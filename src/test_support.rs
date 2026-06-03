#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new(prefix: &str) -> Self {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}-{suffix}"));

        fs::create_dir_all(&path).expect("temporary directory should be created");

        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub fn create_bare_git_repo(prefix: &str) -> TempDir {
    let seed = TempDir::new(&format!("{prefix}-seed"));
    let remote = TempDir::new(&format!("{prefix}-remote"));

    run_git(seed.path(), &["init"]);
    run_git(
        seed.path(),
        &["config", "user.email", "pao@example.invalid"],
    );
    run_git(seed.path(), &["config", "user.name", "PAO Test"]);
    fs::write(seed.path().join("README.md"), "# test\n").expect("seed file should be written");
    run_git(seed.path(), &["add", "README.md"]);
    run_git(seed.path(), &["commit", "-m", "initial"]);
    run_git(seed.path(), &["branch", "-M", "main"]);

    let output = Command::new("git")
        .arg("clone")
        .arg("--bare")
        .arg(seed.path())
        .arg(remote.path())
        .output()
        .expect("git clone --bare should run");

    assert!(
        output.status.success(),
        "git clone --bare failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    remote
}

fn run_git(path: &Path, args: &[&str]) {
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
