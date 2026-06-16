## Context

rojcad operates in Janet bootstrap mode (`JANET_BOOTSTRAP=1`), where the standard module system (`import`/`require`) is unavailable. All Janet code is currently embedded at compile time via `include_str!` + `janet_dostring` in `src/main.rs`. At runtime, the only way to execute Janet code is over the TCP REPL, where `connect-handler` reads a single line, parses it with `my-parse`, and evaluates it with `my-eval` against `core-env`.

The `io`, `parse`, `compile`, and `os` Janet libraries are all registered, providing file I/O and parser primitives that can be composed into a `load-file` function without any Rust/C changes.

The existing `my-eval` function in `boot.janet` already handles shape tracking (auto-purge on redef, auto-show on def), so loading a file that creates shapes behaves identically to typing those same forms into the REPL.

## Goals / Non-Goals

**Goals:**
- Add a `load-file` function callable from the REPL or from other Janet code
- Load an arbitrary `.janet` file from an absolute path, evaluating all forms in sequence
- Integrate with `my-eval` for shape lifecycle management
- Error on first failure with a descriptive message
- Document the function so it appears in REPL discovery tools

**Non-Goals:**
- Module system / namespace isolation (`import-file` that returns a table is future work)
- Relative path resolution or `LOAD_PATH` search (absolute paths only in MVP)
- File watching or hot-reload
- Saving REPL state to file
- Loading non-Janet formats (STEP/STL import already exists as `read-step`)

## Decisions

### Pure Janet implementation (no C/Rust changes)

`load-file` will be implemented entirely in `boot.janet` using existing registered libraries.

**Rationale:**
- `file/open`, `file/read`, `file/close` are available via `janet_lib_io`
- `parser/new`, `parser/consume`, `parser/produce` are available via `janet_lib_parse`
- `my-eval` already exists and handles shape tracking
- No recompile of Rust/C code needed — only a `just build`
- Faster iteration: changes to `load-file` behavior only require editing Janet code
- Integration with existing error handling and shape management is natural

**Alternative considered: C bridge calling `janet_dostring`.**
- Would require a new C function in `bridge.c` + Rust extern decl + registration
- `janet_dostring` evaluates a whole string and returns the last value, but doesn't integrate with `my-eval`'s shape tracking
- Would lose auto-purge on redef and auto-show on def
- Rejected in favor of reusing the proven `my-eval` path

### Multi-form parsing via parser/consume loop

The Janet `parser` library supports incremental consumption. We feed the entire file content, then loop calling `parser/produce` until it returns nil.

**Rationale:**
- Handles any number of top-level forms
- Parser handles whitespace, comments, and form boundaries automatically
- Same parsing mechanism as the REPL's `connect-handler`, ensuring consistency

### Abort on first error

If any form fails to compile or evaluate, `load-file` signals an error and stops.

**Rationale:**
- Predictable semantics: either the file loads completely or not at all
- A half-loaded file with partial bindings is confusing and hard to debug
- Users can always structure their files so early forms don't depend on later ones, or use `protect` if partial loading is desired

### Absolute paths only

`load-file` requires an absolute path. No relative path resolution, no search path.

**Rationale:**
- No ambiguity about which file is being loaded
- Avoids needing a `*load-path*` variable or cwd-relative resolution
- Simple contract: give it a path, it loads that file

## Risks / Trade-offs

- **Risk**: Large files with many forms evaluate synchronously, blocking the event loop. **Mitigation**: This is consistent with the existing REPL model (every `my-eval` is synchronous). Could use `ev/go` + fiber in the future if needed.
- **Risk**: `load-file` re-evaluates code every time it's called, including redefining functions and recreating shapes. **Mitigation**: Documented behavior — users structure their files accordingly. Caching or idempotent loading is future work.
- **Risk**: `parser/produce` returns nil when the parser has no complete form yet, but also returns nil at EOF. If the parser is in an error state (e.g., unmatched paren), `parser/status` returns `:error`. **Mitigation**: Check `parser/status` to distinguish EOF from syntax errors.
