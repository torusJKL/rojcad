## 1. Visibility functions (already wrapped)

- [x] 1.1 Strip `cad_show` → ensure `_show` primitive exists in C, verify Janet wrapper
- [x] 1.2 Strip `cad_hide` → ensure `_hide` primitive exists in C, verify Janet wrapper
- [x] 1.3 Strip `cad_purge` → ensure `_purge` primitive exists in C, verify Janet wrapper
- [x] 1.4 Strip `cad_registry_remove` → ensure `_registry-remove` primitive exists in C, verify Janet wrapper

## 2. Query functions

- [x] 2.1 Strip `cad_visible_q` → save C primitive, verify Janet variadic wrapper
- [x] 2.2 Strip `cad_wire_q` → save C primitive, verify Janet variadic wrapper
- [x] 2.3 Strip `cad_face_q` → save C primitive, verify Janet variadic wrapper
- [x] 2.4 Strip `cad_solid_q` → save C primitive, verify Janet variadic wrapper
- [x] 2.5 Strip `cad_shape_type` → save C primitive, verify Janet variadic wrapper

## 3. Verification

- [x] 3.1 Build and run (`just build`)
- [x] 3.2 Verify each function via REPL
- [x] 3.3 Run tests (`just test`)
