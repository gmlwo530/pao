use crate::cli::{ChatArgs, Cli, ClientCommand, Commands, RepoCommand};
use crate::config::{config_path, UserConfig};
use crate::error::{ErrorCode, PaoError};
use crate::git::{command_version, repository_status};
use crate::workspace::Workspace;
use crate::RuntimeEnv;
use crate::VERSION;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandReport {
    pub stdout: String,
}

impl CommandReport {
    pub fn stdout(stdout: impl Into<String>) -> Self {
        Self {
            stdout: stdout.into(),
        }
    }
}

pub async fn execute(cli: Cli, runtime: &RuntimeEnv) -> Result<CommandReport, PaoError> {
    match cli.command {
        Some(Commands::Init) => init(runtime),
        Some(Commands::Repo { command }) => repo(command, runtime),
        Some(Commands::Sync) => sync(runtime),
        Some(Commands::Chat(args)) => chat(args, runtime),
        Some(Commands::Client { command }) => client(command, runtime),
        Some(Commands::Doctor) => doctor(runtime),
        None => Err(PaoError::new(
            ErrorCode::NotImplemented,
            "TUI startup is not implemented in this CLI skeleton",
        )),
    }
}

fn init(runtime: &RuntimeEnv) -> Result<CommandReport, PaoError> {
    Workspace::init(&runtime.cwd)?;

    Ok(CommandReport::stdout(
        "Initialized PAO workspace at .pao/workspace.yaml\n",
    ))
}

fn repo(command: RepoCommand, runtime: &RuntimeEnv) -> Result<CommandReport, PaoError> {
    match command {
        RepoCommand::Add(args) => {
            let mut workspace = Workspace::load(&runtime.cwd)?;
            workspace.add_repo(&args.name, &args.remote, &args.branch)?;

            Ok(CommandReport::stdout(format!(
                "Registered repository `{}` on branch `{}`\n",
                args.name, args.branch
            )))
        }
        RepoCommand::List => {
            let workspace = Workspace::load(&runtime.cwd)?;

            if workspace.file.repos.is_empty() {
                return Ok(CommandReport::stdout("No repositories registered.\n"));
            }

            let mut output = String::from("NAME\tBRANCH\tREMOTE\tPATH\n");

            for (name, repo) in &workspace.file.repos {
                output.push_str(&format!(
                    "{}\t{}\t{}\t{}\n",
                    name, repo.branch, repo.remote, repo.path
                ));
            }

            Ok(CommandReport::stdout(output))
        }
        RepoCommand::Status(args) => {
            let workspace = Workspace::load(&runtime.cwd)?;
            let mut output = String::from("NAME\tSTATUS\tPATH\n");

            if let Some(name) = args.name {
                let repo = workspace.repo(&name)?;
                let status = repository_status(&workspace.repo_checkout_path(repo));
                output.push_str(&format!("{}\t{}\t{}\n", name, status.as_str(), repo.path));

                return Ok(CommandReport::stdout(output));
            }

            for (name, repo) in &workspace.file.repos {
                let status = repository_status(&workspace.repo_checkout_path(repo));
                output.push_str(&format!("{}\t{}\t{}\n", name, status.as_str(), repo.path));
            }

            Ok(CommandReport::stdout(output))
        }
    }
}

fn sync(runtime: &RuntimeEnv) -> Result<CommandReport, PaoError> {
    let workspace = Workspace::load(&runtime.cwd)?;

    Ok(CommandReport::stdout(format!(
        "Workspace metadata is current. repositories={}\n",
        workspace.file.repos.len()
    )))
}

fn chat(args: ChatArgs, runtime: &RuntimeEnv) -> Result<CommandReport, PaoError> {
    let workspace = Workspace::load(&runtime.cwd)?;

    if let Some(repo_name) = &args.repo {
        workspace.repo(repo_name)?;
    }

    Err(PaoError::new(
        ErrorCode::NotImplemented,
        "AI client execution is not implemented in this CLI skeleton",
    ))
}

fn client(command: ClientCommand, runtime: &RuntimeEnv) -> Result<CommandReport, PaoError> {
    match command {
        ClientCommand::Add(args) => {
            let mut config = UserConfig::load(runtime)?;
            config.add_client(&args.name, &args.command)?;

            Ok(CommandReport::stdout(format!(
                "Registered client `{}` with command `{}`\n",
                args.name, args.command
            )))
        }
        ClientCommand::List => {
            let config = UserConfig::load(runtime)?;

            if config.file.clients.is_empty() {
                return Ok(CommandReport::stdout("No clients registered.\n"));
            }

            let mut output = String::from("NAME\tCOMMAND\tDEFAULT\n");

            for (name, client) in &config.file.clients {
                let is_default = config.file.default_client.as_deref() == Some(name.as_str());
                let default_marker = if is_default { "yes" } else { "no" };
                output.push_str(&format!(
                    "{}\t{}\t{}\n",
                    name, client.command, default_marker
                ));
            }

            Ok(CommandReport::stdout(output))
        }
        ClientCommand::SetDefault(args) => {
            let mut config = UserConfig::load(runtime)?;
            config.set_default(&args.name)?;

            Ok(CommandReport::stdout(format!(
                "Set default client to `{}`\n",
                args.name
            )))
        }
    }
}

fn doctor(runtime: &RuntimeEnv) -> Result<CommandReport, PaoError> {
    let workspace_status = if runtime.cwd.join(".pao/workspace.yaml").exists() {
        "found"
    } else {
        "missing"
    };
    let config_path = config_path(runtime)?;
    let git_status = command_version("git").unwrap_or_else(|| "missing".to_string());
    let rg_status = command_version("rg").unwrap_or_else(|| "missing".to_string());

    let output = format!(
        "pao doctor\nversion: {VERSION}\nworkspace: {workspace_status}\nconfig: {}\ngit: {git_status}\nrg: {rg_status}\n",
        config_path.display()
    );

    Ok(CommandReport::stdout(output))
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::PathBuf;

    use crate::run_with_env;
    use crate::test_support::TempDir;
    use crate::RuntimeEnv;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[tokio::test]
    async fn init_and_repo_list_workflow() {
        let temp_dir = TempDir::new("pao-command-workspace");
        let runtime = RuntimeEnv {
            cwd: temp_dir.path().to_path_buf(),
            home: None,
            config_home: Some(temp_dir.path().join("config")),
        };

        run_with_env(args(&["pao", "init"]), &runtime)
            .await
            .expect("workspace should initialize");
        run_with_env(
            args(&[
                "pao",
                "repo",
                "add",
                "app",
                "--remote",
                "https://example.com/app.git",
                "--branch",
                "main",
            ]),
            &runtime,
        )
        .await
        .expect("repo should be added");

        let report = run_with_env(args(&["pao", "repo", "list"]), &runtime)
            .await
            .expect("repos should list");

        assert!(report.stdout.contains("app"));
        assert!(report.stdout.contains("repos/app"));
    }

    #[tokio::test]
    async fn client_commands_use_config_home() {
        let temp_dir = TempDir::new("pao-command-client");
        let runtime = RuntimeEnv {
            cwd: PathBuf::from("/workspace"),
            home: None,
            config_home: Some(temp_dir.path().join("config")),
        };

        run_with_env(
            args(&["pao", "client", "add", "codex", "--command", "codex"]),
            &runtime,
        )
        .await
        .expect("client should be added");

        let report = run_with_env(args(&["pao", "client", "list"]), &runtime)
            .await
            .expect("clients should list");

        assert!(report.stdout.contains("codex"));
        assert!(report.stdout.contains("yes"));
    }
}
