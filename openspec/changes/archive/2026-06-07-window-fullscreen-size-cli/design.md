## Context

The viewer thread creates a winit window in `ApplicationHandler::resumed()` with a hardcoded `LogicalSize::new(1024.0, 768.0)`. Window dimensions, fullscreen, and maximized state are completely static — no CLI args, no Janet API, no atomics. The REPL thread and viewer thread communicate via:

- **Atomics** (in `src/types.rs`): for state the viewer reads every frame (projection mode, stats/help visibility, edge settings). Zero-latency reads from either thread.
- **mpsc channel** (`ReplToViewer`): for one-shot imperative commands (fit-to-bounds, set view angles). The viewer drains these in `check_repl_commands()` each frame.

Window creation flow:

```
main() → spawn_viewer(repl_rx, config) → thread::spawn → run_viewer(rx, config)
  → EventLoop::builder().with_any_thread(true)
  → event_loop.run_app(&mut ViewerApp)
  → winit calls ViewerApp::resumed()
     → Window::default_attributes().with_inner_size(1024, 768)
     → create wgpu device, surface, renderers
```

## Goals / Non-Goals

**Goals:**

- Viewer starts maximized by default (fills screen, keeps title bar / window decorations)
- `--width <PX>` and `--height <PX>` CLI flags customize initial size and disable maximized
- Janet API for runtime window resize, fullscreen toggle, and maximized toggle (6 functions)
- Zero-latency queries from the REPL thread for current size, fullscreen, and maximized state
- Backward compatible: existing headless mode, port, and eval flags unaffected

**Non-Goals:**

- Window position control (move window)
- Exclusive fullscreen (borderless only)
- GUI-based resize handles in the 3D view (OS window decorations handle this)

## Decisions

### 1. Maximized by default, CLI args imply windowed mode

| Scenario | Behavior |
|---|---|
| No `--width` or `--height` | Maximized window (fills screen, keeps decorations) |
| `--width 1280` only | Windowed 1280×768 (default height) |
| `--height 720` only | Windowed 1024×720 (default width) |
| `--width 800 --height 600` | Windowed 800×600 |

**Rationale:** Simple, no extra flags. Maximized is less intrusive than fullscreen — keeps the title bar accessible and works uniformly across all window managers. If a user specifies dimensions they want a window at that exact size.

### 2. `with_maximized(true)` in window attributes

Set `with_maximized(true)` on `WindowAttributes` during window creation, rather than calling `set_maximized()` after. This ensures the window manager applies maximized state immediately, before the window is mapped.

**Rationale:** Build-time attributes are more reliable across X11 and Wayland than post-creation calls.

### 3. Atomics for queries, channel for commands

```
Janet (window-size?)       → reads WINDOW_WIDTH / WINDOW_HEIGHT atomics   → instant
Janet (window-size w h)    → sends SetWindowSize via ReplToViewer channel → next frame
Janet (window-fullscreen?) → reads WINDOW_FULLSCREEN atomic               → instant
Janet (window-fullscreen)  → sets atomic + sends SetFullscreen via channel → next frame
Janet (window-maximized?)  → reads WINDOW_MAXIMIZED atomic               → instant
Janet (window-maximized)   → sets atomic + sends SetMaximized via channel → next frame
```

**Rationale:** Queries must be synchronous from the REPL thread; atomics provide zero-latency reads. Window operations (`request_inner_size`, `set_fullscreen`, `set_maximized`) are imperative and require the viewer thread's `Window` handle, so they go through the existing channel.

### 4. Atomics updated by viewer on `Resized` event

The existing `WindowEvent::Resized` handler already runs on every resize (including fullscreen transitions). We add writes to `WINDOW_WIDTH` / `WINDOW_HEIGHT` there, ensuring the atomics always reflect the true window size regardless of how it changed (OS drag, fullscreen toggle, `request_inner_size`).

### 5. `ViewerConfig` struct passed through spawn chain

```rust
pub struct ViewerConfig {
    pub width: u32,      // default 1024
    pub height: u32,     // default 768
    pub maximized: bool, // true by default (unless --width/--height given)
}
```

Passed as: `main()` → `spawn_viewer(rx, config)` → `run_viewer(tx, rx, running, config)` → `ViewerApp { config, .. }`.

**Rationale:** Avoids global statics for initial setup. The config is consumed once in `resumed()` and never changes after that (runtime changes use the channel).

### 6. Borderless fullscreen for runtime toggle

`Fullscreen::Borderless(None)` — uses the current monitor's native resolution. `None` tells winit to fullscreen on whichever monitor the window is currently on, avoiding the need to acquire a `MonitorHandle`.

### 7. `set_maximized()` for runtime maximized toggle

`window.set_maximized(mx)` — winit's built-in method. The viewer thread calls it when it receives `SetMaximized` from the channel. The `WINDOW_MAXIMIZED` atomic is set optimistically on the REPL thread before the channel send, then confirmed on the viewer side.

## Risks / Trade-offs

- **[Race: query before first frame]** `(window-size?)`, `(window-fullscreen?)`, or `(window-maximized?)` called before the viewer thread has created the window will return the default atomic values. In practice the REPL doesn't accept input until boot.janet finishes, by which time the viewer has already created the window. Acceptable.
- **[Wayland fullscreen]** Some Wayland compositors may not support `set_fullscreen`. Runtime fullscreen toggle may be a no-op on some compositors — the atomic and channel will still reflect the intended state.
- **[Fullscreen → windowed size restoration]** When exiting fullscreen, winit restores the window to its size *before* the last `set_fullscreen()` call. If the user calls `(window-size 800 600)` while in fullscreen, exiting fullscreen may revert to the pre-fullscreen size, not 800×600.
- **[Maximized + fullscreen interaction]** If a user starts maximized then calls `(window-fullscreen true)`, exiting fullscreen returns to a windowed state (not maximized). winit does not preserve the maximized flag across fullscreen transitions.
