## 1. ShapeData — Decouple registration from creation

- [x] 1.1 Add `mesh: Option<MeshData>` and `edge_polylines: Vec<Vec<[f64; 3]>>` fields to `ShapeData` in `types.rs`
- [x] 1.2 Add `registered: bool` field to `ShapeData`
- [x] 1.3 Add `purged: bool` field to `ShapeData`
- [x] 1.4 Remove `global_shape_registry().register(entry)` call from `ShapeData::new()`
- [x] 1.5 Remove `tessellate_and_update()` call from `ShapeData::new()`
- [x] 1.6 Implement `Drop for ShapeData` that calls `global_shape_registry().remove(self.shape_id)` when `self.registered` is true
- [x] 1.7 Removed `tessellate_and_update` (replaced by `ShapeData::tessellate_if_needed()` + `ShapeData::show()`)
- [x] 1.8 Add helper `ShapeData::tessellate_if_needed(&mut self)` that checks `self.mesh.is_none()`, tessellates, and stores

## 2. Rust creation functions — accept `:eager`

- [x] 2.1 Add `eager: bool` parameter to `make_box` / `make_cube` / `make_box_from_corners`
- [x] 2.2 Add `eager: bool` parameter to `make_sphere`
- [x] 2.3 Add `eager: bool` parameter to `make_cylinder`
- [x] 2.4 Add `eager: bool` parameter to `make_cone`
- [x] 2.5 Add `eager: bool` parameter to `make_torus`
- [x] 2.6 Add `eager: bool` parameter to `cut`, `common`, `fuse`
- [x] 2.7 Add `eager: bool` parameter to `translate`, `rotate`, `scale`, `mirror`
- [x] 2.8 Update all `rust_init_*` functions in `main.rs` to accept and forward the `eager` flag

## 3. C bridge — `:eager` keyword parsing

- [x] 3.1 Add shared `has_eager` helper to `bridge.c` that checks for `:eager` keyword in argv
- [x] 3.2 Update `cad_box` JANET_FN to parse `:eager` and pass to `rust_init_box`/`rust_init_cube`/`rust_init_box_from_corners`
- [x] 3.3 Update `cad_sphere` to parse `:eager` and pass to `rust_init_sphere`
- [x] 3.4 Update `cad_cylinder` to parse `:eager` and pass to `rust_init_cylinder`/`rust_init_cylinder_point_dir`/`rust_init_cylinder_from_points`
- [x] 3.5 Update `cad_cone` to parse `:eager` and pass to `rust_init_cone`
- [x] 3.6 Update `cad_torus` to parse `:eager` and pass to `rust_init_torus`
- [x] 3.7 Update `cad_cut`, `cad_common`, `cad_fuse` to parse `:eager` and pass to respective `rust_init_*`
- [x] 3.8 Update `cad_translate`, `cad_rotate`, `cad_scale`, `cad_mirror` to parse `:eager` and pass to respective `rust_init_*`

## 4. C bridge — show, hide, registry-remove functions

- [x] 4.1 Implement `cad_show` JANET_FN: tessellate if needed, register if not registered, set visible
- [x] 4.2 Implement `cad_hide` JANET_FN: set visible flag to false
- [x] 4.3 Implement `cad_registry_remove` JANET_FN: remove from registry, mark purged, return nil
- [x] 4.4 Register all new functions in `cad_register_functions`

## 5. Janet macros — purge, display

- [x] 5.1 Add `purge` macro to `boot.janet`
- [x] 5.2 Add `display` macro to `boot.janet`

## 6. Remove dead ReplToViewer channel

- [x] 6.1 Remove `ReplToViewer` enum from `src/viewer/mod.rs`
- [x] 6.2 Remove `repl_tx`/`repl_rx` channel creation from `spawn_viewer()`
- [x] 6.3 Remove `repl_rx` field from `ViewerApp` struct in `src/viewer/app.rs`
- [x] 6.4 Remove `repl_rx` parameter and field init from `run_viewer()` in `src/viewer/app.rs`

## 7. Update existing callers and tests

- [x] 7.1 Audit all callers of `ShapeData::new()` in `cad.rs` — each passes `eager` correctly (default false)
- [x] 7.2 Removed `tessellate_and_update` entirely — `ShapeData::tessellate_if_needed()` handles tessellation
- [x] 7.3 Update unit tests in `cad.rs` to reflect that shapes are not auto-registered
- [x] 7.4 Add unit tests for `show`/`hide`/`purge` lifecycle
- [x] 7.5 Verify `just build` compiles cleanly
- [x] 7.6 Run `just test` to confirm no regressions
