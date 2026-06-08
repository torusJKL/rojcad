## Context

rojcad currently supports 3D primitives (box, sphere, cylinder, cone, torus) and 2D primitives (rect, circle, polygon) via `opencascade-rs`. Text shapes require: (1) loading a TrueType/OpenType font, (2) extracting glyph outlines as bezier curves, (3) converting those outlines to OCCT edges/wires/faces, and (4) handling inner contours (holes) via boolean subtract.

No changes to `opencascade-rs` are needed — the boolean subtract approach (`Face::subtract(&Face) → CompoundFace`) is available in the current API.

## Goals / Non-Goals

**Goals:**
- Add `(text ...)` and `(text3d ...)` Janet functions for 2D/3D text
- Add `(list-fonts)` for system font discovery
- Support `:plane`, `:at`, `:depth`, `:eager`, `:hide` keywords
- Use `ttf-parser` for glyph outline extraction (already a transitive dep)
- Handle glyph holes via boolean subtract

**Non-Goals:**
- No RTL/BiDi text support
- No OpenType shaping (ligatures, contextual alternates)
- No multi-line text layout
- No vertical text
- No font subsetting or embedding
- No changes to opencascade-rs

## Decisions

### Decision 1: ttf-parser over ab_glyph for outline extraction

`ttf-parser` exposes explicit `move_to`/`close` events in its `OutlineBuilder` trait, which are essential for splitting glyph outlines into separate contours and detecting holes. `ab_glyph` omits `move_to`/`close` from its `OutlineCurve` enum (issue #82), making contour reconstruction fragile.

The layout and kerning logic is simple enough to implement manually on top of `ttf-parser`.

### Decision 2: Boolean subtract for holes over CXX bridge change

`Face::subtract(&Face)` is available in the current opencascade-rs API. Adding a CXX bridge for `BRepBuilderAPI_MakeFace::Add(wire)` would be cleaner but requires forking or vendoring opencascade-rs. Boolean subtract is sufficient for v1 and proven by the existing `letter_a.rs` example.

### Decision 3: File path as primary FFI argument

Consistent with all existing Janet CAD functions. Font bytes support is exposed at the Rust internal API level for embed scenarios, but the Janet API takes a file path string.

### Decision 4: Largest-area contour heuristic for outer/hole detection

Each glyph's contours are collected via `ttf-parser::OutlineBuilder`. The contour with the largest bounding box area is treated as the outer boundary; all others are holes. This works for Latin, Cyrillic, Greek, CJK, and most scripts. Some decorative fonts may fail — the user can work around with manual shape construction.

### Decision 5: OS font directory scanning for list-fonts

Pure Rust directory traversal is simpler than binding to fontconfig/CoreText/DirectWrite. Scans well-known OS font directories recursively for `*.ttf`, `*.otf`, `*.ttc` files, parses the `name` table for the human-readable font name and aspect.

## Risks / Trade-offs

- **[Boolean subtract fragility]**: OCCT boolean operations may fail on degenerate geometry — very thin strokes, near-tangent boundaries. **Mitigation**: Wrap in `catch_unwind`, surface error to user at Janet level with actionable message.
- **[Performance on complex CJK text]**: Each glyph with N holes requires N boolean subtract operations. A CJK character with 20+ holes would be slow. **Mitigation**: Acceptable for v1; revisit with CXX bridge if performance is reported.
- **[Font discovery misses some fonts]**: Directory scanning won't find fonts managed by fontconfig but not in standard directories. **Mitigation**: Users can always use explicit font file paths.
- **[TTF name table parsing]**: Font name extraction from TTF `name` table is locale-dependent. **Mitigation**: Take first valid English (platform=3, encoding=1) name entry, fall back to first available.
