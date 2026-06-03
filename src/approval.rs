use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::ai_client::AiClientRunRequest;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ApprovalAction {
    AiClientRun,
    ShellCommand,
    WorkspaceConfigChange,
    PaoOwnedDestructiveAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RiskLevel {
    ReadOnly,
    Mutating,
    Destructive,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovalPayload {
    pub action: ApprovalAction,
    pub action_name: String,
    pub target_repo: Option<String>,
    pub cwd: PathBuf,
    pub command: String,
    pub timeout_seconds: u64,
    pub expected_impact: String,
    pub risk_level: RiskLevel,
    pub affected_files: Vec<String>,
    pub existing_dirty_state: String,
    pub reason: String,
}

impl ApprovalPayload {
    pub fn requires_approval(&self) -> bool {
        matches!(
            self.risk_level,
            RiskLevel::Mutating | RiskLevel::Destructive | RiskLevel::Unknown
        )
    }
}

pub fn ai_client_approval_payload(
    request: &AiClientRunRequest,
    target_repo: Option<String>,
    expected_impact: impl Into<String>,
    existing_dirty_state: impl Into<String>,
) -> ApprovalPayload {
    ApprovalPayload {
        action: ApprovalAction::AiClientRun,
        action_name: "AI client run".to_string(),
        target_repo,
        cwd: request.cwd.clone(),
        command: format_command(&request.command.program, &request.command.args),
        timeout_seconds: duration_seconds(request.timeout),
        expected_impact: expected_impact.into(),
        risk_level: RiskLevel::Mutating,
        affected_files: Vec::new(),
        existing_dirty_state: existing_dirty_state.into(),
        reason: "AI client execution may modify files in the selected repository".to_string(),
    }
}

pub fn shell_command_risk(command: &str, cwd: &Path) -> RiskLevel {
    let lowered = command.to_ascii_lowercase();

    if contains_destructive_command(&lowered) {
        return RiskLevel::Destructive;
    }

    if contains_mutating_operator(command) {
        return RiskLevel::Mutating;
    }

    if cwd.as_os_str().is_empty() {
        return RiskLevel::Unknown;
    }

    RiskLevel::ReadOnly
}

fn format_command(program: &str, args: &[String]) -> String {
    if args.is_empty() {
        return program.to_string();
    }

    format!("{} {}", program, args.join(" "))
}

fn duration_seconds(duration: Duration) -> u64 {
    duration.as_secs()
}

fn contains_mutating_operator(command: &str) -> bool {
    command
        .split_whitespace()
        .any(|part| matches!(part, ">" | ">>"))
}

fn contains_destructive_command(command: &str) -> bool {
    command
        .split_whitespace()
        .any(|part| matches!(part, "rm" | "mv" | "reset" | "clean" | "checkout"))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use super::{ai_client_approval_payload, shell_command_risk, ApprovalAction, RiskLevel};
    use crate::ai_client::{validate_client_command, AiClientRunRequest};

    #[test]
    fn ai_client_payload_requires_mutating_approval() {
        let request = AiClientRunRequest {
            client_name: "codex".to_string(),
            command: validate_client_command("codex --model gpt-5").expect("command is valid"),
            cwd: PathBuf::from("/workspace/repos/app"),
            prompt: "implement feature".to_string(),
            timeout: Duration::from_secs(300),
        };

        let payload = ai_client_approval_payload(
            &request,
            Some("app".to_string()),
            "modify repository files",
            "dirty",
        );

        assert_eq!(payload.action, ApprovalAction::AiClientRun);
        assert_eq!(payload.risk_level, RiskLevel::Mutating);
        assert!(payload.requires_approval());
        assert_eq!(payload.timeout_seconds, 300);
    }

    #[test]
    fn classifies_destructive_shell_commands() {
        let risk = shell_command_risk("rm -rf target", std::path::Path::new("/workspace"));

        assert_eq!(risk, RiskLevel::Destructive);
    }
}
