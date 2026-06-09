## ADDED Requirements

### Requirement: Parametric model functions have API doc metadata

Each parametric model function (`build`, `graph`, `highlight`, `highlight-clear`) and macro (`defmodel`) SHALL have:
- `:source` set to `"rojcad"` in its core-env binding
- `:category` set to `"parametric-models"` in its core-env binding
- `:doc` set to a docstring following the established format (usage line, body, examples, return)

#### Scenario: functions appear in generated API docs

- **WHEN** `(dump-docs "doc")` is called after boot
- **THEN** `doc/janet-api.md` SHALL contain entries for `defmodel`, `build`, `graph`, `highlight`, and `highlight-clear`
- **AND** each entry SHALL appear under a "Parametric Models" section heading

#### Scenario: each function has usage and examples

- **WHEN** a user views the generated docs
- **THEN** each parametric model entry SHALL include a usage line, body text, example code, and return type description

#### Scenario: defmodel macro is documented

- **WHEN** `(doc defmodel)` is called in the REPL
- **THEN** the docstring SHALL describe the macro syntax including `:parts` and `:result` keywords

### Requirement: `cad-groups` includes `parametric-models` category

The `cad-groups` table in `boot.janet` SHALL include a `"parametric-models"` entry mapping to a human-readable display name.

#### Scenario: parametric models group renders in docs

- **WHEN** docs are generated
- **THEN** the HTML sidebar and markdown heading SHALL include a "Parametric Models" section

### Requirement: README has parametric models workflow section

The README SHALL include a "Parametric Models" section demonstrating the end-to-end workflow with a real example.

#### Scenario: README shows defmodel + build + graph + highlight

- **WHEN** a user reads the README
- **THEN** it SHALL include a code example showing:
  1. Defining a model with `defmodel` including `:parts` and `:result`
  2. Building the model with `build`
  3. Inspecting structure with `graph`
  4. Highlighting a named part with `highlight`
  5. Rebuilding with different parameters

#### Scenario: README example is self-contained

- **WHEN** a user copies the README example into a REPL session
- **THEN** it SHALL produce visible shapes without requiring additional setup
