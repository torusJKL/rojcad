## Why

Today, binding a shape to a name requires two steps: `(def b (box 10))` to create and bind, then `(display b)` or `(show b)` to make it visible in the viewer. The `display` C function combines show+return but still requires the outer `(def ... (display ...))` wrapper. A would-be `(display name expr)` macro fails because Janet's bootstrap mode prevents `def` inside a `do` from registering bindings in `core-env`. This friction breaks the interactive CAD workflow where every `def`'d shape should just appear.

## What Changes

- **Auto-show on `def`**: When the REPL evaluates a top-level `(def name shape-expr)` and the result is a shape, the shape is automatically shown in the viewer.
- **`:hide` keyword**: All shape constructors gain an optional `:hide` keyword. When present, the shape is created with `visible=false`, suppressing the auto-show. The user can later call `(show name)` explicitly.
- **Remove `display`** **BREAKING**: The `display` C function is removed. It's redundant — `(def b (box 10))` now does what `(def b (display (box 10)))` used to do.
- Unbound shape expressions like `(box 10)` or `(translate (box 10) 5 0 0)` are NOT auto-shown — only `def`-bound shapes trigger the auto-show.

## Capabilities

### New Capabilities
- `shape-visibility`: Auto-show shapes bound via top-level `def`, with `:hide` keyword for opt-out

### Modified Capabilities

<!-- No existing specs to modify -->

## Impact

- `bridge/bridge.c`: Add `:hide` keyword to all shape constructors (box, sphere, cylinder, cone, torus, cut, common, fuse, translate, rotate, scale, mirror). Remove `cad_display` function and its registration.
- `boot.janet`: Modify `my-eval` to inspect the top-level form and auto-show `def`'d shapes.
- No Rust changes needed (ShapeData, cad.rs, main.rs, types.rs unchanged).
- Backward compatible for non-`def` usage. Code using `(display ...)` will break (removed) — migration is to drop `display` and rely on auto-show.
