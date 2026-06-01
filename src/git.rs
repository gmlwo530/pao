use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoStatus {
    MissingCheckout,
    Clean,
    Dirty,
    NotGitRepository,
    GitUnavailable,
}

impl RepoStatus {
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

pub fn repository_status(path: &Path) -> RepoStatus {
    if !path.exists() {
        return RepoStatus::MissingCheckout;
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("status")
        .arg("--short")
        .output();

    let Ok(output) = output else {
        return RepoStatus::GitUnavailable;
    };

    if !output.status.success() {
        return RepoStatus::NotGitRepository;
    }

    if output.stdout.is_empty() {
        RepoStatus::Clean
    } else {
        RepoStatus::Dirty
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
