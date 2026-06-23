## 1. OCCT Wrapper Bindings

- [ ] 1.1 Add `GeomAbs_SurfaceType` enum and `BRepAdaptor_Surface` wrapper (new, GetType) in opencascade-sys + opencascade-rs
- [ ] 1.2 Add curve geometry access: `BRepAdaptor_Curve::Circle()` → `gp_Circ` → `.Radius()`, `.Position()` and `::Line()` → `gp_Lin` → `.Direction()`, `.Location()`
- [ ] 1.3 Add curve length utility via `GCPnts_AbscissaPoint`

## 2. Rust Core Extraction (cad.rs)

- [ ] 2.1 Add `FaceInfo` struct with Serialize and `face_info_json()` extraction function
- [ ] 2.2 Enrich `EdgeInfo` struct with `length`, `radius`, `axis`, `center` fields; update `edge_info_json()`
- [ ] 2.3 Implement `get_nth_face()` and `get_nth_edge()` using `TopTools_IndexedMapOfShape` for O(1) stable lookup
- [ ] 2.4 Add face operation wrappers: `face_offset()`, `face_fillet()`, `face_chamfer()`, `face_workplane()`

## 3. C Bridge + FFI (bridge.c + main.rs)

- [ ] 3.1 Add `_cad_face_info_raw` JANET_FN in bridge.c; add `rust_face_info` / `rust_face_info_free` in main.rs
- [ ] 3.2 Add `_cad_get_face` and `_cad_get_edge` JANET_FN; add `rust_get_face` and `rust_get_edge` FFI bridge functions
- [ ] 3.3 Add `_cad_face_offset`, `_cad_face_fillet`, `_cad_face_chamfer`, `_cad_face_workplane` JANET_FN
- [ ] 3.4 Add corresponding `rust_face_offset`, `rust_face_fillet`, `rust_face_chamfer`, `rust_face_workplane` FFI bridge functions
- [ ] 3.5 Register all new functions in `cad_register_functions()` function table

## 4. Janet Wrappers (boot.janet)

- [ ] 4.1 Add `face-info` Janet function (wraps `_face-info-raw` + JSON decode + struct conversion, following `edge-info` pattern)
- [ ] 4.2 Add `get-face` and `get-edge` Janet functions
- [ ] 4.3 Add `faces` and `edges` convenience functions (combine info query + extraction)
- [ ] 4.4 Add `face-offset`, `face-fillet`, `face-chamfer`, `face-workplane` Janet wrappers with keyword arguments
- [ ] 4.5 Add docstrings for all new functions in boot.janet

## 5. Tests

- [ ] 5.1 Add unit tests for `edge_info_json` enrichment in cad.rs (verify length, radius, axis fields)
- [ ] 5.2 Add unit tests for `face_info_json` in cad.rs (verify type, area, normal, axis, radius for different primitives)
- [ ] 5.3 Add unit tests for `get_nth_face` / `get_nth_edge` (valid indices, out-of-range, type checking)
- [ ] 5.4 Add REPL integration tests for `face-info` output on box, cylinder, sphere
- [ ] 5.5 Add REPL integration tests for `get-face` + extrude/revolve chaining
- [ ] 5.6 Add REPL integration tests for filter → extract → operate pattern (filter faces, get-face, extrude)
- [ ] 5.7 Add REPL integration tests for face operations (offset, fillet, chamfer, workplane)
- [ ] 5.8 Add REPL integration tests for error cases (non-shape argument, out-of-range index, wrong shape type)
