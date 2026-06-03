<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/pao-emblem-dark.png">
    <img src="assets/pao-emblem.png" width="140" alt="PAO railway switch lever emblem">
  </picture>
</p>

# PAO

[English](README.md)

PAO(Project Agent Orchestrator)는 하나의 TUI에서 여러 Git repository를 관리하기 위한 macOS 우선 터미널 AI coding agent입니다.

PAO는 여러 프로젝트에서 로컬 AI coding CLI들을 지휘하고, 각 CLI의 context, 진행 상황, 변경 사항을 하나의 터미널 UI에서 보여주는 orchestrator입니다.

## 초기 명령 형태

```bash
pao init
pao repo add <name> --remote <git-url> --branch <branch>
pao repo remove <name> --keep-checkout
pao repo list
pao repo status
pao sync
pao task create <task-id>
pao
pao chat --repo <name> --prompt <prompt>
pao client add <name> --command <command>
pao client list
pao client set-default <name>
```

## 설치

v0.1.0 개발 빌드는 로컬 checkout에서 설치합니다.

```bash
cargo install --path .
```

## 기본 사용

```bash
pao init
pao repo add app --remote <git-url> --branch main
pao client add codex --command codex
pao chat --repo app --prompt "작은 변경을 만들어줘"
pao doctor
```

`pao chat`은 AI client 실행 전에 `.pao/sessions/<session-id>/` 아래에 baseline과 approval artifact를 준비합니다.

## 검증

의미 있는 PR을 열거나 갱신하기 전에 다음 명령을 실행합니다.

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
```

## 에러 코드

PAO는 안정적인 에러 코드와 사람이 읽을 수 있는 메시지를 함께 출력합니다. 자세한 목록은 [docs/error-codes.md](docs/error-codes.md)를 봅니다.

## 버전 관리

첫 개발 라인은 `v0`로 둡니다. PAO는 Semantic Versioning과 `vX.Y.Z` Git tag를 사용하고, 릴리즈 범위는 GitHub milestone으로 추적합니다.

자세한 규칙은 [docs/versioning.md](docs/versioning.md)를 따릅니다.

## 라이선스

PAO는 [MIT License](LICENSE)로 배포됩니다.

## 현재 상태

이 저장소에는 v0.1.0 개발 릴리즈를 위한 초기 Rust CLI 골격이 있습니다.

구현된 명령 표면:

```bash
pao --version
pao init
pao repo add <name> --remote <git-url> --branch <branch>
pao repo remove <name> --keep-checkout
pao repo list
pao repo status
pao sync
pao task create <task-id>
pao chat --repo <name> --prompt <prompt>
pao client add <name> --command <command>
pao client list
pao client set-default <name>
pao doctor
```

기본 `pao` TUI 진입점과 `pao chat` AI client 실행 경로는 후속 v0 작업으로 추적합니다.
