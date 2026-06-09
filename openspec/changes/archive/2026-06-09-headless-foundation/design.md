## Context

rojcad is a greenfield parametric CAD system. The Rust binary embeds a Janet interpreter via the Janet C amalgamation (`janet.c` / `janet.h`), and links OCCT 8.0 through the `opencascade-rs` crate. The Janet interpreter runs the event loop, accepts REPL connections over TCP, and evaluates user s-expressions that call into Rust-registered CAD functions.

This design covers the headless foundation: project scaffolding, the Rust↔Janet↔OCCT bridge, the CAD API surface (box, sphere, cut, common), and the TCP REPL server.

## Goals / Non-Goals

**Goals:**
- A working Rust binary that embeds Janet and OCCT
- TCP REPL on port 9000 where the user can type CAD s-expressions
- `(make-box width depth height &keys :center)` and `(make-sphere radius &keys :center)` creating OCCT solids
- `(cut a b)` and `(common a b)` performing boolean operations
- Shape values are first-class Janet objects (abstract type) with metadata
- STEP and STL export
- GPLv3 license with all dependencies compatible
- Janet tuple `'(x y z)` for coordinate arguments
- Janet keywords `:center` for optional parameters

**Non-Goals:**
- No 3D viewer (deferred to a later phase with AIS)
- No fillet, chamfer, loft, sweep, or other complex operations
- No 2D sketching / workplane system
- No scene tree or named shape registry beyond Janet's `def`
- No WASM support
- No CI/CD pipelines (though Cargo workspace is set up for them)

## Decisions

### 1. Use opencascade-rs crate vs hand-roll OCCT FFI

**Decision:** Use `opencascade-rs` for the modeling layer.

**Rationale:**
- `opencascade-rs` provides a comprehensive Rust API for OCCT primitives, booleans, fillets, import/export, and mesh generation. Rolling our own FFI for each OCCT class is months of redundant work.
- It bundles and compiles OCCT from source via git submodule — saving us from build-system complexity.
- It handles the complex C++ ownership and ref-counting (`Handle<>`) correctly via cxx.rs.
- It exposes lower-level `occt-sys` if we need direct OCCT calls for things it doesn't wrap.
- TODO items we need to add on top: `Shape::translated()`, `Shape::rotated()` — these exist on Wire/Face but not Shape.

**Alternatives considered:**
- Hand-roll C++ wrappers + bindgen: More control but massive duplication of effort. Each OCCT class needs a C wrapper, Rust binding, and safe wrapper.
- Use `occt-sys` directly: Too low-level — we'd be writing unsafe FFI calls for every shape operation.

### 2. Janet C API via amalgamation vs janetrs crate

**Decision:** Use the raw Janet C amalgamation (`janet.c` / `janet.h`) compiled via `build.rs` + `cc` crate, with a C bridge (`bridge.c`) that handles Janet C API calls and delegates to Rust via `extern "C"`.

**Rationale:**
- Full control over the abstract type lifecycle, GC integration, and error handling.
- `janetrs` (v0.8.0) is still early-stage — abstract type support may be incomplete.
- The raw C API is well-documented, stable, and designed for embedding.
- The C bridge is thin (~20 functions), each following the same pattern: parse args → call Rust → wrap result.

**Alternatives considered:**
- `janetrs` crate: Higher-level Rust bindings, but may not expose the abstract type API we need for `ShapeData` metadata.
- Direct `extern "C"` from Rust to Janet: Possible but requires mirroring all Janet C types and macros in Rust — fragile.

### 3. Shape representation as Janet abstract type

**Decision:** Wrap a Rust `ShapeData` struct (containing an `opencascade::primitives::Shape` + visibility flag + optional color) as a Janet abstract type with a custom finalizer and `tostring` method.

**Rationale:**
- Shapes are first-class Janet values that can be `def`'d, passed to functions, stored in tables/arrays.
- Metadata (visibility, color) is part of the value — `show`/`hide` mutate it in place, the viewer picks it up on next frame.
- GC finalizer frees the Rust allocation when Janet collects the shape.
- `tostring` gives human-readable `#<Shape(SOLID)>` output.

**Key design:**
```rust
struct ShapeData {
    shape: Shape,           // OCCT TopoDS_Shape
    visible: bool,          // for future viewer
    color: Option<Rgb>,     // for future viewer
}
```

### 4. Keyword arguments versus positional center

**Decision:** Required args are positional, optional args use Janet keywords (`:center '(x y z)`, `:name "my-shape"`).

