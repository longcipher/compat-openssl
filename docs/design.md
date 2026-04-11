# Design: Switchable OpenSSL / BoringSSL Compatibility Layer

| Metadata | Details |
| :--- | :--- |
| Status | Implementation Ready |
| Created | 2026-04-11 |
| Repository | compat-openssl |
| Reference Fork | <https://github.com/longcipher/rust-openssl> branch compat-openssl |
| Scope | Workspace architecture, dependency replacement, API compatibility, and verification strategy |

## 1. Summary

This repository will provide a path- or git-patched compatibility workspace that can replace both `openssl` and `openssl-sys` for downstream Cargo consumers while allowing the application root to choose either an OpenSSL backend or a BoringSSL backend.

The design uses three layers:

1. A real backend layer containing the actual OpenSSL and BoringSSL bindings.
2. An `openssl-sys` shim crate that owns `links = "openssl"` and relays backend build metadata back to downstream crates as `DEP_OPENSSL_*`.
3. An `openssl` shim crate that presents the safe Rust API surface and applies backend-specific polyfills.

The repository is currently still a scaffold. `bin/` and `crates/` are empty, `features/checkout.feature` is a template leftover, and the current `Justfile` assumes `cargo test --all-features`, which is incompatible with mutually exclusive SSL backend features. This design therefore includes both the compatibility architecture and the repository normalization needed to make the workspace implementable. Backend selection for purely transitive consumers is handled by a direct root dependency on patched `openssl`, not by adding another selector crate.

## 2. Problem Statement

The compatibility layer must solve four separate problems at the same time:

1. Cargo allows only one package with `links = "openssl"` in the full dependency graph.
2. Downstream crates such as `native-tls` may rely on `DEP_OPENSSL_*` build metadata emitted by `openssl-sys`.
3. Some crates depend on the safe `openssl` API, while others depend directly on `openssl-sys`.
4. Cargo features are additive, so any default backend on the patched crates means explicit backend-selection commands must use `--no-default-features` once patched crates appear transitively.

The compatibility layer must therefore preserve package identity, avoid `links` collisions, relay build metadata, and expose a coherent API surface under one globally selected backend.

## 3. Goals

- Provide a drop-in replacement path for crates.io-sourced `openssl` and `openssl-sys` through `[patch.crates-io]`.
- Support exactly one backend per build: OpenSSL or BoringSSL.
- Preserve the `DEP_OPENSSL_*` contract for direct downstream build scripts.
- Support both safe API consumers (`openssl`) and raw binding consumers (`openssl-sys`).
- Make backend selection explicit and deterministic at the application root.
- Keep unsafe compatibility work isolated to the smallest possible surface.
- Verify the contract with real downstream fixture builds, not only unit tests.

## 4. Non-Goals

- Publishing these replacement crates to crates.io. The official package names are already taken; distribution is via path patch, git patch, or an internal registry only.
- Supporting arbitrary non-crates.io OpenSSL dependencies. A dependency that uses its own git or path source for `openssl` or `openssl-sys` is outside the patch mechanism.
- Claiming universal compatibility with every historical or niche OpenSSL API on day one.
- Silently degrading or re-enabling insecure legacy crypto behavior that BoringSSL intentionally removed.
- Preserving the current template-specific `checkout/common` naming and test flow.

## 5. Key Constraints

### 5.1 Cargo Constraints

- `links` values are globally unique. The shim crate must own `links = "openssl"`, so the real OpenSSL backend cannot keep that same `links` value.
- Cargo features are additive across the graph. This repository defaults the shim crates to `backend-boring` for local `cargo build`, so explicit backend matrix commands must use `--no-default-features`, and downstream roots that need deterministic switching should keep `default-features = false` on their patched dependencies.
- `cargo test --all-features`, `cargo check --all-features`, and similar template commands are invalid once `backend-openssl` and `backend-boring` are mutually exclusive.
- Build metadata from `links` crates is only visible to direct dependents, so the shim `build.rs` must explicitly relay selected backend metadata under the `openssl` namespace.

### 5.2 Repository Constraints

- Workspace members live under `bin/*` and `crates/*`.
- The current `Justfile` targets a nonexistent `common` crate and must be replaced with backend-aware commands.
- The root `features/checkout.feature` is unrelated to this repository and must be replaced with compatibility scenarios.
- `tests/fixtures/*` must remain outside the root workspace member set so they behave like independent downstream consumers.
- Documentation, comments, and commit messages are required to stay in English.

### 5.3 Safety Constraints

- Unsupported legacy APIs may only be polyfilled in ways that fail deterministically.
- No compatibility shim may silently claim support for a backend behavior that does not actually exist.
- Unsafe wrappers must document their invariants and stay isolated in the sys compatibility layer.

