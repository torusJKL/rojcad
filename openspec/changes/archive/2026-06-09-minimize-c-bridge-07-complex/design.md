## Context

These three functions are the most complex in bridge.c. Each has multiple construction modes that are dispatched based on which positional args and keywords are present. The C code uses a mix of positional counting and keyword lookup to decide which underlying `rust_init_*` to call.

## Box — Construction Modes

1. `(box 10)` → cube, size=10
2. `(box 10 20 30)` → box, w=10 d=20 h=30
3. `(box :w 10 :d 20 :h 30)` → keyword box
4. `(box :pl [0 0 0] :ph [10 20 30])` → box from corners
5. All + `:c [x y z]` for centering
6. All + `:eager` / `:hide`

Manual variadic parser: ~22 lines
C primitives needed: `_init-box`, `_init-cube`, `_init-box-from-corners`, `_alloc-shape`, `_show`

## Cylinder — Construction Modes

1. `(cylinder 5 10)` → simple cylinder, r=5 h=10 along Z
2. `(cylinder :r 5 :h 10)` → keyword style
3. `(cylinder :fp [0 0 0] :tp [0 0 10] :r 5)` → from point to point
4. `(cylinder :r 5 :h 10 :dir [0 1 0])` → along custom direction
5. All + `:c [x y z]`
6. All + `:eager` / `:hide`

Manual variadic parser: ~22 lines
C primitives needed: `_init-cylinder`, `_init-cylinder-from-points`, `_init-cylinder-point-dir`

## Torus — Construction Modes

1. `(torus 20 10)` → simple torus, rr=20 tr=10
2. `(torus :rr 20 :tr 10)` → keyword style
3. All + `:c [x y z]` for center
4. All + `:dir [dx dy dz]` for axis
5. All + `:a deg` / `:ar rad` for partial torus
6. All + `:as deg` / `:asr rad` + `:ae deg` / `:aer rad` for angle range
7. All + `:eager` / `:hide`

Manual variadic parser: ~25 lines (most keywords)
C primitives needed: `_init-torus`
