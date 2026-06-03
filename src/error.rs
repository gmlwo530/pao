use std::fmt;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    CliUsage,
    NotImplemented,
    WorkspaceExists,
    WorkspaceMissing,
    WorkspaceInvalid,
    RepositoryInvalid,
    RepositoryExists,
    RepositoryMissing,
    RepositoryCheckoutExists,
    GitCommandFailed,
    ConfigInvalid,
    ClientInvalid,
    ClientMissing,
    TaskInvalid,
    TaskExists,
    TaskMissing,
    Io,
    Serialization,
}

impl ErrorCode {
    pub const ALL: &'static [Self] = &[
        Self::CliUsage,
        Self::NotImplemented,
        Self::WorkspaceExists,
        Self::WorkspaceMissing,
        Self::WorkspaceInvalid,
        Self::RepositoryInvalid,
        Self::RepositoryExists,
        Self::RepositoryMissing,
        Self::RepositoryCheckoutExists,
        Self::GitCommandFailed,
        Self::ConfigInvalid,
        Self::ClientInvalid,
        Self::ClientMissing,
        Self::TaskInvalid,
        Self::TaskExists,
        Self::TaskMissing,
        Self::Io,
        Self::Serialization,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::CliUsage => "PAO-0001",
            Self::NotImplemented => "PAO-0004",
            Self::WorkspaceExists => "PAO-1001",
            Self::WorkspaceMissing => "PAO-1002",
            Self::WorkspaceInvalid => "PAO-1003",
            Self::RepositoryInvalid => "PAO-1101",
            Self::RepositoryExists => "PAO-1102",
            Self::RepositoryMissing => "PAO-1103",
            Self::RepositoryCheckoutExists => "PAO-1104",
            Self::GitCommandFailed => "PAO-1105",
            Self::ConfigInvalid => "PAO-1201",
            Self::ClientInvalid => "PAO-1202",
            Self::ClientMissing => "PAO-1203",
            Self::TaskInvalid => "PAO-1301",
            Self::TaskExists => "PAO-1302",
            Self::TaskMissing => "PAO-1303",
            Self::Io => "PAO-9001",
            Self::Serialization => "PAO-9002",
        }
    }

    pub fn summary(self) -> &'static str {
        match self {
            Self::CliUsage => "CLI arguments are invalid",
            Self::NotImplemented => "The requested command is not implemented",
            Self::WorkspaceExists => "A PAO workspace already exists",
            Self::WorkspaceMissing => "A PAO workspace is required",
            Self::WorkspaceInvalid => "The PAO workspace file is invalid or unsupported",
            Self::RepositoryInvalid => "Repository input is invalid",
            Self::RepositoryExists => "The repository is already registered",
            Self::RepositoryMissing => "The repository is not registered",
            Self::RepositoryCheckoutExists => "The target checkout path already exists",
            Self::GitCommandFailed => "A git command failed",
            Self::ConfigInvalid => "The user configuration is invalid",
            Self::ClientInvalid => "AI client input is invalid",
            Self::ClientMissing => "An AI client is required",
            Self::TaskInvalid => "Task input is invalid",
            Self::TaskExists => "The task already exists",
            Self::TaskMissing => "The task does not exist",
            Self::Io => "A filesystem or process I/O operation failed",
            Self::Serialization => "A YAML or JSON serialization operation failed",
        }
    }

    pub fn exit_code(self) -> u8 {
        match self {
            Self::CliUsage
            | Self::NotImplemented
            | Self::WorkspaceExists
            | Self::WorkspaceMissing
            | Self::WorkspaceInvalid
            | Self::RepositoryInvalid
            | Self::RepositoryExists
            | Self::RepositoryMissing
            | Self::RepositoryCheckoutExists
            | Self::ConfigInvalid
            | Self::ClientInvalid
            | Self::ClientMissing
            | Self::TaskInvalid
            | Self::TaskExists
            | Self::TaskMissing => 2,
            Self::GitCommandFailed | Self::Io | Self::Serialization => 1,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug)]
pub struct PaoError {
    code: ErrorCode,
    message: String,
}

impl PaoError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn io(error: io::Error) -> Self {
        Self::new(ErrorCode::Io, error.to_string())
    }

    pub fn serialization(error: impl fmt::Display) -> Self {
        Self::new(ErrorCode::Serialization, error.to_string())
    }

    pub fn code(&self) -> ErrorCode {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn exit_code(&self) -> u8 {
        self.code.exit_code()
    }

    pub fn render(&self) -> String {
        format!("{}: {}", self.code, self.message)
    }
}

impl fmt::Display for PaoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.render())
    }
}

impl std::error::Error for PaoError {}

#[cfg(test)]
mod tests {
    use super::{ErrorCode, PaoError};

    #[test]
    fn render_includes_stable_code_and_message() {
        let error = PaoError::new(ErrorCode::WorkspaceMissing, "workspace is missing");

        assert_eq!(error.render(), "PAO-1002: workspace is missing");
        assert_eq!(error.exit_code(), 2);
    }

    #[test]
    fn catalog_codes_are_unique_and_have_summaries() {
        let mut codes = ErrorCode::ALL
            .iter()
            .map(|code| code.as_str())
            .collect::<Vec<_>>();
        codes.sort_unstable();
        codes.dedup();

        assert_eq!(codes.len(), ErrorCode::ALL.len());

        for code in ErrorCode::ALL {
            assert!(!code.summary().is_empty());
        }
    }
}