## 6. Alternatives Considered

### Option A: Fork `openssl` only

Replace the safe Rust layer and leave `openssl-sys` untouched.

Rejected because it does not solve `links = "openssl"` conflicts and does not preserve `DEP_OPENSSL_*` metadata for build scripts.

### Option B: Ask downstream crates to import new package names

Publish or vend a separate `compat-openssl` package and update every consumer.

Rejected because it defeats the purpose of transparent replacement and requires invasive changes throughout the downstream dependency graph.

### Option C: Dual-layer shim with tracked real backend and metadata relay

Patch both `openssl` and `openssl-sys`, keep package identity intact, and isolate backend switching behind the shim crates.

Selected because it is the only option that addresses package identity, `links` uniqueness, metadata propagation, and safe/raw API replacement together.

## 7. Selected Architecture

### 7.1 Real Backend Layer

This layer contains the actual backend implementations:

- `boring` and `boring-sys` from crates.io.

- A tracked fork of `rust-openssl` where package identity changes prevent `links` collisions:
  - `openssl-sys` becomes package `real-openssl-sys` with `links = "real_openssl"`.
  - `openssl` becomes package `real-openssl` and depends on `real-openssl-sys`.
    - The current reference branch already exposes `real-openssl-sys` version `0.9.112` and `real-openssl` version `0.10.76`.

This layer does not attempt to preserve public package identity. Its only responsibility is to provide the real implementation behind the shim. The fork is maintained as an actual Git repository and consumed via pinned `git` + `rev` metadata, or mirrored locally as a Git submodule if the repository later needs a checked-in copy. Raw copied vendor snapshots are explicitly out of scope.

### 7.2 `openssl-sys` Shim Layer

This crate keeps the official package identity and owns `links = "openssl"`.

Responsibilities:

- Select exactly one backend sys crate.
- Re-export the raw ABI-facing surface expected from `openssl-sys`, including functions, constants, aliases, opaque handles, and exposed C types.
- Relay backend build metadata back to downstream build scripts as `DEP_OPENSSL_*`.
- Add backend-specific raw polyfills where BoringSSL removes symbols that downstream crates still compile against.

### 7.3 `openssl` Shim Layer

This crate keeps the official `openssl` package identity.

Responsibilities:

- Select exactly one safe backend crate.
- Re-export the safe Rust API surface expected from `openssl`.
- Apply adapters and polyfills for safe APIs missing under BoringSSL.
- Forward backend selection to the `openssl-sys` shim.

### 7.4 No Selector Crate

No dedicated backend-selector crate will be introduced. For applications that only depend on OpenSSL transitively, the root package should add a direct dependency on patched `openssl` solely to activate `backend-openssl` or `backend-boring`. Cargo feature propagation already solves this problem, while an extra selector crate would only add maintenance surface and one more place for version skew.

## 8. Proposed Workspace Layout

```text
compat-openssl/
├── crates/
│   ├── openssl/
│   └── openssl-sys/
├── docs/
│   └── design.md
├── examples/
│   └── consumer-app/
├── features/
│   ├── backend-selection.feature
│   ├── metadata-proxy.feature
│   └── downstream-compat.feature
└── tests/
│   └── fixtures/
│       ├── downstream-build-metadata/
│       ├── downstream-native-tls/
│       ├── downstream-tokio-native-tls/
│       ├── downstream-reqwest-native-tls/
│       ├── downstream-postgres-native-tls/
│       ├── downstream-openssl-sys/
│       └── downstream-openssl-exact-pin/
```

Notes:

- The forked `rust-openssl` lives in a separate Git repository referenced by pinned `rev`; if it is ever mirrored locally, it must be added as a Git submodule rather than copied into a `vendor/` directory.
- `crates/openssl` and `crates/openssl-sys` use the official package names because they are the patch targets.
- `tests/fixtures/*` are standalone consumer packages used to verify real-world patch behavior. They are excluded from the root workspace and carry their own Cargo manifests and patch configuration.
- `examples/consumer-app` is also a standalone root package. It exists as checked-in consumer documentation and is compiled separately from the workspace matrix.
- Future optional tooling such as `bin/api-surface-sync` is intentionally excluded from the initial layout and only becomes relevant if manual compatibility-gap management proves too expensive.

## 9. Detailed Design

### 9.1 Tracked Real OpenSSL Fork

The tracked fork is the canonical OpenSSL API source of truth.

Required fork changes:

