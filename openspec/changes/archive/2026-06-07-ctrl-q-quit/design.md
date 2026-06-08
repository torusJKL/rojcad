## Context

The application has two threads: a main thread running the Janet REPL (TCP server on port 9365) and a background thread running the wgpu/winit 3D viewer. Currently, ESC and the window close button (`CloseRequested`) both close only the viewer thread — the `ViewerToRepl::ViewerClosed` message is sent but the receiver is discarded, so the REPL continues running with no window. There is no mechanism for the viewer to signal the main thread to exit.

The existing communication pattern between Rust ↔ Janet uses `AtomicBool`/`AtomicU64` statics in `types.rs`, exposed via `extern "C"` functions in `main.rs`, registered as Janet functions in `bridge/bridge.c`, and polled from a Janet fiber in `boot.janet` (e.g., `LAST_SELECTION` → `poll-selection`). This change follows the same pattern.

## Goals / Non-Goals

**Goals:**
- ESC does nothing at the winit level (egui can consume it freely)
- Ctrl+Q closes the entire application (viewer + REPL)
- Window close button (X) also closes the entire application
- The quit mechanism uses the established atomic-FFI-Janet pattern
- Maximum latency between Ctrl+Q and process exit: ~100ms (matches `poll-viewer` sleep interval)

**Non-Goals:**
- No changes to `--headless` mode (no viewer, no key handler, no quit mechanism needed)
- No signal-based interruption of Janet's event loop (polling is sufficient)
- No refactoring of the viewer thread lifecycle beyond the handler changes

## Decisions

1. **Atomic flag + Janet polling over `std::process::exit`** — The existing codebase uses atomic statics for cross-thread signaling (selection, visibility toggles, projection mode). Following the same pattern keeps the architecture consistent. A raw `process::exit` from the viewer thread would work but is not idiomatic for a system designed around cooperative signaling.

2. **`os/exit` in `poll-viewer` fiber over trying to break the event loop cleanly** — Janet's event loop keeps running as long as any fiber is alive. The `accept-loop` fiber is blocked on `net/accept` and can't be easily interrupted. `os/exit(0)` from the polling fiber is the simplest reliable approach. The viewer thread has already exited by this point.

3. **One-shot flag (fetch-and-swap) over sticky flag** — `QUIT_REQUESTED` uses `swap(false, Ordering::SeqCst)` so it returns `true` exactly once. This prevents a race where `poll-viewer` could read the flag twice and call `os/exit` in a loop.

4. **No `cad_fn_categories` entry** — The `quit-requested` function is internal (used only by `boot.janet`, not user-facing). Skipping categorization avoids adding a new "system" category and keeps it hidden from `(cad-fns)` listings.

## Risks / Trade-offs

- **[Race condition]** If Ctrl+Q is pressed and the poll-viewer fiber is mid-execution (after the check but before `ev/sleep`), the quit won't be detected for up to 100ms. This is acceptable latency for a user-triggered quit action.
- **[Stale viewer thread]** After `os/exit(0)`, the viewer thread's `ViewerHandle::Drop` won't run. The GPU context is cleaned up by the OS on process exit. This is the same behavior as any SIGTERM/SIGKILL.
- **[No quit in headless mode]** `--headless` users must kill the process via SIGINT or similar. This is pre-existing behavior and out of scope.

## Data Flow

```
Ctrl+Q pressed (or X clicked)
       │
       ▼
app.rs: handler fires
  QUIT_REQUESTED.store(true)
  event_loop.exit()           ← viewer thread event loop stops
       │
       ▼
poll-viewer fiber (boot.janet)
  ev/sleep 0.1 → wake
  quit-requested() → true     ← C FFI → main.rs → QUIT_REQUESTED.swap(false)
  os/exit(0)                  ← process terminates
```
