## Context

The viewer runs on a dedicated OS thread (`wgpu-viewer`) with its own winit event loop. The Janet REPL runs on the main thread. Cross-thread communication uses:

- **Global atomics** for simple state (edge visibility, colors, projection mode) — read lock-free each frame by the viewer
- **mpsc channel** for one-shot REPL→Viewer commands (existing `ReplToViewer` enum with `FitToBounds` variant)
- **`ShapeRegistry` (RwLock + generation counter)** for mesh data — dirty-checked each frame

The `ReplToViewer` mpsc channel was recently added for `view-fit` / `view-fit-all` (zoom-to-bounds commands). This design extends it with a new variant for angular view presets. The viewer already has infrastructure for camera animation (`CameraAnimation` for yaw/pitch only, `FitAnimation` for target+radius+yaw+pitch) and an array of preset view angles (`VIEW_TARGETS[6]` at `app.rs:1082`), but these are only accessible via keyboard shortcuts (Ctrl+1/3/7).

## Goals / Non-Goals

**Goals:**
- Expose 7 named view presets (front, back, left, right, top, bottom, isometric) as Janet-callable functions
- Provide a generic `view-angle` function for arbitrary yaw/pitch/distance
- Use the existing `ReplToViewer` mpsc channel for communication
- Reuse the existing `FitAnimation` system for smooth 0.5s animated transitions
- Keep all angle constants in Janet (`math/pi`, `math/sqrt`, `math/asin`) — no C math.h needed
- All preset functions accept an optional distance argument for zoom level
- Docstrings on every function documenting the yaw/pitch values

**Non-Goals:**
- Changing the keyboard shortcut behavior (Ctrl+1/3/7 continue to work as-is)
- Adding per-shape camera framing (already covered by `view-fit` / `view-fit-all`)
- Camera path recording or keyframe animation
- Perspective/orthographic toggle during view animation (already handled by `PROJECTION_PERSPECTIVE` atomic)

## Decisions

### 1. Communication mechanism: mpsc channel over atomics

**Chosen**: Add `SetViewAngles` variant to the existing `ReplToViewer` mpsc channel.
**Rejected**: Using atomic statics (like `VIEW_REQUEST: AtomicI64`).

The mpsc channel already exists and is plumbed through the system for `FitToBounds`. Adding a new variant is the path of least resistance. Atomics would require a new polling pattern and would be a parallel mechanism alongside the channel, adding complexity without benefit.

### 2. Named presets in Janet vs. C

**Chosen**: 7 named presets (`view-front`, `view-back`, `view-left`, `view-right`, `view-top`, `view-bottom`, `view-iso`) defined in `boot.janet` as pure Janet wrappers around `view-angle`.
**Rejected**: Defining them as individual C `JANET_FN` handlers in `bridge/bridge.c`.

Moving presets to Janet eliminates the need for `math.h` in C, keeps angle constants in a single place, makes docstrings easier to edit, and allows users to define their own presets. The C bridge only needs a single `view-angle` function.

### 3. Animation: FitAnimation over CameraAnimation

**Chosen**: Use `FitAnimation` (which animates target + radius + yaw + pitch) with target pinned to current value.
**Rejected**: Using `CameraAnimation` (which forces orthographic on completion).

`CameraAnimation` has a side effect: it sets `camera.perspective = false` on completion (designed for keyboard shortcuts that want drafting-style orthographic views). `FitAnimation` has no such side effect and preserves the user's current projection mode. Pinning target to the current camera position means only radius, yaw, and pitch animate.

### 4. Distance as absolute radius

**Chosen**: Optional distance argument sets the camera's absolute radius from target.
**Rejected**: Relative zoom factor (multiply/divide).

Absolute radius is simpler to reason about and document. Users can discover good distance values experimentally and reuse them. A relative mode could be added later if needed.

### 5. Single Rust FFI function

**Chosen**: One `rust_view_set_angles(yaw, pitch, has_distance, distance)` function handling all presets.
**Rejected**: Individual `rust_view_front()`, `rust_view_back()`, etc.

All presets do the same thing — send yaw/pitch/distance through the channel. A single function minimizes the Rust FFI surface and keeps all angle math in Janet.

### 6. C bridge: no math.h

The `view-angle` C JANET_FN passes through raw doubles without any math operations. The angle constants live in Janet where `math/pi`, `math/sqrt`, `math/asin` are available through the registered `janet_lib_math` module.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Channel buffer fills if viewer thread is blocked | mpsc channels have unlimited capacity in Rust; worst case is delayed command execution |
| Depth of `1/(sqrt 3)` computed in Janet could have precision loss | `math/asin` and `math/sqrt` in Janet use C `double` internally — precision is identical to C math |
| `FitAnimation` currently doesn't have a way to "not change target" | Design sets `start_target == end_target == camera.target`, which is a no-op for that field |
| If both keyboard shortcut and Janet command fire simultaneously, one animation overrides the other | Both trigger animations independently; `FitAnimation` will be replaced by `SetViewAngles` in `check_repl_commands` and vice versa for keyboard polling order |