1. Rename package `openssl-sys` to `real-openssl-sys`.
2. Change `links = "openssl"` to `links = "real_openssl"`.
3. Rename package `openssl` to `real-openssl`.
4. Update the forked `openssl` crate to depend on `real-openssl-sys`.
5. Keep the fork pinned to a known Git revision and upgrade it intentionally, not automatically.
6. Consume the fork from a dedicated Git repository via `git = "..."` and `rev = "..."`, or via a Git submodule if a local mirror is required.

The fork must remain thin. It is not the place for compatibility logic. All compatibility behavior belongs in the shim crates. Preserving Git history is part of the design because upstream `rust-openssl` security fixes must remain mergeable.

### 9.1.1 Current Fork Reference and Consumption

The current reference fork already exists at `https://github.com/longcipher/rust-openssl` on branch `compat-openssl`.

At the time of writing, that branch exposes:

- `real-openssl-sys` version `0.9.112` with `links = "real_openssl"`.
- `real-openssl` version `0.10.76` depending on `real-openssl-sys`.

Representative dependency wiring for the shim crates during active development:

```toml
# crates/openssl-sys/Cargo.toml
[dependencies]
real-openssl-sys = { package = "real-openssl-sys", git = "https://github.com/longcipher/rust-openssl", branch = "compat-openssl", version = "0.9.112", optional = true }

# crates/openssl/Cargo.toml
[dependencies]
real-openssl = { package = "real-openssl", git = "https://github.com/longcipher/rust-openssl", branch = "compat-openssl", version = "0.10.76", optional = true }
```

Branch-based references are acceptable while the compatibility line is still moving. Once the first fully green fixture matrix exists, the workspace should switch from `branch = "compat-openssl"` to an exact `rev = "..."` so CI and downstream reproduction stop depending on a floating branch head.

### 9.2 Backend Feature Model

The backend model must avoid Cargo feature union traps while still keeping repo-local verification ergonomic.

Design rules:

- `crates/openssl/Cargo.toml` and `crates/openssl-sys/Cargo.toml` use `default = ["backend-boring"]` so plain `cargo build` in this repository exercises the BoringSSL path.
- Both crates define `backend-openssl` and `backend-boring` features.
- Both crates fail the build when both backend features are enabled.
- Both crates fail the build when no backend feature is enabled after defaults are disabled.
- Commands that explicitly select `backend-openssl` or `backend-boring` in this repository must pass `--no-default-features`.
- The application root should still enable one backend explicitly by directly depending on patched `openssl`, even if that dependency exists only to activate features for transitive consumers.

Representative feature wiring:

```toml
[features]
default = ["backend-boring"]
backend-openssl = ["dep:real-openssl", "openssl-sys/backend-openssl"]
backend-boring = ["dep:boring", "openssl-sys/backend-boring"]
```

And for the sys shim:

```toml
[features]
default = ["backend-boring"]
backend-openssl = ["dep:real-openssl-sys"]
backend-boring = ["dep:boring-sys"]
```

This default is a repo-local development convenience, not a recommendation that downstream consumers rely on transitive defaults. Consumers that need deterministic switching should continue to disable defaults on their direct patched dependencies and select a backend from the root package.

### 9.2.1 Root Activation Pattern

The root application must activate the backend explicitly, even when it only consumes OpenSSL transitively.

Representative root manifest shape:

```toml
[features]
default = ["tls-boring"]
tls-openssl = ["openssl/backend-openssl"]
tls-boring = ["openssl/backend-boring"]

[dependencies]
openssl = { version = "0.10.76", default-features = false }
```

`default-features = false` is critical here when the root wants to control backend selection explicitly. `[patch.crates-io]` does not bypass feature union rules, so leaving dependency defaults enabled opts the consumer into the shim crates' `backend-boring` fallback.

### 9.3 `openssl-sys` Build Metadata Proxy

The sys shim owns the downstream build contract. For the OpenSSL backend it mostly relays metadata. For the BoringSSL backend it acts as a translation layer that emits a documented OpenSSL compatibility profile rather than blindly rewriting environment-variable prefixes.

Behavior:

1. The chosen backend sys crate runs its own build script as usual.
2. Cargo exposes its metadata to the shim build script as `DEP_BORINGSSL_*` or `DEP_REAL_OPENSSL_*`.
3. The shim maps backend metadata into an `openssl` compatibility profile.
4. Under the OpenSSL backend, most compatibility keys are forwarded from `DEP_REAL_OPENSSL_*`.
5. Under the BoringSSL backend, required keys are synthesized where downstream crates expect OpenSSL semantics, especially `DEP_OPENSSL_VERSION_NUMBER`, and include discovery falls back to `DEP_BORINGSSL_ROOT` when `DEP_BORINGSSL_INCLUDE` is absent.
6. Cargo then exposes the resulting values to downstream crates as `DEP_OPENSSL_*`.

Important rules:

