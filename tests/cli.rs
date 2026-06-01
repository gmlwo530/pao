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
            "https://example.com/app.git",
            "--branch",
            "main",
        ])
        .current_dir(&workspace_dir)
        .env("PAO_CONFIG_HOME", &config_dir)
        .output()
        .expect("pao repo add should run");

    assert!(add.status.success());

    let list = Command::new(env!("CARGO_BIN_EXE_pao"))
        .args(["repo", "list"])
        .current_dir(&workspace_dir)
        .env("PAO_CONFIG_HOME", &config_dir)
        .output()
        .expect("pao repo list should run");

    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("app"));

    let _ = fs::remove_dir_all(workspace_dir);
    let _ = fs::remove_dir_all(config_dir);
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
