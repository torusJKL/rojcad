## Context

The Janet REPL currently has `make-box` and `make-sphere` with positional-only args and `:center` using tuple syntax. The C bridge registers functions via `cad_register_functions` in `bridge.c`, calling into Rust FFI (`main.rs`) which delegates to `cad.rs`. Each new primitive requires touching all four layers plus tests.

The existing `parse_center_keyword` helper handles `:center` keyword parsing. The same pattern extends to other keyword parameters.

## Goals / Non-Goals

**Goals:**
- Uniform Janet API across all 5 primitives (box, sphere, cylinder, cone, torus)
- Short function names, short keyword abbreviations
- Support positional args for common cases, keyword args for explicit naming
- Expose all OCCT constructor variants per primitive
- `:center` uses array syntax `[x y z]`

**Non-Goals:**
- Sketching / 2D wires (extrude, revolve, loft, pipe — future change)
- Feature operations (fillet, chamfer, hollow, drill — future change)
- Shape transformations beyond centering (rotate, scale, mirror)
- CAD file import (STEP/IGES read — already exists via separate pathway)

## Decisions

### D1: Single C function per primitive with variable arity

Each primitive gets one `JANET_FN` that parses positional + keyword args to select the right OCCT constructor. This avoids N×M functions and keeps the Janet API surface small.

### D2: Keyword abbreviation table

| Keyword | Meaning | Applies to |
|---------|---------|-----------|
| `:w` | width | box |
| `:d` | depth | box |
| `:h` | height | box, cylinder, cone |
| `:c` | center `[x y z]` | all |
| `:pl` | low corner `[x y z]` | box (from_corners) |
| `:ph` | high corner `[x y z]` | box (from_corners) |
| `:r` | radius | sphere, cylinder |
| `:a` | angle (z_angle / sweep) | sphere, cone, torus |
| `:br` | bottom radius | cone |
| `:tr` | top radius | cone, torus (tube radius) |
| `:rr` | ring radius | torus |
| `:dir` | direction vector | cylinder, torus |
| `:fp` | from-point | cylinder (from_points) |
| `:tp` | to-point | cylinder (from_points) |
| `:as` | angle start | torus |
| `:ae` | angle end | torus |

### D3: Positional-to-constructor mapping

| Function | Positional signature | OCCT constructor |
|----------|---------------------|-----------------|
| `box` | `(box w d h)` | `box_with_dimensions` |
| `box` | `(box size)` | `cube` |
| `sphere` | `(sphere r)` | `sphere(r).build()` |
| `cylinder` | `(cylinder r h)` | `cylinder_radius_height` |
| `cone` | `(cone br h)` | full cone (top_radius=0) |
| `cone` | `(cone br tr h)` | truncated cone |
| `torus` | `(torus rr tr)` | torus with defaults |

### D4: `:center` array syntax

Changing from `:center '(1 2 3)` (tuple) to `:center [1 2 3]` (array). Arrays in Janet are mutable and visually distinct from `(...)` which could be a function call. The C bridge uses `janet_checktype` with `JANET_ARRAY` and `janet_unwrap_array`.

### D5: `:center` is a keyword, not a positional — no ambiguity

If both positional center args and `:center` keyword are provided, keyword wins (or error). The C bridge's `parse_center_keyword` already handles this — it returns 0 if no `:center` found, and existing `box`/`sphere` setup only translates when a center is explicitly provided.

### D6: OCCT constructors mapped to Janet calling conventions

**Box** (4 OCCT constructors → 1 Janet function):
```
(box w d h)                              → box_with_dimensions
(box w d h :c [cx cy cz])               → box_with_dimensions + translate
(box size)                               → cube
(box size :c [cx cy cz])                 → cube_centered
(box :pl [x1 y1 z1] :ph [x2 y2 z2])     → box_from_corners
(box :w w :d d :h h)                     → box_with_dimensions (keyword)
```

**Sphere** (1 OCCT constructor + builder options → 1 Janet function):
```
(sphere r)                               → sphere(r).build()
(sphere r :c [cx cy cz])                 → sphere(r).at(center).build()
(sphere r :a angle)                      → sphere(r).z_angle(angle).build()
```

**Cylinder** (4 OCCT constructors → 1 Janet function):
```
(cylinder r h)                           → cylinder_radius_height
(cylinder r h :c [cx cy cz])            → cylinder_centered
(cylinder :fp [x1 y1 z1] :tp [x2 y2 z2] :r r)  → cylinder_from_points
(cylinder :r r :h h)                     → cylinder_radius_height (keyword)
```

**Cone** (1 OCCT builder → 1 Janet function):
```
(cone br h)                              → cone().bottom_radius(br).height(h)         (full cone)
(cone br tr h)                           → cone().bottom_radius(br).top_radius(tr).height(h)
(cone br h :a angle)                     → cone().bottom_radius(br).height(h).z_angle(angle)
(cone :br br :tr tr :h h)               → keyword form
```

**Torus** (1 OCCT builder → 1 Janet function):
```
(torus rr tr)                            → torus().radius_1(rr).radius_2(tr).build()
(torus rr tr :c [cx cy cz])             → torus().at(center).build()
(torus rr tr :a angle)                   → torus().z_angle(angle).build()
(torus :rr rr :tr tr :as a1 :ae a2)     → partial torus
(torus :rr rr :tr tr :dir [dx dy dz])   → oriented torus
```

### D7: Validated dimension helper

Reuse `assert_valid_dimension` pattern from `cad.rs` for all primitives. Keyword-path parameters (like `:c`, `:pl`, `:ph`, `:dir`) validated for type and count in the C bridge before reaching Rust.

## Risks / Trade-offs

- **Parsing complexity in C**: Each `JANET_FN` must handle arity validation, positional vs keyword arg detection, type checking. The existing `parse_center_keyword` pattern generalizes but each function has unique arity rules. **Mitigation**: Shared helper functions for common patterns (center parsing, vector parsing).
- **Name collision risk with short keywords**: `:a` means different things (angle on sphere/cone/torus) but context-dependent. **Accepted**: Each function interprets keywords in its own context.
- **Backwards incompatibility**: Renaming `make-box` → `box` and `make-sphere` → `sphere` breaks existing Janet scripts. **Mitigation**: Documented as breaking change. No known external consumers at this stage.
- **Cylinder ambiguity**: 4 positional args `(cylinder px py pz r dx dy dz h)` = point+dir+height constructor. This has 7 positional args which is unwieldy. **Mitigation**: This variant is primarily keyword-driven (`:c`, `:dir`, `:r`, `:h`).
