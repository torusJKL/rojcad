## Why

The axis gizmo uses 2D flat quads for its axis shafts, which become invisible from certain camera angles (when the viewing direction aligns with the quad's normal). This makes the gizmo unreliable as a visual reference.

## What Changes

- Replace the 2D quad axis shafts with 3D cylinder (prism) meshes
- Add a small gray hub sphere at the origin to cover the cylinder junctions
- Keep sphere tips and letter labels unchanged
- No new user-facing capabilities — purely an internal rendering fix

## Capabilities

### New Capabilities

None — this is an internal implementation change with no spec-level requirements.

### Modified Capabilities

None.

## Impact

- `src/viewer/gizmo.rs`: Add `generate_cylinder()` function, replace `generate_line_quad()` call, add origin hub sphere
- `src/viewer/gizmo.wgsl`: No change needed (passthrough vertex color shader works for cylinders)
- Vertex count increases by ~250 (trivial for GPU)
