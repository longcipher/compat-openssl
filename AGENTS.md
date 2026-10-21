# compat-openssl Agent Instructions

## Non-negotiables

- Surgical edits only: every changed line must trace to the request. No drive-by refactors, no reformatting untouched code, no deleting pre-existing dead code (mention it instead).
- Fix root causes. Never silence a warning, weaken a check, or patch a symptom to make a gate pass.
- Never report completion while any `just` gate below fails. Report the failure verbatim and stop.
- Stop and ask when a request is ambiguous, conflicts with existing conventions, or requires credentials, publishing, or destructive operations.
- When a correction reveals a missing rule, add one specific rule here ("always X for Y", never "be careful with Y"); delete rules that never catch real errors.

## Scope

- Two shim crates keep the crates.io package identities: `crates/openssl` (package `openssl`) and `crates/openssl-sys` (package `openssl-sys`, owns `links = "openssl"`).
- Consumers install them through `[patch.crates-io]`; publishing from this repository is intentionally unsupported and `just publish` refuses.
- The real OpenSSL backend is the pinned `longcipher/rust-openssl` fork, whose packages are renamed `real-openssl` / `real-openssl-sys` with `links = "real_openssl"` so the `links` value stays unique.
- `examples/consumer-app` and every directory under `tests/fixtures/` are standalone workspace roots, excluded from the root workspace, and must keep their own `[workspace]` and `[patch.crates-io]` tables.
- English only in code, comments, docs, and commit messages.

## Execution

- Parallelize aggressively: dispatch subagents for independent research, exploration, and implementation.
- NEVER run two `cargo` commands concurrently, and never overlap file edits with a running `cargo` command (`target/` file lock): finish edits, wait, then run.
- NEVER use `git worktree`. Edit the current branch in place.
- TDD for behavior changes: failing test first, smallest change to pass, then run the full backend matrix with `just test-all`.

## Verification loop

After each feature or bug fix, in order; any failure means the task is not done:

```bash
just format
just lint
just test
just test-all
```

`just test-all` runs the backend matrix; `just fixtures` additionally builds every downstream fixture against both backends.

## Workspace rules

- Never hand-type dependency versions. Add per-crate deps with `cargo add <crate> -p <crate>`. Shared deps go into root `[workspace.dependencies]` — numeric versions only, no features — and are referenced with `dep = { workspace = true }`.
- Sub-crates inherit `version`/`edition`/`license`/`repository` via `workspace = true`.
- There is no `rust-toolchain.toml`; CI installs nightly. Use `cargo +nightly fmt` / `cargo +nightly clippy` as the Justfile does.
- `backend-openssl` and `backend-boring` are mutually exclusive. Enabling both, or enabling neither after `--no-default-features`, must fail the build with a clear error.
- Never use `--all-features`: the two backend features cannot be enabled together, so every explicit backend command passes `--no-default-features --features backend-<name>`.
- The shim crates default to `backend-boring` for repo-local convenience only. Backend selection belongs at the consumer root, where the patched `openssl` and `openssl-sys` deps carry `default-features = false`.
- Version mirror policy: the shim packages mirror the upstream pair they replace (`openssl` 0.10.x / `openssl-sys` 0.9.x). Exact pins like `openssl = "=0.10.76"` only resolve when the shim version matches exactly; older pins need their own branch or tag.
- Workspace lints deny `unwrap_used`, `expect_used`, `panic`, `allow_attributes`, `todo`, `unimplemented`, `dbg_macro`, `print_stdout`, and `print_stderr`. The only escape hatch is `#[expect(lint, reason = "...")]`.
- `cargo machete` runs over the workspace; crates that are declared but intentionally unused are listed in `[workspace.metadata.cargo-machete]`.

## Dependency choices

- `openssl` and `openssl-sys` here are the shims, not the upstream crates. Never add the real upstream crates as new dependencies.
- The OpenSSL backend is the tracked `real-openssl` / `real-openssl-sys` fork pinned by git rev; the BoringSSL backend uses `boring` / `boring-sys` from crates.io.
- Errors: `thiserror` in `crates/`; `eyre` is reserved for binaries and examples.
- Logging: `tracing` only. Never `log`.
- Forbidden by default: `anyhow`, `log`, `reqwest`, `dashmap`.

## Code conventions

- `crates/openssl-sys/build.rs` owns the downstream build contract: it relays a curated whitelist of `DEP_*` keys into the `openssl` namespace and never mirrors every backend key blindly.
- The BoringSSL backend emits a documented OpenSSL-1.1.1 compatibility profile: `DEP_OPENSSL_VERSION_NUMBER` is synthesized as `1010100f`, `DEP_OPENSSL_INCLUDE` falls back to `${DEP_BORINGSSL_ROOT}/boringssl/include`, and `DEP_OPENSSL_BACKEND` is a diagnostic marker.
- A key that the shim cannot honestly uphold must be omitted, not faked. Unsupported legacy APIs fail deterministically using the upstream failure convention.
- Raw polyfills live in the sys shim only; the safe shim re-exports and adapts. Keep unsafe code isolated in `crates/openssl-sys`.
- `cargo hack` is required by `just check-feature` and is not part of the CI image; install it with `just setup`.
