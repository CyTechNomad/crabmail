default:
    @just --list

# Run in debug mode
run:
    cargo run

# Build release binary
build:
    cargo build --release

# Check for compile errors
check:
    cargo check

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt
    deno fmt '**/*.md'

# Check formatting without modifying
fmt-check:
    cargo fmt -- --check
    deno fmt --check '**/*.md'

# Run tests
test:
    cargo test

# Clean build artifacts
clean:
    cargo clean

# Build and run release binary
release: build
    ./target/release/crabmail
