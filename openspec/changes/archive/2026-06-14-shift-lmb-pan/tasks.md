## 1. Implement Shift+LMB Pan in CursorMoved Handler

- [x] 1.1 In `src/viewer/app.rs`, modify the `if state.mouse_pressed[0]` block inside `CursorMoved` to check `state.modifiers.shift_key()` and call `camera.pan(dx, dy)` when Shift is held, `camera.rotate(dx, dy)` otherwise

## 2. Verify

- [x] 2.1 Build and run: `just build && just run`
- [x] 2.2 Test LMB drag (no Shift) — camera orbits
- [x] 2.3 Test Shift+LMB drag — camera pans
- [x] 2.4 Test Shift+LMB click (no drag) — shape adds to selection
- [x] 2.5 Test MMB drag — camera pans (unchanged)
- [x] 2.6 Test RMB drag — camera zooms (unchanged)
- [x] 2.7 Test scroll wheel — camera zooms (unchanged)
- [x] 2.8 Test Ctrl+click — shape toggles selection (unchanged)
