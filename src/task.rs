use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{ErrorCode, PaoError};
use crate::validation::validate_name;
use crate::workspace::Workspace;

pub const TASK_FILE: &str = "task.yaml";
pub const TASK_FILE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskFile {
    pub version: u32,
    pub id: String,
    pub status: TaskStatus,
    pub created_at_unix_seconds: u64,
    pub sessions_path: String,
    pub command_log_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    Open,
}

pub fn create_task(workspace: &Workspace, task_id: &str) -> Result<TaskFile, PaoError> {
    validate_name("task", task_id, ErrorCode::TaskInvalid)?;

    let task_dir = task_dir(workspace, task_id);

    if task_dir.exists() {
        return Err(PaoError::new(
            ErrorCode::TaskExists,
            format!("task `{task_id}` already exists"),
        ));
    }

    let sessions_dir = task_dir.join("sessions");
    let command_log_dir = task_dir.join("command-log");

    fs::create_dir_all(&sessions_dir).map_err(PaoError::io)?;
    fs::create_dir_all(&command_log_dir).map_err(PaoError::io)?;

    let task_file = TaskFile {
        version: TASK_FILE_VERSION,
        id: task_id.to_string(),
        status: TaskStatus::Open,
        created_at_unix_seconds: unix_seconds()?,
        sessions_path: format!(".pao/tasks/{task_id}/sessions"),
        command_log_path: format!(".pao/tasks/{task_id}/command-log"),
    };
    let content = serde_yaml::to_string(&task_file).map_err(PaoError::serialization)?;

    fs::write(task_dir.join(TASK_FILE), content).map_err(PaoError::io)?;

    Ok(task_file)
}

fn task_dir(workspace: &Workspace, task_id: &str) -> PathBuf {
    workspace.pao_dir().join("tasks").join(task_id)
}

fn unix_seconds() -> Result<u64, PaoError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| PaoError::new(ErrorCode::Io, error.to_string()))?;

    Ok(duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::{create_task, TaskStatus};
    use crate::test_support::TempDir;
    use crate::workspace::Workspace;

    #[test]
    fn creates_task_file_and_storage_directories() {
        let temp_dir = TempDir::new("pao-task-create");
        let workspace = Workspace::init(temp_dir.path()).expect("workspace should initialize");

        let task = create_task(&workspace, "task-1").expect("task should be created");

        assert_eq!(task.id, "task-1");
        assert_eq!(task.status, TaskStatus::Open);
        assert!(temp_dir.path().join(".pao/tasks/task-1/task.yaml").exists());
        assert!(temp_dir.path().join(".pao/tasks/task-1/sessions").exists());
        assert!(temp_dir
            .path()
            .join(".pao/tasks/task-1/command-log")
            .exists());
    }

    #[test]
    fn rejects_duplicate_tasks() {
        let temp_dir = TempDir::new("pao-task-duplicate");
        let workspace = Workspace::init(temp_dir.path()).expect("workspace should initialize");

        create_task(&workspace, "task-1").expect("task should be created");
        let duplicate = create_task(&workspace, "task-1");

        assert!(duplicate.is_err());
    }
}
