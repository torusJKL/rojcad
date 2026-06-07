## Context

rojcad exposes CAD operations to a Janet REPL via a 3-layer bridge: Janet → C (`bridge/bridge.c`) → Rust FFI (`src/main.rs`, `src/bridge.rs`) → CAD logic (`src/cad.rs`) → OCCT (`opencascade-rs`). Currently only 3D solid primitives, boolean ops, and transforms exist.

The `opencascade-rs` crate already provides all the 2D infrastructure (Workplane, Sketch, Wire, Face, extrusion, revolution) in its Rust API, but none of it is exposed to Janet. The `temp-merge` branch additionally provides safe downcasts (`Shape::as_wire()`, `as_face()`, `as_solid()`).

## Goals / Non-Goals

**Goals:**
- Expose 2D primitives, freeform sketch, extrusion, revolution, and wire operations to Janet
- Match existing keyword conventions (`:c`, `:dir`, `:a`/`:ar`, `:eager`, `:hide`)
- Support both REPL interactive use and file-based script evaluation
- Require zero viewer changes (WIRE and FACE types already render correctly)

**Non-Goals:**
- 2D face/fillet/chamfer on wires (covered by `wire-fillet`, `wire-chamfer`)
- Compound face operations (multiple disconnected faces extruded together)
- Parametric constraints or dimension-driven sketching
- Import/export of 2D profiles (STEP/IGES already handle all shape types)

## Decisions

### 1. Sketch as a new abstract type (`rojcad/sketch`)
- **Decision**: Create a `rojcad/sketch` GC-managed abstract type in `bridge.c`, backed by a Rust `SketchData` struct in `src/sketch.rs`
- **Why**: The sketch accumulates OCCT `Edge` objects in a `Vec<Edge>` as the user draws. These edges are OCCT handles that need proper lifecycle management (drop via GC finalizer). A Janet table can't manage this.
- **Alternatives considered**: Storing edges in a Janet array — rejected because edges are OCCT objects, not Janet values.

### 2. Pure threading (functional, no mutation)
- **Decision**: Each sketch operation (`move-to`, `line-to`, etc.) consumes `self` and returns a new `SketchData` with one more edge
- **Why**: Threading via Janet's `->` macro makes sketches composable in one expression. REPL use still works via `(set sk (line-to sk 10 0))`. No hidden mutation.
- **Cost**: N intermediate `SketchData` allocations for N operations. Negligible compared to OCCT geometry computation.

### 3. Workplane: explicit with fallback
- **Decision**: All 2D creation functions accept `:plane` keyword (e.g., `:xy`, `:xz`, `:yz`) and `:at` for offset. Defaults to XY at origin.
- **Why**: Predictable for file evaluation (no hidden global state). Explicit is better than implicit in a scripting context.

### 4. Direct downcast via `temp-merge` branch
- **Decision**: Update Cargo.toml to `branch = "temp-merge"` which provides `Shape::as_wire()`, `as_face()`, `as_solid()`
- **Why**: Avoids the edges→collect→reconnect→wire workaround which, while functionally correct, is more code and has a small performance tax.

### 5. Reuse existing ShapeData for Wire/Face/Solid
- **Decision**: Faces and Wires are stored in the same `ShapeData` type as Solids. The `shape-type` function already returns `:wire` and `:face`. The viewer already handles both.
- **Why**: No type proliferation. The tessellator works on Faces (produces mesh); edge extraction works on any shape type.

### 6. Extrusion direction: face normal by default
- **Decision**: `(extrude face :h 20)` extrudes along the face's normal vector. `:z`/`:x`/`:y` overrides to cardinal axes. `:dir [dx dy dz]` for custom.
- **Why**: Face normal is the most intuitive default — it "pulls" the face outward in 3D space. OCCT's `BRepPrimAPI_MakePrism` handles this naturally.

### 7. Revolve naming
- **Decision**: `:a` for degrees, `:ar` for radians (matching `rotate`). `:c` for axis point (matching `:c` on box/sphere/cylinder). `:dir` for axis direction (matching `:dir` on cylinder/torus).
- **Why**: Consistency with existing API keywords.

## Risks / Trade-offs

- **[Low] `temp-merge` branch instability**: This is a working branch, not a release. If upstream merges it to `main` or removes it, we update the Cargo.toml reference. The downcast methods are trivially simple (casts + constructor calls), so we could vendor them locally if needed.
- **[Low] OCCT edge iterator ordering**: `Shape::edges()` via `TopExp_Explorer` returns edges in topological order, which for a manifold face boundary is naturally connected sequence. The wire reconstruction fallback (without temp-merge) uses `ConnectEdgesToWires` which handles arbitrary ordering.
- **[None] Viewer compatibility**: No risk — WIRE types render as edge polylines, FACE types render as meshes with wireframe overlay. Both work.
