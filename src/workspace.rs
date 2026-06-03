use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{ErrorCode, PaoError};
use crate::validation::{validate_name, validate_relative_path};

pub const WORKSPACE_DIR: &str = ".pao";
pub const WORKSPACE_FILE: &str = "workspace.yaml";
pub const WORKSPACE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceFile {
    pub version: u32,
    #[serde(default)]
    pub repos: BTreeMap<String, RepoConfig>,
}

impl Default for WorkspaceFile {
    fn default() -> Self {
        Self {
            version: WORKSPACE_VERSION,
            repos: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoConfig {
    pub remote: String,
    pub branch: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub root: PathBuf,
    pub file: WorkspaceFile,
}

impl Workspace {
    pub fn init(root: &Path) -> Result<Self, PaoError> {
        let pao_dir = root.join(WORKSPACE_DIR);
        let workspace_path = pao_dir.join(WORKSPACE_FILE);

        if workspace_path.exists() {
            return Err(PaoError::new(
                ErrorCode::WorkspaceExists,
                "PAO workspace already exists",
            ));
        }

        fs::create_dir_all(pao_dir.join("repos")).map_err(PaoError::io)?;
        fs::create_dir_all(pao_dir.join("tasks")).map_err(PaoError::io)?;
        fs::create_dir_all(pao_dir.join("sessions")).map_err(PaoError::io)?;
        fs::create_dir_all(root.join("repos")).map_err(PaoError::io)?;

        let workspace = Self {
            root: root.to_path_buf(),
            file: WorkspaceFile::default(),
        };
        workspace.save()?;

        Ok(workspace)
    }

    pub fn load(root: &Path) -> Result<Self, PaoError> {
        let workspace_path = root.join(WORKSPACE_DIR).join(WORKSPACE_FILE);

        if !workspace_path.exists() {
            return Err(PaoError::new(
                ErrorCode::WorkspaceMissing,
                "run `pao init` before using workspace commands",
            ));
        }

        let content = fs::read_to_string(&workspace_path).map_err(PaoError::io)?;
        let file: WorkspaceFile =
            serde_yaml::from_str(&content).map_err(PaoError::serialization)?;

        if file.version != WORKSPACE_VERSION {
            return Err(PaoError::new(
                ErrorCode::WorkspaceInvalid,
                format!("unsupported workspace version {}", file.version),
            ));
        }

        for repo in file.repos.values() {
            validate_relative_path(&repo.path)?;
        }

        Ok(Self {
            root: root.to_path_buf(),
            file,
        })
    }

    pub fn add_repo(&mut self, name: &str, remote: &str, branch: &str) -> Result<(), PaoError> {
        self.validate_new_repo(name, remote, branch)?;

        self.file.repos.insert(
            name.to_string(),
            RepoConfig {
                remote: remote.to_string(),
                branch: branch.to_string(),
                path: format!("repos/{name}"),
            },
        );
        self.save()
    }

    pub fn validate_new_repo(
        &self,
        name: &str,
        remote: &str,
        branch: &str,
    ) -> Result<(), PaoError> {
        validate_name("repository", name, ErrorCode::RepositoryInvalid)?;

        if remote.trim().is_empty() {
            return Err(PaoError::new(
                ErrorCode::RepositoryInvalid,
                "repository remote cannot be empty",
            ));
        }

        if branch.trim().is_empty() {
            return Err(PaoError::new(
                ErrorCode::RepositoryInvalid,
                "repository branch cannot be empty",
            ));
        }

        if self.file.repos.contains_key(name) {
            return Err(PaoError::new(
                ErrorCode::RepositoryExists,
                format!("repository `{name}` is already registered"),
            ));
        }

        Ok(())
    }

    pub fn remove_repo(&mut self, name: &str, keep_checkout: bool) -> Result<RepoConfig, PaoError> {
        if !keep_checkout {
            return Err(PaoError::new(
                ErrorCode::NotImplemented,
                "checkout removal is not implemented; use --keep-checkout",
            ));
        }

        let repo = self.file.repos.remove(name).ok_or_else(|| {
            PaoError::new(
                ErrorCode::RepositoryMissing,
                format!("repository `{name}` is not registered"),
            )
        })?;

        self.save()?;

        Ok(repo)
    }

    pub fn repo(&self, name: &str) -> Result<&RepoConfig, PaoError> {
        self.file.repos.get(name).ok_or_else(|| {
            PaoError::new(
                ErrorCode::RepositoryMissing,
                format!("repository `{name}` is not registered"),
            )
        })
    }

    pub fn repo_checkout_path(&self, repo: &RepoConfig) -> PathBuf {
        self.root.join(&repo.path)
    }

    pub fn repo_checkout_path_for_name(&self, name: &str) -> PathBuf {
        self.root.join("repos").join(name)
    }

    fn save(&self) -> Result<(), PaoError> {
        let workspace_path = self.root.join(WORKSPACE_DIR).join(WORKSPACE_FILE);
        let content = serde_yaml::to_string(&self.file).map_err(PaoError::serialization)?;

        fs::write(workspace_path, content).map_err(PaoError::io)
    }
}

#[cfg(test)]
mod tests {
    use super::{Workspace, WorkspaceFile};
    use crate::test_support::TempDir;

    #[test]
    fn initializes_workspace_file() {
        let temp_dir = TempDir::new("pao-workspace-init");

        let workspace = Workspace::init(temp_dir.path()).expect("workspace should initialize");

        assert_eq!(workspace.file, WorkspaceFile::default());
        assert!(temp_dir.path().join(".pao/workspace.yaml").exists());
        assert!(temp_dir.path().join(".pao/repos").exists());
        assert!(temp_dir.path().join("repos").exists());
    }

    #[test]
    fn stores_registered_repositories() {
        let temp_dir = TempDir::new("pao-workspace-repo");
        let mut workspace = Workspace::init(temp_dir.path()).expect("workspace should initialize");

        workspace
            .add_repo("app", "https://example.com/app.git", "main")
            .expect("repo should be registered");

        let loaded = Workspace::load(temp_dir.path()).expect("workspace should load");
        let repo = loaded.repo("app").expect("repo should exist");

        assert_eq!(repo.branch, "main");
        assert_eq!(repo.path, "repos/app");
    }

    #[test]
    fn removes_registered_repository_without_deleting_checkout() {
        let temp_dir = TempDir::new("pao-workspace-remove");
        let mut workspace = Workspace::init(temp_dir.path()).expect("workspace should initialize");

        workspace
            .add_repo("app", "https://example.com/app.git", "main")
            .expect("repo should be registered");

        let removed = workspace
            .remove_repo("app", true)
            .expect("repo should be removed");

        assert_eq!(removed.path, "repos/app");
        assert!(workspace.repo("app").is_err());
    }
}
