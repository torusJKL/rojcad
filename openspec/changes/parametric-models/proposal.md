## Why

Parametric CAD is fundamentally about models that can be re-evaluated with different parameters, but rojcad currently has no concept of a "model" — just imperative Janet code that creates shapes. Users must manually track intermediate shapes, purge old geometry on parameter changes, and have no way to introspect or navigate a model's structure. Adding a declarative model layer makes rojcad a true parametric CAD tool.

## What Changes

- Add `defmodel` macro for defining parametric models with named parts
- Add `build` function to instantiate a model, returning a shape
- Add auto-purge on rebuild via existing `my-eval` shape-binding mechanism
- Add `graph` function for model introspection (AST walk of the source form)
- Add `highlight`/`highlight-clear` functions for visual feature highlighting in the viewer
- Add `*model-context*` dynamic variable and CAD function wrapping for shape tracking
- Extend viewer to support per-shape highlighting (edge + mesh tint)
- Support model composition (nested model calls create sub-model instances)

## Capabilities

### New Capabilities
- `parametric-model-definition`: The `defmodel` macro, `build` function, and model lifecycle (auto-purge, `*model-context*` tracking)
- `model-graph-introspection`: The `graph` function, AST walking of model source forms, and shape-to-node mapping
- `model-feature-highlighting`: Visual highlighting of named parts in the viewer (edge + mesh tint), highlight/clear API

### Modified Capabilities
- (none)

## Impact

- **New file**: `boot/model.janet` — parametric model runtime (pure Janet)
- **Modified**: `boot.janet` — wrap CAD functions at boot time for model tracking
- **Modified**: `src/main.rs` — add viewer commands for highlight/unhighlight
- **Modified**: `src/types.rs` — add highlight state atomics
- **Modified**: viewer rendering — support highlighted shape rendering (edge + mesh tint)
- No C bridge changes needed
