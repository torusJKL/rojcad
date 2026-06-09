## Context

The `OrbitCamera` struct uses a spherical coordinate model: `(target, radius, yaw, pitch)`. The `pan()` method translates the `target` point to simulate moving the scene under the cursor. Currently it uses two vectors computed from different bases:

- `self.right()` → `(cos(yaw), 0, sin(yaw))` — flattened to the XZ plane, ignoring pitch
- `self.up()` → computed from `forward × Y → right × forward` — pitch-aware

When pitch ≠ 0 (default is 0.4 rad ≈ 23°), these vectors are not orthogonal. Mouse-left/right (dx) maps to off-screen diagonal movement instead of screen-left/right, making panning feel broken.

The `right()` and `up()` public methods are only called from within `pan()` — no external consumers exist. The change is fully encapsulated.

## Goals / Non-Goals

**Goals:**
- Mouse left/right drag always pans the scene left/right on screen, regardless of pitch or yaw
- Mouse up/down drag always pans the scene up/down on screen
- Pan axes remain orthogonal at all camera orientations
- Zero external API changes

**Non-Goals:**
- No new input methods (keyboard pan, WASD, etc.)
- No changes to zoom, orbit, or view-preset animation
- No changes to Janet/Lisp camera command interface
- No changes to the `right()` / `up()` / `forward()` public methods (they remain available but are no longer used by `pan()`)
- No orthographic/perspective projection changes

## Decisions

1. **Compute right/up locally in pan() rather than fixing right()/up()**
   - `right()` returning a flat XZ vector is arguably intentional — it gives a "compass" right that never tilts, useful for UI gizmos or constrained orbit. Changing it would be a broader semantic change.
   - `pan()` needs screen-space axes specifically. Computing them locally keeps the concern isolated.
   - **Alternatives considered**: Fix `right()` to return true camera right, and have `up()` use it. Rejected because it changes the contract of the public method for no benefit outside `pan()`.

2. **Derive pan axes from `forward()`**
   - `screen_right = normalize(forward × Y)` gives the view-horizontal direction
   - `screen_up = normalize(screen_right × forward)` gives the view-vertical direction
   - These are always orthogonal by construction, and match what the user sees
   - At pitch=0 they simplify to the current `right()` and `Y`, preserving existing feel for horizontal views

3. **Keep pan speed proportional to radius**
   - `speed = radius * 0.002` is unchanged. The factor scales linearly with distance, giving consistent screen-space feel at any zoom level — a common CAD convention.
   - **Alternatives considered**: Constant speed (feels too fast when zoomed in, too slow when zoomed out). Exponential scaling (unnecessary complexity).

4. **No test coverage added in this change**
   - Camera math testing would require rendering validation or mock expectations, which is disproportionate effort for a 6-line change with no branching logic.
   - Manual verification: at pitch=0, default, and pitch=0.8, drag left/right and confirm scene follows cursor.

## Risks / Trade-offs

- [Low] `right()` and `up()` public methods remain but are no longer called internally. If new code uses them expecting screen-space semantics, it may get wrong results. → Mitigation: document that these return world-aligned vectors, not screen-space. Add a doc-comment to both methods.
- [Low] The pan speed multiplier (0.002) was tuned with the old non-orthogonal vectors. Since only the direction changes, not the magnitude, pan speed is preserved. Verify visually.
- [None] No migration needed — this is purely behavioral within an internal method.
