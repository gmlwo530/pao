use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use crate::error::{ErrorCode, PaoError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoState {
    MissingCheckout,
    Clean,
    Dirty,
    NotGitRepository,
    GitUnavailable,
}

impl RepoState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MissingCheckout => "missing-checkout",
            Self::Clean => "clean",
            Self::Dirty => "dirty",
            Self::NotGitRepository => "not-git-repository",
            Self::GitUnavailable => "git-unavailable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryStatus {
    pub state: RepoState,
    pub branch: Option<String>,
    pub upstream: Option<String>,
    pub staged: usize,
    pub unstaged: usize,
    pub untracked: usize,
    pub conflicts: usize,
}

impl RepositoryStatus {
    fn with_state(state: RepoState) -> Self {
        Self {
            state,
            branch: None,
            upstream: None,
            staged: 0,
            unstaged: 0,
            untracked: 0,
            conflicts: 0,
        }
    }

    pub fn branch_display(&self) -> &str {
        self.branch.as_deref().unwrap_or("-")
    }

    pub fn upstream_display(&self) -> &str {
        self.upstream.as_deref().unwrap_or("-")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncState {
    Fetched,
    MissingCheckout,
    NotGitRepository,
    GitUnavailable,
    Failed,
}

impl SyncState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fetched => "fetched",
            Self::MissingCheckout => "missing-checkout",
            Self::NotGitRepository => "not-git-repository",
            Self::GitUnavailable => "git-unavailable",
            Self::Failed => "failed",
        }
    }
}

pub fn clone_repository(remote: &str, branch: &str, path: &Path) -> Result<(), PaoError> {
    if path.exists() {
        return Err(PaoError::new(
            ErrorCode::RepositoryCheckoutExists,
            format!("repository checkout already exists at {}", path.display()),
        ));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(PaoError::io)?;
    }

    let output = Command::new("git")
        .arg("clone")
        .arg("--branch")
        .arg(branch)
        .arg("--single-branch")
        .arg(remote)
        .arg(path)
        .output()
        .map_err(PaoError::io)?;

    if output.status.success() {
        return Ok(());
    }

    Err(git_command_failed("git clone", &output))
}

pub fn fetch_repository(path: &Path) -> SyncState {
    if !path.exists() {
        return SyncState::MissingCheckout;
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("fetch")
        .arg("--prune")
        .output();

    let Ok(output) = output else {
        return SyncState::GitUnavailable;
    };

    if output.status.success() {
        return SyncState::Fetched;
    }

    if repository_status(path).state == RepoState::NotGitRepository {
        return SyncState::NotGitRepository;
    }

    SyncState::Failed
}

pub fn repository_status(path: &Path) -> RepositoryStatus {
    if !path.exists() {
        return RepositoryStatus::with_state(RepoState::MissingCheckout);
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("status")
        .arg("--porcelain=v1")
        .output();

    let Ok(output) = output else {
        return RepositoryStatus::with_state(RepoState::GitUnavailable);
    };

    if !output.status.success() {
        return RepositoryStatus::with_state(RepoState::NotGitRepository);
    }

    let mut status = RepositoryStatus::with_state(RepoState::Clean);
    status.branch = current_branch(path);
    status.upstream = current_upstream(path);

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        update_counts(&mut status, line);
    }

    if status.staged > 0 || status.unstaged > 0 || status.untracked > 0 || status.conflicts > 0 {
        status.state = RepoState::Dirty;
    }

    status
}

fn update_counts(status: &mut RepositoryStatus, line: &str) {
    let bytes = line.as_bytes();

    if bytes.len() < 2 {
        return;
    }

    let index_status = bytes[0] as char;
    let worktree_status = bytes[1] as char;

    if index_status == '?' && worktree_status == '?' {
        status.untracked += 1;
        return;
    }

    if is_conflict_status(index_status, worktree_status) {
        status.conflicts += 1;
        return;
    }

    if index_status != ' ' {
        status.staged += 1;
    }

    if worktree_status != ' ' {
        status.unstaged += 1;
    }
}

fn is_conflict_status(index_status: char, worktree_status: char) -> bool {
    matches!(
        (index_status, worktree_status),
        ('D', 'D') | ('A', 'U') | ('U', 'D') | ('U', 'A') | ('D', 'U') | ('A', 'A') | ('U', 'U')
    )
}

fn current_branch(path: &Path) -> Option<String> {
    let branch = git_stdout(path, &["branch", "--show-current"])?;

    if !branch.is_empty() {
        return Some(branch);
    }

    let revision = git_stdout(path, &["rev-parse", "--short", "HEAD"])?;

    Some(format!("detached:{revision}"))
}

fn current_upstream(path: &Path) -> Option<String> {
    git_stdout(
        path,
        &[
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ],
    )
}

fn git_stdout(path: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value = stdout.trim();

    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

pub fn command_version(command: &str) -> Option<String> {
    let output = Command::new(command).arg("--version").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next()?.trim();

    Some(first_line.to_string())
}

fn git_command_failed(command: &str, output: &Output) -> PaoError {
    PaoError::new(
        ErrorCode::GitCommandFailed,
        format!("{command} failed with status {}", output.status),
    )
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::Command;

    use super::{repository_status, RepoState};
    use crate::test_support::TempDir;

    #[test]
    fn repository_status_counts_dirty_files() {
        let temp_dir = TempDir::new("pao-git-status");
        run_git(temp_dir.path(), &["init"]);
        run_git(
            temp_dir.path(),
            &["config", "user.email", "pao@example.invalid"],
        );
        run_git(temp_dir.path(), &["config", "user.name", "PAO Test"]);
        fs::write(temp_dir.path().join("tracked.txt"), "initial\n").expect("file should write");
        run_git(temp_dir.path(), &["add", "tracked.txt"]);
        run_git(temp_dir.path(), &["commit", "-m", "initial"]);
        fs::write(temp_dir.path().join("tracked.txt"), "changed\n").expect("file should write");
        fs::write(temp_dir.path().join("untracked.txt"), "new\n").expect("file should write");

        let status = repository_status(temp_dir.path());

        assert_eq!(status.state, RepoState::Dirty);
        assert_eq!(status.unstaged, 1);
        assert_eq!(status.untracked, 1);
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
}
