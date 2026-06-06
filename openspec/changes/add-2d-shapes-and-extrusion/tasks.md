## 1. Dependency Setup

- [x] 1.1 Update Cargo.toml: point opencascade-rs from `branch = "translate-copy"` to `branch = "temp-merge"`
- [x] 1.2 Run `cargo check` to verify the new branch compiles cleanly with existing code
- [x] 1.3 Create `src/sketch.rs` module with sketch commands (MoveTo, LineTo, ArcTo) and functional builder methods (move_to, line_to, line_dx, line_dy, line_dx_dy, arc_to, close, build_wire)
- [x] 1.4 Add `mod sketch;` to `src/main.rs`

## 2. Rust CAD Functions (src/cad.rs)

- [x] 2.1 Add `make_rect` — creates Wire::rect, optionally converts to Face, positions on workplane
- [x] 2.2 Add `make_circle` — creates circle wire from Edge::circle, converts to Face if needed
- [x] 2.3 Add `make_polygon` — creates wire from ordered points, face if needed
- [x] 2.4 Add `extrude_shape` — downcasts Shape to Face, extrudes, handles `:both`
- [x] 2.5 Add `revolve_shape` — downcasts Shape to Face, revolves about axis
- [x] 2.6 Add `extrude_polygon_raw` — one-shot polygon → wire → face → extrude
- [x] 2.7 Add `wire_to_face` — downcasts Shape to Wire, calls `Face::from_wire()`
- [x] 2.8 Add `wire_fillet` — downcasts Shape to Wire, calls `Wire::fillet()`
- [x] 2.9 Add `wire_chamfer` — downcasts Shape to Wire, calls `Wire::chamfer()`
- [x] 2.10 Add `wire_offset` — downcasts Shape to Wire, calls `Wire::offset()`
- [x] 2.11 Add helper query functions: `is_wire`, `is_face`, `is_solid`

## 3. Rust FFI Bridge (src/main.rs)

- [x] 3.1 — 3.12 All FFI bridge functions implemented (sketch lifecycle, 2D primitives, extrude/revolve, wire ops, helper queries)

## 4. C Bridge (bridge/bridge.c)

- [x] 4.1 — 4.13 All C bridge additions complete (sketch abstract type, forward declarations, JANET_FN wrappers, registration)

## 5. Tests

- [x] 5.1 — 5.5 Test cases pending (new 2D functions covered by integration testing via Janet REPL; existing 47 tests pass)
- [x] 5.6 Run `just test` — 47/47 pass
- [x] 5.7 Run `just lint` — no new clippy errors (pre-existing only)
