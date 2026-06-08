## Context

The 3D viewer runs on a dedicated OS thread (`wgpu-viewer`) with its own winit event loop. The Janet REPL runs on the main thread. Currently, the only REPL→Viewer communication is via atomic globals (`AtomicBool`, `AtomicU64` in `types.rs`) for persistent state like edge visibility and colors. These work for toggle/state data but not for one-shot commands with structured parameters.

The `OrbitCamera` (`src/viewer/camera.rs`) stores position as spherical coordinates: `target` (DVec3 look-at point), `radius` (distance from target), `yaw`, `pitch`. Default is `target=origin, radius=50, yaw=0, pitch=0.4, perspective=true, fov_y=PI/4`.

Shapes are tessellated into `MeshData { vertices: Vec<[f32; 3]>, ... }` either in `ShapeData` (Janet-side abstract values) or in `ShapeEntry` within the `ShapeRegistry`. Both carry mesh data that can be used for bounding box computation.

A `CameraAnimation` struct already exists (`app.rs:55-109`) that animates yaw/pitch with `ease_in_out(t)` over 0.5s, but it does not animate target or radius.

## Goals / Non-Goals

**Goals:**
- Two Janet functions: `(view-fit & shapes ; :keep-angle)` and `(view-fit-all ; :keep-angle)`
- Compute union AABB from mesh vertex data (explicit shapes or registry visible shapes)
- Animate camera target + radius + (optionally) yaw/pitch over 0.5s
- REPL→Viewer communication via dedicated mpsc channel
- `view-fit-all` with no visible shapes resets camera to default
- Hidden shapes ARE included when explicitly passed to `view-fit`

**Non-Goals:**
- Not changing existing `CameraAnimation` (yaw/pitch snaps); new `FitAnimation` is separate
- Not adding instant (non-animated) mode — future concern
- Not adding `view-fit` as a zero-arg synonym for `view-fit-all` — panics instead
- Not tessellating shapes on-the-fly; shapes without mesh are silently skipped
- Not adding a Janet wrapper in boot.janet — C handles keyword parsing natively

## Decisions

### Decision 1: mpsc channel for REPL→Viewer commands

**Choice**: Add a dedicated `mpsc::channel::<ReplToViewer>` for one-shot commands from the REPL thread to the viewer thread.

**Rationale**: The existing atomic-global pattern works for persistent state toggles but not for structured one-shot commands (need to pass `center: DVec3` + `radius: f64` + `keep_angle: bool`). An mpsc channel is type-safe, supports queuing, and is extensible for future REPL→Viewer commands (e.g., camera animation presets, viewport controls).

**Alternatives considered:**
- `Mutex<Option<FitCommand>>` + `AtomicBool` flag: simpler but not extensible; every new command type adds another Mutex
- `AtomicU64` bit-packing for f64: fragile, unreadable
- Writing directly to viewer thread's state via `Arc<RwLock>`: over-engineered for one-shot commands

The sender is stored in `OnceLock<mpsc::Sender<ReplToViewer>>` (`src/main.rs`) for access by `extern "C"` functions. The receiver is passed to `spawn_viewer()` then into `ViewerApp`, polled each frame.

**`ReplToViewer` enum** (in `src/types.rs`):
```rust
pub enum ReplToViewer {
    FitToBounds {
        center: DVec3,
        radius: f64,
        animate: bool,
        keep_angle: bool,
    },
}
```

### Decision 2: Bounding box computed on the REPL thread

**Choice**: Compute union AABB from mesh vertex data on the main thread (in the `extern "C"` Rust function), then send only the result (center, radius) to the viewer.

**Rationale**: The mesh data (`MeshData.vertices`) is already on the main thread, accessible from `ShapeData` pointers (for `view-fit`) and from `ShapeRegistry` (for `view-fit-all`). No need to serialize mesh data across threads — just compute min/max, send the compact result.

### Decision 3: Separate `FitAnimation` struct (not extending `CameraAnimation`)

**Choice**: Create a new `FitAnimation` struct in `src/viewer/app.rs` that animates target, radius, yaw, and pitch. The existing `CameraAnimation` is unchanged.

**Rationale**: The existing animation only handles yaw/pitch and hardcodes `camera.perspective = false` on completion (a quirk for view snaps). Mixing fit logic into it would risk breaking existing behavior. A separate struct is cleaner, with no risk of regressions.

### Decision 4: Fit math adapts to perspective vs orthographic

**Choice**: Compute optimal camera radius differently depending on `camera.perspective`.

**Rationale**:
- Perspective: visible height at distance `d` = `2 * d * tan(fov_y/2)`. We need `d` such that the bounding sphere fits both vertically (`R / tan(fov_y/2)`) and horizontally (`R / (tan(fov_y/2) * aspect)`). Take the max and multiply by 1.3 margin.
- Orthographic: `half_size = radius * 0.5`. Visible height = `radius`, visible width = `radius * aspect`. Need `radius >= max(2R, 2R/aspect) * 1.3`.

### Decision 5: `:keep-angle` keyword parsed in C

**Choice**: The C `JANET_FN` loops through `argv` to identify keyword arguments; non-keywords are treated as shapes.

**Rationale**: Janet keywords are just symbols with a `:` prefix. The C API `janet_checktype(argv[i], JANET_KEYWORD)` + `janet_keyword()` makes detection straightforward. No need for a boot.janet wrapper.

```c
// Pseudocode:
for (int i = 0; i < argc; i++) {
    if (janet_checktype(argv[i], JANET_KEYWORD)) {
        if (janet_keyword_eq(argv + i, "keep-angle")) keep_angle = true;
    } else {
        shape_count++;
    }
}
```

### Decision 6: Visibility semantics

- `view-fit` with explicit shapes includes ALL of them regardless of visibility — the user named them explicitly, so their mesh data is used even if hidden.
- `view-fit-all` only includes shapes where `ShapeEntry.visible == true` — because "all" in the registry context means "all visible."
- Shapes never tessellated (never passed through `show`) have `mesh: None` and are silently skipped in both functions.

### Decision 7: Camera reset for `view-fit-all` with no visible shapes

When no visible shapes exist, `view-fit-all` resets the camera to its default position (origin target, radius 50, yaw 0, pitch 0.4). The `:keep-angle` keyword is ignored in this case (always resets). This is the same behavior as application startup.

## Risks / Trade-offs

- **[Complexity] Adding a second animation system** — Two animation structs (`CameraAnimation` + `FitAnimation`) both run in `render()`. If both are active simultaneously, behavior is undefined. Mitigation: `FitAnimation` replaces `CameraAnimation` semantics during its duration; only one should be active. The view-snap keyboard shortcuts could overwrite a fit animation. Acceptable for now.
- **[Extensibility] `ReplToViewer` enum growth** — Adding more command variants to the enum requires `match` arms in `check_repl_commands()`. Mitigation: this is trivial and the pattern is already established.
- **[Thread safety] `REPL_TO_VIEWER` OnceLock** — Must be set before the viewer thread starts receiving. Since `spawn_viewer()` spawns the thread and returns, and we set the lock before spawning, this is safe.
- **[Performance] Per-frame channel poll** — `try_recv()` on an empty channel is near-zero cost. No measurable impact.
