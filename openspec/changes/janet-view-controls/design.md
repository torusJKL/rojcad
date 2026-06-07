## Context

The viewer runs on a dedicated OS thread (`wgpu-viewer`) with its own winit event loop. The Janet REPL runs on the main thread. Cross-thread communication currently uses:

- **Global atomics** for simple state (edge visibility, colors, thickness) — read lock-free each frame by the viewer
- **`ShapeRegistry` (RwLock + generation counter)** for mesh data — dirty-checked each frame

`show_back_edges` is currently a local `ViewerState` field, settable only via the X keyboard shortcut. Projection mode (`perspective` bool on `OrbitCamera`) is settable only via O/P keyboard shortcuts. Neither is exposed to Janet.

## Goals / Non-Goals

**Goals:**
- Expose hidden edge toggle and projection mode toggle as Janet-callable functions
- Make `SHOW_BACK_EDGES` default to `false`
- Keep keyboard shortcuts (X, O/P) working identically
- Use Rust atomics as single source of truth (consistent with existing `SHOW_INACTIVE_EDGES`, `SHOW_ACTIVE_EDGES`)

**Non-Goals:**
- Adding a REPL→Viewer command channel (future concern if more viewer controls are needed)
- Per-shape opacity/transparency controls
- Viewport color or lighting control
- Animation or transition between projection modes

## Decisions

### 1. Atomics vs. Janet globals

**Chosen**: Rust `AtomicBool` globals.
**Rejected**: Storing state in Janet globals.

The viewer thread cannot safely access the Janet VM (runs on REPL thread). Atomics are lock-free, read-every-frame, and consistent with the three existing edge-state atomics. A channel-based approach would add a second communication path without benefit for simple booleans.

### 2. Single atomic per feature vs. toggle-request atomic

**Chosen**: Single authoritative atomic per feature.
- `SHOW_BACK_EDGES`: viewer reads it in `render()`, keyboard and Janet both write it.
- `PROJECTION_PERSPECTIVE`: viewer syncs camera field from it in `render()`, keyboard and Janet both write it.

**Rejected**: Separate request/action atomics. Simpler to keep one source of truth.

### 3. Projection sync strategy

The viewer syncs `state.camera.perspective` from `PROJECTION_PERSPECTIVE` at the top of `render()`, before the animation update. This means the change takes effect on the next frame — imperceptible latency. The keyboard handler (`O`/`P`) now writes to the atomic instead of calling `toggle_projection()` directly, so all paths converge on a single source of truth.

### 4. Janet API naming

Follow existing conventions:
- `edge-*-toggle` / `edge-*-show?` for edge controls
- `projection-toggle` / `projection-perspective` (get/set) for camera
- `edge-hidden` (get/set) mirrors the `edge-thickness` pattern (0-arg query, 1-arg set)

### 5. Default state

`SHOW_BACK_EDGES` defaults to `false` (user request). This matches common CAD behavior where hidden lines are off by default and enabled when needed.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Race: Janet sets projection between frames | Accepted — single frame delay is imperceptible |
| Keyboard and Janet out of sync | Impossible — both write to same atomic, viewer reads it |
| Future controls need richer API (e.g., background color) | Can add REPL→Viewer channel then; atomics remain for high-frequency reads |
