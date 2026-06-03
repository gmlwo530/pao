use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::error::{ErrorCode, PaoError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoSnapshot {
    pub repo_path: PathBuf,
    pub files: BTreeMap<String, FileState>,
    pub outside_repo_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileState {
    pub path: String,
    pub index_status: char,
    pub worktree_status: char,
    pub content_hash: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangeSet {
    pub changed_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub deleted_files: Vec<String>,
    pub overlap_files: Vec<String>,
    pub outside_repo_paths: Vec<PathBuf>,
}

pub fn repo_snapshot(repo_path: &Path) -> Result<RepoSnapshot, PaoError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("status")
        .arg("--porcelain=v1")
        .output()
        .map_err(PaoError::io)?;

    if !output.status.success() {
        return Err(PaoError::new(
            ErrorCode::GitCommandFailed,
            format!("git status failed with status {}", output.status),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files = stdout
        .lines()
        .filter_map(|line| parse_status_line(repo_path, line))
        .map(|file_state| (file_state.path.clone(), file_state))
        .collect();

    Ok(RepoSnapshot {
        repo_path: repo_path.to_path_buf(),
        files,
        outside_repo_paths: Vec::new(),
    })
}

pub fn change_set(before: &RepoSnapshot, after: &RepoSnapshot) -> ChangeSet {
    let all_paths = before
        .files
        .keys()
        .chain(after.files.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut changed_files = Vec::new();
    let mut untracked_files = Vec::new();
    let mut deleted_files = Vec::new();
    let mut overlap_files = Vec::new();

    for path in all_paths {
        let before_state = before.files.get(&path);
        let after_state = after.files.get(&path);

        if let Some(state) = after_state {
            if state.is_untracked() {
                untracked_files.push(path.clone());
            }

            if state.is_deleted() {
                deleted_files.push(path.clone());
            }
        }

        if before_state != after_state {
            changed_files.push(path.clone());
        }

        if before_state.is_some() && after_state.is_some() && before_state != after_state {
            overlap_files.push(path);
        }
    }

    let outside_repo_paths = before
        .outside_repo_paths
        .iter()
        .chain(after.outside_repo_paths.iter())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    ChangeSet {
        changed_files,
        untracked_files,
        deleted_files,
        overlap_files,
        outside_repo_paths,
    }
}

pub fn path_outside_repo(repo_path: &Path, path: &Path) -> bool {
    let Ok(repo_path) = repo_path.canonicalize() else {
        return true;
    };
    let Ok(path) = path.canonicalize() else {
        return true;
    };

    !path.starts_with(repo_path)
}

fn parse_status_line(repo_path: &Path, line: &str) -> Option<FileState> {
    let bytes = line.as_bytes();

    if bytes.len() < 4 {
        return None;
    }

    let path = line.get(3..)?.to_string();

    let content_hash = file_content_hash(&repo_path.join(&path));

    Some(FileState {
        path,
        index_status: bytes[0] as char,
        worktree_status: bytes[1] as char,
        content_hash,
    })
}

impl FileState {
    fn is_untracked(&self) -> bool {
        self.index_status == '?' && self.worktree_status == '?'
    }

    fn is_deleted(&self) -> bool {
        self.index_status == 'D' || self.worktree_status == 'D'
    }
}

fn file_content_hash(path: &Path) -> Option<u64> {
    let bytes = fs::read(path).ok()?;
    let mut hash = 0xcbf29ce484222325_u64;

    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    Some(hash)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::Command;

    use super::{change_set, path_outside_repo, repo_snapshot};
    use crate::test_support::TempDir;

    #[test]
    fn detects_changed_untracked_deleted_and_overlap_files() {
        let temp_dir = TempDir::new("pao-change-set");
        run_git(temp_dir.path(), &["init"]);
        run_git(
            temp_dir.path(),
            &["config", "user.email", "pao@example.invalid"],
        );
        run_git(temp_dir.path(), &["config", "user.name", "PAO Test"]);
        fs::write(temp_dir.path().join("modified.txt"), "initial\n").expect("file should write");
        fs::write(temp_dir.path().join("deleted.txt"), "delete me\n").expect("file should write");
        run_git(temp_dir.path(), &["add", "modified.txt", "deleted.txt"]);
        run_git(temp_dir.path(), &["commit", "-m", "initial"]);
        fs::write(temp_dir.path().join("modified.txt"), "user change\n")
            .expect("file should write");
        let before = repo_snapshot(temp_dir.path()).expect("snapshot should collect");

        fs::write(temp_dir.path().join("modified.txt"), "ai change\n").expect("file should write");
        fs::write(temp_dir.path().join("new.txt"), "new\n").expect("file should write");
        fs::remove_file(temp_dir.path().join("deleted.txt")).expect("file should delete");
        let after = repo_snapshot(temp_dir.path()).expect("snapshot should collect");
        let change_set = change_set(&before, &after);

        assert!(change_set
            .changed_files
            .contains(&"modified.txt".to_string()));
        assert!(change_set.untracked_files.contains(&"new.txt".to_string()));
        assert!(change_set
            .deleted_files
            .contains(&"deleted.txt".to_string()));
        assert!(change_set
            .overlap_files
            .contains(&"modified.txt".to_string()));
    }

    #[test]
    fn detects_paths_outside_repo() {
        let temp_dir = TempDir::new("pao-outside-repo");
        fs::create_dir_all(temp_dir.path().join("repo")).expect("repo dir should create");
        fs::write(temp_dir.path().join("outside.txt"), "outside\n").expect("file should write");

        assert!(path_outside_repo(
            &temp_dir.path().join("repo"),
            &temp_dir.path().join("outside.txt")
        ));
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
