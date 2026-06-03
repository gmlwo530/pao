use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ai_client::validate_client_command;
use crate::error::{ErrorCode, PaoError};
use crate::validation::validate_name;
use crate::RuntimeEnv;

pub const CONFIG_FILE: &str = "config.yaml";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigFile {
    pub default_client: Option<String>,
    #[serde(default)]
    pub clients: BTreeMap<String, ClientConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClientConfig {
    pub command: String,
}

#[derive(Debug, Clone)]
pub struct UserConfig {
    path: PathBuf,
    pub file: ConfigFile,
}

impl UserConfig {
    pub fn load(runtime: &RuntimeEnv) -> Result<Self, PaoError> {
        let path = config_path(runtime)?;

        if !path.exists() {
            return Ok(Self {
                path,
                file: ConfigFile::default(),
            });
        }

        let content = fs::read_to_string(&path).map_err(PaoError::io)?;
        let file: ConfigFile = serde_yaml::from_str(&content).map_err(PaoError::serialization)?;

        if let Some(default_client) = &file.default_client {
            if !file.clients.contains_key(default_client) {
                return Err(PaoError::new(
                    ErrorCode::ConfigInvalid,
                    format!("default client `{default_client}` is not registered"),
                ));
            }
        }

        Ok(Self { path, file })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn add_client(&mut self, name: &str, command: &str) -> Result<(), PaoError> {
        validate_name("client", name, ErrorCode::ClientInvalid)?;
        validate_client_command(command)?;

        self.file.clients.insert(
            name.to_string(),
            ClientConfig {
                command: command.to_string(),
            },
        );

        if self.file.default_client.is_none() {
            self.file.default_client = Some(name.to_string());
        }

        self.save()
    }

    pub fn set_default(&mut self, name: &str) -> Result<(), PaoError> {
        if !self.file.clients.contains_key(name) {
            return Err(PaoError::new(
                ErrorCode::ClientMissing,
                format!("client `{name}` is not registered"),
            ));
        }

        self.file.default_client = Some(name.to_string());
        self.save()
    }

    fn save(&self) -> Result<(), PaoError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(PaoError::io)?;
        }

        let content = serde_yaml::to_string(&self.file).map_err(PaoError::serialization)?;

        fs::write(&self.path, content).map_err(PaoError::io)
    }
}

pub fn config_path(runtime: &RuntimeEnv) -> Result<PathBuf, PaoError> {
    if let Some(config_home) = &runtime.config_home {
        return Ok(config_home.join(CONFIG_FILE));
    }

    let home = runtime.home.as_ref().ok_or_else(|| {
        PaoError::new(
            ErrorCode::ConfigInvalid,
            "HOME is not set and PAO_CONFIG_HOME was not provided",
        )
    })?;

    Ok(home.join(".config").join("pao").join(CONFIG_FILE))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::UserConfig;
    use crate::test_support::TempDir;
    use crate::RuntimeEnv;

    #[test]
    fn client_add_creates_config_and_default() {
        let temp_dir = TempDir::new("pao-config-client");
        let runtime = RuntimeEnv {
            cwd: PathBuf::from("/workspace"),
            home: None,
            config_home: Some(temp_dir.path().to_path_buf()),
        };

        let mut config = UserConfig::load(&runtime).expect("config should load");
        config
            .add_client("codex", "codex")
            .expect("client should be added");

        let loaded = UserConfig::load(&runtime).expect("config should reload");

        assert_eq!(loaded.file.default_client.as_deref(), Some("codex"));
        assert_eq!(loaded.file.clients["codex"].command, "codex");
    }

    #[test]
    fn client_add_rejects_shell_control_operators() {
        let temp_dir = TempDir::new("pao-config-client-invalid");
        let runtime = RuntimeEnv {
            cwd: PathBuf::from("/workspace"),
            home: None,
            config_home: Some(temp_dir.path().to_path_buf()),
        };

        let mut config = UserConfig::load(&runtime).expect("config should load");
        let result = config.add_client("unsafe", "codex && rm -rf target");

        assert!(result.is_err());
    }
}
