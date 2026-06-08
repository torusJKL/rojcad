## Why

rojcad currently only supports 3D solid primitives (box, sphere, cylinder, cone, torus). Real CAD workflows require sketching 2D profiles and extruding/revolving them into 3D solids. This is the single biggest capability gap — every parametric CAD system needs this as a foundation.

## What Changes

- Add 2D primitives: `rect`, `circle`, `polygon` — return Face (or Wire with `:wire`)
- Add freeform sketch builder: `sketch`, `move-to`, `line-to`, `line-dx`, `line-dy`, `line-dx-dy`, `arc-to`, `close-sketch`, `build-wire` — pure threading, no mutation
- Add extrusion: `extrude` on a Face → Solid, along face normal or custom direction
- Add revolution: `revolve` on a Face → Solid, about an axis
- Add wire operations: `wire-to-face`, `wire-fillet`, `wire-chamfer`, `wire-offset`
- Add one-shot `extrude-polygon` for point-list → Solid
- Add helper queries: `wire?`, `face?`, `solid?`
- Update Cargo.toml to point opencascade-rs to the `temp-merge` branch (adds `Shape::as_wire`, `as_face`, `as_solid` downcasts)

## Capabilities

### New Capabilities
- `2d-primitives`: Rect, circle, and polygon creation from workplane coordinates; returns Face or Wire
- `sketch`: Freeform 2D profile builder with move-to, line-to, arc, close — functional thread-safe API
- `wire-operations`: Wire-to-face conversion, fillet, chamfer, and offset on existing wires
- `extrusion`: Extrude a Face along normal or custom direction, with `:both` option
- `revolution`: Revolve a Face about an axis with angle control

### Modified Capabilities

- (none)

## Impact

- **New file**: `src/sketch.rs` — `SketchData` struct and functional sketch operations
- **Modified files**: `src/cad.rs`, `src/main.rs`, `src/bridge.rs`, `bridge/bridge.c`
- **Dependency**: opencascade-rs branch `translate-copy` → `temp-merge` (adds downcast methods, no breaking changes)
- **Viewer**: No viewer changes needed — `ShapeData` already handles FACE and WIRE types in tessellation and edge rendering
- `Cargo.lock` will update with new `paste` dependency (transitive via opencascade-rs)
