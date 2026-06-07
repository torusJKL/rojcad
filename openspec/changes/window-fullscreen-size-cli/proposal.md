## Why

The 3D viewer always starts at a hardcoded 1024×768 window. Users who want a larger layout — or prefer different dimensions — must resize manually. Adding maximized-by-default, CLI size overrides, and runtime Janet controls gives users flexible window management without leaving their workflow.

## What Changes

- Viewer starts **maximized** by default (fills screen, keeps title bar)
- `--width <PX>` and `--height <PX>` CLI flags set initial windowed size and **disable maximized**
- New Janet functions for runtime control:
  - `(window-size w h)` — resize the viewer window
  - `(window-size?)` — query current size → `[width height]`
  - `(window-fullscreen true\|false)` — enter/exit fullscreen
  - `(window-fullscreen?)` — query fullscreen state → `true/false`
  - `(window-maximized true\|false)` — enter/exit maximized
  - `(window-maximized?)` — query maximized state → `true/false`
- Window state exposed as atomics for zero-latency queries from the REPL thread

## Capabilities

### New Capabilities

- `window-size`: Viewer window sizing — default dimensions, CLI override via `--width`/`--height`, and Janet API for resize and query at runtime
- `window-fullscreen`: Fullscreen mode — runtime toggle and query via Janet API
- `window-maximized`: Maximized mode — default startup behavior, CLI override semantics, and runtime toggle/query via Janet API

### Modified Capabilities

<!-- No existing capabilities have requirement changes. -->

## Impact

- **`src/main.rs`**: CLI arg parsing (add `--width`, `--height`); `extern "C"` bridge functions for window size, fullscreen, and maximized
- **`src/types.rs`**: New `ReplToViewer` variants (`SetWindowSize`, `SetFullscreen`, `SetMaximized`); new atomics (`WINDOW_WIDTH`, `WINDOW_HEIGHT`, `WINDOW_FULLSCREEN`, `WINDOW_MAXIMIZED`)
- **`src/viewer/mod.rs`**: New `ViewerConfig` struct (with `maximized` field); updated `spawn_viewer()` signature
- **`src/viewer/app.rs`**: Consume `ViewerConfig` in `resumed()` (use `with_maximized`); handle new commands in `check_repl_commands()`; update atomics on `Resized`
- **`bridge/bridge.c`**: 6 `JANET_FN` wrappers + registration entries
- **`src/viewer/help.rs`**: Document `--width`, `--height` in CLI args overlay
