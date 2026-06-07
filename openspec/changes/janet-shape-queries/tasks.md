## 1. Shared State Infrastructure

- [x] 1.1 Add `SHAPE_PTR_MAP` global (`OnceLock<RwLock<HashMap<ShapeId, *mut c_void>>>`) and helper functions (`register_shape_pointer`, `unregister_shape_pointer`, `get_shape_pointer`) to `src/types.rs`
- [x] 1.2 Add `SELECTED_IDS` global (`OnceLock<RwLock<HashSet<ShapeId>>>`) and helper function `get_selected_ids` to `src/types.rs`
- [x] 1.3 Initialize both globals in `main()` before the viewer starts
- [x] 1.4 Extend `ShapeData::drop` to call `unregister_shape_pointer(self.shape_id)`
- [x] 1.5 Add `ShapeRegistry::hidden_shapes()` method in `src/types.rs`

## 2. Shape Pointer Registration

- [x] 2.1 Add `rust_register_abstract` call at the end of each `rust_init_*` function in `src/main.rs` after successful `ptr::write`: `rust_init_box`, `rust_init_sphere`, `rust_init_cube`, `rust_init_box_from_corners`, `rust_init_cylinder`, `rust_init_cylinder_from_points`, `rust_init_cylinder_point_dir`, `rust_init_cone`, `rust_init_torus`
- [x] 2.2 Add `rust_register_abstract` call in Boolean/transform functions: `rust_init_cut`, `rust_init_common`, `rust_init_fuse`, `rust_init_translate`, `rust_init_rotate`, `rust_init_scale`, `rust_init_mirror`
- [x] 2.3 Add `rust_register_abstract` call in import/text functions: `rust_init_read_step`, `rust_init_text`, `rust_init_text_extruded`
- [x] 2.4 Add `rust_register_abstract` call in 2D/extrusion functions: `rust_init_rect`, `rust_init_circle`, `rust_init_polygon`, `rust_init_extrude`, `rust_init_revolve`, `rust_init_extrude_polygon`
- [x] 2.5 Add `rust_register_abstract` call in wire/sketch functions: `rust_init_wire_to_face`, `rust_init_wire_fillet`, `rust_init_wire_chamfer`, `rust_init_wire_offset`, `rust_sketch_close`, `rust_sketch_build_wire`

## 3. Selection State Sync (Viewer)

- [x] 3.1 In `src/viewer/app.rs::handle_click`, after every code path that modifies `state.selected_ids`, sync the full set to `SELECTED_IDS`: `*SELECTED_IDS.get().unwrap().write().unwrap() = state.selected_ids.clone();`

## 4. Rust FFI Query Functions

- [x] 4.1 Add `extern "C" fn rust_get_selected_shape_ids(count_out: *mut usize) -> *mut u64` to `src/main.rs`
- [x] 4.2 Add `extern "C" fn rust_get_registered_shape_ids(filter: u8, count_out: *mut usize) -> *mut u64` to `src/main.rs`
- [x] 4.3 Add `extern "C" fn rust_get_shape_pointer(id: u64) -> *mut c_void` to `src/main.rs`
- [x] 4.4 Add `extern "C" fn rust_free_u64_array(ptr: *mut u64, count: usize)` to `src/main.rs`

## 5. C Bridge: New JANET_FN Functions

- [x] 5.1 Add `extern` declarations for the new Rust FFI functions in `bridge/bridge.c`
- [x] 5.2 Implement `cad_selected_shapes` JANET_FN in `bridge/bridge.c`: call `rust_get_selected_shape_ids`, iterate IDs, look up pointers via `rust_get_shape_pointer`, build Janet tuple
- [x] 5.3 Implement `cad_list_shapes` JANET_FN in `bridge/bridge.c`: parse `:visible`/`:hidden` keywords, call `rust_get_registered_shape_ids`, look up pointers, build Janet tuple
- [x] 5.4 Register both functions in `cad_register_functions`'s `cfuns[]` array in `bridge.c`
- [x] 5.5 Add both function names to `cad_fn_categories[]` in `bridge.c` (category: `"queries"`)

## 6. Verification

- [x] 6.1 Build and verify no compilation errors: `just build`
- [x] 6.2 Run existing tests: `just test`
- [x] 6.3 Run lint: `just lint`
- [x] 6.4 Manual smoke test: start server, create a shape, select it in viewer, call `(selected-shapes)` and `(list-shapes)` from REPL
