## 1. Project Scaffolding

- [x] 1.1 Create Cargo workspace with `Cargo.toml` (workspace root), `rust-toolchain.toml` targeting Rust edition 2024
- [x] 1.2 Add dependencies to `Cargo.toml`: `opencascade` (with default-features for builtin OCCT), `glam`, `thiserror`, `cc` (build dep)
- [x] 1.3 Create `.gitmodules` for `opencascade-rs` to fetch OCCT as submodule (`git submodule update --init`)
- [x] 1.4 Add `LICENSE` file with GPLv3 text
- [x] 1.5 Add `.gitignore` (Rust standard + OCCT build artifacts)

## 2. Janet Amalgamation Integration

- [x] 2.1 Download/extract Janet amalgamation (`janet.c`, `janet.h`) into `vendor/` directory
- [x] 2.2 Create `build.rs` that compiles `vendor/janet.c` and `bridge/bridge.c` using the `cc` crate, linking against `opencascade-rs`'s built OCCT
- [x] 2.3 Create `src/bridge.rs` with `extern "C"` declarations for all Janet C API functions used (janet_init, janet_deinit, janet_core_env, janet_dostring, janet_cfuns, janet_wrap_abstract, janet_abstract_type, janet_getkwargs, janet_panic, etc.)

## 3. Shape Abstract Type

- [x] 3.1 Create `src/types.rs` with `ShapeData` struct containing `Shape` (from opencascade-rs), `visible: bool`, and a future-ready `color: Option<[f64;3]>`
- [x] 3.2 Implement `ShapeData` construction (new, new_with_center) and helper methods
- [x] 3.3 In `bridge/bridge.c`, register the `rojcad/shape` Janet abstract type with `gcfinish` finalizer (calls `rust_shape_drop`) and `tostring` (prints `#<Shape(SOLID)>` etc.)
- [x] 3.4 In `bridge/bridge.c`, implement the `tostring` method that queries `rust_shape_type_string` and formats the Janet buffer
- [x] 3.5 Add Rust `extern "C"` functions for shape lifecycle: `rust_shape_drop`, `rust_shape_type_string`, `rust_is_shape`

## 4. CAD Primitives

- [x] 4.1 Implement `rust_make_box` in Rust (`src/cad.rs`) — calls `Shape::box_with_dimensions(w, d, h)`, optionally applies translation via `BRepBuilderAPI_Transform` when `:center` is provided
- [x] 4.2 Implement `rust_make_sphere` in Rust — calls `Shape::sphere(r).build()`, optionally applies translation for `:center`
- [x] 4.3 Add `Shape::translated()` helper to `src/cad.rs` wrapping `BRepBuilderAPI_Transform` for general shape translation (needed because opencascade-rs only has it on Wire/Face, not Shape)
- [x] 4.4 In `bridge/bridge.c`, implement `cad_make_box` JANET_FN — parse 3 required numbers + optional `:center` keyword tuple, call `rust_make_box`, wrap result as abstract
- [x] 4.5 In `bridge/bridge.c`, implement `cad_make_sphere` JANET_FN — parse 1 required number + optional `:center` keyword tuple, call `rust_make_sphere`, wrap result as abstract
- [x] 4.6 Validate inputs: reject zero/negative dimensions with `janet_panic`

## 5. CAD Booleans

- [x] 5.1 Implement `rust_cut` in Rust (`src/cad.rs`) — calls `shape_a.subtract(&shape_b).shape`, validates `BooleanShape` is non-empty
- [x] 5.2 Implement `rust_common` in Rust — calls `shape_a.intersect(&shape_b).shape`, validates result is non-empty
- [x] 5.3 In `bridge/bridge.c`, implement `cad_cut` JANET_FN — parse 2 abstract shape args, type-check, call `rust_cut`, wrap result
- [x] 5.4 In `bridge/bridge.c`, implement `cad_common` JANET_FN — parse 2 abstract shape args, type-check, call `rust_common`, wrap result
- [x] 5.5 Handle OCCT boolean failures (empty results, non-intersecting shapes) — return Janet error with descriptive message

