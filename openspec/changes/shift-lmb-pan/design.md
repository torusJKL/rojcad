## Context

Camera panning is currently mapped to middle mouse button (MMB) drag. On Wayland (Ubuntu 25.10), horizontal MMB motion can produce unexpected zoom behavior due to compositor-level event handling or scroll wheel physics interference. The existing click-vs-drag mechanism (3px threshold on release) already distinguishes clicks from drags correctly.

## Goals / Non-Goals

**Goals:**
- Shift+LMB drag pans the camera (primary pan gesture)
- MMB drag continues to pan (secondary/fallback)
- LMB drag without Shift continues to orbit/rotate
- LMB click (release within 3px) still selects — regardless of Shift state
- Zero new dependencies, zero API changes

**Non-Goals:**
- Changing the selection model (Shift+click additive selection is unaffected)
- Changing zoom behavior
- Adding configuration UI for key bindings

## Decisions

1. **Check Shift on CursorMoved, not on MouseInput press**
   - We evaluate `state.modifiers.shift_key()` on each `CursorMoved` event rather than capturing it at button-down time
   - This means pressing/releasing Shift mid-drag switches the action immediately
   - Rationale: responsive to user intent; if they hold Shift mid-rotate they clearly want to switch to pan

2. **MMB pan is preserved as-is**
   - No changes to the MMB path (`mouse_pressed[1]` → `camera.pan()`)
   - Rationale: zero risk of regression; users with functional MMB can keep using it

3. **Single-location change**
   - Only the `CursorMoved` handler in `src/viewer/app.rs` needs modification
   - The existing `if state.mouse_pressed[0]` block splits into a Shift check that routes to `pan()` or `rotate()`

## Risks / Trade-offs

- **[DX] Shift used for both additive selection and pan**: A Shift+click (<3px movement) selects additively. A Shift+drag (>3px) pans. The 3px threshold is the only distinction. If a user has shaky hands or a high-DPI mouse at low sensitivity, they might trigger a tiny pan when intending to Shift+click. Mitigation: the existing 3px threshold already handles this for orbit-vs-click, and the pan amount at <3px is imperceptible (dx * 0.005 radians or radius * 0.002 units).
- **[Perception] Mid-drag Shift toggle toggles action**: Pressing Shift mid-drag switches from rotate to pan. This is intentional but might surprise users who try to "add Shift" after starting a rotate. This matches Blender's behavior.
