## 1. Simple primitives

- [x] 1.1 Strip `cad_sphere` → add Janet wrapper (manual variadic: radius + :r :c :a :ar :eager :hide)
- [x] 1.2 Strip `cad_cone` → add Janet wrapper (manual variadic: br tr h + :br :tr :h :c :a :ar :eager :hide)

## 2. Operations

- [x] 2.1 Strip `cad_extrude` → add Janet wrapper (:h :z :y :x :dir :both :eager :hide)
- [x] 2.2 Strip `cad_revolve` → add Janet wrapper (:a :ar :c :dir :eager :hide)
- [x] 2.3 Strip `cad_extrude_polygon` → add Janet wrapper (points height + :h :plane :at :eager :hide)

## 3. 2D primitives

- [x] 3.1 Strip `cad_rect` → add Janet wrapper (w d + :w :d :h :wire :plane :at :eager :hide, manual variadic)
- [x] 3.2 Strip `cad_circle` → add Janet wrapper (r + :r :wire :plane :at :eager :hide)
- [x] 3.3 Strip `cad_polygon` → add Janet wrapper (:pts :wire :plane :at :eager :hide)

## 4. Text

- [x] 4.1 Strip `cad_text` → add Janet wrapper (str font size + :depth :plane :at :eager :hide)
- [x] 4.2 Strip `cad_text3d` → add Janet wrapper (str font size depth + :plane :at :both :eager :hide)
- [x] 4.3 Strip `cad_list_fonts` → add Janet wrapper (constructs array from C primitive)

## 5. Booleans

- [x] 5.1 Strip `cad_cut` → add Janet wrapper (a b + :eager :hide)
- [x] 5.2 Strip `cad_common` → add Janet wrapper (a b + :eager :hide)
- [x] 5.3 Strip `cad_fuse` → add Janet wrapper (a b + :eager :hide)

## 6. Transforms

- [x] 6.1 Strip `cad_translate` → add Janet wrapper (shape + :t :eager :hide, manual variadic)
- [x] 6.2 Strip `cad_rotate` → add Janet wrapper (shape + :a :ar :x :y :z :r :eager)
- [x] 6.3 Strip `cad_scale` → add Janet wrapper (shape factor + :o :eager :hide)
- [x] 6.4 Strip `cad_mirror` → add Janet wrapper (shape ox oy oz dx dy dz + :eager :hide)

## 7. Verification

- [x] 7.1 Build and run (`just build`)
- [x] 7.3 Run tests (`just test`) — 82/82 pass
- [x] 7.2 Verify each function via REPL (test all keyword combos) — sphere, cone, cut/common/fuse, rect, circle, polygon, extrude, revolve, extrude-polygon, translate, rotate, scale, mirror, list-fonts all working
