# Versioning Policy

PAO's first development line is `v0`. The first stable release is `1.0.0`.

## Rules

1. PAO uses Semantic Versioning.
2. Git tags use a `v` prefix, such as `v0.1.0`.
3. Release scope is tracked with GitHub milestones in `vX.Y.Z` format.
4. GitHub Release notes are prepared from the release milestone, merged PRs, and user-impacting change summaries.

## Version Bump Rules

PAO uses one release tracking process across `v0` and later releases. Version number bumps follow SemVer.

Expected flow:

1. `0.1.0`: first usable development release.
2. `0.2.0`: command, config, TUI, and JSON output contracts may still evolve before the stable release.
3. `0.2.1`: bug fixes, documentation fixes, and small compatible improvements.
4. `1.0.0`: first stable release for regular users.

Rules:

1. Breaking command, option, JSON output, config, or workspace schema changes require a minor bump while the major version is `0`, and a major bump after `1.0.0`.
2. Backwards-compatible features use a minor bump.
3. Bug fixes use a patch bump.
4. Documentation-only changes may use a patch bump or ship with the next planned release.
5. Security fixes use a patch, minor, or major bump based on impact.

## Version Output

Basic output:

```bash
pao --version
```

```text
pao 0.1.0
```

Detailed diagnostics should be available through `pao doctor` or a verbose version output.

```text
pao 0.1.0
git_sha: abc1234
build_date: 2026-06-01
target: aarch64-apple-darwin
```
