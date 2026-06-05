# rojcad — Justfile (https://just.systems)
#
# Common commands for the headless parametric CAD system.
# Install `just` via: cargo install just

# ── Variables ──────────────────────────────────────────────────────────────────

# Local cargo home to work around sandbox permission issues
_cargo_home := justfile_directory() + "/.local-cargo"

# Default environment for sandbox builds
# - HOME=/tmp bypasses ~/.gitconfig permission issues
# - CC=clang / CXX=clang++ avoids collect2 linker permission issues
# - CARGO_HOME points to local dir to avoid ~/.cargo permission issues
# - RUSTFLAGS="-Clinker=clang" makes rustc use clang as linker driver
_env := "HOME=/tmp GIT_CONFIG_NOSYSTEM=1 CC=clang CXX=clang++ CARGO_HOME=" + _cargo_home + " RUSTFLAGS=-Clinker=clang"

# ── Default ────────────────────────────────────────────────────────────────────

# Default recipe — shown when running `just` with no arguments
default:
    @echo "rojcad v0.1.0 — Headless parametric CAD with embedded Janet DSL"
    @echo ""
    @echo "Usage: just <recipe>"
    @echo ""
    @echo "Recipes:"
    @echo "  check          Check code compiles (fastest feedback)"
    @echo "  build          Build in debug mode"
    @echo "  build-release  Build in release mode"
    @echo "  test           Run all tests"
    @echo "  run            Start the TCP REPL server on port 9000"
    @echo "  run-release    Start the server (release build)"
    @echo "  lint           Run clippy"
    @echo "  fmt            Format code with rustfmt"
    @echo "  doc            Build Rust documentation"
    @echo "  doc-janet      Generate Janet API reference (Markdown + HTML)"
    @echo "  deps           Show dependency tree"
    @echo "  clean          Remove build artifacts"
    @echo "  clean-all      Remove all artifacts + local cargo cache"
    @echo "  info           Print project info and all recipes"
    @echo ""
    @echo "First build? OCCT compiles from source (10-15 min)."
    @echo "After that, successive builds are incremental."

# ── Submodule management ───────────────────────────────────────────────────────

# Init and update git submodules (required for opencascade-rs OCCT bundle)
submodule:
    git submodule update --init --recursive

# ── Build ──────────────────────────────────────────────────────────────────────

# Build in debug mode (default)
build:
    {{_env}} cargo build

# Build in release mode (recommended for actual use)
build-release *args="":
    {{_env}} cargo build --release {{args}}

# Check code compiles without producing artifacts (fastest feedback)
check:
    {{_env}} cargo check

# ── Test ────────────────────────────────────────────────────────────────────────

# Run all tests
test:
    {{_env}} cargo test

# Run only the unit tests (faster, no integration tests)
test-unit:
    {{_env}} cargo test --lib

# Run a specific test by name, e.g.: just test-name test_make_box
test-name name:
    {{_env}} cargo test {{name}} -- --nocapture

# ── Run ────────────────────────────────────────────────────────────────────────

# Run the server (debug)
run:
    {{_env}} cargo run

# Run the server (release)
run-release:
    {{_env}} cargo run --release

# ── Lint ───────────────────────────────────────────────────────────────────────

# Run clippy (Rust linter)
lint:
    {{_env}} cargo clippy -- -D warnings

# Format code with rustfmt
fmt:
    cargo fmt

# Check formatting without changing files
fmt-check:
    cargo fmt --check

# ── Documentation ──────────────────────────────────────────────────────────────

# Build docs
doc:
    {{_env}} cargo doc --no-deps

# Open docs in browser
doc-open:
    {{_env}} cargo doc --no-deps --open

# Generate Janet API reference (Markdown + HTML)
doc-janet:
    {{_env}} cargo run -- --headless --eval '(do (dump-docs "doc") (os/exit 0))'

# ── Clean ──────────────────────────────────────────────────────────────────────

# Remove build artifacts
clean:
    cargo clean

# Clean everything including the local cargo cache
clean-all: clean
    -rm -rf {{_cargo_home}}
    -rm -rf target

# ── Utility ────────────────────────────────────────────────────────────────────

# Print project metadata
info:
    @echo "rojcad v0.1.0 — Headless parametric CAD with embedded Janet DSL"
    @echo ""
    @echo "Recipes:"
    @just --list

# Show dependencies
deps:
    {{_env}} cargo tree

# Show outdated dependencies
outdated:
    {{_env}} cargo outdated

# ── OCCT build helpers ─────────────────────────────────────────────────────────

# Check if OCCT build is cached (fast indicator)
check-occt-cache:
    test -d {{_cargo_home}}/git/db/opencascade-rs* && echo "OCCT cached" || echo "OCCT not cached — first build will be slow"

# Full fresh build (debug): init submodules, then build
full-build: submodule build

# Full fresh build (release)
full-build-release: submodule build-release
