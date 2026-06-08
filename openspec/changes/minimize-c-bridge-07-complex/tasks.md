## 1. Box

- [x] 1.1 Strip `cad_box` → add Janet wrapper
      Modes: (box w d h), (box size) as cube, (box :w :d :h), (box :pl :ph), (box :c), all + :eager :hide
      Manual variadic parser ~22 lines

## 2. Cylinder

- [x] 2.1 Strip `cad_cylinder` → add Janet wrapper
      Modes: (cylinder r h), (cylinder :fp :tp :r), (cylinder :dir), (cylinder :c), all + :eager :hide
      Manual variadic parser ~22 lines

## 3. Torus

- [x] 3.1 Strip `cad_torus` → add Janet wrapper
      Modes: (torus rr tr), (torus :rr :tr), (torus :c :dir :a :as :ae), all + :eager :hide
      Manual variadic parser ~25 lines

## 4. Verification

- [x] 4.1 Build and run (`just build`)
- [x] 4.2 Verify each function with all mode combos via REPL
- [x] 4.3 Run tests (`just test`)
- [x] 4.4 Final line count: bridge.c 2103 lines (2163 → 2103, −60 in this change; cumulative −509 across series; original was 339)
