## Why

Code-based parametric CAD is powerful for version control, automation, and reproducibility. Existing tools like ClotCAD (Common Lisp + OCCT) demonstrate the value of a Lisp-based DSL for CAD, but are tied to the SBCL runtime. A Rust-hosted equivalent with an embedded Janet DSL gives us the safety and ecosystem of Rust, the embeddable lightness of Janet (<1MB runtime), and a familiar s-expression syntax — all while being a single small binary the user can nc into.

## What Changes

- Create a new Rust binary (`rojcad`) that embeds the Janet interpreter
- Register CAD primitives (box, sphere) and boolean operations (cut, common) as Janet functions
- Ship a `boot.janet` script that starts a TCP REPL server on port 9000
- Wrap OCCT shapes as Janet abstract types (`#<Shape(SOLID)>`) with metadata (visibility)
- Use Janet's built-in `def` for shape binding — no separate name registry
- Use Janet tuples for coordinate arguments (`'(1 2 3)`)
- Use Janet keyword arguments for optional parameters (`:center`)
- Build on top of `opencascade-rs` for OCCT modeling
- Build on top of Janet's C amalgamation (`janet.c` / `janet.h`) for the interpreter
- Link OCCT with the `TKService` / `TKV3d` / `TKOpenGl` modules compiled but inert (for future AIS viewer)
- Export models via STEP and STL
- License: GPLv3

## Capabilities

### New Capabilities
- `cad-primitives`: Creation of 3D primitives — box and sphere, with positional dimensions and optional keyword-based center position.
- `cad-booleans`: Boolean operations on shapes — cut (subtract) and common (intersect).
- `cad-inspection`: Query shape type and edges/faces, check visibility state.
- `cad-export`: Export shapes to STEP and STL file formats.
- `janet-repl`: TCP REPL server on port 9000 using Janet's networking and event loop, accepting multiple concurrent client sessions.

### Modified Capabilities

*None — new project, no existing capabilities.*

## Impact

- **New crate**: `rojcad` binary crate, `rojcad-core` library crate, and `rojcad-bridge` C → Rust FFI layer.
- **Build dependencies**: `opencascade-rs` (LGPL-2.1, compiled alongside), Janet amalgamation (`janet.c`), C compiler toolchain via `cc` crate + `build.rs`.
- **Runtime dependencies**: OCCT shared libraries, single-threaded Janet VM.
- **No existing code affected** — greenfield project.
