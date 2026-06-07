## ADDED Requirements

### Requirement: Wrapped functions preserve metadata in env tables
When a C function is replaced with a Janet wrapper via table mutation (putting the new function as `:value`), the env table's `:doc`, `:source`, and `:category` fields SHALL remain intact so that `doc`, `cad-fns`, and `group` continue to work.

#### Scenario: doc still works on wrapped function
- **WHEN** `(doc 'hide)` is called after the wrapper is installed
- **THEN** the documentation string is returned (same as before wrapping)

#### Scenario: cad-fns includes wrapped functions
- **WHEN** `(cad-fns)` is called after wrapping
- **THEN** `hide`, `show`, `cut`, `common`, `fuse`, `shape-type`, `visible?`, `wire?`, `face?`, `solid?`, `purge`, and `registry-remove` are all listed

#### Scenario: group includes wrapped functions in correct category
- **WHEN** `(group "booleans")` is called after wrapping
- **THEN** `cut`, `common`, and `fuse` are all listed

### Requirement: all-fns and apropos discover wrapped rojcad functions
`all-fns` and `apropos` SHALL discover rojcad functions even after they are wrapped in Janet, not just bare C functions.

#### Scenario: all-fns includes wrapped hide
- **WHEN** `(all-fns)` is called after wrapping
- **THEN** the result includes `hide` even though it is no longer a `:cfunction`

#### Scenario: apropos finds wrapped functions
- **WHEN** `(apropos "cut")` is called after wrapping
- **THEN** the result includes `cut`
