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
pao repo list
pao repo status
pao sync
pao
pao chat --repo <name>
pao client add <name> --command <command>
pao client list
pao client set-default <name>
```

## 버전 관리

첫 개발 라인은 `v0`로 둡니다. PAO는 Semantic Versioning과 `vX.Y.Z` Git tag를 사용하고, 릴리즈 범위는 GitHub milestone으로 추적합니다.

자세한 규칙은 [docs/versioning.md](docs/versioning.md)를 따릅니다.

## 라이선스

PAO는 [MIT License](LICENSE)로 배포됩니다.

## 현재 상태

이 저장소는 초기 설계 단계입니다. 아직 설치 가능한 패키지, Rust 프로젝트 골격, 실제 CLI/TUI 구현은 없습니다.
