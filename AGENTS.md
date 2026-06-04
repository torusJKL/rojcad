# rojcad — Agent Guide

## Build sandbox
All `just` commands (build, test, run, check, lint, etc.) run with a sandboxed env:
`HOME=/tmp GIT_CONFIG_NOSYSTEM=1 CC=clang CXX=clang++ CARGO_HOME=.local-cargo RUSTFLAGS=-Clinker=clang`

**Always use `just` commands, never raw `cargo`**, or the build will fail with permission errors. Raw `cargo` is only safe for `fmt`, `clean`, and read-only operations.

## First build
Compiles OCCT from source via `opencascade-rs`'s `builtin` feature (~10-15 min). Subsequent builds are incremental.
Check OCCT cache: `just check-occt-cache`

## Janet is vendored, not a Cargo dependency
`vendor/` contains a copy of the [Janet](https://janet-lang.org/) language — compiled from C source in `build.rs`. `boot.janet` (TCP REPL server) is embedded at compile time via `include_str!("../boot.janet")`.

To add a new Janet-callable CAD function:
1. Rust implementation (e.g., in `src/cad.rs`)
2. C wrapper in `bridge/bridge.c` (JANET_FN + register in `cad_register_functions`)
3. Rust `extern "C"` declaration in `src/bridge.rs`
4. Rust FFI bridge function in `src/main.rs`

## Janet bootstrap mode
`JANET_BOOTSTRAP=1` means Janet core library modules are **not** auto-registered. They are manually registered in `src/main.rs` — any new core module dependency must be added there.

## Common commands
| Task | Command |
|------|---------|
| Build (debug) | `just build` |
| Build (release) | `just build-release` |
| Check (fast) | `just check` |
| Run server | `just run` |
| All tests | `just test` |
| Unit tests only | `just test-unit` |
| Single test | `just test-name <name>` |
| Lint | `just lint` (clippy with `-D warnings`) |
| Format check | `just fmt-check` |
| Run headless | `cargo run -- --headless` |

## Viewer
- Runs on a background thread (`wgpu-viewer`), using wgpu + winit.
- **Compiled out on macOS/iOS** (`#[cfg(not(any(target_os = "macos", target_os = "ios")))]`).
- Disabled with `--headless` flag.
- REPL ↔ viewer communication via `mpsc` channels.
- Shared state via `ShapeRegistry` (RwLock + atomic generation counter).

## Testing quirks
- Export tests write to `/tmp/` (STEP/STL files).
- Janet-level operations are tested indirectly (integration tests from `boot.janet` are not automated; all tests are Rust unit tests in `src/cad.rs`).

## OpenSpec
Change proposals live in `openspec/changes/`. Use the OpenSpec skills (loaded from `.opencode/skills/`) for propose/apply/explore/archive workflows.

## Style
- Make methods private unless they must be public.
- Use Yoda conditions: `if (false == condition)` not `if (!condition)`.
- Rust edition 2024.

## Janet function docstrings
Every public Janet function must have a docstring. Simple functions (e.g. `visible?`) get a short description. Functions with non-trivial arguments (e.g. `make-box`, `make-sphere`, `on-select`) must include a usage example in the docstring.

## Dependencies
| Dependency | How |
|---|---|
| opencascade-rs | Git dep, compiles OCCT from source (C++) |
| Janet language | Vendored in `vendor/`, compiled from C |
| Viewer | wgpu 24, winit 0.30, glam 0.24, bytemuck, pollster |
