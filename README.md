# rojcad

> Headless parametric CAD system with embedded Janet DSL.

**rojcad** is a parametric CAD system that embeds a [Janet](https://janet-lang.org/) interpreter with [OpenCASCADE](https://www.opencascade.com/) modeling via [opencascade-rs](https://github.com/bschwind/opencascade-rs). It provides a TCP REPL server — connect with `nc` and start modeling in s-expressions.

## Features

- **CAD Primitives**: `(make-box width depth height)` and `(make-sphere radius)` with optional `:center` positioning
- **Boolean Operations**: `(cut a b)` and `(common a b)` for shape subtraction and intersection
- **Shape Inspection**: `(shape-type s)`, `(visible? s)`, `(hide s)`, `(show s)`
- **Export**: `(write-step s "path.step")` and `(write-stl s "path.stl")`
- **TCP REPL**: Connect via `nc 127.0.0.1 9000` for an interactive modeling session

## Build Prerequisites

- **Rust toolchain** (install via [rustup.rs](https://rustup.rs/))
- **CMake** 3.5+ (for building OCCT)
- **C++ compiler** with C++11 support (gcc, clang, or MSVC)

On Debian/Ubuntu:

```bash
sudo apt install build-essential cmake
```

On macOS:

```bash
xcode-select --install
brew install cmake
```

## Build

```bash
# Clone with submodules (OCCT)
git clone --recursive https://github.com/your-org/rojcad.git
cd rojcad

# Or if already cloned:
git submodule update --init --recursive

# Build (first build compiles OCCT from source, takes 10-15 min)
cargo build --release
```

> **Note**: The first build compiles the full OpenCASCADE library from source (via `opencascade-rs`'s `builtin` feature). Subsequent builds are incremental and much faster.

## Usage

### Start the server

```bash
cargo run --release
```

You should see:
```
◆ rojcad ready — connect via: nc 127.0.0.1 9000
```

### Connect and model

```bash
nc 127.0.0.1 9000
```

Then in the REPL:

```janet
# Create a box 10×20×30mm
(def b (make-box 10 20 30))
# => #<Shape(SOLID)>

# Create a sphere centered at (5, 10, 0)
(def s (make-sphere 15 :center '(5 10 0)))
# => #<Shape(SOLID)>

# Subtract the sphere from the box
(def result (cut b s))
# => #<Shape(SOLID)>

# Check shape type
(shape-type result)
# => :solid

# Export
(write-step result "result.step")
(write-stl result "result.stl")

# Inspect visibility
(visible? result)
# => true
(hide result)
(visible? result)
# => false
(show result)
```

### Multiple clients

Multiple `nc` connections can be active simultaneously — each gets an independent REPL session.

## Architecture

```
┌──────────────────────────────────────────┐
│           rojcad binary                   │
│  ┌──────────┐  ┌──────────────────────┐  │
│  │  main.rs  │  │  boot.janet          │  │
│  │  (entry)  │  │  (TCP REPL server)   │  │
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
│  │  └─ opencascade-sys (occt-sys)  │    │
│  │     └─ OCCT (C++ library)       │    │
│  └──────────────────────────────────┘    │
└──────────────────────────────────────────┘
```

## License

GPLv3 — see [LICENSE](LICENSE).
