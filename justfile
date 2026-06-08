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

# Version from Cargo.toml (evaluated at load time)
_version := `sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml`

# Version for documentation: git tag (with -dirty if uncommitted),
# short hash (with -dirty), or Cargo.toml fallback
_doc_version := `desc=$(git describe --tags --exact-match --dirty 2>/dev/null); if [ -n "$desc" ]; then echo "$desc"; else hash=$(git rev-parse --short HEAD 2>/dev/null); if [ -n "$hash" ]; then dirty=$(git status --porcelain 2>/dev/null | grep -q . && echo "-dirty"); echo "$hash$dirty"; else sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml; fi; fi`

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
    @echo "  run            Start the TCP REPL server on port 9365"
    @echo "  run-release    Start the server (release build)"
    @echo "  lint           Run clippy"
    @echo "  fmt            Format code with rustfmt"
    @echo "  doc            Build Rust documentation"
    @echo "  doc-janet      Generate Janet API reference (Markdown + HTML)"
    @echo "  appimage       Build release + package as AppImage"
    @echo "  tarball        Build release + package as tar.gz"
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

# Run REPL integration tests for variadic CAD function wrappers
test-repl:
    {{_env}} tests/test-variadic.sh

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
    {{_env}} cargo clippy --all-targets -- -D warnings

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
    {{_env}} cargo run -- --headless --eval '(do (dump-docs "doc" "{{_doc_version}}") (os/exit 0))'

# ── Packaging ──────────────────────────────────────────────────────────────────

# Download AppImage tooling (linuxdeploy + appimagetool)
_appimage-tools:
    mkdir -p .appimage
    test -f .appimage/linuxdeploy || curl -sL \
      "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage" \
      -o .appimage/linuxdeploy
    chmod +x .appimage/linuxdeploy
    test -f .appimage/appimagetool || curl -sL \
      "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage" \
      -o .appimage/appimagetool
    chmod +x .appimage/appimagetool

# Package as AppImage
appimage: build-release _appimage-tools
    rm -rf .appimage/rojcad.AppDir
    mkdir -p dist
    APPIMAGE_EXTRACT_AND_RUN=1 \
    ./.appimage/linuxdeploy \
      --appdir .appimage/rojcad.AppDir \
      --executable target/release/rojcad \
      --desktop-file packaging/rojcad.desktop \
      --icon-file packaging/rojcad.svg
    # Bundle Vulkan loader (wgpu loads it at runtime via dlopen)
    mkdir -p .appimage/rojcad.AppDir/usr/lib
    cp /usr/lib/x86_64-linux-gnu/libvulkan.so.1 \
      .appimage/rojcad.AppDir/usr/lib/ 2>/dev/null || true
    APPIMAGE_EXTRACT_AND_RUN=1 \
    ./.appimage/appimagetool \
      --no-appstream \
      .appimage/rojcad.AppDir \
      dist/rojcad-{{_version}}-x86_64.AppImage

# Package as tar.gz with binary, docs, and README
tarball: build-release
    mkdir -p dist/rojcad-{{_version}}-x86_64
    {{_env}} cargo run --release -- \
      --headless --eval '(do (dump-docs "doc" "{{_doc_version}}") (os/exit 0))'
    cp -r doc dist/rojcad-{{_version}}-x86_64/
    cp target/release/rojcad dist/rojcad-{{_version}}-x86_64/
    cp README.md dist/rojcad-{{_version}}-x86_64/
    cp CHANGELOG.md dist/rojcad-{{_version}}-x86_64/ 2>/dev/null || true
    tar czf dist/rojcad-{{_version}}-x86_64.tar.gz \
      -C dist rojcad-{{_version}}-x86_64

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
