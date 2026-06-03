use std::fs;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::ai_client::{validate_client_command, AiClientRunRequest};
use crate::approval::ai_client_approval_payload;
use crate::change::repo_snapshot;
use crate::cli::{ChatArgs, Cli, ClientCommand, Commands, RepoCommand, TaskCommand};
use crate::config::{config_path, UserConfig};
use crate::error::{ErrorCode, PaoError};
use crate::git::{clone_repository, command_version, fetch_repository, repository_status};
use crate::task::create_task;
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
        Some(Commands::Task { command }) => task(command, runtime),
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
            workspace.validate_new_repo(&args.name, &args.remote, &args.branch)?;
            let checkout_path = workspace.repo_checkout_path_for_name(&args.name);

            clone_repository(&args.remote, &args.branch, &checkout_path)?;
            workspace.add_repo(&args.name, &args.remote, &args.branch)?;

            Ok(CommandReport::stdout(format!(
                "Registered repository `{}` on branch `{}` at {}\n",
                args.name,
                args.branch,
                checkout_path.display()
            )))
        }
        RepoCommand::Remove(args) => {
            let mut workspace = Workspace::load(&runtime.cwd)?;
            let removed = workspace.remove_repo(&args.name, args.keep_checkout)?;

            Ok(CommandReport::stdout(format!(
                "Removed repository `{}` from workspace and kept checkout at {}\n",
                args.name, removed.path
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
            let mut output = String::from(
                "NAME\tSTATE\tBRANCH\tUPSTREAM\tSTAGED\tUNSTAGED\tUNTRACKED\tCONFLICTS\tPATH\n",
            );

            if let Some(name) = args.name {
                let repo = workspace.repo(&name)?;
                let status = repository_status(&workspace.repo_checkout_path(repo));
                output.push_str(&format_repo_status(&name, repo.path.as_str(), &status));

                return Ok(CommandReport::stdout(output));
            }

            for (name, repo) in &workspace.file.repos {
                let status = repository_status(&workspace.repo_checkout_path(repo));
                output.push_str(&format_repo_status(name, repo.path.as_str(), &status));
            }

            Ok(CommandReport::stdout(output))
        }
    }
}

fn sync(runtime: &RuntimeEnv) -> Result<CommandReport, PaoError> {
    let workspace = Workspace::load(&runtime.cwd)?;
    let mut output = String::from(
        "NAME\tSYNC\tSTATE\tBRANCH\tUPSTREAM\tSTAGED\tUNSTAGED\tUNTRACKED\tCONFLICTS\tPATH\n",
    );

    if workspace.file.repos.is_empty() {
        return Ok(CommandReport::stdout("No repositories registered.\n"));
    }

    for (name, repo) in &workspace.file.repos {
        let checkout_path = workspace.repo_checkout_path(repo);
        let sync_state = fetch_repository(&checkout_path);
        let status = repository_status(&checkout_path);
        output.push_str(&format!(
            "{}\t{}\t{}",
            name,
            sync_state.as_str(),
            format_repo_status_fields(repo.path.as_str(), &status)
        ));
    }

    Ok(CommandReport::stdout(output))
}

fn task(command: TaskCommand, runtime: &RuntimeEnv) -> Result<CommandReport, PaoError> {
    match command {
        TaskCommand::Create(args) => {
            let workspace = Workspace::load(&runtime.cwd)?;
            let task = create_task(&workspace, &args.task_id)?;

            Ok(CommandReport::stdout(format!(
                "Created task `{}` with sessions at {} and command log at {}\n",
                task.id, task.sessions_path, task.command_log_path
            )))
        }
    }
}

fn chat(args: ChatArgs, runtime: &RuntimeEnv) -> Result<CommandReport, PaoError> {
    let workspace = Workspace::load(&runtime.cwd)?;
    let config = UserConfig::load(runtime)?;
    let repo_name = args.repo.ok_or_else(|| {
        PaoError::new(
            ErrorCode::RepositoryMissing,
            "chat requires a target repository via --repo",
        )
    })?;
    let repo = workspace.repo(&repo_name)?;
    let client_name = config.file.default_client.as_ref().ok_or_else(|| {
        PaoError::new(
            ErrorCode::ClientMissing,
            "configure a default client before running chat",
        )
    })?;
    let client = config.file.clients.get(client_name).ok_or_else(|| {
        PaoError::new(
            ErrorCode::ClientMissing,
            format!("client `{client_name}` is not registered"),
        )
    })?;

    let command = validate_client_command(&client.command)?;
    let repo_path = workspace.repo_checkout_path(repo);
    let baseline = repo_snapshot(&repo_path)?;
    let request = AiClientRunRequest {
        client_name: client_name.clone(),
        command,
        cwd: repo_path,
        prompt: args.prompt,
        timeout: Duration::from_secs(args.timeout_seconds),
    };
    let approval = ai_client_approval_payload(
        &request,
        Some(repo_name.clone()),
        "AI client may modify files in the target repository",
        baseline_state_summary(&baseline),
    );
    let session_id = format!("session-{}", unix_nanos()?);
    let session_dir = workspace.pao_dir().join("sessions").join(&session_id);

    fs::create_dir_all(&session_dir).map_err(PaoError::io)?;
    write_yaml(session_dir.join("baseline.yaml"), &baseline)?;
    write_yaml(session_dir.join("approval.yaml"), &approval)?;

    Ok(CommandReport::stdout(format!(
        "Approval required for `{}` using client `{}`.\nsession: {}\nbaseline_files: {}\napproval: {}\n",
        repo_name,
        client_name,
        session_id,
        baseline.files.len(),
        session_dir.join("approval.yaml").display()
    )))
}

fn format_repo_status(name: &str, path: &str, status: &crate::git::RepositoryStatus) -> String {
    format!("{}\t{}", name, format_repo_status_fields(path, status))
}

fn format_repo_status_fields(path: &str, status: &crate::git::RepositoryStatus) -> String {
    format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
        status.state.as_str(),
        status.branch_display(),
        status.upstream_display(),
        status.staged,
        status.unstaged,
        status.untracked,
        status.conflicts,
        path
    )
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

fn baseline_state_summary(snapshot: &crate::change::RepoSnapshot) -> String {
    if snapshot.files.is_empty() {
        return "clean".to_string();
    }

    format!("dirty files={}", snapshot.files.len())
}

fn write_yaml<T: serde::Serialize>(path: std::path::PathBuf, value: &T) -> Result<(), PaoError> {
    let content = serde_yaml::to_string(value).map_err(PaoError::serialization)?;

    fs::write(path, content).map_err(PaoError::io)
}

fn unix_nanos() -> Result<u128, PaoError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| PaoError::new(ErrorCode::Io, error.to_string()))?;

    Ok(duration.as_nanos())
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
        let remote_dir = crate::test_support::create_bare_git_repo("pao-command-remote");

        run_with_env(
            args(&[
                "pao",
                "repo",
                "add",
                "app",
                "--remote",
                remote_dir.path().to_str().expect("path should be utf-8"),
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

    #[tokio::test]
    async fn task_create_stores_task_metadata() {
        let temp_dir = TempDir::new("pao-command-task");
        let runtime = RuntimeEnv {
            cwd: temp_dir.path().to_path_buf(),
            home: None,
            config_home: Some(temp_dir.path().join("config")),
        };

        run_with_env(args(&["pao", "init"]), &runtime)
            .await
            .expect("workspace should initialize");

        let report = run_with_env(args(&["pao", "task", "create", "release-0.1"]), &runtime)
            .await
            .expect("task should be created");

        assert!(report.stdout.contains("release-0.1"));
        assert!(temp_dir
            .path()
            .join(".pao/tasks/release-0.1/task.yaml")
            .exists());
    }

    #[tokio::test]
    async fn chat_creates_baseline_and_approval_artifacts() {
        let temp_dir = TempDir::new("pao-command-chat");
        let runtime = RuntimeEnv {
            cwd: temp_dir.path().to_path_buf(),
            home: None,
            config_home: Some(temp_dir.path().join("config")),
        };
        let remote_dir = crate::test_support::create_bare_git_repo("pao-command-chat-remote");

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
                remote_dir.path().to_str().expect("path should be utf-8"),
                "--branch",
                "main",
            ]),
            &runtime,
        )
        .await
        .expect("repo should be added");
        run_with_env(
            args(&["pao", "client", "add", "codex", "--command", "codex"]),
            &runtime,
        )
        .await
        .expect("client should be added");

        let report = run_with_env(
            args(&[
                "pao",
                "chat",
                "--repo",
                "app",
                "--prompt",
                "make a small change",
            ]),
            &runtime,
        )
        .await
        .expect("chat should prepare approval");

        assert!(report.stdout.contains("Approval required"));
        assert!(temp_dir.path().join(".pao/sessions").exists());
    }
}