- Relay only a curated whitelist of keys that are part of the compatibility contract.
- Do not blindly mirror every backend-specific metadata key.
- Treat BoringSSL metadata as a versioned compatibility profile. The initial baseline is OpenSSL 1.1.1 compatibility, represented by `version_number = 1010100f`.
- Any synthesized key such as `version_number` or `conf` is an explicit downstream promise and must be covered by fixture builds.
- Do not emit a key whose semantics the raw and safe compatibility layers cannot actually uphold.
- Let the real backend crate consume discovery environment variables such as include/library locations. The shim only selects a backend and relays output metadata.
- Add one extra internal metadata key such as `backend=openssl` or `backend=boring` for fixture tests and diagnostics.

Representative build script shape:

```rust
use std::{env, path::PathBuf};

const BORINGSSL_INCLUDE_ENV: &str = "DEP_BORINGSSL_INCLUDE";
const BORINGSSL_ROOT_ENV: &str = "DEP_BORINGSSL_ROOT";

fn main() {
	match (
		env::var_os("CARGO_FEATURE_BACKEND_OPENSSL"),
		env::var_os("CARGO_FEATURE_BACKEND_BORING"),
	) {
		(Some(_), None) => relay_real_openssl(),
		(None, Some(_)) => relay_boring_as_ossl111(),
		(Some(_), Some(_)) => panic!("backend-openssl and backend-boring are mutually exclusive"),
		(None, None) => panic!("one SSL backend must be selected"),
	}
}

fn relay_real_openssl() {
	forward_if_present("DEP_REAL_OPENSSL_INCLUDE", "include");
	forward_if_present("DEP_REAL_OPENSSL_VERSION_NUMBER", "version_number");
	forward_if_present("DEP_REAL_OPENSSL_LIBRESSL", "libressl");
	forward_if_present(
		"DEP_REAL_OPENSSL_LIBRESSL_VERSION_NUMBER",
		"libressl_version_number",
	);
	forward_if_present("DEP_REAL_OPENSSL_CONF", "conf");
	println!("cargo:backend=openssl");
}

fn relay_boring_as_ossl111() {
	if !forward_if_present(BORINGSSL_INCLUDE_ENV, "include") {
		forward_boring_include_from_root();
	}
	println!("cargo:version_number=1010100f");
	println!("cargo:backend=boring");
}

fn forward_boring_include_from_root() {
	if let Some(include_dir) = boring_include_from_root() {
		println!("cargo:include={}", include_dir.to_string_lossy());
	}
}

fn boring_include_from_root() -> Option<PathBuf> {
	let root = env::var_os(BORINGSSL_ROOT_ENV)?;
	let include_dir = PathBuf::from(root).join("boringssl").join("include");
	include_dir.is_dir().then_some(include_dir)
}

fn forward_if_present(env_key: &str, cargo_key: &str) -> bool {
	if let Some(value) = env::var_os(env_key) {
		println!("cargo:{cargo_key}={}", value.to_string_lossy());
		return true;
	}

	false
}
```

The exact synthesized BoringSSL profile can evolve, but it must be deliberate and documented. The compatibility layer is not just a forwarder; it is an opinionated contract that tells downstream crates which OpenSSL feature baseline they may assume.

### 9.3.1 Initial Compatibility Metadata Table

The initial compatibility profile should be treated as an explicit table, not an implicit side effect of whatever the backend crate happens to emit.

| Downstream key | OpenSSL backend source | BoringSSL backend source | Notes |
| :--- | :--- | :--- | :--- |
| `DEP_OPENSSL_INCLUDE` | Forward from `DEP_REAL_OPENSSL_INCLUDE` | Forward from `DEP_BORINGSSL_INCLUDE` if present, otherwise derive `${DEP_BORINGSSL_ROOT}/boringssl/include` | Required for downstream header consumers |
| `DEP_OPENSSL_VERSION_NUMBER` | Forward from `DEP_REAL_OPENSSL_VERSION_NUMBER` | Synthesize `1010100f` | Declares the initial OpenSSL 1.1.1 compatibility baseline |
| `DEP_OPENSSL_LIBRESSL` | Forward if present | Omit by default | Do not invent LibreSSL identity on BoringSSL |
| `DEP_OPENSSL_LIBRESSL_VERSION_NUMBER` | Forward if present | Omit by default | Same rationale as above |
| `DEP_OPENSSL_CONF` | Forward if present | Omit unless a fixture proves it is required and semantics are understood | `conf` is not a free-form dump key |
| `DEP_OPENSSL_BACKEND` | Synthesize `openssl` | Synthesize `boring` | Internal diagnostic key used by fixtures |

