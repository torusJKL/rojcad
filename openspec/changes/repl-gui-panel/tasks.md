## 1. Channel infrastructure

- [x] 1.1 Add `GuiToRepl` and `ReplToGui` enum types to `src/types.rs` (implemented as `gui_repl.rs` module with init/FFI)
- [x] 1.2 Define the cross-thread channel statics in `src/main.rs` (mpsc channels for viewer->REPL and REPL->viewer)
- [x] 1.3 Add FFI functions: `rust_gui_repl_poll_request`, `rust_gui_repl_send_response`, registered via `janet_cfuns` in main.rs
- [x] 1.4 Create `poll-gui-repl` fiber in `boot.janet` with pipe-delimited request parsing and JSON response encoding
- [x] 1.5 Register `poll-gui-repl` in the Janet event loop via `ev/go`

## 2. Egui side panel shell

- [x] 2.1 Create `src/viewer/repl.rs` with `ReplPanel` struct
- [x] 2.2 Add `ReplPanel` to `ViewerState`
- [x] 2.3 Add `--repl` and `--no-repl` CLI flags
- [x] 2.4 Wire Ctrl+R toggle via `SHOW_REPL_PANEL` atomic
- [x] 2.5 Add egui side panel rendering using `SidePanel::right()`
- [x] 2.6 Panel layout: scrollable history, separator, input editor, toolbar

## 3. Eval flow

- [x] 3.1 Implement `submit()` with pipe-delimited `e\x02<id>\x02<code>` format
- [x] 3.2 Implement REPL-side `handle-eval` in boot.janet
- [x] 3.3 Viewer-side response polling (drain response channel each frame)
- [x] 3.4 History rendering with syntax-coloured `LayoutJob` from `code_to_job()`
- [x] 3.5 Ctrl+Enter intercepted at winit level, flag-checked on next frame
- [x] 3.6 Clickable shape results (shape_id not extracted from Janet — low priority)

## 4. Syntax highlighting

- [x] 4.1 Janet-side `highlight-scan` character scanner (replaced freja's PEG grammar)
- [x] 4.2 `handle-highlight` dispatch (deprecated — local tokenization preferred)
- [x] 4.3 Local tokenization via `egui_code_editor::Token::highlight()` — no channel needed
- [x] 4.4 `code_to_job()` builds LayoutJob via shared `HistoryEditor` + `ROJCAD_THEME`
- [x] 4.5 Rainbow bracket coloring in history (Cell depth tracker, 6-colour palette)

## 5. Code completion

- [x] 5.1 Pre-load function names (`fn_names` set exists, not populated from startup)
- [x] 5.2 Extract current word from input buffer (last whitespace/bracket to end)
- [x] 5.3 Filter `fn_names` by prefix match, limit to 20 sorted results
- [x] 5.4 Render completion popup as egui `Frame` below the code editor
- [x] 5.5 Auto-trigger on input change when prefix >= 2 chars
- [x] 5.6 Click selection inserts word into input, replaces prefix
- [x] 5.7 Dismiss on prefix < 2 chars, no matches, or Escape key

## 6. Polish and integration

- [x] 6.1 Wire SHOW_REPL_PANEL to --repl/--no-repl CLI flags
- [x] 6.2 Toolbar with Run and Clear buttons
- [x] 6.3 Panel width displayed in stats overlay (REPL_PANEL_WIDTH atomic)
- [x] 6.4 Decision: eval entire buffer on Ctrl+Enter
- [x] 6.5 Tests: 93/94 pass (1 pre-existing SIGSEGV in step file reading)
- [x] 6.6 Dynamic gizmo positioning with DPI-aware f64 arithmetic
- [x] 6.7 Upgrade egui 0.32->0.34, wgpu 25->29, add egui_code_editor
- [x] 6.8 Custom Janet Syntax definition with keywords, types, comments, quotes
- [x] 6.9 Shared ROJCAD_THEME constant for input and history colour consistency
