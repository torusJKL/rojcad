## Why

ESC currently closes the 3D viewer window, which is surprising and disruptive — users who press ESC to close an egui help dialog accidentally close the entire viewer. There's no standard desktop shortcut (Ctrl+Q) to quit the application, and the window's close button (X) only stops the viewer thread while the REPL keeps running in the background.

## What Changes

- **REMOVE** ESC keybinding for closing the viewer. ESC falls through to the default handler (egui consumes it for closing help dialogs).
- **ADD** Ctrl+Q keybinding to quit the entire application (viewer + REPL process).
- **UPDATE** `WindowEvent::CloseRequested` to also exit the entire application (not just the viewer thread).
- **ADD** Janet-visible `quit-requested` function so `boot.janet` can cleanly exit its event loop when a quit is signaled.
- **UPDATE** `boot.janet`'s `poll-viewer` fiber to check `(quit-requested)` each cycle and call `(os/exit 0)`.

## Capabilities

### New Capabilities
- `keyboard-quit`: Ctrl+Q shortcut to exit the application, plus ESC passthrough for egui

### Modified Capabilities

<!-- No existing capability specs are affected. This is purely a keyboard shortcut change. -->

## Impact

- **Viewer thread** (`src/viewer/app.rs`): Key event matching and close-requested handler
- **Shared types** (`src/types.rs`): New `QUIT_REQUESTED` atomic flag
- **Rust FFI** (`src/main.rs`): New `rust_quit_requested()` C-callable function
- **C bridge** (`bridge/bridge.c`): New `(quit-requested)` Janet function
- **Boot script** (`boot.janet`): Quit check in `poll-viewer` fiber
- No new dependencies. The mechanism follows the existing pattern used by `poll-selection` (atomic + C FFI + Janet function).
