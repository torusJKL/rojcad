## 1. Add guard to `cad::cut`

- [x] 1.1 Add Face-Face guard at the top of `cad::cut` (line 361): check `shape_type() == ShapeType::Face` for both inputs, return `Err` with descriptive message
- [x] (common and fuse are NOT guarded — OCCT investigation confirmed they handle Face-Face correctly)

## 2. Unit tests

- [x] 2.1 Add unit test for `cut` Face-Face rejection: create two face shapes, call `cut`, assert `Err` with expected message substring
- [x] 2.2 Add unit test for `cut` with Face+Solid: verify it passes through (no guard error)
- [x] 2.3 Add unit test for `cut` with Solid+Solid: verify it passes through (no guard error)

## 3. Verify build and tests

- [x] 3.1 Run `just build` to confirm compilation
- [x] 3.2 Run `just test-unit` to confirm all tests pass
