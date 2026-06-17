## 1. Vendor — spork JSON + Dependencies

- [x] 1.1 Copy `src/json.c` from spork master to `vendor/core/spork_json.c`
- [x] 1.2 Rename entry point: replace `JANET_MODULE_ENTRY(JanetTable *env)` with `void janet_lib_spork_json(JanetTable *env)`
- [x] 1.3 Add `serde_json` and `serde` (with derive feature) to `Cargo.toml`
- [x] 1.4 Add `pub fn janet_lib_spork_json(env: *mut JanetTable);` to `src/bridge.rs`
- [x] 1.5 Add `bridge::janet_lib_spork_json(env);` to `src/main.rs` registration block

## 2. Rust — Core CAD functions (cad.rs)

- [x] 2.1 Add `EdgeInfo` struct with `#[derive(Serialize)]` and EdgeInfo fields (index, edge_type, start, end)
- [x] 2.2 Implement `shape_fillet()` with `Option<&[usize]>` for edge selection
- [x] 2.3 Implement `shape_chamfer()` with `Option<&[usize]>` for edge selection
- [x] 2.4 Implement `edge_info()` that iterates `Shape::edges()` and collects `Vec<EdgeInfo>`, serialized to JSON via `serde_json::to_string()`

## 3. Rust — FFI Bridge (main.rs)

- [x] 3.1 Add `rust_init_fillet(dest, data, radius, idxs, count, eager)` extern "C" function
- [x] 3.2 Add `rust_init_chamfer(dest, data, distance, idxs, count, eager)` extern "C" function
- [x] 3.3 Add `rust_edge_info(data)` returning `*mut c_char` (JSON CString)
- [x] 3.4 Add `rust_highlight_edges(data, idxs, count)` and `rust_highlight_edges_clear()` extern "C" functions

## 4. C Bridge — JANET_FN wrappers (bridge/bridge.c)

- [x] 4.1 Add forward declarations for all new Rust extern functions
- [x] 4.2 Add `_cad_fillet` — keyword fn with `:r` (required), `:e` (optional tuple), `:eager`, `:hide`
- [x] 4.3 Add `_cad_chamfer` — keyword fn with `:d` (required), `:e` (optional tuple), `:eager`, `:hide`
- [x] 4.4 Add `_cad_edge_info_raw` — calls `rust_edge_info`, wraps JSON CString as Janet string
- [x] 4.5 Add `_cad_highlight_edge` — takes shape + variadic int indices
- [x] 4.6 Add `_cad_highlight_edge_clear` — no-arg function
- [x] 4.7 Register all 5 in the `cfuns[]` array

## 5. Viewer — Edge highlight state (src/types.rs, src/viewer/)

- [x] 5.1 Add `ReplToViewer::HighlightEdges { shape_id, indices: Vec<usize> }` and `ClearEdgeHighlight` variants
- [x] 5.2 Add `highlighted_edges: HashMap<ShapeId, HashSet<usize>>` to viewer state
- [x] 5.3 Add edge rendering color logic: highlighted edges in orange, normal edges in existing color

## 6. Janet — boot.janet + model.janet

- [x] 6.1 Add `(def edge-info (compose |(json/decode $ true) _edge-info-raw))` to boot.janet
- [x] 6.2 Replace custom `json-encode`/`json-escape` with spork's `(string (json/encode resp))` in GUI REPL code
- [x] 6.3 Add `wrap-c-fn` wrapper for `fillet` with `:r`, `:e`, `:eager`, `:hide` keywords
- [x] 6.4 Add `wrap-c-fn` wrapper for `chamfer` with `:d`, `:e`, `:eager`, `:hide` keywords
- [x] 6.5 Add `defmeta` for `fillet`, `chamfer`, `edge-info` with docstrings and examples
- [x] 6.6 Add `wrap-c-fn` wrapper for `highlight-edge` and `defn highlight-edge-clear`
- [x] 6.7 Add `defmeta` for `highlight-edge` and `highlight-edge-clear`
- [x] 6.8 Add `fillet`, `chamfer` to `cad-shape-fns` in `boot/model.janet`

## 7. Tests

- [x] 7.1 Add unit tests for `shape_fillet` and `shape_chamfer` in `src/cad.rs` (all-edges + selected-edges cases)
- [x] 7.2 Add unit test for `edge_info` in `src/cad.rs`
- [x] 7.3 Add unit tests for validation (negative radius/distance errors)
- [x] 7.4 Run `just build` to verify compilation
- [x] 7.5 Run `just test-unit` to verify unit tests pass

## 8. Degenerate Geometry Safety (Three-Layer Protection)

### Layer 1 — Pre-validation (before OCCT call)

- [x] 8.1 Add `validate_edge_dimension()` helper in `cad.rs` — checks distance/radius against `start_point()`/`end_point()` edge lengths
- [x] 8.2 Reject when `value × 2 >= edge_length` (must be strictly less than half edge length)
- [x] 8.3 Show descriptive error with max allowed value

### Layer 2 — Post-check (after OCCT returns)

- [x] 8.4 Add `validate_has_geometry()` helper in `cad.rs` — tessellates and checks for mesh vertices OR edge polylines
- [x] 8.5 Wire shapes are valid with edge polylines alone (no mesh required)
- [x] 8.6 Apply to `fillet` / `chamfer`
- [x] 8.7 Apply to `wire-fillet` / `wire-chamfer` / `wire-offset`
- [x] 8.8 Apply to `extrude` / `revolve`

### Layer 3 — Viewer guard (rendering safety net)

- [x] 8.9 `types.rs::tessellate_if_needed()` — set `mesh = None` when OCCT returns no vertices
- [x] 8.10 `app.rs` — skip rendering entries with empty vertex/index buffers
- [x] 8.11 Verify viewer does not crash on shapes with `mesh = None`

### Artifacts

- [x] 8.12 Update specs with three-layer protection requirements
- [x] 8.13 Update design doc with three-layer architecture and diagram
- [x] 8.14 Verify all tests pass
- [x] 8.8 Verify all tests pass
