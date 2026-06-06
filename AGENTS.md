# rojcad — Agent Guide

## Build sandbox
All `just` commands (build, test, run, check, lint, etc.) run with a sandboxed env:
`HOME=/tmp GIT_CONFIG_NOSYSTEM=1 CC=clang CXX=clang++ CARGO_HOME=.local-cargo RUSTFLAGS=-Clinker=clang`

**Always use `just` commands, never raw `cargo`**, or the build will fail with permission errors. Raw `cargo` is only safe for `fmt`, `clean`, and read-only operations.

## First build
Compiles OCCT from source via `opencascade-rs`'s `builtin` feature (~10-15 min). CI caches OCCT using the opencascade-rs commit from Cargo.lock. Check local cache: `just check-occt-cache`. Use `just full-build` or `just full-build-release` for a fresh build from submodules.

## Janet is vendored + bootstrapped
`vendor/` contains Janet compiled from C in `build.rs`. `boot.janet` (TCP REPL server) is embedded at compile time via `include_str!`. `JANET_BOOTSTRAP=1` means core library modules must be manually registered in `src/main.rs` — any new core module dependency must be added there.

## Default port is 9365
The TCP REPL listens on port **9365** by default (override with `--port <PORT>` or `--port=<PORT>`). The README incorrectly says 9000.

## CLI flags
`--headless` disables the 3D viewer. `--eval <EXPR>` or `--eval=<EXPR>` runs Janet code after boot, then exits.

## Adding a Janet-callable CAD function
1. Rust implementation (e.g., in `src/cad.rs`)
2. C wrapper in `bridge/bridge.c` (JANET_FN + register in `cad_register_functions`)
3. Rust `extern "C"` declaration in `src/bridge.rs`
4. Rust FFI bridge function in `src/main.rs`

## Common commands
| Task | Command |
|------|---------|
| Build (debug) | `just build` |
| Build (release) | `just build-release` |
| Check (fast) | `just check` |
| Run server | `just run` / `just run-release` |
| Run headless | `cargo run -- --headless` |
| All tests | `just test` |
| Unit tests only | `just test-unit` |
| Single test (with stdout) | `just test-name <name>` |
| Lint | `just lint` (clippy with `-D warnings`) |
| Format | `just fmt` / `just fmt-check` |
| Generate Janet API docs | `just doc-janet` |
| Full fresh build | `just full-build` |
| Clean (including cargo cache) | `just clean-all` |

## CI quirks
- CI runs `cargo test --all-targets` (includes benches/examples); `just test` runs `cargo test` (lib+tests only).
- CI uses **nightly** rustfmt; `just fmt`/`just fmt-check` use whatever toolchain is installed.
- CI needs `submodules: "recursive"` for checkout.

## Viewer
- Runs on a background thread (`wgpu-viewer`), using wgpu + winit.
- **Compiled out on macOS/iOS** (`#[cfg(not(any(target_os = "macos", target_os = "ios")))]`).
- Disabled with `--headless`.
- REPL ↔ viewer communication via `mpsc` channels.
- Shared state via `ShapeRegistry` (RwLock + atomic generation counter).

## Tests
- All tests are `#[cfg(test)]` inline in `src/cad.rs` (no separate test files).
- Export tests write to `/tmp/test_rojcad_*.step` / `.stl`.
- Janet-level operations are tested indirectly only — no automated Janet integration tests.

## Style
- Make methods private unless they must be public.
- Use Yoda conditions: `if (false == condition)` not `if (!condition)`.
- Rust edition 2024. Crate-level allows `non_upper_case_globals`, `non_camel_case_types`, `non_snake_case`, and `clippy::missing_safety_doc`.
- Every public Janet function must have a docstring. Simple functions get a short description; functions with non-trivial arguments must include a usage example.
- OpenSpec change proposals live in `openspec/changes/`. Load `.opencode/skills/` for propose/apply/explore/archive workflows.
