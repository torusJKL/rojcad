## Context

Three I/O functions that validate a string path, call a Rust FFI function, and propagate errors.

## Pattern

C primitive (stays in bridge.c):
```c
JANET_FN(cad_write_step, "(write-step shape path)", "") {
    janet_arity(argc, 2, 2);
    void *data = unwrap_shape_or_panic(argv[0], 0);
    const char *path = (const char *)janet_unwrap_string(argv[1]);
    int result = rust_write_step(data, path);
    if (result != 0) {
        const char *msg = rust_take_last_error();
        janet_panic(msg);
    }
    return janet_wrap_nil();
}
```

Janet wrapper:
```janet
(def _write-step (get (get core-env 'write-step) :value))
(put (get core-env 'write-step) :value
  (fn [shape path]
    (_write-step shape path)))
```

`read-step` has `:eager` and `:hide` keywords, so slightly more complex — it needs the manual variadic `&` pattern to pass `:eager` through.

## Functions

| Function | Args | Notes |
|----------|------|-------|
| write-step | shape, path | String path, error propagation |
| write-stl | shape, path | Same pattern |
| read-step | path, :eager, :hide | Allocates shape, has keywords |
