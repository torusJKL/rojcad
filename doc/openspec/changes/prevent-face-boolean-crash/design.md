## Context

`BRepAlgoAPI_Cut` crashes with `SIGSEGV` when both operands are coplanar `Face` shapes. OCCT source investigation (`BOPAlgo_BOP.cxx:BuildRC`) reveals the root cause: `Cut` must build a face with an inner wire (hole) for coplanar Face-Face input, while `Fuse` and `Common` only gather or filter existing splits — they never need to insert holes. The crash is specific to `Cut`. The guard must be at the Rust layer (`cad.rs`) to catch it before reaching opencascade-rs.

The relevant function is `cad::cut` in `src/cad.rs:361`. The guard calls `a.shape.shape_type()` which is already available. Helper queries `is_face` and `is_solid` exist at `cad.rs:1132-1140`. `common` and `fuse` are NOT guarded — they pass Face-Face input through to OCCT without issue.

## Goals / Non-Goals

**Goals:**
- Prevent `SIGSEGV` when users pass two planar faces to `cut`
- Return a clear, actionable error message: which operation failed, why, and how to fix it (extrude faces to solids)
- Tests verify the guard fires for Face-Face cut and passes for mixed/solid input
- **Not** guarding `common` or `fuse` — OCCT investigation confirmed they handle Face-Face correctly

**Non-Goals:**
- Fix the upstream OCCT/open-cascade-rs issue (deferred to upstream feature request)
- Add `BRepBuilderAPI_MakeFace::Add` or any hole-creation API (separate change)
- Handle mixed Face-Solid input (these work fine in OCCT and are unaffected)
- Change the C bridge, Janet API, or any user-facing signatures

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Guard location | `cad.rs` Rust layer (before opencascade-rs) | Catches it before any C++ FFI call; `catch_cad` in `main.rs` already returns errors as Janet panics |
| Which ops are guarded | `cut` only | OCCT source confirmed `BOPAlgo_BOP::BuildRC` — Fuse/Common handle Face-Face via simple split gather, only Cut needs hole creation |
| Check condition | Both `shape_type() == ShapeType::Face` | Mixed Face-Solid works fine; only coplanar Face-Face Cut triggers the crash |
| Error message format | `"cut: cannot cut two planar faces. Extrude one or both to solids first (e.g. with `prism`)"` | Tells user the operation, the problem, and the fix in one line |

Rejected alternatives:
- **C bridge check (`bridge.c`)**: Duplicates logic already available in Rust; the Rust error flows cleanly through `catch_cad` → `janet_panic`
- **Janet wrapper check**: Would need to expose shape-type query at every call site, more invasive and harder to maintain
- **Guarding `common` and `fuse`**: Unnecessary — OCCT C++ source shows they handle Face-Face correctly

## Risks / Trade-offs

- **False positive risk** → None: non-coplanar Face-Face also crashes in Cut, so the guard is always correct for Face-Face. The condition is conservative and safe.
- **Missed crash paths** → Mitigation: `Face::subtract` in opencascade-rs also calls `BRepAlgoAPI_Cut` and could crash if called directly. rojcad doesn't currently expose it, but worth noting.
- **User confusion** → Mitigation: error message is explicit about both the cause and the workaround.
