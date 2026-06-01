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

## Branch and PR Flow

1. `main` is the release-ready branch.
2. All changes go through a branch and pull request.
3. Regular work branches use `feature/<short-name>`, `fix/<short-name>`, `docs/<short-name>`, or `chore/<short-name>`.
4. Release preparation branches use `release/vX.Y.Z`.
5. Pull requests start as draft until they are ready for review or merge.
6. Pull requests use squash merge.
7. Merged branches are deleted.
8. Release scope is tracked with the matching `vX.Y.Z` milestone.
9. Release tags are created from the `main` commit that contains the merged release preparation PR.
10. Long-lived `release/vX.Y` maintenance branches are created only when an older minor line needs patch support.

Regular work branches always use `main` as the base branch. Release preparation branches also start from `main`.

`release/vX.Y.Z` is a short-lived release preparation branch. It is not a feature integration branch.

Allowed release preparation changes:

1. Version bump.
2. Lockfile update.
3. Release notes or GitHub Release draft text.
4. Packaging, workflow, or installation documentation finalization.
5. Release verification notes.

Do not use a release preparation branch for new features, general bug fixes, broad refactors, or unrelated documentation cleanup.

If a product bug is found during release preparation, fix it through a regular branch targeting `main`, then update the release preparation branch from the latest `main`.

## Milestone Flow

1. Create the `vX.Y.Z` milestone when release planning starts.
2. Assign every Issue and PR planned for that release to the milestone.
3. Regular feature, fix, documentation, and chore PRs still target `main`.
4. The release preparation PR also uses the same milestone.
5. Before merging the release preparation PR, the milestone must have no open Issue or PR except the release preparation PR itself.
6. After tagging and publishing the GitHub Release, close the milestone.

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
