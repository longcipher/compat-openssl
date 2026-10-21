# Default recipe to display help
default:
  @just --list

# Format all code
format:
  rumdl fmt .
  taplo fmt
  cargo +nightly fmt --all

# Auto-fix linting issues
fix:
  rumdl check --fix .
  just fix-openssl
  just fix-boring

# Auto-fix clippy findings for the OpenSSL backend
fix-openssl:
  cargo +nightly clippy --fix --workspace --no-default-features --features backend-openssl --allow-dirty

# Auto-fix clippy findings for the BoringSSL backend
fix-boring:
  cargo +nightly clippy --fix --workspace --no-default-features --features backend-boring --allow-dirty

# Run all lints
lint:
  typos
  rumdl check .
  taplo fmt --check
  cargo +nightly fmt --all -- --check
  just lint-openssl
  just lint-boring
  cargo machete

# Lint the OpenSSL backend
lint-openssl:
  cargo +nightly clippy --workspace --no-default-features --features backend-openssl -- -D warnings

# Lint the BoringSSL backend
lint-boring:
  cargo +nightly clippy --workspace --no-default-features --features backend-boring -- -D warnings

# Check the OpenSSL backend compiles
check-openssl:
  cargo check --workspace --all-targets --no-default-features --features backend-openssl

# Check the BoringSSL backend compiles
check-boring:
  cargo check --workspace --all-targets --no-default-features --features backend-boring

# Check all targets compile
check:
  just check-openssl
  just check-boring

# Build the OpenSSL backend
build-openssl:
  cargo build --workspace --no-default-features --features backend-openssl

# Build the BoringSSL backend
build-boring:
  cargo build --workspace --no-default-features --features backend-boring

# Build entire workspace
build:
  just build-openssl
  just build-boring

# Check the positive feature powerset of both shim crates
check-feature:
  cargo hack check --feature-powerset --depth 1 --no-dev-deps --exclude-no-default-features --exclude-all-features --mutually-exclusive-features backend-openssl,backend-boring

# Build the standalone consumer example with the OpenSSL backend
example-openssl:
  cargo build --manifest-path examples/consumer-app/Cargo.toml --no-default-features --features backend-openssl

# Build the standalone consumer example with the BoringSSL backend
example-boring:
  cargo build --manifest-path examples/consumer-app/Cargo.toml --no-default-features --features backend-boring

# Build the standalone consumer example with both backends
examples:
  just example-openssl
  just example-boring

# Build every downstream fixture against both backends
fixtures:
  #!/usr/bin/env bash
  set -euo pipefail
  for dir in tests/fixtures/*/; do
    dir="${dir%/}"
    for backend in backend-openssl backend-boring; do
      echo "==> ${dir} (${backend})"
      cargo build --manifest-path "${dir}/Cargo.toml" --no-default-features --features "${backend}"
    done
  done

# Test the OpenSSL backend
test-openssl:
  cargo test --workspace --no-default-features --features backend-openssl

# Test the BoringSSL backend
test-boring:
  cargo test --workspace --no-default-features --features backend-boring

# Run tests
test:
  just test-openssl
  just test-boring

# Run the full backend matrix
test-all:
  just test

# Coverage for the OpenSSL backend
test-coverage-openssl:
  cargo tarpaulin --workspace --no-default-features --features backend-openssl --timeout 300

# Coverage for the BoringSSL backend
test-coverage-boring:
  cargo tarpaulin --workspace --no-default-features --features backend-boring --timeout 300

# Run tests with coverage
test-coverage:
  just test-coverage-openssl
  just test-coverage-boring

# Crates.io publishing is intentionally unsupported for this repository
publish-check:
  @echo "crates.io publishing is intentionally unsupported for compat-openssl" >&2; exit 1

# Crates.io publishing is intentionally unsupported for this repository
publish:
  @echo "crates.io publishing is intentionally unsupported for compat-openssl" >&2; exit 1

# Check for Chinese characters in English-only sources and docs
check-cn:
  #!/usr/bin/env bash
  set -euo pipefail
  if rg --line-number --column "\p{Han}" --glob '*.md' --glob '*.toml' --glob '!target/**' --glob '!specs/**'; then
    echo "error: Chinese characters found in English-only files" >&2
    exit 1
  fi

# Full CI check
ci: lint test-all build examples

# ============================================================
# Maintenance & Tools
# ============================================================

# Clean build artifacts
clean:
  cargo clean

# Install all required development tools
setup:
  cargo install cargo-machete
  cargo install cargo-hack
  cargo install cargo-tarpaulin
  cargo install cargo-nextest
  cargo install cargo-audit
  cargo install cargo-outdated
  cargo install rumdl
  cargo install taplo-cli
  cargo install typos-cli

# Generate documentation for the workspace
docs:
  cargo doc --no-deps --open

# Run security audit
audit:
  cargo audit

# Check for outdated dependencies
outdated:
  cargo outdated

# Run clippy with all lints against the OpenSSL backend
clippy-all-openssl:
  cargo +nightly clippy --workspace --no-default-features --features backend-openssl -- -W clippy::pedantic -W clippy::nursery -W clippy::cargo

# Run clippy with all lints against the BoringSSL backend
clippy-all-boring:
  cargo +nightly clippy --workspace --no-default-features --features backend-boring -- -W clippy::pedantic -W clippy::nursery -W clippy::cargo

# Run clippy with all lints against both backends
clippy-all:
  just clippy-all-openssl
  just clippy-all-boring

# Run tests with nextest against both backends
test-nextest:
  cargo nextest run --workspace --no-default-features --features backend-openssl
  cargo nextest run --workspace --no-default-features --features backend-boring

# Run miri on unsafe code
miri:
  cargo +nightly miri test --workspace --no-default-features --features backend-boring

# Check for unused dependencies
unused-deps:
  cargo machete

# Update all dependencies
update-deps:
  cargo update

# Full check (lint + test + build + audit)
full-check: lint test build audit
