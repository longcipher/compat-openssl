# consumer-app

This standalone crate demonstrates the full consumer-side integration surface for compat-openssl.

## What It Shows

- Direct safe-layer usage through `openssl`
- Direct raw-layer usage through `openssl-sys`
- Transitive TLS usage through `native-tls`, `tokio-native-tls`, and `reqwest` with `native-tls`
- Database adapter wiring through `postgres-native-tls`
- Build-script metadata consumption through `DEP_OPENSSL_*`

## Build Commands

```bash
cargo build --manifest-path examples/consumer-app/Cargo.toml
cargo build --manifest-path examples/consumer-app/Cargo.toml --no-default-features --features backend-boring
cargo build --manifest-path examples/consumer-app/Cargo.toml --no-default-features --features backend-openssl
```

## Current Compatibility Line

- `openssl = "0.10.76"`
- `openssl-sys = "0.9.112"`
- Local path patch from this example points to `../../crates/openssl` and `../../crates/openssl-sys`

## Important Notes

- Keep `default-features = false` on both patched crates so the example root, rather than the dependencies themselves, selects exactly one backend.
- This example root defaults to `backend-boring`; use `--no-default-features --features backend-openssl` when you want to validate the OpenSSL path.
- Keep the direct `openssl-sys` dependency only if you need raw FFI items or `DEP_OPENSSL_*` inside `build.rs`.
- The build script in this example exports the forwarded backend metadata to `env!` so the binary can compile against it.
- For an external repository, replace the local path patch with a git patch to this repository at a fixed revision you control.
