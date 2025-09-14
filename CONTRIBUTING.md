# Contributing to datalit

Thank you for your interest in contributing!

## How to Contribute

- Please use GitHub Issues to report bugs or request features (the templates
  will guide you).
- Pull requests are welcome. For small, obvious fixes (docs, typos, tiny
  tweaks), open a PR directly. For larger changes (new syntax/behavior), open
  an issue or discussion first so we can align early.

### PR workflow (quick version)

- Keep PRs focused and small; split large work when possible.
- CI must pass. We deny warnings and treat rustdoc warnings as errors.
- Don’t bump versions in PRs; We have our own scripts on release.

### Toolchains & MSRV

- We target stable Rust and maintain a workspace MSRV via `rust-version`.
- Avoid raising MSRV without prior discussion; CI has an MSRV job to catch regressions.

## Running checks locally

Before opening a PR, run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo test --workspace --all-targets
# Optional: inspect packaged files for the leaf crate
cargo package --no-verify -p datalit-macros-internals --allow-dirty --list
```

Notes:

- Docs/examples are part of CI. Please update/add tests and docs for
  user‑visible changes.
- README doctests run via a small non‑published doctest crate in this
  workspace; `cargo test --workspace` covers it.

## Copyright and Licensing

- By contributing, you agree that your contributions will be licensed under the same dual MIT/Apache-2.0 license as the rest of the project.
- Please do not submit code you do not have the right to license.

## Code of Conduct

- By participating, you agree to follow the [Code of Conduct](./CODE_OF_CONDUCT.md).

## Security

Please don’t report vulnerabilities in public issues. For private disclosure,
see the [Security Policy](./SECURITY.md).

## Project Scope

- This is a small, community-driven project. Maintainer time is limited, and not all requests or contributions may be accepted.

Thank you for helping make this project better!