Phase 2 confirmed that the chosen `boring-sys` revision emits `DEP_BORINGSSL_*` keys because it uses `links = "boringssl"`. If a backend key is absent, the shim must not emit a matching `DEP_OPENSSL_*` key unless the compatibility profile explicitly defines a synthesized value and the fixture matrix proves the contract is sound.

### 9.4 `openssl-sys` Raw API Surface

Simple `pub use boring_sys::*;` is not sufficient for full compatibility. Some downstream crates depend directly on the public Rust items of `openssl-sys`, not only the linked native libraries.

The sys shim therefore starts with two layers:

1. Direct backend re-exports for functions, constants, aliases, structs, unions, and opaque handles whose Rust surface already matches.
2. Manual polyfills for items that do not exist in BoringSSL or need signature-level adaptation.

Policy for raw polyfills:

- Constants and type aliases may be forwarded directly when semantically equivalent.
- Missing functions may be implemented as Rust-defined compatibility functions with matching signatures when callers only require the Rust-level symbol, not an exact foreign export from the native library.
- Unsupported legacy operations must fail deterministically using the native failure convention for that API, such as returning `0`, `false`, or a null pointer where the upstream API already models failure that way.
- If an item cannot be represented safely or honestly, it is explicitly unsupported and documented as such.

### 9.5 `openssl` Safe API Surface

The safe shim follows a compiler-driven pattern:

1. Re-export the OpenSSL backend directly when `backend-openssl` is selected.
2. Re-export the BoringSSL backend where APIs match.
3. Build target fixtures and let compiler errors identify the missing APIs that matter.
4. Overlay manual adapters and polyfills only for those missing BoringSSL APIs.

Polyfill tiers:

- Tier 1: exact re-export where both backends already match.
- Tier 2: adapter wrapper where signatures can be preserved with a translation layer.
- Tier 3: deterministic failure for removed or intentionally unsupported legacy APIs.

The compatibility layer must never silently downgrade security semantics. If BoringSSL removed an unsafe legacy feature, the compatibility layer may preserve compile-time shape but must fail predictably at runtime or return an error result where the API contract allows it.

### 9.6 Compiler-Driven Gap Closure

The initial implementation deliberately avoids an AST-based code generator. A robust Rust AST diff and scaffolding tool is expensive to build and is not on the critical path to proving the compatibility design.

The implementation loop is:

1. Start with direct re-exports from `real-openssl` and `boring`.
2. Build the downstream fixture set.
3. Let compiler errors and failing fixtures identify missing APIs and metadata gaps.
4. Add the smallest manual polyfill or adapter needed to satisfy those target consumers.
5. Repeat until the targeted fixture matrix passes.

An AST-driven helper such as `bin/api-surface-sync` is deferred to a later phase and should only be introduced if manual gap closure becomes the primary maintenance bottleneck.

### 9.7 Consumer Integration Contract

The downstream application uses `[patch.crates-io]` to redirect crates.io package resolution:

```toml
[patch.crates-io]
openssl = { path = "../compat-openssl/crates/openssl" }
openssl-sys = { path = "../compat-openssl/crates/openssl-sys" }
```

If the application already depends on `openssl`, it should select the backend there:

```toml
[features]
default = ["ssl-backend-openssl"]
ssl-backend-openssl = ["openssl/backend-openssl"]
ssl-backend-boring = ["openssl/backend-boring"]

[dependencies]
openssl = { version = "0.10.76", default-features = false }
```

If the application only depends on OpenSSL transitively, it should still add a direct dependency on patched `openssl` solely to choose the backend feature. That dependency exists to drive Cargo feature propagation across the graph even if no application code imports `openssl` directly.

The design intentionally avoids backend selection by environment variable. Backend choice must be explicit in Cargo features so lockfiles, CI, and reproducible builds stay deterministic.

### 9.7.1 Patch Resolution Rules

Cargo's `[patch.crates-io]` mechanism does not ignore normal version resolution. It adds the patched package version to the source and still resolves the highest compatible version that satisfies the dependency requirement.

Implications for this repository:

- `openssl = "^0.10"` can resolve through a shim at `0.10.76`.
- `openssl = "=0.10.76"` can resolve through a shim at `0.10.76`.
- `openssl = "=0.10.64"` cannot resolve through a shim at `0.10.76`.
- The same rule applies independently to `openssl-sys`, currently mirrored as `0.9.112` on the reference fork.

The compatibility line therefore has to mirror the upstream version pair it claims to replace. Supporting multiple exact pins requires separate maintained tags, branches, or release lines rather than one universal shim version.

### 9.8 Shim Version Policy and Patch SemVer

The shim packages still participate in normal Cargo dependency resolution before patch replacement, so their package versions must satisfy downstream version requirements.

Rules:

