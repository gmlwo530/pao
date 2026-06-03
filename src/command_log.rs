use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

pub const COMMAND_LOG_ENTRY_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandLogEntry {
    pub version: u32,
    pub command_id: String,
    pub command_summary: String,
    pub command_display: String,
    pub cwd: PathBuf,
    pub timeout_seconds: u64,
    pub started_at_unix_seconds: u64,
    pub duration_ms: u128,
    pub exit_status: Option<i32>,
    pub stdout_summary: String,
    pub stderr_summary: String,
    pub redaction_status: RedactionStatus,
    pub owner_repo: Option<String>,
    pub owner_session: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RedactionStatus {
    Clean,
    Redacted,
}

pub struct CommandLogInput {
    pub command_id: String,
    pub command_summary: String,
    pub command_display: String,
    pub cwd: PathBuf,
    pub timeout: Duration,
    pub started_at_unix_seconds: u64,
    pub duration: Duration,
    pub exit_status: Option<i32>,
    pub stdout_summary: String,
    pub stderr_summary: String,
    pub owner_repo: Option<String>,
    pub owner_session: Option<String>,
}

pub fn command_log_entry(input: CommandLogInput) -> CommandLogEntry {
    let command_display = redact_text(&input.command_display);
    let stdout_summary = redact_text(&input.stdout_summary);
    let stderr_summary = redact_text(&input.stderr_summary);
    let redaction_status =
        if command_display.redacted || stdout_summary.redacted || stderr_summary.redacted {
            RedactionStatus::Redacted
        } else {
            RedactionStatus::Clean
        };

    CommandLogEntry {
        version: COMMAND_LOG_ENTRY_VERSION,
        command_id: input.command_id,
        command_summary: input.command_summary,
        command_display: command_display.text,
        cwd: input.cwd,
        timeout_seconds: input.timeout.as_secs(),
        started_at_unix_seconds: input.started_at_unix_seconds,
        duration_ms: input.duration.as_millis(),
        exit_status: input.exit_status,
        stdout_summary: stdout_summary.text,
        stderr_summary: stderr_summary.text,
        redaction_status,
        owner_repo: input.owner_repo,
        owner_session: input.owner_session,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RedactedText {
    text: String,
    redacted: bool,
}

fn redact_text(input: &str) -> RedactedText {
    let mut redacted = false;
    let parts = input
        .split_whitespace()
        .map(|part| redact_part(part, &mut redacted))
        .collect::<Vec<_>>();

    RedactedText {
        text: parts.join(" "),
        redacted,
    }
}

fn redact_part(part: &str, redacted: &mut bool) -> String {
    let lowered = part.to_ascii_lowercase();

    if contains_secret_key(&lowered) {
        if let Some((key, _value)) = part.split_once('=') {
            *redacted = true;
            return format!("{key}=[redacted]");
        }

        *redacted = true;
        return "[redacted]".to_string();
    }

    part.to_string()
}

fn contains_secret_key(value: &str) -> bool {
    value.contains("api_key")
        || value.contains("apikey")
        || value.contains("token")
        || value.contains("password")
        || value.contains("secret")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use super::{command_log_entry, CommandLogInput, RedactionStatus};

    #[test]
    fn command_log_redacts_secret_like_values() {
        let entry = command_log_entry(CommandLogInput {
            command_id: "cmd-1".to_string(),
            command_summary: "run client".to_string(),
            command_display: "codex --api-key=secret-value".to_string(),
            cwd: PathBuf::from("/workspace"),
            timeout: Duration::from_secs(30),
            started_at_unix_seconds: 1,
            duration: Duration::from_millis(25),
            exit_status: Some(0),
            stdout_summary: "token=secret-value ok".to_string(),
            stderr_summary: String::new(),
            owner_repo: Some("app".to_string()),
            owner_session: Some("session-1".to_string()),
        });

        assert_eq!(entry.redaction_status, RedactionStatus::Redacted);
        assert!(entry.command_display.contains("[redacted]"));
        assert!(entry.stdout_summary.contains("[redacted]"));
    }

    #[test]
    fn command_log_keeps_clean_values() {
        let entry = command_log_entry(CommandLogInput {
            command_id: "cmd-1".to_string(),
            command_summary: "git status".to_string(),
            command_display: "git status --short".to_string(),
            cwd: PathBuf::from("/workspace"),
            timeout: Duration::from_secs(30),
            started_at_unix_seconds: 1,
            duration: Duration::from_millis(25),
            exit_status: Some(0),
            stdout_summary: "clean".to_string(),
            stderr_summary: String::new(),
            owner_repo: Some("app".to_string()),
            owner_session: None,
        });

        assert_eq!(entry.redaction_status, RedactionStatus::Clean);
    }
}
