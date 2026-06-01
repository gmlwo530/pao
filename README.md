<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/pao-emblem-dark.png">
    <img src="assets/pao-emblem.png" width="140" alt="PAO railway switch lever emblem">
  </picture>
</p>

# PAO

[한국어](README.ko.md)

PAO (Project Agent Orchestrator) is a macOS-first terminal AI coding agent for managing multiple Git repositories from one TUI.

PAO orchestrates local AI coding CLIs across multiple projects and keeps their context, progress, and changes visible in one terminal UI.

## Initial Command Shape

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

## Versioning

The first development line is `v0`. PAO uses Semantic Versioning with `vX.Y.Z` Git tags and tracks release scope with GitHub milestones.

See [docs/versioning.md](docs/versioning.md) for the versioning and release rules.

## Current Status

This repository is in the initial design stage. There is no installable package, Rust project skeleton, or working CLI/TUI implementation yet.
