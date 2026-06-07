## ADDED Requirements

### Requirement: List all cfunctions

The system SHALL provide an `(all-fns)` function that returns an array of all cfunction names bound in the environment, sorted alphabetically.

#### Scenario: all-fns returns cfunctions

- **WHEN** the user calls `(all-fns)` from the REPL
- **THEN** the result SHALL include all 30 CAD function names and all core library cfunction names (e.g., `math/sin`, `string/find`, `type`, `print`)
- **THEN** the result SHALL be sorted alphabetically
- **THEN** values of types other than `:cfunction` SHALL NOT appear in the result

#### Scenario: all-fns includes user-defined cfunctions

- **WHEN** the user defines a function in the REPL via `(def my-fn (fn [x] x))` and then calls `(all-fns)`
- **THEN** the result SHALL include `my-fn`

### Requirement: Search for functions by name

The system SHALL provide an `(apropos pattern)` function that returns an array of cfunction names containing the given pattern string, sorted alphabetically.

#### Scenario: apropos finds matching functions

- **WHEN** the user calls `(apropos "edge")` from the REPL
- **THEN** the result SHALL include `edge-toggle-inactive`, `edge-toggle-active`, `edge-inactive-show?`, `edge-active-show?`, `edge-thickness`, `edge-color-inactive`, `edge-color-active`
- **THEN** the result SHALL NOT include functions without "edge" in their name

#### Scenario: apropos with no matches

- **WHEN** the user calls `(apropos "zzzzz")` from the REPL
- **THEN** the result SHALL be an empty array

### Requirement: Read function docstrings

The system SHALL provide a `(doc symbol)` function that prints or returns the docstring for a registered function.

#### Scenario: doc returns known function docstring

- **WHEN** the user calls `(doc 'box)` from the REPL
- **THEN** the output SHALL include the usage line `"(box width depth height &keys :w :d :h :c :pl :ph :eager :hide)"`
- **THEN** the output SHALL include the description `"Create a box or cube."`

#### Scenario: doc on function without docstring

- **WHEN** the user calls `(doc 'some-undefined-symbol)` from the REPL
- **THEN** the function SHALL return a message indicating no documentation is available

### Requirement: List only CAD functions

The system SHALL provide a `(cad-fns)` function that returns an array of only the rojcad-specific cfunction names, sorted alphabetically.

#### Scenario: cad-fns returns CAD functions

- **WHEN** the user calls `(cad-fns)` from the REPL
- **THEN** the result SHALL include `box`, `sphere`, `cylinder`, `cone`, `torus`, `cut`, `common`, `fuse`, `translate`, `rotate`, `scale`, `mirror`, `shape-type`, `purge`, `hide`, `show`, `registry-remove`, `visible?`, `write-step`, `write-stl`, `read-step`, `on-select`, `poll-selection`, `edge-toggle-inactive`, `edge-toggle-active`, `edge-inactive-show?`, `edge-active-show?`, `edge-thickness`, `edge-color-inactive`, `edge-color-active`
- **THEN** the result SHALL NOT include core library functions like `math/sin`, `print`, `type`, `string/find`

### Requirement: Group CAD functions by category

The system SHALL provide a `(group &opt category)` function that groups CAD functions by their registered category.

#### Scenario: group returns all groups

- **WHEN** the user calls `(group)` from the REPL
- **THEN** the result SHALL be a table mapping category names to arrays of function names
- **THEN** every CAD function SHALL appear in exactly one group

#### Scenario: group returns a specific category

- **WHEN** the user calls `(group "primitives")` from the REPL
- **THEN** the result SHALL include `box`, `sphere`, `cylinder`, `cone`, `torus`

#### Scenario: uncategorized functions fall under "other"

- **WHEN** a CAD function exists without a registered category
- **THEN** it SHALL appear under the `"other"` key when calling `(group)`

### Requirement: CAD functions tagged with metadata at C level

The system SHALL tag each CAD function's environment binding with `:source "rojcad"` and a `:category` keyword at registration time in `bridge/bridge.c`.

#### Scenario: CAD functions have source tag

- **WHEN** the user queries `(get (get core-env 'box) :source)` from the REPL
- **THEN** the result SHALL be `"rojcad"`
- **WHEN** the user queries `(get (get core-env 'math/sin) :source)` from the REPL
- **THEN** the result SHALL be nil

#### Scenario: CAD functions have category tag

- **WHEN** the user queries `(get (get core-env 'box) :category)` from the REPL
- **THEN** the result SHALL be a non-nil string (e.g., `"primitives"`)
- **WHEN** the user queries `(get (get core-env 'math/sin) :category)` from the REPL
- **THEN** the result SHALL be nil
