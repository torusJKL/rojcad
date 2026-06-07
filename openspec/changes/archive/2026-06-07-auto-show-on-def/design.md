## Context

The Janet REPL (`boot.janet`) evaluates forms via `my-eval`, which compiles them against a captured `core-env` and resumes a fiber. Each shape-constructing C function (box, sphere, etc.) allocates a `ShapeData` via `janet_abstract`, initializes it with OCCT geometry, and returns the abstract value to Janet. Shapes are NOT automatically registered in the viewer — registration (tessellation + entry in `ShapeRegistry`) only happens when `show()` is called explicitly.

The `display` C function called `show()` then returned the shape, requiring the pattern `(def b (display (box 10)))`. A macro `(display name expr)` failed because bootstrapped Janet cannot register `core-env` bindings from `def` inside `do` inside a macro expansion.

The viewer runs on a separate thread and reads `ShapeRegistry` each frame. `ShapeData` has fields `visible: bool` (default `true`), `registered: bool` (default `false`). Calling `hide()` on an unregistered shape just sets `visible = false` — no viewer interaction.

### Affected files

| File | Role |
|---|---|
| `bridge/bridge.c` | C JANET_FN constructors + registration |
| `boot.janet` | REPL eval loop |

### Unchanged files

`src/types.rs`, `src/cad.rs`, `src/main.rs`, `src/bridge.rs` — no Rust changes needed.

## Goals / Non-Goals

**Goals:**
- Auto-show shapes bound via top-level `(def name shape-expr)` in the REPL
- Provide `:hide` keyword to suppress auto-show on a per-shape basis
- Remove the now-redundant `display` C function
- Non-`def` shape expressions remain invisible (correct GC behavior)
- Zero Rust changes

**Non-Goals:**
- Not changing special forms (`def`, `set`, `do`) — all logic stays in the REPL eval layer
- Not adding `:hide` to non-CAD Janet functions
- Not modifying the viewer or shape registry internals

## Decisions

### Decision 1: Form inspection in boot.janet (not C-level auto-show)

Auto-show on every shape construction would leak intermediate values. E.g., `(translate (box 10) 5 0 0)` would show both the inner box and the translated result. Instead, `my-eval` inspects the **original parsed form**:

```
form = (def name expr)  →  is result a shape?  →  visible?  →  show
form = (box 10)         →  skip (not a def)
```

This is a single `if` condition in `boot.janet` with zero changes to the C compilation pipeline.

### Decision 2: `:hide` via simple C helper

A tiny `maybe_hide` helper checks `argv` for `:hide` and calls `rust_shape_hide` before returning. This leverages the existing behavior of `hide()` on unregistered shapes (just sets visible=false). No new Rust exports, no new ShapeData fields.

### Decision 3: Remove `display` entirely

The `cad_display` JANET_FN and its registration entry are deleted. It served only to combine `show` + return, which is now subsumed by auto-show. Any existing usage `(display <shape>)` becomes `(<shape>)` (a no-op expression) or simply `(def b <shape>)`.

### Decision 4: No post-eval check on non-`def` forms

Only `(def name expr)`-shaped tuples trigger the auto-show check. This correctly excludes:
- `(set name expr)` — unusual in REPL, excluded for simplicity
- `(do ...)` — compound forms where auto-show would be ambiguous
- Bare expressions like `(box 10)` — not bound, should GC

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| `(do (def b (box 10)) (fn b))` won't auto-show b (top-level form is `do`, not `def`) | Rare in REPL usage. User writes `(show b)` explicitly. |
| Users who relied on `(display shape)` must migrate | Simple mechanical change: drop `display` wrapper. If bound, switch to `(def name shape)`. If unbound, just `shape` was meaningless. |
| `:hide` keyword could conflict with future keyword | No existing `:hide` usage. Namespace collision unlikely. Rename to `:invisible` or `:hidden` if conflict arises. |
| Form inspection checks `(= (first form) 'def)` — fragile against future Janet changes to `def` syntax | `def` is a fundamental special form unlikely to change. If it does, the condition fails open (no auto-show, user calls `show` manually). |
