## Why

Camera panning (middle-mouse-drag or Shift+left-drag) is broken when pitch ≠ 0. The `pan()` method mixes a pitch-ignorant horizontal right vector with a pitch-aware up vector, producing non-orthogonal pan axes. This means dragging left/right moves the scene diagonally on screen instead of cleanly left/right, making the viewport feel imprecise and disorienting. This is the most immediate usability issue in the viewer.

## What Changes

- Fix `OrbitCamera::pan()` to use a consistent orthogonal pair of right/up vectors derived from the camera's forward direction, giving true screen-space panning
- The change is entirely within `src/viewer/camera.rs` — no API or Janet bindings change
- The `right()` and `up()` public methods remain unchanged (they are only used internally by `pan()`, so no external breakage)

## Capabilities

### New Capabilities
- `camera-pan`: Screen-space camera panning that correctly maps mouse movement to viewport movement at any camera orientation

### Modified Capabilities
*(none — no existing specs)*

## Impact

- **Single file affected**: `src/viewer/camera.rs` (the `pan()` method, ~6 lines)
- **No API changes**: Public interface unchanged
- **No Janet/Lisp changes**: Camera commands continue to work identically
- **No test changes**: Existing pan behavior wasn't tested; new behavior is a strict improvement
