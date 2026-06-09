## 1. CAD Function Wrapping Infrastructure

- [x] 1.1 Create `boot/model.janet` with `*model-context*` dynamic variable and wrapping utility for CAD functions
- [x] 1.2 Write boot-time loop that wraps all registered CAD functions to register shapes in `*model-context*`
- [x] 1.3 Integrate `model.janet` loading into the boot sequence
- [x] 1.4 Removed auto-show (`(unless hide (show ...))`) from all boot.janet CAD wrappers — shapes no longer auto-show on creation

## 2. defmodel Macro

- [x] 2.1 Implement `defmodel` macro that captures parameter vector, `:parts` table, `:result` expression, and stores source form
- [x] 2.2 Generate `:body-fn` that binds parameters, evaluates `:parts` expressions into a `parts` local, evaluates `:result`, and returns the result shape
- [x] 2.3 Produce model record with `:params`, `:body-fn`, `:source`, `:parts`, `:shapes`, `:shape-map`, `:current-params` fields
- [x] 2.4 Handle error cases: `:parts` without `:result`, result expression errors, arity mismatch
- [x] 2.5 Emit bare `(def name ...)` not wrapped in `do` — needed for compile-time symbol visibility

## 3. Build Function

- [x] 3.1 Implement `build` function that purges old shapes, clears tracking state, sets `*model-context*`, executes body-fn, records shape-map, and returns result shape
- [x] 3.2 Implement cleanup to restore `*model-context*` on error during build (fiber-based error handling)
- [x] 3.3 Verified interaction with `my-eval` `shape-bindings` auto-purge for `(def x (build ...))` + re-def pattern

## 4. Sub-Model Instance Support

- [x] 4.1 When `build` is called inside a tracked context, the inner model's shapes are tracked separately
- [x] 4.2 Sub-model builds get their own `:shapes` and `:shape-map`, parent records only the result shape
- [x] 4.3 Sub-model instance results are registered in parent's `:shapes` for correct purge on rebuild

## 5. Graph Introspection

- [x] 5.1 Implement `graph` function that walks the model's `:source` form and builds the node tree
- [x] 5.2 Map AST nodes to shape values from `:shape-map` for built models
- [x] 5.3 Handle sub-model instances: `_sub-instances` recorded in shape-map during nested builds
- [x] 5.4 Return the graph table with `:name`, `:params`, `:current`, `:nodes`, `:shape-map`

## 6. Viewer Highlight Support

- [x] 6.1 Extend `ReplToViewer` enum in `src/types.rs` with `HighlightShape { id: ShapeId }` and `ClearHighlight` variants
- [x] 6.2 Add highlight state to viewer (`ViewerState` with `highlighted_shape` field)
- [x] 6.3 Modify viewer mesh renderer to apply tint overlay on highlighted shape
- [x] 6.4 Modify viewer edge renderer to use active-edge style on highlighted shape
- [x] 6.5 Wire REPL→viewer channel handling for highlight/clear commands in `check_repl_commands`

## 7. Highlight/Unhighlight API

- [x] 7.1 Implement `highlight` function: show + highlight a part, track in `_hl-shapes`
- [x] 7.2 Implement `highlight-clear` with optional model and part-id to hide tracked shapes
- [x] 7.3 Handle error: highlight on unbuilt model signals an error
- [x] 7.4 Handle sub-model highlighting: recursively highlights sub-instance result shapes

## 8. Integration and Testing

- [x] 8.1 Verified through compilation — ReplToViewer enum variants compile, match arms are exhaustive
- [x] 8.2 Verified via --eval: defmodel, build, graph, highlight, highlight-clear, rebuild all work correctly
- [ ] 8.3 Manual verification: nested model composition with introspection (requires interactive REPL)
- [ ] 8.4 Manual verification: highlight edge + mesh tint rendering in viewer (requires interactive REPL + display)
- [x] 8.5 Verified existing CAD functions work identically outside model context — headless boot without --eval succeeds with 90 passing tests
