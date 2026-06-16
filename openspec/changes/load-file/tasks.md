## 1. Core Implementation

- [x] 1.1 Add `load-file` function to `boot.janet` alongside `my-parse`/`my-eval` (around line 1068)

     Function reads file, reads content, creates parser, loops `produce` until nil, evaluates each form with `my-eval`, returns last result. Must check `parser/status` for `:error` to abort on syntax errors.

- [x] 1.2 Add `defmeta` metadata for `load-file` with category `"io"`, source `"rojcad"`, and a docstring with usage, description, and examples

## 2. Integration Tests

- [x] 2.1 Add REPL integration test: load a file containing a single arithmetic expression, verify the returned value

- [x] 2.2 Add REPL integration test: load a file that defines a function, call the function, verify result

- [x] 2.3 Add REPL integration test: load a file that creates a shape with `def`, verify the shape is visible and has the correct type

- [x] 2.4 Add REPL integration test: attempt to load a non-existent file, verify error is signaled

- [x] 2.5 Add REPL integration test: load a file with a syntax error, verify error is signaled

- [x] 2.6 Add REPL integration test: load an empty file, verify nil is returned
