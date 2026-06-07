## 1. Core Help Window UI

- [x] 1.1 Add `SHOW_HELP_OVERLAY: AtomicBool` (default `true`) to `src/types.rs` next to `SHOW_STATS_OVERLAY`
- [x] 1.2 Create `src/viewer/help.rs` with `Help` struct and `ui()` method rendering a centered egui window with four sections (keyboard shortcuts, REPL docs, connection info, CLI args)
- [x] 1.3 Register `pub mod help;` in `src/viewer/mod.rs`
- [x] 1.4 Add `help: Help` field to `ViewerState` in `src/viewer/app.rs`, initialize in `resumed()`, and call `state.help.ui(ctx)` in the egui render pass

## 2. Keyboard Shortcuts

- [x] 2.1 Add `h`/`H` key handler in `window_event()` that toggles `SHOW_HELP_OVERLAY`, guarded by `!state.egui_ctx.wants_keyboard_input()`
- [x] 2.2 Add Escape handler that closes help if visible (no-op otherwise)
- [x] 2.3 Update help window content to list the correct keybindings (Esc closes help, Ctrl+Q quits)

## 3. Janet Integration

- [x] 3.1 Add `SHOW_HELP_OVERLAY` to the `use types::{...}` import in `src/main.rs`
- [x] 3.2 Add three `#[unsafe(no_mangle)] extern "C"` functions in `src/main.rs`: `rust_help_overlay_toggle`, `rust_help_overlay_showing`, `rust_help_overlay_set`
- [x] 3.3 Add `extern` declarations for the three Rust functions in `bridge/bridge.c` (in the extern C block)
- [x] 3.4 Add three `JANET_FN` implementations in `bridge/bridge.c`: `cad_help_toggle`, `cad_help_showing`, `cad_help_set`
- [x] 3.5 Register the three functions in `cad_register_functions` and add group entries to `cad_groups` (category: `"view"`)

## 4. Verification

- [x] 4.1 Run `cargo check` to verify compilation
- [x] 4.2 Run `cargo test` to verify no regressions
