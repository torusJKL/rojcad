## ADDED Requirements

### Requirement: Model definition macro

Users SHALL be able to define a parametric model using a `defmodel` macro.

The macro SHALL accept:
- A name symbol for the model
- A parameter vector of symbols (positional)
- Optional `:parts` keyword with a table of named part definitions
- Optional `:result` keyword with an expression producing the final shape
- A body expression (the `:result` expression or the last expression if `:result` is omitted)

Inside the macro body, each symbol in the parameter vector SHALL be bound as a local variable. If `:parts` is provided, a `parts` local variable SHALL be bound to the table of built part shapes.

The macro SHALL produce a model record (Janet table) created via `(def ,name (table ...))` — a bare top-level `def`, not wrapped in a `do` block. This ensures the binding is visible to the Janet compiler for subsequent forms.

The model record SHALL contain at minimum:
- `:params` — the parameter vector
- `:body-fn` — a compiled function that executes the model
- `:source` — the source S-expression of the body
- `:parts` — the parts table declaration (from `:parts` keyword), or nil if absent
- `:shapes` — array of all shapes created by the last `build` call
- `:shape-map` — table mapping part name or path to shape value
- `:current-params` — the parameter values from the last `build`

#### Scenario: Define a simple model

- **WHEN** a user writes `(defmodel bracket [w h] :parts {:base (box w h 30)} :result base)`
- **THEN** a model record SHALL be bound to `bracket`
- **AND** `(bracket :params)` SHALL be `(w h)`
- **AND** `(bracket :parts)` SHALL be `{:base (box w h 30)}`

#### Scenario: Define a model without parts

- **WHEN** a user writes `(defmodel cube [s] (box s s s))`
- **THEN** `(cube :parts)` SHALL be nil
- **AND** `(cube :body-fn)` SHALL be a callable function

#### Scenario: Parameters are bound in body

- **WHEN** a user writes `(defmodel rect [w h] :parts {:face (rect w h)} :result face)`
- **THEN** inside the body, `w` and `h` SHALL refer to the supplied parameter values at build time

### Requirement: Build function

A `build` function SHALL be provided that instantiates a model.

`build` SHALL accept a model record followed by positional parameter values matching the model's parameter vector.

`build` SHALL:
1. Purge all shapes in the model's `:shapes` array from the viewer
2. Clear the model's `:shapes` and `:shape-map`
3. Execute the model's `:body-fn` with the supplied parameters
4. Store created shapes in the model's `:shapes` array
5. Populate `:shape-map` with part-name-to-shape mappings
6. Store the parameter values in `:current-params`
7. Return the result shape

#### Scenario: Build returns a shape

- **WHEN** `(def br (build bracket 100 50))` is evaluated
- **THEN** `br` SHALL be a `rojcad/shape` abstract value
- **AND** `(shape-type br)` SHALL match the model's result type

#### Scenario: Build auto-shows via def

- **WHEN** the result of `build` is assigned with `(def br (build bracket 100 50))`
- **THEN** the shape SHALL be auto-shown in the viewer (via existing `my-eval` behavior)

#### Scenario: Re-build purges old shapes

- **WHEN** `(def br (build bracket 100 50))` is followed by `(def br (build bracket 200 80))`
- **THEN** the first `br` shape SHALL be purged
- **AND** intermediate shapes from the first build (parts) SHALL be purged
- **AND** the new `br` shape SHALL be displayed

#### Scenario: Build without def does not auto-show

- **WHEN** `(build bracket 100 50)` is called without a `def`
- **THEN** the shape SHALL NOT be visible in the viewer

#### Scenario: Build with wrong arity signals error

- **WHEN** `(build bracket 100)` is called (parameter count mismatch)
- **THEN** an error SHALL be signaled

### Requirement: Shape tracking via `*model-context*`

All CAD functions (`box`, `sphere`, `cylinder`, `cone`, `torus`, `cut`, `fuse`, `common`, `translate`, `rotate`, `scale`, `mirror`, `extrude`, `revolve`, `rect`, `circle`, `polygon`, `wire-to-face`, `wire-fillet`, `wire-chamfer`, `wire-offset`, `close-sketch`, `build-wire`) SHALL be wrapped at boot time to check `(dyn '*model-context*)`.

When `*model-context*` is set to a model record:
- Each shape returned by a CAD function SHALL be pushed onto the model's `:shapes` array
- Shape-to-path mapping SHALL be recorded in the model's `:shape-map`

#### Scenario: Untracked code is unaffected

- **WHEN** a user calls `(box 10 20 30)` outside a `build`
- **THEN** no model tracking SHALL occur
- **AND** behavior SHALL be identical to before the wrapping

### Requirement: Model composition (sub-model instances)

When a model body calls another model's `build` (via `(model-name ...)`), the called model SHALL create a sub-model instance with its own `:shapes` and `:shape-map`.

The parent model SHALL:
- Track only the returned shape (result of the sub-model build), not internal sub-shapes
- Record the sub-model instance as part of the parent's shape-map under the part name

#### Scenario: Nested model creates sub-instance

- **WHEN** model A contains `(bracket 100 50 10 5)` in its body
- **THEN** the bracket shapes SHALL be tracked in a separate sub-instance
- **AND** model A's shape-map SHALL contain an entry for the sub-instance's result shape