**Rationale:**
- `(make-sphere 10)` is the common case — no extra noise.
- `(make-sphere 10 :center '(1 2 3))` reads naturally when you need it.
- Janet's C API supports keyword parsing directly (`janet_getkwargs`).
- Extensible: new options don't break existing code.

**Alternatives considered:**
- All-positional with optional trail args: `(make-sphere 10 1 2 3)` — user hated this.
- Table-based options: `(make-sphere 10 {:center '(1 2 3)})` — allocates a table every call, wasteful for the common case.

### 5. TUPLES for coordinates

**Decision:** Use Janet tuples `'(x y z)` not arrays `@[x y z]` for coordinate positions.

**Rationale:**
- Coordinates are immutable value types — tuples are the semantically correct choice.
- Tuples are also passed by reference (not copied), so no performance penalty.
- Using mutable arrays for coordinates would allow accidental mutation after passing, which is a class of bugs.

### 6. TCP REPL via Janet's built-in networking

**Decision:** The REPL server is written in Janet (`boot.janet`) using `net/listen`, `ev/spawn`, and `repl`. No Rust networking code needed.

**Rationale:**
- `(repl stream)` is built into Janet — it reads s-expressions, evaluates, prints results.
- `ev/spawn` creates a green thread per connection — concurrent clients are free.
- The entire REPL server is ~15 lines of Janet.
- When a viewer arrives, the Janet event loop runs on a separate OS thread from the render loop.

### 7. License: GPLv3

**Decision:** GPLv3 for all rojcad code.

**Rationale:**
- Compatible with all dependencies: opencascade-rs (LGPL-2.1), OCCT itself (LGPL with exception), Janet (MIT), opencascade-sys (LGPL-2.1).
- GPLv3 is a strong copyleft license matching the spirit of the project.

### 8. Crate structure

**Decision:** Cargo workspace with three crates:

```
rojcad/                     # workspace root
├── Cargo.toml              # [workspace] members = ["crates/*"]
├── rust-toolchain.toml     # Rust edition 2024
├── vendor/
│   ├── janet.h             # Janet amalgamation header
│   └── janet.c             # Janet amalgamation source
├── bridge/
│   └── bridge.c            # C glue: Janet C API ↔ Rust CAD functions
├── boot.janet              # TCP REPL bootstrap (embedded via include_str!)
├── build.rs                # Compiles janet.c + bridge.c
├── src/
│   ├── main.rs             # Entry: init Janet, register fns, run boot, ev_loop
│   ├── bridge.rs           # Rust extern "C" declarations for bridge functions
│   ├── cad.rs              # Rust CAD ops wrapping opencascade-rs
│   └── types.rs            # ShapeData struct, abstract type helpers
└── openspec/
    └── changes/
        └── headless-foundation/
            ├── proposal.md
            ├── design.md
            ├── specs/
            │   ├── cad-primitives/spec.md
            │   ├── cad-booleans/spec.md
            │   ├── cad-inspection/spec.md
            │   ├── cad-export/spec.md
            │   └── janet-repl/spec.md
            └── tasks.md
```

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| opencascade-rs may have bugs or missing features for our use case | It's OSS with 243 stars and active development. We can patch or contribute fixes upstream. The lower-level `occt-sys` is always available as a fallback. |
| OCCT compilation is slow (10-15 min first build) | opencascade-rs caches the build. Only affects first `cargo build`. Subsequent builds are incremental. |
| OCCT build requires CMake and a C++ compiler | These are standard dev tools. Document in README. |
| Shape metadata (visible, color) must survive serialization? | Not an issue yet — we only need runtime metadata. File formats (STEP, STL) carry no metadata. |
| Single-threaded Janet VM blocks on long OCCT ops | For headless: irrelevant. For future viewer: Janet VM runs on its own thread, rendering on main thread. Channel-based queue for shape updates. |
| `opencascade-rs` doesn't have `Shape::translated()` / `Shape::rotated()` | We implement these ourselves using the OCCT BRepBuilderAPI_Transform, exposed through opencascade-rs's existing FFI or direct `occt-sys` calls. |
| C bridge (`bridge.c`) is unsafe C code | The bridge is thin (~20 functions), all following the same pattern. Unsafe is contained to argument parsing and abstract type wrapping. |

## Open Questions

- Should the OCCT build be integrated as a git submodule (opencascade-rs's approach) or should we require pre-installed OCCT? Decision: use opencascade-rs's builtin feature which bundles OCCT as a submodule.
- What Rust edition to target? 2024 is stable at time of writing (June 2026).
- Should `Shape::translated` be upstreamed to opencascade-rs or kept in our wrapper? For now: keep in our wrapper, consider upstreaming later.