- `crates/openssl` and `crates/openssl-sys` should mirror the official upstream version pair they are currently targeting.
- One branch or checkout exposes one version pair at a time.
- Dependencies written as `^0.10` can resolve through the shim as long as the shim stays on that compatible line.
- Dependencies written as `=x.y.z` require the shim package version to match that exact upstream version.
- Supporting multiple older exact pins requires separate maintenance branches or tags, not one universal shim version.

The fixture suite must include at least one exact-pin consumer matching the currently targeted upstream version pair so the patch behavior is verified under strict resolution as well as caret-range resolution.

### 9.9 Template Cleanup Required by This Repository

The current scaffold must be normalized before meaningful implementation work begins.

Required cleanup:

- Replace `features/checkout.feature` with compatibility-related scenarios.
- Replace the `bdd` recipe targeting `common` with a backend-aware BDD target.
- Replace `test`, `check`, `build`, and `lint` recipes that currently use `--all-features` with sequential backend matrix commands.
- Revisit publish-oriented recipes because crates named `openssl` and `openssl-sys` cannot be published to crates.io from this repository.

## 10. Verification Strategy

### 10.1 Command Matrix

Because backend features are mutually exclusive, verification must run sequentially per backend.

Recommended command shape:

```just
check-openssl:
	cargo check --workspace --no-default-features --features backend-openssl

check-boring:
	cargo check --workspace --no-default-features --features backend-boring

test-openssl:
	cargo test --workspace --no-default-features --features backend-openssl

test-boring:
	cargo test --workspace --no-default-features --features backend-boring

bdd-openssl:
	cargo test -p openssl --test bdd --no-default-features --features backend-openssl,internal-bdd

bdd-boring:
	cargo test -p openssl --test bdd --no-default-features --features backend-boring,internal-bdd

check:
	just check-openssl
	just check-boring

test:
	just test-openssl
	just test-boring

bdd:
	just bdd-openssl
	just bdd-boring

example-openssl:
	cargo build --manifest-path examples/consumer-app/Cargo.toml --no-default-features --features backend-openssl

example-boring:
	cargo build --manifest-path examples/consumer-app/Cargo.toml --no-default-features --features backend-boring

examples:
	just example-openssl
	just example-boring
```

Cargo commands must run sequentially, never in parallel, because the workspace shares a `target/` directory.

A plain `cargo build --workspace` continues to validate the default BoringSSL path.

The BDD runner lives at `crates/openssl/tests/bdd.rs` and should stay behind an internal-only feature gate so ordinary `cargo test -p openssl` does not execute cucumber unless the caller explicitly requests it.

`cargo-hack` is a useful supplemental tool for checking the positive feature powerset without relying on `--all-features`, for example:

```bash
cargo hack check --workspace --feature-powerset --mutually-exclusive-features backend-openssl,backend-boring
```

It does not replace explicit negative tests for the `both backends enabled` and `no backend enabled` failure modes, and it does not replace fixture builds.

### 10.2 Fixture Builds

Real compatibility must be proven with standalone consumer fixtures:

These fixtures are not workspace members. The root workspace should explicitly exclude `tests/fixtures/*`, and each fixture should carry its own Cargo manifest and workspace boundary so `[patch.crates-io]` is evaluated from the fixture itself rather than inherited accidentally from the repository root.

Representative root workspace change:

```toml
[workspace]
members = ["crates/*"]
exclude = [".github/", "tests/fixtures/*"]
resolver = "3"
```

Representative fixture manifest shape:

```toml
[package]
name = "downstream-build-metadata"
version = "0.1.0"
edition = "2024"

[workspace]

[features]
default = []
backend-openssl = ["openssl/backend-openssl", "openssl-sys/backend-openssl"]
backend-boring = ["openssl/backend-boring", "openssl-sys/backend-boring"]

[dependencies]
openssl = { version = "0.10.76", default-features = false }
openssl-sys = { version = "0.9.112", default-features = false }

[patch.crates-io]
openssl = { path = "../../../crates/openssl" }
openssl-sys = { path = "../../../crates/openssl-sys" }
```

The metadata fixture must keep a direct dependency on `openssl-sys`, not only `openssl`, because Cargo only exposes `DEP_OPENSSL_*` keys to the build scripts of immediate dependents of the linked sys crate. The safe-layer `openssl` dependency keeps the patch graph realistic, while the direct `openssl-sys` edge is what makes the metadata contract observable inside the fixture build script.

The empty `[workspace]` table makes the fixture its own workspace root and stops Cargo from walking upward into the repository workspace.

- `tests/fixtures/downstream-build-metadata`
  - contains a `build.rs` that asserts required `DEP_OPENSSL_*` variables are present.
