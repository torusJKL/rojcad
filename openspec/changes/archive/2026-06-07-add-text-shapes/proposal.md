## Why

rojcad currently has no way to create text-based geometry. Users must manually construct letter shapes as polygons or import from STEP files. Adding font-rendered text shapes unblocks labels, signage, embossing, engraving, and parametric text parts — directly within the Janet DSL.

## What Changes

- New `text` Janet function that renders a string as a 2D Face from a TTF/OTF font file, with optional extrusion depth for 3D text
- New `text3d` Janet convenience function that renders extruded 3D text in one call
- New `list-fonts` Janet function that discovers and returns available system fonts
- Internal font → geometry pipeline using `ttf-parser` for glyph outline extraction and OCCT boolean subtract for hole (inner contour) support
- Internal font discovery via scanning standard OS font directories

## Capabilities

### New Capabilities

- `text-shapes`: Create 2D and 3D text shapes from TrueType/OpenType fonts, list system fonts, with workplane and positioning support

### Modified Capabilities

<!-- No existing capabilities are modified -->

## Impact

- **New dependency**: `ttf-parser` crate added to `Cargo.toml`
- **No changes to opencascade-rs** — boolean subtract approach uses existing API
- **New module**: `src/text.rs` with font loading, glyph outline extraction, contour-to-Edge conversion, and face construction
- **Bridge additions**: 2 new Janet functions (`text`, `text3d`) and 1 helper (`list-fonts`) in `bridge/bridge.c` + `src/main.rs`
