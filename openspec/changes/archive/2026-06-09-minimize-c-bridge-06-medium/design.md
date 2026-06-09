## Context

This is the largest phase (~300 lines C removed). These functions have real argument parsing — keyword options (some positional, some keyword-only), optional params, and multiple construction modes.

Because `:eager` and `:hide` appear in most functions, pure `&keys` doesn't work — we need manual variadic parsing (`& args` in Janet) for each function.

## Manual Variadic Parsing Pattern

Each function follows this pattern in Janet:

```janet
(def _sphere (get (get core-env 'sphere) :value))
(put (get core-env 'sphere) :value
  (fn [& args]
    (var radius nil) (var cx nil) (var cy nil) (var cz nil)
    (var angle nil) (var eager false) (var hide false)
    (var pos-count 0)
    (var i 0)
    (while (< i (length args))
      (if (= :keyword (type (args i)))
        (do (def kw (args i)) (def v (args (+ i 1)))
          (match kw
            :r (set radius v) :c (do (set cx (v 0)) (set cy (v 1)) (set cz (v 2)))
            :a (set angle (* v (/ math/pi 180)))
            :ar (set angle v)
            :eager (set eager true) :hide (set hide true))
          (set i (+ i 2)))
        (do
          (case pos-count 0 (set radius (args i)))
          (set pos-count (+ pos-count 1))
          (set i (+ i 1)))))
    (def shape (alloc-shape))
    (_init-sphere shape radius cx cy cz angle eager)
    (if (not hide) (show shape))
    shape))
```

This is ~25 lines per complex function, ~15 for simpler ones.

## Function Categories

### Simple primitives (sphere, cone)
Positional radius/height + keyword options. Manual variadic parser.

### Operations (extrude, revolve, extrude-polygon)
Shape argument + keyword options. `extrude` has axis selection (:x/:y/:z/:dir). `revolve` has angle + axis. `extrude-polygon` takes point arrays.

### 2D primitives (rect, circle, polygon)
Width/depth/radius positionals + :plane/:at/:wire keywords. `circle` is simplest, `polygon` takes :pts array.

### Text (text, text3d, list-fonts)
String + font + size positionals + keywords (:depth, :plane, :at). `list-fonts` has no args — returns array of tuples. The C version does string parsing (splitting "name|path|aspect"). This can move to Janet.

### Booleans (cut, common, fuse)
Shape a + shape b + :eager :hide. Simplest in this group — already partially wrapped in boot.janet (variadic chaining).

### Transforms (translate, rotate, scale, mirror)
Shape + params. `translate` has mixed positional/keyword (:t). `rotate` has keyword-only angle + axis selection. `scale` has :o. `mirror` is all positional.
