## Why

New users have no way to discover keyboard shortcuts, REPL documentation commands, or CLI arguments without reading external documentation. A floating help window provides an in-app reference, reducing the learning curve and making the tool more self-documenting.

## What Changes

- New egui `Help` window anchored center-screen, visible on startup
- Dismiss with `h`, X button, or Escape (when help is open)
- Toggle with `h` key (guarded against egui keyboard focus)
- Shows four sections: keyboard shortcuts, REPL docs, connecting to REPL, CLI arguments
- Three new Janet functions: `window-help-toggle`, `window-help-show?`, `window-help-show`
- All categorized under the `"view"` group in documentation

## Capabilities

### New Capabilities
- `help-window`: In-app help overlay showing keyboard shortcuts, REPL documentation commands, connection info, and CLI arguments

### Modified Capabilities

<!-- No existing capabilities change -->

## Impact

- **New file**: `src/viewer/help.rs` — `Help` struct + `ui()` method
- **Modified files**:
  - `src/types.rs` — add `SHOW_HELP_OVERLAY` atomic
  - `src/viewer/mod.rs` — register `help` module
  - `src/viewer/app.rs` — add `Help` to state, render it, handle `h`/Escape keys
  - `src/main.rs` — add three `extern "C"` FFI functions, import the new atomic
  - `bridge/bridge.c` — add three `JANET_FN` + extern decls + registration + group entries
- **No new dependencies**
