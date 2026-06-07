## Context

The gizmo renders axis shafts as flat quads (two triangles each) with zero thickness in one perpendicular direction. When the camera aligns with that direction, the quad becomes edge-on and invisible. The gizmo also renders sphere tips (14×8 tessellation) and letter labels (X/Y/Z).

The shader (`gizmo.wgsl`) is a simple passthrough — vertex position transformed by orthographic view-projection, vertex color output directly. No lighting, no normals.

## Goals / Non-Goals

**Goals:**
- Axis shafts are always visible from any camera angle
- Match the current visual style (unlit, faceted, consistent with sphere tips)
- Match current line thickness (`LINE_WIDTH = 0.055` half-width)
- Add a gray hub sphere at origin to cleanly cover cylinder junctions

**Non-Goals:**
- Changing the shader or adding lighting
- Modifying the sphere tips or letter labels
- Changing hit-testing behavior
- Performance optimization beyond what's needed

## Decisions

**Decision 1: Cylinder (prism) mesh instead of box or crossed quads**

A cylinder with N=14 segments (matching `SPHERE_LON`) gives a faceted look consistent with the existing spheres. At typical gizmo viewport size (~200px), the facets aren't individually discernible — it reads as a smooth rod.

Alternatives considered:
- **Flat quad (current)**: Invisible from certain angles. Rejected.
- **Crossed quads**: Still has a vanishing angle (at 45° between quads). Rejected.
- **Rectangular box**: 8 verts, 12 tris. Simplest but looks blocky. Fine but cylinder is cleaner.

```
     Top view of N=14 prism:
     
            ┌──┐
          ╱    │ ╲
         │     │  │
         │  ●  │  │   always presents a face toward camera
         │     │  │
          ╲    │ ╱
            └──┘
```

**Decision 2: Body-only cylinders (no end caps)**

The tip sphere covers the far end of each cylinder completely. The origin hub sphere covers the near end. No cap triangles needed, saving 6N vertices per axis.

```
      gray hub ──●═══════════════●── red tip sphere
                 │   body only    │
                 │  (no caps)     │
```

**Decision 3: Gray hub sphere at origin**

A small sphere (`radius = LINE_WIDTH = 0.055`) using the same `generate_sphere()` function, placed at `Vec3::ZERO`. Color: medium gray (e.g. `[0.4, 0.4, 0.4, 1.0]`). Same tessellation as tip spheres.

This handles the origin junction cleanly instead of leaving three exposed cylinder ends.

**Decision 4: Reuse existing vertex format and shader**

`GizmoVertex` (position + color) and `gizmo.wgsl` work unchanged. Cylinder vertices just need position and color — no normals, no UVs. The faceted prism look is intentional and consistent.

**Decision 5: Generate cylinder in `build_static_vertices()`**

The cylinder replaces the quad in the same function. No structural pipeline changes needed.

## Risks / Trade-offs

- **~250 additional vertices** (252 for 3 cylinders + ~41 for hub sphere). Negligible — the spheres alone use 2016 verts.
- **Cylinder segments visible at very close zoom** — N=14 creates a 14-sided prism, not a smooth cylinder. Acceptable for a gizmo. Increase to N=20 if faceting is objectionable.
- **No rollback needed** — this is additive rendering, no state migration or data change.
