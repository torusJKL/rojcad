## ADDED Requirements

### Requirement: Graph introspection function

A `graph` function SHALL be provided that returns the structure of a model as a Janet table.

`graph` SHALL accept a model record and return a table with:
- `:name` — the model's name symbol
- `:params` — the model's parameter vector
- `:current` — the current parameter values (or nil if not yet built)
- `:nodes` — an array of node descriptors, each containing:
  - `:id` — a string or keyword identifying the part (e.g., `:base`, `:hole`, `:result`)
  - `:type` — the CAD function name as a keyword (e.g., `:box`, `:cylinder`, `:cut`)
  - `:args` — the argument expressions as a tuple (source form, not evaluated)
  - `:path` — a vector of keys locating this node in the source form
  - `:children` — an array of child node IDs (for composite operations like `:cut`)
- `:shape-map` — a table mapping node IDs to shape values (only for built models)

#### Scenario: Graph of an unbuilt model

- **WHEN** `(graph bracket)` is called before any `build`
- **THEN** the returned table SHALL have `:name`, `:params`, `:nodes`
- **AND** `:current` SHALL be nil
- **AND** each node SHALL have `:type`, `:args`, `:path`

#### Scenario: Graph of a built model includes shape references

- **WHEN** `(def br (build bracket 100 50))` followed by `(graph bracket)`
- **THEN** each node in `:nodes` SHALL have a `:shape` field
- **AND** `:shape-map` SHALL map node IDs to shape values

#### Scenario: Graph of a model with named parts

- **WHEN** `(graph bracket)` is called on a model with `:parts {:base (box ...) :hole (cylinder ...)}`
- **THEN** `:nodes` SHALL include entries for `:base`, `:hole`, and `:result`
- **AND** each node SHALL correctly identify its CAD type (`:box`, `:cylinder`, etc.)

#### Scenario: Graph of a nested model

- **WHEN** `(graph assembly)` is called on a model containing sub-model calls
- **THEN** sub-model call nodes SHALL have `:type :model-instance`
- **AND** the node SHALL include `:model` referencing the sub-model record
- **AND** `:instances` SHALL list each call with its params and sub-shape-map

### Requirement: Source form preservation

The `defmodel` macro SHALL store the body's source S-expression verbatim in the model's `:source` field.

#### Scenario: Source reflects original structure

- **WHEN** `(defmodel m [x] :parts {:a (box x 10 20)} :result a)` is defined
- **THEN** `(m :source)` SHALL be `(fn [x] (def parts {:a (box x 10 20)}) (def result a))` or equivalent

#### Scenario: Source preserves macro arguments

- **WHEN** `(defmodel m [x] :parts {:a (box x x x)} :result a)` is defined
- **THEN** `(m :source)` SHALL preserve the literal argument expressions `x`, `x`, `x`
