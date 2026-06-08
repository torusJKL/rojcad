## 1. Sketch creation and cursor movement

- [x] 1.1 Strip `cad_sketch` → add Janet wrapper (:plane :at keywords)
- [x] 1.2 Strip `cad_move_to` → add Janet wrapper (sketch x y)
- [x] 1.3 Strip `cad_line_to` → add Janet wrapper (sketch x y)

## 2. Relative sketch operations

- [x] 2.1 Strip `cad_line_dx` → add Janet wrapper (sketch dx)
- [x] 2.2 Strip `cad_line_dy` → add Janet wrapper (sketch dy)
- [x] 2.3 Strip `cad_line_dx_dy` → add Janet wrapper (sketch dx dy)
- [x] 2.4 Strip `cad_arc_to` → add Janet wrapper (sketch x2 y2 x3 y3)

## 3. Sketch completion

- [x] 3.1 Strip `cad_close_sketch` → add Janet wrapper (sketch + :eager :hide)
- [x] 3.2 Strip `cad_build_wire` → add Janet wrapper (sketch + :eager :hide)

## 4. Wire operations

- [x] 4.1 Strip `cad_wire_to_face` → add Janet wrapper (wire + :eager :hide)
- [x] 4.2 Strip `cad_wire_fillet` → add Janet wrapper (wire :r + :eager :hide)
- [x] 4.3 Strip `cad_wire_chamfer` → add Janet wrapper (wire :d + :eager :hide)
- [x] 4.4 Strip `cad_wire_offset` → add Janet wrapper (wire :d + :eager :hide)

## 5. Verification

- [x] 5.1 Build and run (`just build`)
- [x] 5.2 Verify sketch pipeline: (-> (sketch) (line-to 10 0) (line-to 10 10) (close-sketch))
- [x] 5.3 Verify wire operations
- [x] 5.4 Run tests (`just test`)
