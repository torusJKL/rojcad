## 1. Rename cryptic thin primitives in C

- [x] 1.1 Rename `"_bx"` → `"_init-box"` in cfuns[] array
- [x] 1.2 Rename `"_cb"` → `"_init-cube"` in cfuns[]
- [x] 1.3 Rename `"_bfc"` → `"_init-box-from-corners"` in cfuns[]
- [x] 1.4 Rename `"_cyfp"` → `"_init-cylinder-from-points"` in cfuns[]
- [x] 1.5 Rename `"_cydir"` → `"_init-cylinder-point-dir"` in cfuns[]
- [x] 1.6 Strip `"_cy"` (replaced by non-underscore `"cylinder"`)
- [x] 1.7 Strip `"_tr"` (replaced by non-underscore `"torus"`)

## 2. Strip redundant underscore C registrations (keep non-underscore)

- [x] 2.1 Strip `"_sphere"`, `"_cone"`, `"_cut"`, `"_common"`, `"_fuse"`
- [x] 2.2 Strip `"_translate"`, `"_rotate"`, `"_scale"`, `"_mirror"`
- [x] 2.3 Strip `"_rect"`, `"_circle"`, `"_polygon"`
- [x] 2.4 Strip `"_extrude"`, `"_revolve"`, `"_extrude-polygon"`
- [x] 2.5 Strip `"_wire-to-face"`, `"_wire-fillet"`, `"_wire-chamfer"`, `"_wire-offset"`
- [x] 2.6 Strip `"_sketch"`, `"_move-to"`, `"_line-to"`, `"_line-dx"`, `"_line-dy"`, `"_line-dx-dy"`, `"_arc-to"`, `"_close-sketch"`, `"_build-wire"`
- [x] 2.7 Strip `"_text"`, `"_text3d"`, `"_list-fonts"`
- [x] 2.8 Strip `"_view-fit"`, `"_view-fit-all"`, `"_view-angle"`
- [x] 2.9 Strip `"_edge-thickness"`, `"_edge-color-inactive"`, `"_edge-color-active"`
- [x] 2.10 Strip `"_quit-requested"` (keep `"quit-requested"`)
- [x] 2.11 Keep `"_poll-selection-raw"`, `"_get-selected-ids"`, `"_get-registered-ids"`, `"_get-shape"` (underscore-only helpers)

## 3. Update boot.janet captures for renamed primitives

- [x] 3.1 Change `'_bx` → `'_init-box` (line 230)
- [x] 3.2 Change `'_cb` → `'_init-cube` (line 232)
- [x] 3.3 Change `'_bfc` → `'_init-box-from-corners` (line 234)
- [x] 3.4 Change `'_cy` → `'cylinder` (line 291)
- [x] 3.5 Change `'_cyfp` → `'_init-cylinder-from-points` (line 293)
- [x] 3.6 Change `'_cydir` → `'_init-cylinder-point-dir` (line 295)
- [x] 3.7 Change `'_tr` → `'torus` (line 352)

## 4. Remove C metadata table and add Janet-side metadata

- [x] 4.1 Remove `cad_fn_categories` static array (bridge.c lines 1892–1925)
- [x] 4.2 Remove the metadata-enrichment loop (bridge.c lines 2094–2102)
- [x] 4.3 Add `:source`/`:category` metadata in boot.janet for all wrappers (Pattern A + C)

## 5. Compress metadata with category-keyed groups

- [x] 5.1 Replace ~126 repetitive `(put ...)` lines with category-keyed table + `while`/`next` loop
- [x] 5.2 Verify group iteration handles all categories correctly
- [x] 5.3 Build and test

## 6. Verification

- [x] 6.1 Build (`just build`)
- [x] 6.2 Run tests (`just test`) — 82/82 pass
- [x] 6.3 Verify REPL: sphere, box, shape-type, metadata all work
- [x] 6.4 Verify public API names unchanged
