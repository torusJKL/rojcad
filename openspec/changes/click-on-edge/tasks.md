## 1. Data model: types.rs

- [x] 1.1 Add `topo_edge_count: usize` field to `ShapeEntry` and `ShapeData` (initialized in constructor, set before synthetic wireframe is appended)
- [x] 1.2 Add atomics: `LAST_EDGE_SHAPE_ID: AtomicU64 = AtomicU64::new(0)` and `LAST_EDGE_INDEX: AtomicI32 = AtomicI32::new(-1)`
- [x] 1.3 Add `SELECTED_EDGES: OnceLock<RwLock<HashMap<ShapeId, HashSet<usize>>>>` global for Janet queries
- [x] 1.4 Update `ShapeData::tessellate_if_needed()` to record real edge count before appending synthetic wireframe

## 2. Edge polyline extraction: cad.rs

- [x] 2.1 Add `pub fn topological_edge_count(shape: &Shape) -> usize` helper returning `shape.edges().count()`

## 3. Edge picking: pick.rs

- [x] 3.1 Add `EdgePickResult { shape_id: u64, edge_index: usize, distance: f64, hit_point: DVec3 }` struct
- [x] 3.2 Add `pub fn pick_edge(...)` — projects each polyline segment to screen, computes point-to-segment distance, checks z-depth, returns closest edge within threshold
- [x] 3.3 Synthetic edges (index >= topo_edge_count) are skipped in the loop

## 4. Viewer click handling + rendering: app.rs

- [x] 4.1 Add `selected_edges: HashMap<ShapeId, HashSet<usize>>` field to `ViewerState`
- [x] 4.2 In `handle_click()`: edge pick first with modifier semantics, falls back to shape pick
- [x] 4.3 Edge deselection on missed click: clear `selected_edges` when clicking empty space without modifiers
- [x] 4.4 Sync `selected_edges` to global `SELECTED_EDGES` after each mutation
- [x] 4.5 Set `LAST_EDGE_SHAPE_ID` and `LAST_EDGE_INDEX` atomics on edge selection/deselection events
- [x] 4.6 Added `LAST_EDGE_ACTION` atomic (4 = edge selected, 5 = edge deselected)
- [x] 4.7 In edge rendering section: merged `selected_edges` into the highlight check

## 5. Rust FFI: main.rs

- [x] 5.1 Add `rust_poll_edge_selection` — atomically reads and resets `LAST_EDGE_SHAPE_ID`, `LAST_EDGE_INDEX`, `LAST_EDGE_ACTION`
- [x] 5.2 Add `rust_get_selected_edge_ids` — returns flat array of `[shape_id, edge_idx, ...]`

## 6. C bridge: bridge.c

- [x] 6.1 Modify `_cad_poll_selection_raw`: check edge atomics first, return 3-tuple `[action, shape_id, edge_index]`
- [x] 6.2 Add `_cad_edge_selection` JANET_FN registered as `_edge-selection-raw`

## 7. Janet API: boot.janet

- [x] 7.1 Add `shape-name` helper: reverse-lookup a shape in `shape-bindings`, returns string or "nil"
- [x] 7.2 Extend `poll-selection`: decode actions 4/5/1 into structured events with :type, :shape, :name, :edge
- [x] 7.3 Extend `poll-viewer`: handles struct events — prints `"■ edge N of NAME selected"` etc.
- [x] 7.4 Add `(edge-selection)` function: groups flat edge data by shape with names
- [x] 7.5 Add `(fillet-selected ...)` convenience wrapper
- [x] 7.6 Add `(chamfer-selected ...)` convenience wrapper

## 8. Hidden edge occlusion check

- [x] 8.1 Add `is_edge_hidden(hit_point, shape_id, mesh_refs, origin) -> bool` in `pick.rs` — ray-mesh occlusion test for edge midpoint
- [x] 8.2 In `handle_click`: after `pick_edge` returns, check `SHOW_BACK_EDGES`. If false and edge is hidden, reject it (set `edge_hit = None`)
- [x] 8.3 Import `is_edge_hidden` and `SHOW_BACK_EDGES` in `app.rs`

## 9. Tests

- [x] 8.1 Unit test `pick_edge` — mock edge polylines, verify hit detection and closest-edge logic
- [x] 8.2 Unit test synthetic edge exclusion — synthetic edges beyond topo_count not picked
- [x] 8.3 Unit test `topological_edge_count` — verify count > 0 for box, sphere, cylinder
- [x] 8.4 Unit test `test_topo_edge_count_passed_to_entry` — verify ShapeData.topo_edge_count is set

## 9. Cleanup

- [x] 9.1 `just fmt` — formatting applied and verified clean
- [x] 9.2 Clippy clean with `-D warnings`
- [x] 9.3 All 123 tests pass
