## Context

The `OrbitCamera` uses a spherical coordinate model: `target` (look-at point), `radius` (distance), `yaw`/`pitch` (angles). Zoom scales `radius`, moving the camera eye toward/away from a fixed target. Dolly needs to translate `target` along the view direction so the entire camera rig moves through scene space.

The `pan` fix (change `fix-camera-panning`) already introduced screen-space `right`/`up` computation using `forward()`. Dolly reuses `forward()` for the same reason — it's the camera's look direction.

## Goals / Non-Goals

**Goals:**
- Translate the camera rig forward/backward through the scene along the view direction
- Shift+scroll triggers dolly (scroll stays zoom)
- Shift+RMB drag triggers dolly (plain RMB stays zoom)
- Consistent sensitivity with existing zoom/pan

**Non-Goals:**
- No new camera model changes — dolly reuses existing spherical coordinates
- No Janet API changes (can be added later if needed)
- No animation interpolation for dolly

## Decisions

1. **Dolly translates target, not radius**
   - `target += forward * delta * speed` keeps radius unchanged
   - After dollying, orbit still works around the new target point
   - Zoom still works to refine distance

2. **Speed proportional to radius, matching old pan sensitivity**
   - `dolly(amount) = target += forward * amount * radius * 0.002`
   - Uses the same `radius * 0.002` factor as the old broken pan, so dolly feels identical to the forward/backward component users experienced before the pan fix
   - Input handlers pass raw values (pixels for RMB drag, notch×50 for scroll) — no extra multiplier

3. **Shift+RMB replaces zoom when Shift is held**
   - Unmodified RMB = zoom (existing behavior)
   - Shift+RMB = dolly (new)
   - Shift is already used to modify LMB (orbit→pan), so this is consistent

4. **Modifier state queried from egui context, not winit directly**
   - `state.egui_ctx.input(|i| i.modifiers.shift)` reads Shift state from egui's own modifier tracking
   - egui-winit processes `ModifiersChanged` and stores state in `egui::Context` before our handler runs
   - This sidesteps platform-specific event-ordering issues where `ModifiersChanged` events don't arrive before mouse events on some X11 setups
   - Alternatives rejected: manual keyboard tracking (`NamedKey::Shift` events may not fire on all platforms), relying on `ModifiersChanged` directly (timing issues on X11)

## Risks / Trade-offs

- [Low] Shift+scroll dolly sensitivity: `factor * 50.0 * radius * 0.002`. At default radius=50, one scroll notch moves target 5 units. Tweak the 50.0 constant if feel needs adjustment — the 0.002 and radius scaling keep it proportional at all zoom levels.
- [Low] Dependency on egui's modifier state: if egui's own modifier tracking breaks (e.g., winit API change in an update), dolly modifier detection breaks too. This is mitigated because egui already depends on correct modifier state for its own shortcut handling.
