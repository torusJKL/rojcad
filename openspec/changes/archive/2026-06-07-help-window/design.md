## Context

The viewer uses egui (v0.31) for its stats overlay panel (`src/viewer/stats.rs`). That panel is a single `egui::Window` anchored top-left, gated by an `AtomicBool` (`SHOW_STATS_OVERLAY`), and rendered inside `state.egui_ctx.run(...)`. Keyboard shortcuts are handled via raw winit `WindowEvent::KeyboardInput` in `app.rs`.

Ctrl+Q already closes the application. Escape currently falls through as a no-op in the keyboard match.

The bridge pattern for exposing viewer state to Janet is well-established: `AtomicBool` in `types.rs` → `extern "C"` FFI function in `main.rs` → `JANET_FN` in `bridge.c` → registration in `cad_register_functions`.

## Goals / Non-Goals

**Goals:**
- Centered floating egui window showing keyboard shortcuts, REPL docs, connection info, and CLI args
- Visible on first startup
- Toggle with `h` key (guarded against egui keyboard focus)
- Close with Escape, X button, or `h` again
- Three Janet functions for programmatic control: `window-help-toggle`, `window-help-show?`, `window-help-show`
- All new code follows existing patterns (stats overlay for egui, back-edges for Janet FFI)

**Non-Goals:**
- Searchable help or interactive documentation
- Dynamic content (port number, etc.) — static text is fine
- Hot-reload of help content

## Decisions

| Decision | Choice | Alternatives |
|---|---|---|
| Visibility mechanism | `AtomicBool` in `types.rs` | Could use a `ViewerState` field, but atomic allows Janet REPL to toggle it too |
| Startup state | Default `true` (visible) | User sees help immediately, dismisses when ready |
| Close button sync | `.open(&mut visible)` + store back to atomic | Handles both keyboard toggle and X-button close correctly |
| Egui keyboard guard | Check `!state.egui_ctx.wants_keyboard_input()` before `h` | Prevents key capture when egui text fields have focus (future-proofing) |
| Escape priority | Close help first if open, otherwise no-op | Escape doesn't close the app — Ctrl+Q does that |
| Window anchor | `Align2::CENTER_CENTER` | Matches "floating help window in center" requirement |
| Janet function naming | `window-help-toggle`, `window-help-show?`, `window-help-show` | Follows existing `edge-hidden-toggle`/`edge-hidden-show?`/`edge-hidden` triple pattern |
| Category | `"view"` group | Matches `stats-overlay` and `view-fit` |

## Risks / Trade-offs

- [Race] The `.open(&mut visible)` + atomic store-back pattern has a brief window where another thread could toggle the atomic between load and store. Mitigation: only the viewer thread currently toggles help visibility, so this is theoretical.
- [Layout] Help content is hardcoded. If keybindings change, `help.rs` must be updated manually. Mitigation: low-velocity change — keybindings rarely change.
- [Overlap] Help window and stats overlay can be open simultaneously. Mitigation: help is centered, stats is top-left, they don't overlap.

## Migration Plan

N/A — new feature, no migration needed.

## Open Questions

None.
