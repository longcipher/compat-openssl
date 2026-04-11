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

fix-openssl:
  cargo +nightly clippy --fix --workspace --features backend-openssl --allow-dirty

fix-boring:
  cargo +nightly clippy --fix --workspace --features backend-boring --allow-dirty

# Run all lints
lint:
  typos
  rumdl check .
  taplo fmt --check
  cargo +nightly fmt --all -- --check
  just lint-openssl
  just lint-boring
  cargo machete

lint-openssl:
  cargo +nightly clippy --workspace --features backend-openssl -- -D warnings

lint-boring:
  cargo +nightly clippy --workspace --features backend-boring -- -D warnings

check-openssl:
  cargo check --workspace --all-targets --features backend-openssl

check-boring:
  cargo check --workspace --all-targets --features backend-boring

# Check all targets compile
check:
  just check-openssl
  just check-boring

build-openssl:
  cargo build --workspace --features backend-openssl

build-boring:
  cargo build --workspace --features backend-boring

# Build entire workspace
build:
  just build-openssl
  just build-boring

test-openssl:
  cargo test --workspace --features backend-openssl

test-boring:
  cargo test --workspace --features backend-boring

# Run tests
test:
  just test-openssl
  just test-boring

bdd-openssl:
  cargo test -p openssl --test bdd --features backend-openssl,internal-bdd

bdd-boring:
  cargo test -p openssl --test bdd --features backend-boring,internal-bdd

# Run BDD scenarios
bdd:
  just bdd-openssl
  just bdd-boring

# Run both TDD and BDD suites
test-all:
  just test
  just bdd

test-coverage-openssl:
  cargo tarpaulin --workspace --features backend-openssl --timeout 300

test-coverage-boring:
  cargo tarpaulin --workspace --features backend-boring --timeout 300

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

# Check for Chinese characters
check-cn:
  rg --line-number --column "\p{Han}"

# Full CI check
ci: lint test-all build

# ============================================================
# Maintenance & Tools
# ============================================================

# Clean build artifacts
clean:
  cargo clean

# Install all required development tools
setup:
  cargo install cargo-machete
  cargo install rumdl
  cargo install taplo-cli
  cargo install typos-cli

# Generate documentation for the workspace
docs:
  cargo doc --no-deps --open
