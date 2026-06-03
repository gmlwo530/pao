use std::env;
use std::path::Path;

use crate::ai_client::validate_client_command;
use crate::config::{config_path, UserConfig};
use crate::error::{ErrorCode, PaoError};
use crate::git::{command_version, repository_status, RepoState};
use crate::workspace::Workspace;
use crate::{RuntimeEnv, VERSION};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticStatus {
    Ok,
    Warn,
    Error,
}

impl DiagnosticStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub check: String,
    pub status: DiagnosticStatus,
    pub detail: String,
}

impl Diagnostic {
    fn new(check: impl Into<String>, status: DiagnosticStatus, detail: impl Into<String>) -> Self {
        Self {
            check: check.into(),
            status,
            detail: detail.into(),
        }
    }
}

pub fn doctor_report(runtime: &RuntimeEnv) -> String {
    let mut diagnostics = vec![
        Diagnostic::new("version", DiagnosticStatus::Ok, VERSION),
        command_diagnostic("git", "git"),
        command_diagnostic("rg", "rg"),
    ];

    append_workspace_diagnostics(runtime, &mut diagnostics);
    append_config_diagnostics(runtime, &mut diagnostics);

    render_diagnostics(&diagnostics)
}

fn command_diagnostic(check: &str, command: &str) -> Diagnostic {
    match command_version(command) {
        Some(version) => Diagnostic::new(check, DiagnosticStatus::Ok, version),
        None => Diagnostic::new(
            check,
            DiagnosticStatus::Error,
            format!("`{command}` is not available"),
        ),
    }
}

fn append_workspace_diagnostics(runtime: &RuntimeEnv, diagnostics: &mut Vec<Diagnostic>) {
    match Workspace::load(&runtime.cwd) {
        Ok(workspace) => {
            diagnostics.push(Diagnostic::new(
                "workspace",
                DiagnosticStatus::Ok,
                workspace
                    .pao_dir()
                    .join("workspace.yaml")
                    .display()
                    .to_string(),
            ));

            if workspace.file.repos.is_empty() {
                diagnostics.push(Diagnostic::new(
                    "repos",
                    DiagnosticStatus::Warn,
                    "no repositories registered",
                ));
                return;
            }

            for (name, repo) in &workspace.file.repos {
                let checkout_path = workspace.repo_checkout_path(repo);
                let status = repository_status(&checkout_path);
                diagnostics.push(repo_diagnostic(name, repo.path.as_str(), &status));
            }
        }
        Err(error) => diagnostics.push(error_diagnostic("workspace", &error)),
    }
}

fn repo_diagnostic(name: &str, path: &str, status: &crate::git::RepositoryStatus) -> Diagnostic {
    let diagnostic_status = match status.state {
        RepoState::Clean => DiagnosticStatus::Ok,
        RepoState::Dirty => DiagnosticStatus::Warn,
        RepoState::MissingCheckout | RepoState::NotGitRepository | RepoState::GitUnavailable => {
            DiagnosticStatus::Error
        }
    };
    let detail = format!(
        "{} path={} branch={} upstream={} staged={} unstaged={} untracked={} conflicts={}",
        status.state.as_str(),
        path,
        status.branch_display(),
        status.upstream_display(),
        status.staged,
        status.unstaged,
        status.untracked,
        status.conflicts
    );

    Diagnostic::new(format!("repo.{name}"), diagnostic_status, detail)
}

fn append_config_diagnostics(runtime: &RuntimeEnv, diagnostics: &mut Vec<Diagnostic>) {
    let config_path = match config_path(runtime) {
        Ok(path) => path,
        Err(error) => {
            diagnostics.push(error_diagnostic("config", &error));
            return;
        }
    };

    match UserConfig::load(runtime) {
        Ok(config) => {
            diagnostics.push(Diagnostic::new(
                "config",
                config_status(&config_path),
                config_path.display().to_string(),
            ));

            match config.file.default_client.as_deref() {
                Some(default_client) => diagnostics.push(Diagnostic::new(
                    "default-client",
                    DiagnosticStatus::Ok,
                    default_client,
                )),
                None => diagnostics.push(Diagnostic::new(
                    "default-client",
                    DiagnosticStatus::Warn,
                    "no default client configured",
                )),
            }

            if config.file.clients.is_empty() {
                diagnostics.push(Diagnostic::new(
                    "clients",
                    DiagnosticStatus::Warn,
                    "no clients registered",
                ));
                return;
            }

            for (name, client) in &config.file.clients {
                diagnostics.push(client_diagnostic(
                    name,
                    client.command.as_str(),
                    &runtime.cwd,
                ));
            }
        }
        Err(error) => diagnostics.push(error_diagnostic("config", &error)),
    }
}

