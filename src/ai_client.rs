use std::path::PathBuf;
use std::time::Duration;

use crate::error::{ErrorCode, PaoError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientCommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiClientRunRequest {
    pub client_name: String,
    pub command: ClientCommandSpec,
    pub cwd: PathBuf,
    pub prompt: String,
    pub timeout: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiClientRunResult {
    pub exit_status: i32,
    pub stdout_summary: String,
    pub stderr_summary: String,
    pub duration: Duration,
}

pub trait AiClientRunner {
    fn run(&self, request: &AiClientRunRequest) -> Result<AiClientRunResult, PaoError>;
}

#[derive(Debug, Clone)]
pub struct FakeAiClientRunner {
    result: AiClientRunResult,
}

impl FakeAiClientRunner {
    pub fn new(result: AiClientRunResult) -> Self {
        Self { result }
    }
}

impl AiClientRunner for FakeAiClientRunner {
    fn run(&self, _request: &AiClientRunRequest) -> Result<AiClientRunResult, PaoError> {
        Ok(self.result.clone())
    }
}

pub fn validate_client_command(command: &str) -> Result<ClientCommandSpec, PaoError> {
    let trimmed = command.trim();

    if trimmed.is_empty() {
        return Err(PaoError::new(
            ErrorCode::ClientInvalid,
            "client command cannot be empty",
        ));
    }

    if contains_shell_control_operator(trimmed) {
        return Err(PaoError::new(
            ErrorCode::ClientInvalid,
            "client command cannot include shell control operators",
        ));
    }

    let mut parts = trimmed.split_whitespace();
    let program = parts
        .next()
        .ok_or_else(|| PaoError::new(ErrorCode::ClientInvalid, "client command cannot be empty"))?;

    Ok(ClientCommandSpec {
        program: program.to_string(),
        args: parts.map(str::to_string).collect(),
    })
}

fn contains_shell_control_operator(command: &str) -> bool {
    command
        .split_whitespace()
        .any(|part| matches!(part, "|" | "||" | "&" | "&&" | ";" | ">" | ">>" | "<"))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use super::{
        validate_client_command, AiClientRunRequest, AiClientRunResult, AiClientRunner,
        FakeAiClientRunner,
    };

    #[test]
    fn validates_client_command_program_and_args() {
        let command = validate_client_command("codex --model gpt-5").expect("command is valid");

        assert_eq!(command.program, "codex");
        assert_eq!(command.args, vec!["--model", "gpt-5"]);
    }

    #[test]
    fn rejects_shell_control_operators() {
        assert!(validate_client_command("codex && rm -rf target").is_err());
        assert!(validate_client_command("codex | tee output").is_err());
    }

    #[test]
    fn fake_runner_returns_configured_result() {
        let command = validate_client_command("codex").expect("command is valid");
        let expected = AiClientRunResult {
            exit_status: 0,
            stdout_summary: "done".to_string(),
            stderr_summary: String::new(),
            duration: Duration::from_millis(25),
        };
        let runner = FakeAiClientRunner::new(expected.clone());
        let request = AiClientRunRequest {
            client_name: "codex".to_string(),
            command,
            cwd: PathBuf::from("/workspace"),
            prompt: "make a change".to_string(),
            timeout: Duration::from_secs(30),
        };

        let result = runner.run(&request).expect("fake runner should run");

        assert_eq!(result, expected);
    }
}
