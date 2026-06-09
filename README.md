# rojcad

> Parametric CAD system with embedded Janet DSL.

**rojcad** embeds a [Janet](https://janet-lang.org/) interpreter with
[OpenCASCADE](https://www.opencascade.com/) modeling via
[opencascade-rs](https://github.com/bschwind/opencascade-rs).  It provides a TCP
REPL server — connect with `nc` and start modeling in s-expressions, optionally
backed by a real-time 3D viewer (wgpu/winit).

## Quickstart

```bash
# Prerequisites: Rust, CMake 3.5+, C++11 compiler
# Debian/Ubuntu: sudo apt install build-essential cmake
# macOS: xcode-select --install && brew install cmake

git clone --recursive https://github.com/torusJKL/rojcad.git
cd rojcad

# Build (first run compiles OCCT from source — 10-15 min)
just build

# Start the TCP REPL servers (raw on 9364, spork on 9365)
just run

# In another terminal — raw REPL (basic):
nc 127.0.0.1 9364

# Or spork REPL (line editing, tab completion, history):
janet -e "(import spork/netrepl) (netrepl/client)"
```

Then in the REPL:

```janet
(def b (make-box 10 20 30))
(def s (make-sphere 15 :center '(5 10 0)))
(def result (cut b s))
(hide b) (hide s)             # operands are still shown by default
(write-step result "result.step")
```

## CLI

| Flag | Description |
|------|-------------|
| `--headless` | Disable the 3D viewer |
| `--raw-port <PORT>` | Raw TCP REPL port (default: **9364**) |
| `--spork-port <PORT>` | Spork netrepl REPL port (default: **9365**) |
| `--eval <EXPR>` | Run Janet code after boot |

## Common just recipes

| Task | Command |
|------|---------|
| Build (debug) | `just build` |
| Build (release) | `just build-release` |
| Check (fast) | `just check` |
| Run server | `just run` / `just run-release` |
| Run headless | `just run -- --headless` |
| All tests | `just test` |
| Single test | `just test-name <name>` |
| Lint (clippy) | `just lint` |
| Format | `just fmt` / `just fmt-check` |
| Janet API docs | `just doc-janet` |
| Full fresh build | `just full-build` |
| Clean all | `just clean-all` |

Use `just` (not raw `cargo`) for all build/test/run commands — the sandbox
env in `justfile` avoids filesystem permission issues. Raw `cargo` is safe
only for `cargo fmt`, `cargo clean`, and read-only operations.

## Documentation

Generate the Janet API reference (Markdown + HTML):

```bash
just doc-janet
```

This runs the server headless, calls `(dump-docs "doc")`, then exits. The output files are written to `doc/`:

| File | Format |
|------|--------|
| `doc/janet-api.md` | Markdown reference |
| `doc/janet-api.html` | HTML reference (viewable in browser) |

Rust API documentation can be built with:

```bash
just doc
```

## Architecture

```
┌──────────────────────────────────────────┐
│           rojcad binary                  │
│  ┌──────────┐  ┌──────────────────────┐  │
│  │ main.rs  │  │  boot.janet          │  │
│  │ (entry)  │  │  (TCP REPL server)   │  │
│  └────┬─────┘  └──────────────────────┘  │
│       │  include_str!()                  │
│       ▼                                  │
│  ┌──────────┐  ┌──────────────────────┐  │
│  │ bridge.rs│◄─┤  bridge/bridge.c     │  │
│  │ (extern  │  │  (Janet C API glue)  │  │
│  │  "C"     │  └──────────┬───────────┘  │
│  │  decls)  │             │              │
│  └──────────┘             ▼              │
│  ┌──────────┐  ┌──────────────────────┐  │
│  │  cad.rs  │  │  types.rs            │  │
│  │ (OCCT    │  │  (ShapeData,         │  │
│  │  ops)    │  │   metadata)          │  │
│  └────┬─────┘  └──────────────────────┘  │
│       │                                  │
│       ▼                                  │
│  ┌──────────────────────────────────┐    │
│  │  opencascade-rs (opencascade)    │    │
│  │  └─ opencascade-sys (occt-sys)   │    │
│  │     └─ OCCT (C++ library)        │    │
│  └──────────────────────────────────┘    │
└──────────────────────────────────────────┘
```

The 3D viewer runs on a background thread (wgpu + winit + egui) and is compiled
out on macOS/iOS.  REPL ↔ viewer communication is via `mpsc` channels, with
shared state in a `ShapeRegistry` (RwLock + atomic generation counter).

## Dependencies & Licenses

| Dependency | License |
|------------|---------|
| [rojcad](.) (this project) | GPL-3.0-only |
| [OpenCASCADE](https://www.opencascade.com/) (OCCT) | LGPL-2.1 with exception |
| [opencascade-rs](https://github.com/bschwind/opencascade-rs) | LGPL-2.1 |
| [Janet](https://janet-lang.org/) (vendored) | MIT |
| [wgpu](https://github.com/gfx-rs/wgpu) | MIT / Apache-2.0 |
| [winit](https://github.com/rust-windowing/winit) | Apache-2.0 |
| [glam](https://github.com/bitshifter/glam-rs) | MIT / Apache-2.0 / Zlib |
| [thiserror](https://github.com/dtolnay/thiserror) | MIT / Apache-2.0 |
| [bytemuck](https://github.com/Lokathor/bytemuck) | Zlib / Apache-2.0 / MIT |
| [pollster](https://github.com/zesterer/pollster) | MIT / Apache-2.0 |

Full license texts are in [`licenses/`](licenses/) with a mapping in [`licenses/README.md`](licenses/README.md).

## License

GPL-3.0-only — see [LICENSE](LICENSE).
