use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, path::PathBuf};

fn temp_dir(prefix: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}-{suffix}"));

    fs::create_dir_all(&path).expect("temporary directory should be created");

    path
}

fn create_bare_git_repo(prefix: &str) -> PathBuf {
    let seed_dir = temp_dir(&format!("{prefix}-seed"));
    let remote_dir = temp_dir(&format!("{prefix}-remote"));

    run_git(&seed_dir, &["init"]);
    run_git(&seed_dir, &["config", "user.email", "pao@example.invalid"]);
    run_git(&seed_dir, &["config", "user.name", "PAO Test"]);
    fs::write(seed_dir.join("README.md"), "# test\n").expect("seed file should be written");
    run_git(&seed_dir, &["add", "README.md"]);
    run_git(&seed_dir, &["commit", "-m", "initial"]);
    run_git(&seed_dir, &["branch", "-M", "main"]);

    let output = Command::new("git")
        .arg("clone")
        .arg("--bare")
        .arg(&seed_dir)
        .arg(&remote_dir)
        .output()
        .expect("git clone --bare should run");

    assert!(
        output.status.success(),
        "git clone --bare failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let _ = fs::remove_dir_all(seed_dir);

    remote_dir
}

fn run_git(path: &std::path::Path, args: &[&str]) {
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

#[test]
fn version_output_matches_package_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_pao"))
        .arg("--version")
        .output()
        .expect("pao binary should run");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("pao {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn workspace_commands_create_and_list_repositories() {
    let workspace_dir = temp_dir("pao-cli-workspace");
    let config_dir = temp_dir("pao-cli-config");
    let remote_dir = create_bare_git_repo("pao-cli-remote");

    let init = Command::new(env!("CARGO_BIN_EXE_pao"))
        .arg("init")
        .current_dir(&workspace_dir)
        .env("PAO_CONFIG_HOME", &config_dir)
        .output()
        .expect("pao init should run");

    assert!(init.status.success());
    assert!(workspace_dir.join(".pao/workspace.yaml").exists());

    let add = Command::new(env!("CARGO_BIN_EXE_pao"))
        .args([
            "repo",
            "add",
            "app",
            "--remote",
            remote_dir.to_str().expect("path should be utf-8"),
            "--branch",
            "main",
        ])
        .current_dir(&workspace_dir)
        .env("PAO_CONFIG_HOME", &config_dir)
        .output()
        .expect("pao repo add should run");

    assert!(add.status.success());
    assert!(workspace_dir.join("repos/app/.git").exists());

    let list = Command::new(env!("CARGO_BIN_EXE_pao"))
        .args(["repo", "list"])
        .current_dir(&workspace_dir)
        .env("PAO_CONFIG_HOME", &config_dir)
        .output()
        .expect("pao repo list should run");

    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("app"));

    fs::write(workspace_dir.join("repos/app/untracked.txt"), "new\n")
        .expect("untracked file should be written");

    let status = Command::new(env!("CARGO_BIN_EXE_pao"))
        .args(["repo", "status", "app"])
        .current_dir(&workspace_dir)
        .env("PAO_CONFIG_HOME", &config_dir)
        .output()
        .expect("pao repo status should run");

    let status_stdout = String::from_utf8_lossy(&status.stdout);

    assert!(status.status.success());
    assert!(status_stdout.contains("dirty"));
    assert!(status_stdout.contains("\t1\t0\t"));

    let sync = Command::new(env!("CARGO_BIN_EXE_pao"))
        .arg("sync")
        .current_dir(&workspace_dir)
        .env("PAO_CONFIG_HOME", &config_dir)
        .output()
        .expect("pao sync should run");

    assert!(sync.status.success());
    assert!(String::from_utf8_lossy(&sync.stdout).contains("fetched"));

    let task = Command::new(env!("CARGO_BIN_EXE_pao"))
        .args(["task", "create", "release-0.1"])
        .current_dir(&workspace_dir)
        .env("PAO_CONFIG_HOME", &config_dir)
        .output()
        .expect("pao task create should run");

    assert!(task.status.success());
    assert!(workspace_dir
        .join(".pao/tasks/release-0.1/task.yaml")
        .exists());
    assert!(workspace_dir
        .join(".pao/tasks/release-0.1/sessions")
        .exists());
    assert!(workspace_dir
        .join(".pao/tasks/release-0.1/command-log")
        .exists());

    let client = Command::new(env!("CARGO_BIN_EXE_pao"))
        .args(["client", "add", "codex", "--command", "codex"])
        .current_dir(&workspace_dir)
        .env("PAO_CONFIG_HOME", &config_dir)
        .output()
        .expect("pao client add should run");

    assert!(client.status.success());

    let chat = Command::new(env!("CARGO_BIN_EXE_pao"))
        .args(["chat", "--repo", "app", "--prompt", "make a small change"])
        .current_dir(&workspace_dir)
        .env("PAO_CONFIG_HOME", &config_dir)
        .output()
        .expect("pao chat should run");

    let chat_stdout = String::from_utf8_lossy(&chat.stdout);

    assert!(chat.status.success());
    assert!(chat_stdout.contains("Approval required"));
    assert!(chat_stdout.contains("approval:"));

    let remove = Command::new(env!("CARGO_BIN_EXE_pao"))
        .args(["repo", "remove", "app", "--keep-checkout"])
        .current_dir(&workspace_dir)
        .env("PAO_CONFIG_HOME", &config_dir)
        .output()
        .expect("pao repo remove should run");

    assert!(remove.status.success());
    assert!(workspace_dir.join("repos/app/.git").exists());

    let _ = fs::remove_dir_all(workspace_dir);
    let _ = fs::remove_dir_all(config_dir);
    let _ = fs::remove_dir_all(remote_dir);
}

#[test]
fn workspace_missing_error_includes_stable_code() {
    let workspace_dir = temp_dir("pao-cli-missing-workspace");
    let config_dir = temp_dir("pao-cli-missing-config");

    let output = Command::new(env!("CARGO_BIN_EXE_pao"))
        .args(["repo", "list"])
        .current_dir(&workspace_dir)
        .env("PAO_CONFIG_HOME", &config_dir)
        .output()
        .expect("pao repo list should run");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("PAO-1002"));

    let _ = fs::remove_dir_all(workspace_dir);
    let _ = fs::remove_dir_all(config_dir);
}

#[test]
fn client_add_rejects_shell_control_operator_commands() {
    let workspace_dir = temp_dir("pao-cli-client-invalid-workspace");
    let config_dir = temp_dir("pao-cli-client-invalid-config");

    let output = Command::new(env!("CARGO_BIN_EXE_pao"))
        .args([
            "client",
            "add",
            "unsafe",
            "--command",
            "codex && rm -rf target",
        ])
        .current_dir(&workspace_dir)
        .env("PAO_CONFIG_HOME", &config_dir)
        .output()
        .expect("pao client add should run");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("PAO-1202"));

    let _ = fs::remove_dir_all(workspace_dir);
    let _ = fs::remove_dir_all(config_dir);
}