- `tests/fixtures/downstream-native-tls`
  - depends on `native-tls` or a similar transitive OpenSSL consumer and proves end-to-end resolution.
- `tests/fixtures/downstream-tokio-native-tls`
  - depends on `tokio-native-tls` to verify the async wrapper over `native-tls`.
- `tests/fixtures/downstream-reqwest-native-tls`
  - depends on `reqwest` with `native-tls` enabled to verify a real HTTP client path.
- `tests/fixtures/downstream-postgres-native-tls`
  - depends on `postgres-native-tls` to verify a database adapter over the transitive TLS path.
- `tests/fixtures/downstream-openssl-sys`
  - depends directly on `openssl-sys` and compiles against the raw API surface.
- `tests/fixtures/downstream-openssl-exact-pin`
  - depends on the exact upstream version currently mirrored by the shim, such as `openssl = "=0.10.76"`, to verify strict-version patch resolution.
- `examples/consumer-app`
  - demonstrates direct `openssl`, direct `openssl-sys`, build-script metadata usage, and verified transitive adapters in a single standalone example crate.

Each fixture is built twice, once per backend, using `[patch.crates-io]` back to this workspace. Exact-pin coverage for older upstream versions is handled by matching maintenance branches or tags, not by a single branch pretending to satisfy multiple exact versions at once.

Representative build invocation from the repository root:

```bash
cargo build --manifest-path tests/fixtures/downstream-build-metadata/Cargo.toml --no-default-features --features backend-openssl
cargo build --manifest-path tests/fixtures/downstream-build-metadata/Cargo.toml --no-default-features --features backend-boring

cargo build --manifest-path tests/fixtures/downstream-native-tls/Cargo.toml --no-default-features --features backend-openssl
cargo build --manifest-path tests/fixtures/downstream-native-tls/Cargo.toml --no-default-features --features backend-boring

cargo build --manifest-path examples/consumer-app/Cargo.toml --no-default-features --features backend-openssl
cargo build --manifest-path examples/consumer-app/Cargo.toml --no-default-features --features backend-boring
```

### 10.3 BDD Scenario Inventory

The existing checkout scenario should be replaced with compatibility scenarios such as:

- `backend-selection.feature`
  - Scenario: building with the OpenSSL backend succeeds.
  - Scenario: building with the BoringSSL backend succeeds.
  - Scenario: enabling both backends fails fast.
  - Scenario: enabling no backend fails fast.
- `metadata-proxy.feature`
  - Scenario: a downstream build script sees `DEP_OPENSSL_INCLUDE` under the OpenSSL backend.
    - Scenario: a downstream build script sees `DEP_OPENSSL_BACKEND=openssl` under the OpenSSL backend.
    - Scenario: a downstream build script sees the expected compatibility metadata under the BoringSSL backend.
- `downstream-compat.feature`
  - Scenario: a `native-tls` consumer builds under the OpenSSL backend.
  - Scenario: a `native-tls` consumer builds under the BoringSSL backend.
    - Scenario: a `tokio-native-tls` consumer builds under both backends.
    - Scenario: a `reqwest` `native-tls` consumer builds under both backends.
    - Scenario: a `postgres-native-tls` consumer builds under both backends.
    - Scenario: a direct `openssl-sys` consumer builds under both backends.
  - Scenario: an exact-version `openssl = "=x.y.z"` consumer resolves against the current compatibility line.
    - Scenario: the checked-in example consumer builds under both backends.

### 10.4 Unit and Property Testing

Unit tests should cover:

- feature exclusivity guards,
- metadata key normalization,
- backend selection logic,
- raw polyfill behavior,
- safe polyfill behavior.

Property tests are useful for deterministic compatibility transforms such as metadata key rewriting and API gap classification. Benchmarks are not required for the initial implementation because this repository is compatibility glue rather than a known hot path. Fuzzing is only justified if raw compatibility code introduces parser-like or unsafe-heavy logic beyond ordinary FFI wrappers.

## 11. Risks and Mitigations

