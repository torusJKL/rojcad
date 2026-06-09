## Context

Sketch operations take a sketch, return a new sketch — a functional persistent pattern. The C JANET_FN functions just unwrap, call FFI, wrap. In Janet this becomes a natural chain.

Wire operations operate on shapes (not sketches) but follow the same thin-wrapper pattern.

## Pattern

C primitive for sketch operations:
```c
JANET_FN(cad_move_to, "(move-to sketch x y)", "") {
    janet_arity(argc, 3, 3);
    void *src = unwrap_sketch_or_panic(argv[0], 0);
    void *dest = janet_abstract(&rojcad_sketch_type, rust_sketch_data_size());
    rust_sketch_move_to(dest, src,
        janet_unwrap_number(argv[1]), janet_unwrap_number(argv[2]));
    return janet_wrap_abstract(dest);
}
```

Janet wrapper:
```janet
(def _move-to (get (get core-env 'move-to) :value))
(put (get core-env 'move-to) :value
  (fn [sketch x y]
    (_move-to sketch x y)))
```

Wire operations follow the same pattern but work on shapes and may have optional keywords (`:eager`, `:hide`, `:r`, `:d`).

## Functions

| Function | Type | Keywords |
|----------|------|----------|
| sketch | creation | :plane :at |
| move-to | sketch | none (positional) |
| line-to | sketch | none |
| line-dx | sketch | none |
| line-dy | sketch | none |
| line-dx-dy | sketch | none |
| arc-to | sketch | none |
| close-sketch | shape | :eager :hide |
| build-wire | shape | :eager :hide |
| wire-to-face | shape | :eager :hide |
| wire-fillet | shape | :r :eager :hide |
| wire-chamfer | shape | :d :eager :hide |
| wire-offset | shape | :d :eager :hide |
