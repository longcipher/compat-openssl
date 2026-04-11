# compat-openssl

compat-openssl is a patch-based compatibility workspace for consumers that already depend on `openssl` and `openssl-sys`, but need to choose either the upstream OpenSSL implementation or the BoringSSL bindings without renaming those dependencies throughout their graph.

The repository ships two shim crates:

- `crates/openssl-sys` preserves the `openssl-sys` package identity, selects exactly one backend, and relays a curated `DEP_OPENSSL_*` metadata contract to downstream build scripts.
- `crates/openssl` preserves the `openssl` package identity and re-exports either the tracked `rust-openssl` fork or `boring` behind the same feature gate.

This repository is intentionally consumed through `[patch.crates-io]`. Publishing these crates from here is out of scope.

## Backend Model

- `backend-openssl` selects the tracked `rust-openssl` fork.
- `backend-boring` selects `boring` and `boring-sys`.
- No default backend is provided.
- Enabling both backends is an error.
- Enabling neither backend is an error.

Backend choice belongs at the consumer root so Cargo feature resolution stays deterministic across transitive users such as `native-tls`.

## Consumer Usage

Patch both crates from this repository and select exactly one backend at the root:

```toml
[features]
default = ["tls-openssl"]
tls-openssl = ["openssl/backend-openssl"]
tls-boring = ["openssl/backend-boring"]

[dependencies]
openssl = { version = "0.10.76", default-features = false }

[patch.crates-io]
openssl = { path = "../compat-openssl/crates/openssl" }
openssl-sys = { path = "../compat-openssl/crates/openssl-sys" }
```

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
just check
just test
just bdd
just test-all
```

BDD is implemented as a gated integration test at `crates/openssl/tests/bdd.rs`. It does not run during ordinary crate tests unless explicitly requested through the internal feature gate:

```bash
cargo test -p openssl --features backend-openssl
cargo test -p openssl --test bdd --features backend-openssl,internal-bdd
cargo test -p openssl --test bdd --features backend-boring,internal-bdd
```

`just bdd` runs the two BDD commands above sequentially.

## Fixture Matrix

The acceptance suite exercises real standalone consumers under `tests/fixtures/`:

- `downstream-build-metadata` verifies the forwarded `DEP_OPENSSL_*` contract.
- `downstream-native-tls` verifies a transitive safe-layer consumer.
- `downstream-openssl-sys` verifies a direct raw sys consumer.
- `downstream-openssl-exact-pin` verifies patch resolution for `openssl = "=0.10.76"`.

Each fixture is its own workspace root and patches back to this repository locally.

## Development Notes

- Cargo commands must run sequentially because the workspace and fixtures share cargo caches and lock files.
- The BDD runner also executes fixture builds sequentially with `max_concurrent_scenarios(1)`.
- AST or code-generation tooling is intentionally out of scope unless manual compatibility work becomes the bottleneck.

## License

Apache-2.0