## 6. CAD Inspection

- [x] 6.1 Implement `rust_shape_type` in Rust — returns the `ShapeType` enum variant name as a C string
- [x] 6.2 In `bridge/bridge.c`, implement `cad_shape_type` JANET_FN — unwrap abstract, call `rust_shape_type`, return as Janet keyword (`:solid`, etc.)
- [x] 6.3 In `bridge/bridge.c`, implement `cad_hide` JANET_FN — set `shape_data->visible = false`, return nil
- [x] 6.4 In `bridge/bridge.c`, implement `cad_show` JANET_FN — set `shape_data->visible = true`, return nil
- [x] 6.5 In `bridge/bridge.c`, implement `cad_visible_q` JANET_FN — return `shape_data->visible` as Janet boolean

## 7. CAD Export

- [x] 7.1 Implement `rust_write_step` in Rust — calls `shape.write_step(path)`, maps `Result<(), Error>` to success/panic
- [x] 7.2 Implement `rust_write_stl` in Rust — calls `shape.write_stl(path)`, maps `Result<(), Error>` to success/panic
- [x] 7.3 In `bridge/bridge.c`, implement `cad_write_step` JANET_FN — parse shape + string, call `rust_write_step`, panic on error
- [x] 7.4 In `bridge/bridge.c`, implement `cad_write_stl` JANET_FN — parse shape + string, call `rust_write_stl`, panic on error

## 8. TCP REPL Server

- [x] 8.1 Create `boot.janet` with the REPL server: `net/listen` on 127.0.0.1:9000, accept loop with `ev/spawn` and `repl` per connection
- [x] 8.2 Add startup banner to stderr: `"◆ rojcad ready — connect via: nc 127.0.0.1 9000"`
- [x] 8.3 Add connect/disconnect logging to stderr
- [x] 8.4 Handle port-in-use error: catch `net/listen` failure, print message, exit non-zero

## 9. Main Entry Point

- [x] 9.1 Create `src/main.rs`: call `janet_init()`, get core env via `janet_core_env(NULL)`, call bridge registration, run `boot.janet` via `janet_dostring`, enter `janet_ev_loop()`, cleanup via `janet_deinit()`
- [x] 9.2 In `bridge/bridge.c`, create `cad_register_functions(env)` that registers all JANET_FN entries in a single `janet_cfuns` call
- [x] 9.3 Embed `boot.janet` into the binary via `include_str!("../boot.janet")` and pass to `janet_dostring`

## 10. Testing

- [x] 10.1 Test box creation via raw Rust API: verify dimensions and shape type
- [x] 10.2 Test sphere creation via raw Rust API: verify radius via bounding box
- [x] 10.3 Test cut via raw Rust API: verify result is a solid, verify original shapes unchanged
- [x] 10.4 Test common via raw Rust API: verify result is a solid for overlapping shapes
- [x] 10.5 Test cut with non-overlapping shapes: verify error
- [x] 10.6 Test STEP export round-trip: create shape, write .step, read back, verify equality
- [x] 10.7 Test STL export: create shape, write .stl, verify file exists and is non-empty
- [x] 10.8 Test visibility flag: create shape, verify visible?, hide, verify not visible?, show, verify visible?
- [x] 10.9 Test type checking: pass wrong types to each function, verify Janet errors
- [x] 10.10 Test TCP REPL: spawn binary, connect with nc, submit expressions, verify responses (integration test)

## 11. Documentation

- [x] 11.1 Create `README.md` explaining the project, build prerequisites (Rust, CMake, C++ compiler), build instructions (`cargo build --release`), and usage (`cargo run`, then `nc 127.0.0.1 9000`)
- [x] 11.2 Add Janet docstrings to all registered CAD functions (second argument to JANET_FN macro provides the docstring shown by `(doc make-box)`)
