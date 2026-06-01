# AGENTS.md

## Common Rules

- Commit messages, PR titles, and PR descriptions must be written in English.
- This is a public repository. Do not include private company repository names, internal URLs, private infrastructure details, or real credentials in repository files, examples, tests, or fixtures.
- When updating `README.md`, update `README.ko.md` with equivalent content.
- Documentation should describe current decisions and operating rules only. Do not preserve superseded alternatives, historical rationale, or "instead of X" context unless explicitly requested.
- This repository is the PAO (Project Agent Orchestrator) project. Prioritize user approval boundaries, repository safety, and auditability over coding speed.
- Do not store credentials, API keys, OAuth tokens, session tokens, or provider raw responses in the repository.
- Do not leave example values that look like real secrets in examples or test fixtures.

## Commit Message Rules

- Use a lowercase type prefix in the subject, followed by a colon and a concise summary.
- Allowed type prefixes include `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `perf`, `build`, `ci`, and `revert`.
- Use `feat` for user-visible functionality, `fix` for bug fixes, `docs` for documentation-only changes, and `chore` for maintenance work that does not change behavior.
- Write the commit body as thoroughly as practical. Explain what changed, why it changed, important tradeoffs, and any testing or verification performed.
- Include risk notes or follow-up work in the body when relevant.
- Do not include secrets, private repository names, internal URLs, or private infrastructure details in commit messages.

## Branch and PR Rules

- Follow `docs/versioning.md` for branch, pull request, release branch, and milestone rules.
- Do not commit directly to `main`; use a branch and pull request.
- Use draft PRs until the change is ready for review or merge.
- Use squash merge and delete merged branches.
- Keep release preparation branches limited to release preparation work.

## Project Direction

- PAO is a macOS-first terminal AI coding agent.
- The implementation language is Rust.
- The default experience is a TUI.
- PAO orchestrates local AI coding CLIs across multiple projects.
- Each AI CLI manages its own auth, account, model, and credential storage.
- Windows/Linux support, MCP server support, multi-agent team/mailbox features, automatic PR creation, and release version-set management are out of scope for v0.

## Rust Implementation Principles

- Use `clap` for CLI argument parsing.
- Prefer `ratatui` and `crossterm` for the TUI.
- Use `tokio` for async runtime.
- Use `reqwest` for HTTP calls.
- Use `serde`, `serde_json`, and `serde_yaml` for serialization.
- Errors must include stable error codes and human-readable messages.
- Do not handle user errors with panic.
- Separate input validation, business logic, and output formatting in command handlers.
- Keep local AI CLI differences behind an `AiClientRunner` boundary.
- Track AI CLI process execution, stdout/stderr, exit status, and before/after repo diffs.
- Do not store AI provider credentials under a repo checkout or workspace directory.

## Agent Safety Rules

- Read-only PAO tools may run automatically.
- PAO-owned destructive actions, workspace config changes, and direct shell commands require user approval.
- AI CLI processes run from the selected repo cwd and may perform file edits or shell commands according to that CLI's behavior.
- PAO must capture AI CLI before/after repo state and show resulting changes clearly.
- Detect and report changes outside the selected repo when possible.
- In dirty repos, show the current diff before mutation and warn about possible overlap with existing changes.
- Shell command approval must show command, cwd, timeout, and expected impact.
- Session transcripts must not store raw secrets.

## Test Rules

- For bug fixes, add or update a regression test that fails without the fix when practical.
- After the Rust project skeleton exists, run `cargo fmt --check`, `cargo check`, `cargo test`, and `cargo clippy --all-targets -- -D warnings` for non-trivial code changes.
- Add unit tests for workspace parsing, config loading, AI client runner abstraction, prompt/session serialization, and error mapping.
- Add command-level tests for CLI output, exit behavior, and stable error codes.
- Add e2e tests for core user flows such as workspace initialization, repo registration, chat session startup, approval flow, and command failure handling once the CLI/TUI skeleton exists.
- Test file edits, shell execution, path sandboxing, and approval policy separately.
- Keep AI CLI execution behind fake-process or mockable boundaries.
- Run real external API integration tests only with explicit opt-in.
- Test key TUI states such as the main screen, approval modal, and diff preview at snapshot level.
