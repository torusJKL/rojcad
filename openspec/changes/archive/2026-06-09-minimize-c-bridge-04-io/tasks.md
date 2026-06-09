## 1. Export

- [x] 1.1 Strip `cad_write_step` → add Janet wrapper (shape + path)
- [x] 1.2 Strip `cad_write_stl` → add Janet wrapper (shape + path)

## 2. Import

- [x] 2.1 Strip `cad_read_step` → add Janet wrapper (path + :eager :hide keywords)

## 3. Verification

- [x] 3.1 Build and run (`just build`)
- [x] 3.2 Write a test STEP file, import, export as STL, verify round-trip
- [x] 3.3 Run tests (`just test`)
