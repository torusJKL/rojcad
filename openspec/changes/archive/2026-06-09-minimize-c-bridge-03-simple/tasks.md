## 1. Quit and selection

- [x] 1.1 Strip `cad_quit_requested` → add Janet wrapper
- [x] 1.2 Strip `cad_on_select` → add Janet wrapper (stores function in Janet variable)
- [x] 1.3 Strip `cad_poll_selection` → add Janet wrapper (reads atomic, builds event, invokes callback)

## 2. Shape queries

- [x] 2.1 Strip `cad_selected_shapes` → add Janet wrapper (IDs → shape lookup → tuple)
- [x] 2.2 Strip `cad_list_shapes` → add Janet wrapper (filter + tuple construction)

## 3. Edge styling

- [x] 3.1 Strip `cad_edge_thickness` → add Janet wrapper (get/set with &opt)
- [x] 3.2 Strip `cad_edge_color_inactive` → add Janet wrapper
- [x] 3.3 Strip `cad_edge_color_active` → add Janet wrapper

## 4. View control

- [x] 4.1 Strip `cad_view_fit` → add Janet wrapper (variadic shapes + :reset keyword)
- [x] 4.2 Strip `cad_view_fit_all` → add Janet wrapper (keywords :hidden :reset)
- [x] 4.3 Strip `cad_view_angle` → add Janet wrapper (2-3 positional args)

## 5. Verification

- [x] 5.1 Build and run (`just build`)
- [x] 5.2 Verify each function via REPL
- [x] 5.3 Run tests (`just test`)
