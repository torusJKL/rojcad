## Context

Bridge/bridge.c contains ~2600 lines of C JANET_FN wrapper functions. Each wrapper: (1) parses Janet arguments, (2) calls a rust_* FFI function, (3) wraps the result back into a Janet value. The argument parsing is the bulk — for trivial getter/setters it's 10-15 lines of C for what is effectively a 1-line operation.

This phase targets the simplest functions: edge toggles, projection mode, overlay state, window state.

## Pattern

Each function follows this migration pattern:

### Thin C Primitive (kept in bridge.c)

The existing JANET_FN is stripped to its minimum — arity check + FFI call + wrap result:

```c
JANET_FN(cad_edge_toggle_inactive, "(edge-toggle-inactive)", "") {
    janet_arity(argc, 0, 0);
    (void)argv;
    int result = rust_edge_toggle_inactive();
    return result ? janet_wrap_true() : janet_wrap_false();
}
```

### Janet Wrapper (added to boot.janet)

Saves the C primitive then replaces the binding:

```janet
(def _edge-toggle-inactive
  (fn [] ((get (get core-env 'edge-toggle-inactive) :value))))

(put (get core-env 'edge-toggle-inactive) :value
  (fn [] (_edge-toggle-inactive)))
```

For get/set pairs (`edge-hidden`, `projection-perspective`, `stats-overlay`, `window-help-show`, `window-size`, `window-fullscreen`, `window-maximized`), the Janet wrapper uses `&opt`:

```janet
(def _edge-hidden (get (get core-env 'edge-hidden) :value))
(put (get core-env 'edge-hidden) :value
  (fn [&opt value]
    (if (not= nil value)
      (_edge-hidden-set value)
      (_edge-hidden-show?))))
```

## Functions by Category

**Pure toggle (no args → bool):**
`edge-toggle-inactive`, `edge-toggle-active`, `edge-hidden-toggle`, `projection-toggle`, `window-help-toggle`

**Pure query (no args → bool):**
`edge-inactive-show?`, `edge-active-show?`, `edge-hidden-show?`, `window-help-show?`, `window-fullscreen?`, `window-maximized?`

**Get/set (0 or 1 arg):**
`edge-hidden`, `projection-perspective`, `stats-overlay`, `window-help-show`, `window-size` (2 args), `window-fullscreen`, `window-maximized`

**Query returning tuple:**
`window-size?` (returns [w h])

## Files Changed

- `bridge/bridge.c` — strip ~54 lines
- `boot.janet` — add ~60 lines

## Risks

- None — these functions are pure reads/writes of atomics, no complex state