| Risk | Impact | Mitigation |
| :--- | :--- | :--- |
| Default backend on patched crates gets unioned with explicit backend selection | Explicit OpenSSL builds accidentally enable both backends | Use `--no-default-features` for explicit backend matrix commands and keep `default-features = false` on consumer root dependencies that select their own backend |
| BoringSSL compatibility metadata diverges from downstream expectations | Build scripts miscompile or enable the wrong cfg paths | Treat synthesized metadata, especially `DEP_OPENSSL_VERSION_NUMBER=1010100f`, as a versioned OpenSSL-compatibility profile and verify it with consumer fixtures |
| BoringSSL lacks APIs assumed by downstream crates | Compile failures or unsafe runtime mismatches | Use compiler-driven fixture builds, minimal tiered polyfills, and explicit unsupported cases |
| Template `--all-features` commands remain in CI | CI fails or hides backend selection errors | Replace with sequential backend matrix commands |
| The tracked fork drifts from upstream `rust-openssl` | Compatibility surface silently diverges or security fixes arrive late | Pin the fork by Git revision, preserve Git history, and gate upgrades through fixture builds |
| Exact-version consumers target a different upstream release than the current shim branch | Patch resolution fails before the build starts | Mirror official versions closely and maintain separate branches or tags for separate exact-version lines |
| Fixtures accidentally inherit the root workspace | Patch scope and dependency resolution stop reflecting real downstream behavior | Exclude fixtures from the root workspace and give each fixture its own workspace boundary |
| Distribution accidentally targets crates.io | Release process becomes impossible | Document path/git patch distribution only and disable crates.io publish flow |

## 12. Phased Delivery Plan

### Phase 0: Repository Normalization

- Remove checkout-template BDD artifacts.
- Add backend-aware workspace commands.
- Exclude `tests/fixtures/*` from the root workspace and create the standalone fixture layout.

Phase 0 is a hard prerequisite for Phase 1. The repository should not begin shim implementation work until all of the following are complete:

- [ ] `features/checkout.feature` replaced by compatibility-oriented feature files.
- [ ] The `Justfile` no longer contains `--all-features` test or check commands.
- [ ] The `Justfile` no longer references `-p common`.
- [ ] The root `Cargo.toml` excludes `tests/fixtures/*` from the workspace.
- [ ] Each fixture under `tests/fixtures/*` has its own workspace boundary and local `[patch.crates-io]` section.

### Phase 1: OpenSSL Passthrough Baseline

- Wire the tracked `rust-openssl` fork via pinned Git revision or Git submodule.
- Implement `crates/openssl-sys` with the OpenSSL backend only.
- Implement `crates/openssl` as a thin re-export over `real-openssl`.
- Prove the OpenSSL backend builds via patched fixtures.

### Phase 2: Build Metadata Proxy

- Add the shim `build.rs` relay.
- Define and document the initial BoringSSL OpenSSL-1.1.1 compatibility metadata profile.
- Verify `DEP_OPENSSL_*` through the metadata fixture.
- Document the compatibility metadata whitelist.

### Phase 3: BoringSSL Backend

- Add `boring` and `boring-sys` backend wiring.
- Introduce backend exclusivity checks.
- Make the BoringSSL build succeed for the raw and safe shim crates.

### Phase 4: Compiler-Driven API Gap Closure

- Build the target fixture matrix and let compiler failures drive gap discovery.
- Close the raw and safe API gaps needed for the chosen downstream fixture set.
- Implement only the polyfills needed for targeted downstream fixtures.

### Phase 5: Consumer Hardening and Optional Tooling

- Keep AST/codegen out of scope unless manual gap closure becomes the main maintenance bottleneck.
- Add a gated cucumber runner under `crates/openssl/tests/bdd.rs` and point the `Justfile` BDD commands at it.
- Expand fixture coverage where it improves consumer confidence, including direct `openssl-sys` consumers.
- Tighten metadata-fixture assertions so the observed backend marker matches the selected backend feature.
- Finalize root feature examples and user documentation.
- Stabilize CI to run the full backend matrix sequentially.

## 13. Acceptance Criteria

The design is considered successfully implemented when all of the following are true:

- A downstream application can patch `openssl` and `openssl-sys` from this repository without renaming its existing dependencies.
- The application can select exactly one backend at the root and build successfully under that backend.
- Enabling both backends fails immediately with a clear error.
- Enabling no backend fails immediately with a clear error.
- A downstream build script receives the required `DEP_OPENSSL_*` metadata under both supported backends.
- A direct `openssl-sys` consumer compiles against the shim.
- A transitive consumer such as `native-tls` compiles against the shim.
- A fixture with an exact dependency pin to the current upstream compatibility line resolves through the shim.
- Workspace verification no longer relies on `--all-features`.
- Fixtures under `tests/fixtures/*` run as isolated consumer projects rather than root workspace members.
- Template checkout/common artifacts are fully removed from the repository.

## 14. Recommended First Implementation Slice

The first implementation slice should not try to solve universal BoringSSL parity. The practical starting point is:

1. Normalize the repository and replace template tests.
2. Deliver the OpenSSL passthrough path end-to-end.
3. Prove metadata relay through a fixture.
4. Add BoringSSL backend selection and close only the API gaps required by the first target downstream crate set.

That sequence creates a usable contract quickly while keeping the highest-risk compatibility work measurable and incremental.
