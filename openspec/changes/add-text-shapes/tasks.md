## 1. Setup

- [x] 1.1 Add `ttf-parser` dependency to `Cargo.toml`
- [x] 1.2 Create `src/text.rs` module skeleton with `pub mod text;` in `main.rs`

## 2. Font loading and glyph outline extraction

- [x] 2.1 Implement `FontData` struct + `load_font(path)` and `load_font_from_bytes(data)` in `text.rs`
- [x] 2.2 Implement glyph outline extraction via `ttf_parser::OutlineBuilder` — collect segments into contours split by `move_to`/`close`
- [x] 2.3 Implement layout loop: per-character glyph index, advance, with scaling and positioning

## 3. Contour-to-OCCT conversion

- [x] 3.1 Implement contour `→ OCCT Edge` conversion: `line_to` → `Edge::segment`, `quad_to` → `Edge::bezier(3)`, `curve_to` → `Edge::bezier(4)`
- [x] 3.2 Implement largest-contour-as-outer heuristic for hole detection
- [x] 3.3 Implement `text_to_face()`: convert all glyph contours to outer Wire + hole Wires, create Face via `Face::from_wire`, subtract each hole via `Face::subtract`

## 4. Extrusion and 3D text

- [x] 4.1 Implement `text_to_solid()`: wrap `text_to_face()`, extrude each glyph face, return Shape
- [x] 4.2 Support `:both` bidirectional extrusion in the internal API

## 5. System font listing

- [x] 5.1 Implement `list_system_fonts()` scanning standard OS font directories (`/usr/share/fonts`, `~/.fonts`, `/Library/Fonts`, `C:\Windows\Fonts`, etc.)
- [x] 5.2 Parse TTF/OTF name table for human-readable font name and aspect (regular/bold/italic)
- [x] 5.3 Return sorted, deduplicated list of `(name, path, aspect)` tuples

## 6. FFI bridge (Rust side)

- [x] 6.1 Add `extern "C"` declarations for text functions in `src/bridge.rs`
- [x] 6.2 Implement `rust_init_text()` FFI function in `src/main.rs`
- [x] 6.3 Implement `rust_init_text_extruded()` FFI function in `src/main.rs`
- [x] 6.4 Implement `rust_list_fonts()` FFI function in `src/main.rs` (returns serialized array)

## 7. C bridge (Janet side)

- [x] 7.1 Add `extern` forward declarations for Rust text functions in `bridge/bridge.c`
- [x] 7.2 Implement `cad_text` JANET_FN with `:depth`, `:plane`, `:at`, `:eager`, `:hide` keyword support
- [x] 7.3 Implement `cad_text3d` JANET_FN convenience wrapper
- [x] 7.4 Implement `cad_list_fonts` JANET_FN returning array of tuples
- [x] 7.5 Register `text`, `text3d`, `list-fonts` in `cad_register_functions` and `cad_fn_categories`

## 8. Workplane and positioning

- [x] 8.1 Reuse existing `workplane_from_keyword()` from `cad.rs` to support `:plane` and `:at` keywords in text placement
- [x] 8.2 Apply workplane transform to all glyph contour coordinates

## 9. Font path resolution fallback

- [ ] 9.1 Modify `FontData::from_path()` to try direct path first, then filename lookup in system font dirs, then font name lookup
- [ ] 9.2 Add unit tests for path resolution: bare filename, font name, and failure cases

## 10. Tests

- [x] 10.1 Add unit tests for glyph outline extraction (verify segment types and counts for known glyphs)
- [x] 10.2 Add unit test for `text_to_shape()` with a hole-containing glyph
- [x] 10.3 Add unit test for `text_to_solid()` (basic + both directions)
- [x] 10.4 Add unit test for empty string and invalid font error handling
- [x] 10.5 Run `just test` and `just lint` to verify nothing is broken
