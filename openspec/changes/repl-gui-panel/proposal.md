## Why

rojcad currently requires an external TCP client (nc, Conjure) to interact with its REPL. New users must discover this, install tooling, and manage a separate connection. An in-process REPL GUI eliminates this friction — the CAD environment is immediately interactive inside the same window.

## What Changes

- Add a REPL panel docked to the right side of the viewer window, built with egui
- Use a pipe-delimited (`\x02`-separated) eval protocol between viewer and REPL threads
- Syntax highlighting via `egui_code_editor` crate with a custom Janet syntax definition, applied both to the input editor and the history output log
- Rainbow bracket coloring in the history output (depth-based palette)
- Input uses `egui_code_editor::CodeEditor` widget with syntax-colored display
- Code completion popup with local filtering from pre-loaded function names, auto-triggered after 2+ characters
- Keyboard shortcut Ctrl+R to toggle panel visibility
- `--repl` (show panel) and `--no-repl` (hide panel) CLI flags
- Gizmo positioning adjusted dynamically based on panel width with proper DPI-aware scaling
- Panel width displayed in the stats overlay (Ctrl+Shift+Alt+S)
- Upgraded egui 0.32 → 0.34 and wgpu 25 → 29 to support `egui_code_editor`

## Capabilities

### New Capabilities
- `gui-repl-panel`: In-window REPL panel with code input, structured output log, eval, syntax highlighting via `egui_code_editor`, rainbow brackets, code completion popup, and gizmo-aware right-side docking

### Modified Capabilities
<!-- No existing specs have requirement changes — the TCP REPLs and viewer-repl sync remain unchanged -->

## Impact

- **New source files**: `src/viewer/repl.rs` (panel widget, history editor, syntax helper), `src/gui_repl.rs` (channel + FFI functions)
- **Modified files**: `src/viewer/repl.rs` (completion state, popup UI, filtering), `src/viewer/app.rs` (ViewerState, gizmo positioning, egui pass), `src/viewer/mod.rs` (repl module export), `src/viewer/stats.rs` (panel width display), `src/main.rs` (channel setup, FFI registration, CLI flags), `src/types.rs` (SHOW_REPL_PANEL, REPL_PANEL_WIDTH), `src/bridge.rs` (janet_cstring, Janet Clone+Copy), `boot.janet` (poll-gui-repl fiber, highlight scanner, JSON encoder), `Cargo.toml` (upgraded egui/wgpu, added egui_code_editor)
- **Dependency upgrades**: egui 0.32→0.34, wgpu 25→29, added `egui_code_editor` 0.3
- **No breaking changes**: TCP REPLs remain, viewer-repl sync unchanged, existing CLI flags unchanged
