## Context

boot.janet already wraps `show`, `hide`, `purge`, `registry-remove` with variadic versions (lines 18-40). The original C JANET_FN is saved as `_show`, `_hide`, etc. The query functions (`visible?`, `wire?`, `face?`, `solid?`, `shape-type`) have variadic wrappers in boot.janet (lines 44-82) and also need C primitives.

## Pattern

### Already wrapped — just strip C JANET_FN

For `show`, `hide`, `purge`, `registry-remove`: the Janet wrappers already exist and save the C function. We just strip the extra logic from the C JANET_FN down to its minimum — or better, create a dedicated thin primitive and point the Janet wrapper at it.

### New wrappers for queries

For `visible?`, `wire?`, `face?`, `solid?`, `shape-type`:

C primitive (stays in bridge.c):
```c
JANET_FN(cad_visible_q, "(visible? shape)", "") {
    janet_arity(argc, 1, 1);
    void *data = unwrap_shape_or_panic(argv[0], 0);
    return rust_shape_get_visible(data) ? janet_wrap_true() : janet_wrap_false();
}
```

Janet wrapper (boot.janet):
```janet
(def _visible (get (get core-env 'visible?) :value))
(put (get core-env 'visible?) :value
  (fn [& shapes]
    (def results @[])
    (var i 0) (def n (length shapes))
    (while (< i n) (array/push results (_visible (shapes i))) (set i (+ i 1)))
    results))
```

## Functions

| Function | C Complexity | Already Wrapped? |
|----------|-------------|------------------|
| show | 5 lines | Yes (line 24) |
| hide | 5 lines | Yes (line 30) |
| purge | 5 lines | Yes (line 36) |
| registry-remove | 5 lines | Yes (line 42) |
| visible? | 5 lines | Yes (line 57) |
| wire? | 5 lines | Yes (line 66) |
| face? | 5 lines | Yes (line 74) |
| solid? | 5 lines | Yes (line 80) |
| shape-type | 5 lines | Yes (line 49) |
