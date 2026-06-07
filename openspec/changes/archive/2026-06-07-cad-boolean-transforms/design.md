## Context

The system provides a Janet REPL that exposes CAD operations (box, sphere, cylinder, cone, torus, cut, common, inspection, export). Shapes are managed as `rojcad/shape` Janet abstract values backed by OCCT `Shape` objects, with thread-safe synchronization to a wgpu viewer via `ShapeRegistry`.

Currently missing: `fuse` (union) boolean operation, and any transformation operations (translate, rotate, scale, mirror). The `opencascade-rs` library wraps OCCT but does not expose `translated`/`rotated`/`scaled`/`mirrored` methods on `Shape` — only on the lower-level `Wire` type.

The transformer pattern is: `rust_init_*` (Rust FFI) → `cad::*` (CAD op) → `ShapeData::new` (registry registration) → tessellation. The C bridge (`bridge/bridge.c`) parses Janet arguments and wires them through.

## Goals / Non-Goals

**Goals:**
- Add `fuse` (union) boolean operation to the Janet REPL
- Add `translate`, `rotate`, `scale`, `mirror` transformation operations to the Janet REPL
- All operations create new shapes — original shapes are never modified
- Fork `opencascade-rs` to expose the needed transform methods on `Shape`
- Change `:a`/`:as`/`:ae` keyword semantics from radians to degrees (BREAKING)
- Add `:ar`/`:asr`/`:aer` keyword alternatives for radians
- Degrees are the default angle unit in the Janet API

**Non-Goals:**
- Non-uniform scaling (requires `BRepBuilderAPI_GTransform` — not needed yet)
- Shearing or perspective transforms
- In-place mutation operations
- Modifying the viewer or registry architecture
- Changing the Rust internal API (cad.rs stays in radians)

## Decisions

### D1: Fork opencascade-rs rather than work around API limitations

The `Shape` struct wraps a `pub(crate) inner: UniquePtr<TopoDS_Shape>`. This field and the `from_shape(&TopoDS_Shape)` constructor are crate-private. There is no way to:
- Extract `&TopoDS_Shape` from a `Shape`
- Construct a `Shape` from a raw `TopoDS_Shape`
- Clone a `Shape` (no `Clone` impl — `UniquePtr` is move-only)

Without access to the inner shape, we cannot use `BRepBuilderAPI_Transform` at all from our code. The cleanest solution is to add the four methods to `Shape` in `opencascade-rs` itself, following the exact pattern `Wire::transform()` and `Wire::mirror_along_axis()` already use.

Each method is ~8 lines: create `gp_Trsf`, configure it, call `BRepBuilderAPI_Transform_new(shape, &trsf, true)`, extract result via `Shape()`, wrap in `Shape::from_shape()`.

### D2: Use BRepBuilderAPI_Transform with copy=true (geometric transform)

OCCT offers two approaches for shape transformation:
- **`BRepBuilderAPI_Transform`** — bakes the transform into vertex coordinates (geometric). With `copy=true`, the original is untouched.
- **`TopLoc_Location`** — sets a persistent location annotation on the shape (efficient, but not baked into geometry).

We choose `BRepBuilderAPI_Transform` with `copy=true` because:
- STEP/STL export includes the transformed coordinates
- The result is a fully independent shape, not a reference to the original
- Consistent with OCCT's canonical transformation API
- Works identically for all transform types (translate, rotate, scale, mirror)

### D3: C bridge is the degrees↔radians conversion boundary

The conversion `deg *= PI / 180` happens in `bridge/bridge.c` after parsing `:a`/`:as`/`:ae` keyword values. The Rust layer (`cad.rs`) always receives radians. This keeps:
- Rust internal API consistent with OCCT (radians)
- Janet API user-friendly (degrees by default, `:ar` for radians)
- Conversion in one visible place

### D4: Rotate uses keyword-based axis specification

```
(rotate shape :a 45 :z)           ;; 45° about Z (most common)
(rotate shape :ar 0.785 :x)       ;; π/4 radians about X
(rotate shape :a 90 :r [1 1 0])   ;; 90° about custom axis
```

Keyword axis (`:x`/`:y`/`:z`/`:r`) rather than positional avoids confusion about argument order and matches the existing keyword pattern in `bridge/bridge.c`. Cardinal axes are common enough to warrant dedicated keywords.

### D5: Scale uses `:o` for center point, defaults to origin

```
(scale shape 2.0)                  ;; 2× about (0,0,0)
(scale shape 2.0 :o [5 10 0])      ;; 2× about custom point
```

The `gp_Trsf::SetScale` OCCT function takes a point and a factor. Defaulting to origin when `:o` is omitted is the most natural behavior.

## Risks / Trade-offs

- **[Risk] Fork maintenance**: The opencascade-rs fork adds a maintenance burden. Mitigation: the 4 methods are small (~30 lines total) and self-contained. Upstreaming is possible as a PR.
- **[Risk] Breaking change — `:a` semantic shift**: Existing Janet scripts using `:a` with radian values will silently produce wrong results. Mitigation: this is an embedded DSL controlled by us; we can update boot.janet examples. The `:ar` keyword provides backward migration.
- **[Risk] OCCT compilation**: The fork changes only Rust wrapper code, not C++ headers. OCCT compilation is cached — only Rust recompilation is needed on subsequent builds.
- **[Trade-off] Degrees by default**: CAD users prefer degrees; OCCT uses radians. The C bridge conversion adds a tiny maintenance surface but greatly improves usability.
- **[Trade-off] BRepBuilderAPI_Transform vs TopLoc_Location**: Geometric transform means recomputing the mesh. For repeated transformations, this is slower than location-based transforms. Acceptable for a REPL-driven workflow.
