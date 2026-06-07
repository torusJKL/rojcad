## ADDED Requirements

### Requirement: Auto-purge old shape on def redefinition
When a top-level `(def name expr)` form is evaluated and `name` was already bound to a `:rojcad/shape`, the old shape SHALL be purged from the viewer registry before the new shape is evaluated.

#### Scenario: def redefinition purges old shape
- **WHEN** the user evaluates `(def obj (box 10))` followed by `(def obj (sphere 20))`
- **THEN** the box shape is removed from the viewer registry and the sphere is visible

#### Scenario: first def does not trigger purge
- **WHEN** the user evaluates `(def obj (box 10))` and no previous binding for `obj` exists
- **THEN** the box is auto-shown and no purge occurs

#### Scenario: def of non-shape value has no side effect
- **WHEN** the user evaluates `(def x 42)` followed by `(def x 100)`
- **THEN** neither def triggers a purge operation

### Requirement: Auto-purge old shape on set redefinition
When a top-level `(set name expr)` form is evaluated and `name` was bound to a `:rojcad/shape`, the old shape SHALL be purged from the viewer registry and the new shape SHALL be auto-shown.

#### Scenario: set redefinition purges and shows
- **WHEN** the user evaluates `(def obj (box 10))` followed by `(set obj (sphere 20))`
- **THEN** the box is purged from the viewer registry and the sphere is auto-shown

### Requirement: Same-shape redefinition does not purge
When `(def name name)` or `(set name name)` is evaluated where the old and new values are the same `:rojcad/shape` object, the shape SHALL NOT be purged or re-shown — it SHALL remain visible and registered as before.

#### Scenario: redef to self is a no-op
- **WHEN** the user evaluates `(def obj (box 10))` followed by `(def obj obj)`
- **THEN** the box remains visible in the viewer and no purge or re-show occurs

### Requirement: Non-top-level def/set forms do not trigger auto-purge
Compound forms like `(do (def obj ...) ...)` SHALL NOT trigger auto-purge — only top-level `def`/`set` forms at the REPL prompt are intercepted.

#### Scenario: def inside do is not auto-purged
- **WHEN** the user evaluates `(do (def obj (box 10)) (def obj (sphere 20)) obj)`
- **THEN** the old box is NOT automatically purged (the `do` form is not intercepted)

### Requirement: Error during evaluation preserves old shape
If the expression in a `(def name expr)` or `(set name expr)` form signals an error, the old binding SHALL remain in the environment and no purge SHALL occur.

#### Scenario: error in redef expression preserves old shape
- **WHEN** the user evaluates `(def obj (box 10))` followed by `(def obj (bad-function))`
- **THEN** `obj` remains bound to the box shape and the box remains visible in the viewer

### Requirement: Purge behavior on shared references
When a shape is referenced by multiple symbols (e.g., `(def a (box 10)) (def b a)`), and one symbol is redefined, the other symbol SHALL continue to point to the same `ShapeData` — which is now in a purged state. Calling `(show other-sym)` on a purged shape SHALL signal an error.

#### Scenario: shared reference after redef
- **WHEN** the user evaluates `(def a (box 10))`, `(def b a)`, and `(def a (sphere 20))`
- **THEN** `a` points to the visible sphere, `b` points to the purged box, and `(show b)` produces an error
