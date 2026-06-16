## ADDED Requirements

### Requirement: Load file from absolute path

The system SHALL provide a `load-file` function that reads a Janet file from an absolute filesystem path.

- `load-file` SHALL accept a single string argument: the absolute path to a `.janet` file
- `load-file` SHALL open the file, read all contents, and close the file
- `load-file` SHALL signal an error if the file does not exist or cannot be opened
- `load-file` SHALL signal an error if the file contains invalid Janet syntax
- `load-file` SHALL parse all top-level forms in the file and evaluate each in sequence
- `load-file` SHALL return the result of the last form, or nil if the file is empty
- `load-file` SHALL abort on the first form that fails to compile or evaluate, and SHALL signal an error with a descriptive message

#### Scenario: Load a file with a single form

- **WHEN** a `.janet` file exists at `/tmp/test-single.janet` containing `(+ 1 2)`
- **THEN** `(load-file "/tmp/test-single.janet")` returns `3`

#### Scenario: Load a file with multiple forms

- **WHEN** a `.janet` file exists at `/tmp/test-multi.janet` containing `(def a 1)\n(def b 2)\n(+ a b)`
- **THEN** `(load-file "/tmp/test-multi.janet")` returns `3`

#### Scenario: Load a file that defines a function

- **WHEN** a `.janet` file exists containing `(defn square [x] (* x x))`
- **THEN** after `(load-file path)`, `(square 5)` returns `25`

#### Scenario: Load a file that defines parametric models

- **WHEN** a `.janet` file defines a model with `defmodel` and accompanying build logic
- **THEN** after `load-file`, the model binding is available in the REPL environment and `build` works as expected

#### Scenario: File not found

- **WHEN** `(load-file "/tmp/nonexistent.janet")` is called
- **THEN** an error is signaled with a message containing "could not open"

#### Scenario: Syntax error in file

- **WHEN** a `.janet` file contains invalid syntax `(def a (`
- **THEN** `load-file` signals an error with a message describing the parse failure

#### Scenario: Empty file

- **WHEN** a `.janet` file is empty
- **THEN** `(load-file path)` returns nil

### Requirement: Shape tracking integration

When `load-file` evaluates forms that create shapes, the existing `my-eval` shape tracking SHALL apply.

- Shapes created via `def` inside a loaded file SHALL be auto-shown (consistent with REPL behavior)
- Redefining a shape variable SHALL auto-purge the old shape (consistent with `my-eval`)
- Models built with `build` inside a loaded file SHALL participate in model tracking

#### Scenario: Loading a file creates and shows a shape

- **WHEN** a `.janet` file contains `(def my-box (box 10))`
- **THEN** after `(load-file path)`, `(visible? my-box)` returns true

### Requirement: Function is discoverable

`load-file` SHALL have metadata (docstring, category, source) so it appears in the REPL's discovery tools.

- `load-file` SHALL have category `"io"`
- `load-file` SHALL have source `"rojcad"`
- `load-file` SHALL have a docstring describing usage, arguments, and examples

#### Scenario: load-file appears in doc tools

- **WHEN** the user runs `(doc load-file)` in the REPL
- **THEN** the docstring is displayed with usage, description, and examples

#### Scenario: load-file appears in group listings

- **WHEN** the user runs `(group :io)`
- **THEN** `load-file` appears in the returned array
