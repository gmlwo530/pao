## Summary

- Add a concise summary of the change.

## Scope

- [ ] User-facing behavior
- [ ] Documentation
- [ ] Tests
- [ ] Internal refactor

## Safety

- [ ] No credentials, tokens, private URLs, or private infrastructure details are included.
- [ ] Changes respect PAO approval and repository safety boundaries.
- [ ] Public examples and fixtures do not look like real secrets.

## Version Tracking

Target milestone: `vX.Y.Z`

## Verification

- [ ] `cargo fmt --check`
- [ ] `cargo check`
- [ ] `cargo test`
- [ ] `cargo clippy --all-targets -- -D warnings`

Notes:

- Add any relevant notes or follow-up work.