fn config_status(config_path: &Path) -> DiagnosticStatus {
    if config_path.exists() {
        DiagnosticStatus::Ok
    } else {
        DiagnosticStatus::Warn
    }
}

fn client_diagnostic(name: &str, command: &str, cwd: &Path) -> Diagnostic {
    let command = match validate_client_command(command) {
        Ok(command) => command,
        Err(error) => return error_diagnostic(format!("client.{name}"), &error),
    };

    if program_available(command.program.as_str(), cwd) {
        return Diagnostic::new(
            format!("client.{name}"),
            DiagnosticStatus::Ok,
            format!("program={} args={}", command.program, command.args.len()),
        );
    }

    Diagnostic::new(
        format!("client.{name}"),
        DiagnosticStatus::Error,
        format!("program `{}` is not available", command.program),
    )
}

fn program_available(program: &str, cwd: &Path) -> bool {
    let program_path = Path::new(program);

    if has_path_separator(program) {
        let path = if program_path.is_absolute() {
            program_path.to_path_buf()
        } else {
            cwd.join(program_path)
        };

        return is_executable_file(&path);
    }

    let Some(paths) = env::var_os("PATH") else {
        return false;
    };

    env::split_paths(&paths).any(|path| is_executable_file(&path.join(program)))
}

fn has_path_separator(program: &str) -> bool {
    program.contains('/') || program.contains('\\')
}

fn is_executable_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let Ok(metadata) = path.metadata() else {
            return false;
        };

        metadata.permissions().mode() & 0o111 != 0
    }

    #[cfg(not(unix))]
    {
        true
    }
}

fn error_diagnostic(check: impl Into<String>, error: &PaoError) -> Diagnostic {
    let status = if error.code() == ErrorCode::WorkspaceMissing {
        DiagnosticStatus::Warn
    } else {
        DiagnosticStatus::Error
    };

    Diagnostic::new(check, status, error.render())
}

fn render_diagnostics(diagnostics: &[Diagnostic]) -> String {
    let mut output = String::from("pao doctor\nCHECK\tSTATUS\tDETAIL\n");

    for diagnostic in diagnostics {
        output.push_str(&format!(
            "{}\t{}\t{}\n",
            diagnostic.check,
            diagnostic.status.as_str(),
            diagnostic.detail
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::Command;

    use super::doctor_report;
    use crate::config::UserConfig;
    use crate::test_support::TempDir;
    use crate::workspace::Workspace;
    use crate::RuntimeEnv;

    #[test]
    fn report_includes_workspace_repo_and_client_diagnostics() {
        let temp_dir = TempDir::new("pao-doctor-report");
        let runtime = RuntimeEnv {
            cwd: temp_dir.path().to_path_buf(),
            home: None,
            config_home: Some(temp_dir.path().join("config")),
        };
        let mut workspace = Workspace::init(temp_dir.path()).expect("workspace should initialize");
        workspace
            .add_repo("app", "https://example.invalid/app.git", "main")
            .expect("repo should be registered");
        run_git(temp_dir.path().join("repos/app").as_path(), &["init"]);

        let mut config = UserConfig::load(&runtime).expect("config should load");
        config
            .add_client("git", "git")
            .expect("client should be added");

        let report = doctor_report(&runtime);

        assert!(report.contains("CHECK\tSTATUS\tDETAIL"));
        assert!(report.contains("workspace\tok\t"));
        assert!(report.contains("repo.app\tok\tclean"));
        assert!(report.contains("config\tok\t"));
        assert!(report.contains("default-client\tok\tgit"));
        assert!(report.contains("client.git\tok\tprogram=git"));
    }

    #[test]
    fn report_includes_invalid_client_command_code() {
        let temp_dir = TempDir::new("pao-doctor-invalid-client");
        let runtime = RuntimeEnv {
            cwd: temp_dir.path().to_path_buf(),
            home: None,
            config_home: Some(temp_dir.path().join("config")),
        };
        fs::create_dir_all(runtime.config_home.as_ref().expect("config path exists"))
            .expect("config dir should be created");
        fs::write(
            temp_dir.path().join("config/config.yaml"),
            "default_client: unsafe\nclients:\n  unsafe:\n    command: codex && rm -rf target\n",
        )
        .expect("config should write");

        let report = doctor_report(&runtime);

        assert!(report.contains("workspace\twarn\tPAO-1002"));
        assert!(report.contains("client.unsafe\terror\tPAO-1202"));
    }

    fn run_git(path: &std::path::Path, args: &[&str]) {
        fs::create_dir_all(path).expect("git directory should be created");
        let output = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(args)
            .output()
            .expect("git should run");

        assert!(
            output.status.success(),
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
