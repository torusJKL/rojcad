## 1. Setup and preparation

- [x] 1.1 Read bridge/bridge.c to identify all 19 target JANET_FN functions
- [x] 1.2 Read boot.janet to understand the existing wrapper pattern (lines 18-82)

## 2. Edge visibility toggles and queries

- [x] 2.1 Strip `cad_edge_toggle_inactive` → add Janet wrapper
- [x] 2.2 Strip `cad_edge_toggle_active` → add Janet wrapper
- [x] 2.3 Strip `cad_edge_inactive_showing` → add Janet wrapper
- [x] 2.4 Strip `cad_edge_active_showing` → add Janet wrapper
- [x] 2.5 Strip `cad_edge_hidden_toggle` → add Janet wrapper
- [x] 2.6 Strip `cad_edge_hidden_showing` → add Janet wrapper
- [x] 2.7 Strip `cad_edge_hidden` → add Janet wrapper (get/set with &opt)

## 3. Projection and overlay toggles

- [x] 3.1 Strip `cad_projection_toggle` → add Janet wrapper
- [x] 3.2 Strip `cad_projection_perspective` → add Janet wrapper (get/set with &opt)
- [x] 3.3 Strip `cad_stats_overlay` → add Janet wrapper (get/set with &opt)

## 4. Help overlay

- [x] 4.1 Strip `cad_help_toggle` → add Janet wrapper
- [x] 4.2 Strip `cad_help_showing` → add Janet wrapper
- [x] 4.3 Strip `cad_help_set` → add Janet wrapper (get/set with &opt)

## 5. Window state

- [x] 5.1 Strip `cad_window_size` → add Janet wrapper
- [x] 5.2 Strip `cad_window_size_query` → add Janet wrapper
- [x] 5.3 Strip `cad_window_fullscreen` → add Janet wrapper
- [x] 5.4 Strip `cad_window_fullscreen_query` → add Janet wrapper
- [x] 5.5 Strip `cad_window_maximized` → add Janet wrapper
- [x] 5.6 Strip `cad_window_maximized_query` → add Janet wrapper

## 6. Verification

- [x] 6.1 Build and run (`just build`) — checked via `just check`
- [x] 6.2 Connect REPL and verify each function works identically
- [x] 6.3 Run existing tests (`just test`) — 82 passed
