## ADDED Requirements

### Requirement: Shape creation without viewer registration

When a shape is created (`box`, `sphere`, `cylinder`, `cone`, `torus`, `cut`, `common`, `fuse`, `translate`, `rotate`, `scale`, `mirror`), the resulting `ShapeData` SHALL NOT be registered in the viewer registry. Registration SHALL only occur when `show` is explicitly called.

#### Scenario: Creating a box without showing
- **WHEN** `(box 10)` is evaluated
- **THEN** the viewer SHALL NOT display any new shape

#### Scenario: Creating a box with def without showing
- **WHEN** `(def b (box 10))` is evaluated
- **THEN** the viewer SHALL NOT display any new shape AND `b` SHALL be bound to the shape

#### Scenario: Fuse with intermediate shapes
- **WHEN** `(fuse (box 10) (sphere 5))` is evaluated
- **THEN** only the fused result MAY be visible if shown; the intermediate box and sphere SHALL NOT be registered in the viewer at any point

### Requirement: Explicit show

`show` SHALL register a shape in the viewer registry and make it visible. On first call, if the shape has not been tessellated, `show` SHALL tessellate it (extract mesh and edge polylines) before registering. On subsequent calls, `show` SHALL only set the visibility flag to true.

#### Scenario: First show registers and tessellates
- **WHEN** `(def b (box 10))` is followed by `(show b)`
- **THEN** the box SHALL appear in the viewer

#### Scenario: Show after hide restores visibility
- **WHEN** a shape is shown, then hidden, then shown again
- **THEN** the shape SHALL reappear in the viewer without re-tessellating

#### Scenario: Show is idempotent
- **WHEN** `(show b)` is called twice on the same shape
- **THEN** no error SHALL occur and the shape SHALL remain visible

### Requirement: Explicit hide

`hide` SHALL set the visibility flag of a registered shape to false. The shape SHALL remain in the registry and its tessellation data SHALL be preserved.

#### Scenario: Hide removes from viewer
- **WHEN** `(hide b)` is called on a visible shape
- **THEN** the shape SHALL no longer be rendered in the viewer

#### Scenario: Hide on unregistered shape is a no-op
- **WHEN** `(hide b)` is called on a shape that was never shown
- **THEN** no error SHALL occur

### Requirement: Deferred tessellation with :eager opt-in

Shapes SHALL NOT be tessellated at creation time by default. Tessellation SHALL be deferred until the first `show` call. All shape-creating functions SHALL accept the `:eager` keyword; when present, the shape SHALL be tessellated immediately at creation time.

#### Scenario: Default is no tessellation
- **WHEN** `(def b (box 10))` is evaluated
- **THEN** `b` SHALL have no mesh data (tessellation deferred)

#### Scenario: :eager forces immediate tessellation
- **WHEN** `(def b (box 10 :eager))` is evaluated
- **THEN** `b` SHALL have mesh data immediately after creation

#### Scenario: :eager on boolean operations
- **WHEN** `(def f (fuse (box 10 :eager) (sphere 5 :eager) :eager))` is evaluated
- **THEN** all three shapes SHALL be tessellated at creation time

### Requirement: Automatic registry cleanup on GC

When Janet's garbage collector collects a `ShapeData` value, the associated entry in the viewer registry SHALL be removed automatically.

#### Scenario: Shape removed from viewer after GC
- **WHEN** `(box 10)` is evaluated (creating an unbound shape) and Janet's GC collects the result
- **THEN** the registry entry for that shape SHALL be removed

#### Scenario: def b nil followed by GC
- **WHEN** `(def b (box 10))` then `(def b nil)` then `(gc)` are evaluated
- **THEN** `b` SHALL be `nil` AND the viewer registry entry SHALL be removed

### Requirement: Purge function

`purge` SHALL be a C function that immediately removes a shape from the viewer registry and marks it as purged. Subsequent operations on a purged shape SHALL produce an error. Variable unbinding is done separately with `(def b nil)`.

Macro-based `(purge b)` expanding to `(do (registry-remove b) (def b nil))` was the original design, but it could not be implemented due to Janet bootstrap environment constraints: `defmacro` is unavailable, and `def` inside a macro-expanded `do` block does not reach the top-level compilation scope (`core-env`), making bindings invisible to subsequent REPL commands.

#### Scenario: Purge removes from viewer
- **WHEN** `(def b (display (box 10)))` then `(purge b)` are evaluated
- **THEN** the shape SHALL disappear from the viewer

#### Scenario: Using a purged shape errors
- **WHEN** `(def b (display (box 10)))` then `(purge b)` then `(show b)` are evaluated
- **THEN** an error SHALL be signaled because the shape is purged

### Requirement: Display function

`display` SHALL be a C function that shows a shape and returns it. It SHALL NOT be a macro — the recommended usage is `(def b (display (box 10)))` which puts `def` at the top-level compilation scope, adding the binding to `core-env`.

A `(display b (box 10))` macro expanding to `(do (def b (box 10)) (show b) b)` was the original design, but it could not be implemented: inside a macro-expanded `do` block, `def` is not at the top-level scope, so `defleaf` (the Janet compiler) does not add the binding to `core-env`. This makes the bound symbol invisible to subsequent REPL commands compiled against `core-env`. Additionally, if the macro expanded to `(def b (display (box 10)))`, the self-referential `display` in the expansion causes infinite recursive macro expansion.

#### Scenario: Display shows immediately
- **WHEN** `(def b (display (box 10)))` is evaluated
- **THEN** `b` SHALL be bound to the box shape AND the shape SHALL appear in the viewer

#### Scenario: Display with eager
- **WHEN** `(def b (display (box 10 :eager)))` is evaluated
- **THEN** `b` SHALL be bound to an eagerly tessellated box AND the shape SHALL appear in the viewer

#### Scenario: Display across REPL connections
- **WHEN** `(def b (display (box 10)))` is evaluated in one REPL connection, then `(purge b)` in a subsequent connection
- **THEN** `b` SHALL be found in `core-env` and the purge SHALL succeed

### Requirement: Remove dead ReplToViewer channel

The `ReplToViewer` message type and its associated channel infrastructure SHALL be removed. The viewer SHALL communicate with the REPL thread only through the global `ShapeRegistry` and the `ViewerToRepl` selection channel.

#### Scenario: Viewer compiles without ReplToViewer
- **WHEN** the project is built after removing `ReplToViewer`
- **THEN** no references to `UpdateShapes`, `RemoveShape`, or `ClearAll` messages SHALL remain in the codebase
