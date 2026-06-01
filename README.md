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

## License

PAO is distributed under the [MIT License](LICENSE).

## Current Status

This repository has an initial Rust CLI skeleton for the v0.1.0 development release.

Implemented command surface:

```bash
pao --version
pao init
pao repo add <name> --remote <git-url> --branch <branch>
pao repo list
pao repo status
pao sync
pao client add <name> --command <command>
pao client list
pao client set-default <name>
pao doctor
```

The default `pao` TUI entrypoint and `pao chat` AI client execution path are tracked as follow-up v0 work.
