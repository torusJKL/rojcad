## Context

rojcad currently has no concept of a "model" — shapes are created imperatively via Janet CAD functions. Each shape is an opaque abstract value. The relationship between shapes (e.g., shape B was produced by cutting shape A with shape C) is not preserved after creation. Users must manually purge old shapes when parameters change.

The viewer renders shapes via a shared `ShapeRegistry` with mesh data. Shape selection exists (for mouse picking) but there is no "feature highlighting" concept.

## Goals / Non-Goals

**Goals:**
- Allow users to define parametric models with `defmodel` macro
- Provide `build` function that executes a model and returns a shape
- Auto-purge old intermediate shapes on re-build via existing `my-eval` shape-binding mechanism
- Expose model structure via `graph` function (AST walk of source form)
- Support model composition (nested model calls as sub-model instances)
- Highlight named parts in the viewer (edge + mesh tint)
- Implement entirely in Janet where possible; minimal viewer changes for highlighting

**Non-Goals:**
- Constraint solving (not adding a constraint engine)
- Full model-to-CAD-kernel undo history (just rebuild/purge)
- File format for model serialization (future work)
- Interactive model tree UI in the viewer (future work)

## Decisions

### Decision 1: Pure Janet model layer, no Rust/C bridge changes for model logic

- **Chosen**: The `defmodel` macro, `build`/`graph`/`highlight` functions are pure Janet code in `boot/model.janet`.
- **Rationale**: Model semantics are about composition, naming, and metadata — OCCT doesn't need to know about them. Keeping the layer in Janet avoids adding more extern "C" functions to the bridge. Janet tables are a natural fit for model graph storage.
- **Alternatives considered**: Rust model graph with abstract types — would be faster but requires significant bridge expansion with no clear benefit.

### Decision 2: `*model-context*` dynamic variable for shape tracking

- **Chosen**: During `build`, a `*model-context*` dynamic variable is set. Wrapped CAD functions check this variable and register created shapes in the current model's shape-map.
- **Rationale**: Dynamic binding is the Janet-native way to manage implicit context. Fibers inherit dynamic bindings naturally, supporting nested model builds.
- **Alternatives considered**: Thread-local storage (not accessible from Janet). Explicit threading of context argument through every CAD function (invasive, breaks existing API).

### Decision 3: CAD function wrapping at boot time

- **Chosen**: At boot, every CAD function (`box`, `sphere`, `cylinder`, `cut`, `fuse`, `common`, `translate`, `rotate`, `scale`, `mirror`, `extrude`, `revolve`, `rect`, `circle`, `polygon`, `wire-to-face`, `wire-fillet`, `wire-chamfer`, `wire-offset`) is wrapped to call the original, then register the result with `*model-context*` if active.
- **Rationale**: Transparent to the user — existing code continues to work unchanged. The wrapping is done once at boot in `model.janet`.
- **Performance**: The `*model-context*` check is a single variable reference — negligible overhead.

### Decision 3a: Removed auto-show from boot.janet CAD wrappers

- **Chosen**: All `(unless hide (show ...))` calls were removed from boot.janet's CAD wrappers. Shapes are no longer automatically visible when created.
- **Rationale**: Previously every CAD function auto-showed its result. This caused intermediate shapes created during `build` to appear in the viewer. With the model system, shapes should only be visible when explicitly assigned (via `def` + `my-eval`) or when `build`/`highlight` explicitly shows them.
- **Impact**: `(box 10 20 30)` in the REPL without `def` no longer shows a visible shape. `(def b (box 10 20 30))` still works because `my-eval` calls `show` on `def` assignment.

### Decision 4: Sub-model instances for nested model calls

- **Chosen**: When a model calls another model inside its body (e.g., `(translate (bracket w h t r) ...)`), the called model's `build` creates a sub-model instance with its own shape-map. The parent model tracks only the returned shape (the result of `build`), not the sub-model's internal shapes.
- **Rationale**: Allows deep introspection — `(graph assembly)` shows sub-model nodes that can be expanded individually. Keeps shape tracking scoped per model instance.
- **Alternatives considered**: Transparent flattening (all shapes in one flat list) loses structure. Opaque function call (no sub-model tracking) loses introspection depth.

### Decision 5: Highlighting via ReplToViewer commands

- **Chosen**: Extend the `ReplToViewer` enum with `HighlightShape { id: ShapeId }` and `ClearHighlight` commands. The viewer renders highlighted shapes with active edges + a tinted mesh overlay.
- **Rationale**: Leverages the existing thread-safe mpsc channel between REPL and viewer. No new synchronization needed.
- **Alternatives considered**: Direct shape registry mutation — would require the REPL thread to push highlight state into the registry, which the viewer would then read. Mpsc is cleaner and already established.

### Decision 6: `defmodel` stores source form explicitly

- **Chosen**: The macro stores the body's source S-expression as `:source` alongside the compiled `:body-fn`. The `graph` function walks `:source` to build the AST tree, mapping nodes to shapes via the shape-map.
- **Rationale**: Source preservation is required for full AST walk without decompilation tricks. The shape-map bridges the gap between source tree nodes and runtime shape values.
- **Alternatives considered**: Function decompilation via `(in fn :source)` — works in standard Janet but may be unavailable in bootstrap mode.

### Decision 6a: Bare `def` output (not wrapped in `do`)

- **Chosen**: The `defmodel` macro emits a bare `(def ,name (table ...))` as its output, NOT wrapped in a `do` block. Metadata (`:source`, `:category`) that was originally planned for core-env binding is skipped.
- **Rationale**: Janet's compiler needs to see bare top-level `def` forms for subsequent expressions to reference the symbol at compile time. Wrapping `def` inside a `do` block causes "unknown symbol" compile errors for later forms that use the model.
- **Trade-off**: Compile-time symbol visibility is fixed, but model metadata is not stored on the core-env binding.

### Decision 7: `highlight` must `show` before highlighting

- **Chosen**: `highlight` calls `(show shape)` before sending the highlight command to the viewer. It tracks shown shapes in a model's `:_hl-shapes` array. `highlight-clear` accepts an optional model and part-id to hide the tracked shapes back.
- **Rationale**: Removing auto-show (Decision 3a) means intermediate shapes are NOT in the viewer registry. The viewer can only highlight shapes that are registered. `highlight` must register them first. `highlight-clear` cleans up by hiding them.
- **API**:
  - `(highlight bracket :hole)` — show + highlight the hole
  - `(highlight-clear bracket :hole)` — hide the hole + clear highlight
  - `(highlight-clear bracket)` — hide all highlighted parts + clear
  - `(highlight-clear)` — clear viewer highlight only

## Risks / Trade-offs

- **CAD wrapping overhead at boot**: All ~20 CAD functions need wrapping. If a new CAD function is added without wrapping, it won't participate in model tracking. **Mitigation**: The wrapping can be automated with a list of function names in `model.janet`, and the boot sequence should verify that all registered CAD functions are covered.
- **`*model-context*` leak**: If a build throws an error mid-execution, `*model-context*` might not be cleared. **Mitigation**: Use `(def *model-context* (dyn '*model-context*'))` set at top of model body and restored in a `(finally ...)` block.
- **Highlight performance**: For models with many parts, highlight state changes trigger a generation counter increment. **Mitigation**: Highlight is a single-shape property — generation counter is only bumped on highlight changes, not per frame.
