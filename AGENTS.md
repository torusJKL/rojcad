# rojcad — Agent Guide

## Build sandbox
All `just` commands sandbox with:
`HOME=/tmp GIT_CONFIG_NOSYSTEM=1 CC=clang CXX=clang++ CARGO_HOME=.local-cargo RUSTFLAGS=-Clinker=clang`

**Always use `just`, never raw `cargo`**, or the build fails with permission errors. Raw `cargo` is safe only for `fmt`, `clean`, and read-only operations.

## First build
Compiles OCCT from source via `opencascade-rs`'s `builtin` feature (~10-15 min). CI caches OCCT by opencascade-sys commit from `Cargo.lock`. Check local cache: `just check-occt-cache`. Fresh build from submodules: `just full-build`.

## Janet is vendored + bootstrapped
`vendor/` contains Janet compiled from C in `build.rs`. `JANET_BOOTSTRAP=1` means core library modules are manually registered in `src/main.rs:2148-2166` — add new ones there.

Standard Janet macros come from `upstream.janet` (loaded via `include_str!` before `boot.janet`). `&form`/`&env` macro arguments are unavailable in bootstrap mode.

**Updating `upstream.janet`**: Fetch new `src/boot/boot.janet` from matching tag, trim from `### Bootstrap` section onward.

## Boot sequence (src/main.rs:2192-2287)
Two-phase: `upstream.janet` (macros) → `boot.janet` (TCP REPL + CAD fn wrappers) → `boot/model.janet` (parametric model runtime). All via `include_str!` + `janet_dostring`.

## Ports
Raw TCP REPL: **9364** (`--raw-port`). Spork netrepl: **9365** (`--spork-port`).

## CLI flags
`--headless`, `--eval <EXPR>`/`--eval=<EXPR>`, `--raw-port`, `--spork-port`, `--width <PX>`, `--height <PX>`.

## Vendored spork
`vendor/spork/` = server-only subset (msg.janet, ev-utils.janet, netrepl-server.janet). Patched at `netrepl-server.janet:60-64` to print strings raw instead of via `pp` — re-apply on update.

## Adding a Janet-callable CAD function
1. Rust impl (e.g., `src/cad.rs`)
2. C wrapper in `bridge/bridge.c` (JANET_FN + register in `cad_register_functions`)
3. Rust `extern "C"` decl in `src/bridge.rs`
4. Rust FFI bridge fn in `src/main.rs`

## Common commands
| Task | Command |
|------|---------|
| Build | `just build` |
| Check (fast) | `just check` |
| Run server | `just run` |
| Run headless | `just run -- --headless` |
| All tests | `just test` |
| Unit tests only | `just test-unit` |
| Single test (+stdout) | `just test-name <name>` |
| REPL integration tests | `just test-repl` |
| Lint (clippy -D warnings) | `just lint` |
| Format / check | `just fmt` / `just fmt-check` |
| Full fresh build | `just full-build` |
| Clean all | `just clean-all` |

## CI quirks
- CI runs `cargo test --all-targets` (includes benches/examples); `just test` runs `cargo test` (lib+tests only).
- CI rustfmt uses **nightly**; `just fmt` uses whatever is installed.
- CI needs `submodules: "recursive"`.
- OCCT cache keyed by opencascade-sys commit from `Cargo.lock`.

## Viewer
- Background thread (wgpu + winit + egui), `mpsc` channels for REPL↔viewer.
- **Compiled out on macOS/iOS**.
- Disabled with `--headless`.
- Shared state via `ShapeRegistry` (RwLock + atomic generation counter).

## Tests
- Unit tests: `#[cfg(test)]` inline in `src/cad.rs`.
- REPL integration: `tests/test-variadic.sh` (`just test-repl`). Type-check error tests use `stdbuf -eL` + `timeout 2` because `janet_panic` longjmp can corrupt the stack.
- Export tests write to `/tmp/test_rojcad_*.step` / `.stl`.

## Style
- Methods private unless they must be public.
- Yoda conditions: `if (false == condition)` not `if (!condition)`.
- Rust edition 2024. Crate-level allows `non_upper_case_globals`, `non_camel_case_types`, `non_snake_case`, `clippy::missing_safety_doc`.
- Every public Janet function needs a docstring. Non-trivial functions must include a usage example.
- Doc format: `#` for example comments, `-` for prose. `\n\n` separates sections (usage, body, examples, returns). Example expressions indented 2 spaces, starting with `(`.
