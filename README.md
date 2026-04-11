# compat-openssl

compat-openssl is a patch-based compatibility workspace for consumers that already depend on `openssl` and `openssl-sys`, but need to choose either the upstream OpenSSL implementation or the BoringSSL bindings without renaming those dependencies throughout their graph.

The repository ships two shim crates:

- `crates/openssl-sys` preserves the `openssl-sys` package identity, selects exactly one backend, and relays a curated `DEP_OPENSSL_*` metadata contract to downstream build scripts.
- `crates/openssl` preserves the `openssl` package identity and re-exports either the tracked `rust-openssl` fork or `boring` behind the same feature gate.

This repository is intentionally consumed through `[patch.crates-io]`. Publishing these crates from here is out of scope.

## Backend Model

- `backend-openssl` selects the tracked `rust-openssl` fork.
- `backend-boring` selects `boring` and `boring-sys`.
- The shim crates default to `backend-boring`, so plain `cargo build` in this repository exercises the BoringSSL path.
- Downstream applications that need deterministic root-controlled backend selection should keep `default-features = false` on patched `openssl` and `openssl-sys` dependencies and choose a backend explicitly.
- Enabling both backends is an error.
- Disabling default features without selecting a backend is an error.

Backend choice belongs at the consumer root so Cargo feature resolution stays deterministic across transitive users such as `native-tls`.

## Consumer Usage

Patch both crates from this repository and, when you want the application root to control backend selection explicitly, disable dependency defaults and select exactly one backend there:

```toml
[features]
default = ["tls-boring"]
tls-openssl = ["openssl/backend-openssl"]
tls-boring = ["openssl/backend-boring"]

[dependencies]
openssl = { version = "0.10.76", default-features = false }

[patch.crates-io]
openssl = { path = "../compat-openssl/crates/openssl" }
openssl-sys = { path = "../compat-openssl/crates/openssl-sys" }
```

If you omit `default-features = false` on the patched crates, the compatibility shims fall back to `backend-boring`.

The checked-in example under `examples/consumer-app` uses the repository-local patch paths that make sense from inside this workspace:

```toml
[patch.crates-io]
openssl = { path = "../../crates/openssl" }
openssl-sys = { path = "../../crates/openssl-sys" }
```

Outside this repository, replace those paths with your own checked-out location or a pinned git patch to a revision you control.

If your application or one of its build scripts depends directly on `openssl-sys`, keep that explicit dependency at the root as well:

```toml
[dependencies]
openssl = { version = "0.10.76", default-features = false }
openssl-sys = { version = "0.9.112", default-features = false }
```

That direct `openssl-sys` dependency is required when a downstream `build.rs` needs to read `DEP_OPENSSL_*`, because Cargo only exposes linked-library metadata to immediate dependents of the sys crate.

## Metadata Contract

The OpenSSL backend forwards a curated subset of `DEP_REAL_OPENSSL_*` keys.

The BoringSSL backend emits an OpenSSL-compatibility profile:

- `DEP_OPENSSL_INCLUDE` comes from `DEP_BORINGSSL_INCLUDE` when available, otherwise from `${DEP_BORINGSSL_ROOT}/boringssl/include`.
- `DEP_OPENSSL_VERSION_NUMBER` is synthesized as `1010100f` to advertise the current OpenSSL 1.1.1 compatibility baseline.
- `DEP_OPENSSL_BACKEND` is synthesized as `openssl` or `boring` for diagnostics and fixture validation.

Observed `boring-sys` metadata uses the `DEP_BORINGSSL_*` prefix, not `DEP_BSSL_*`, because the linked backend crate uses `links = "boringssl"`.

## Repository Verification

The workspace keeps the backend matrix explicit and sequential:

```bash
cargo build
just check
just test
just bdd
just examples
just test-all
```

BDD is implemented as a gated integration test at `crates/openssl/tests/bdd.rs`. It does not run during ordinary crate tests unless explicitly requested through the internal feature gate:

```bash
cargo test -p openssl
cargo test -p openssl --no-default-features --features backend-openssl
cargo test -p openssl --test bdd --no-default-features --features backend-openssl,internal-bdd
cargo test -p openssl --test bdd --no-default-features --features backend-boring,internal-bdd
```

`just bdd` runs the two BDD commands above sequentially.

The standalone example consumer is compiled separately from the workspace because it behaves like a real downstream root package:

```bash
cargo build --manifest-path examples/consumer-app/Cargo.toml
cargo build --manifest-path examples/consumer-app/Cargo.toml --no-default-features --features backend-openssl
```

## Fixture Matrix

The acceptance suite exercises real standalone consumers under `tests/fixtures/`:

- `downstream-build-metadata` verifies the forwarded `DEP_OPENSSL_*` contract.
- `downstream-native-tls` verifies a transitive safe-layer consumer.
- `downstream-tokio-native-tls` verifies the async wrapper over `native-tls`.
- `downstream-reqwest-native-tls` verifies a real HTTP client stack through `reqwest` and `native-tls`.
- `downstream-postgres-native-tls` verifies a database TLS adapter through `postgres-native-tls`.
- `downstream-openssl-sys` verifies a direct raw sys consumer.
- `downstream-openssl-exact-pin` verifies patch resolution for `openssl = "=0.10.76"`.

Each fixture is its own workspace root and patches back to this repository locally.

## Example Consumer

`examples/consumer-app` is a standalone consumer crate that demonstrates the currently supported integration surface in one place:

- direct safe-layer usage through `openssl`
- direct raw-layer usage through `openssl-sys`
- build-script metadata consumption through `DEP_OPENSSL_*`
- transitive TLS usage through `native-tls`, `tokio-native-tls`, `reqwest` with `native-tls`, and `postgres-native-tls`

See `examples/consumer-app/README.md` for the exact dependency list, patch paths, and the local build commands.

## Current Notes

- The currently verified compatibility line is `openssl = "0.10.76"` and `openssl-sys = "0.9.112"`.
- Plain `cargo build` in the repository root and in `examples/consumer-app` defaults to `backend-boring`.
- Exact pins must match that mirrored upstream pair; older exact pins require a different maintenance branch or tag.
- Keep the direct `openssl-sys` dependency only when you need raw FFI items or `DEP_OPENSSL_*` in `build.rs`.
- The current Boring backend verification matrix does not include `tokio-openssl`-based adapters. Those crates expect `openssl::ssl::SslStream` methods such as `ssl_peek`, `read_early_data`, and `write_early_data` that are not currently exposed by the Boring-backed safe layer.

## Development Notes

- Cargo commands must run sequentially because the workspace and fixtures share cargo caches and lock files.
- The BDD runner also executes fixture builds sequentially with `max_concurrent_scenarios(1)`.
- AST or code-generation tooling is intentionally out of scope unless manual compatibility work becomes the bottleneck.

## License

Apache-2.0
