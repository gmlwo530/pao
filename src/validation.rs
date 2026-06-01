use std::path::Path;

use crate::error::{ErrorCode, PaoError};

pub fn validate_name(kind: &str, name: &str, code: ErrorCode) -> Result<(), PaoError> {
    if name.is_empty() {
        return Err(PaoError::new(code, format!("{kind} name cannot be empty")));
    }

    let valid = name
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'));

    if valid && name != "." && name != ".." && !name.starts_with('.') {
        return Ok(());
    }

    Err(PaoError::new(
        code,
        format!("{kind} name must use letters, numbers, dots, dashes, or underscores"),
    ))
}

pub fn validate_relative_path(path: &str) -> Result<(), PaoError> {
    let path = Path::new(path);

    if path.is_absolute() {
        return Err(PaoError::new(
            ErrorCode::WorkspaceInvalid,
            "workspace paths must be relative",
        ));
    }

    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(PaoError::new(
                ErrorCode::WorkspaceInvalid,
                "workspace paths cannot contain parent directory segments",
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_name;
    use crate::error::ErrorCode;

    #[test]
    fn validates_safe_names() {
        assert!(validate_name("repo", "api-service_1", ErrorCode::RepositoryInvalid).is_ok());
    }

    #[test]
    fn rejects_names_that_can_escape_paths() {
        assert!(validate_name("repo", "../api", ErrorCode::RepositoryInvalid).is_err());
        assert!(validate_name("repo", ".hidden", ErrorCode::RepositoryInvalid).is_err());
    }
}
