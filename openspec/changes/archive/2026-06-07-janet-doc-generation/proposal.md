## Why

rojcad has 30+ Janet-callable CAD functions with docstrings embedded in C (via `JANET_FN` macros), accessible at runtime via `(doc 'fn-name)`. But there is no external documentation — AI agents can't read the function signatures, and human users have no browsable reference. As the API grows, this gap widens.

## What Changes

- Add `--eval <expr>` CLI flag to rojcad for one-shot expression evaluation
- Add `(dump-docs)` function in `boot.janet` that generates documentation from runtime metadata
- Generate `doc/janet-api.md` (structured Markdown for AI agents) and `doc/janet-api.html` (single-page HTML for GitHub Pages)
- Add `just doc-janet` recipe
- Create `doc/` directory for generated documentation

## Capabilities

### New Capabilities
- `janet-api-docs`: Auto-generated API reference for all Janet-callable CAD functions, extracted at runtime from Janet's own docstring storage. Produces structured Markdown (for AI agents) and single-file HTML (for humans via GitHub Pages).

### Modified Capabilities

None.

## Impact

| Area | Change |
|------|--------|
| `src/main.rs` | Parse `--eval <expr>` argument, set dynamic variable before boot.janet |
| `src/bridge.rs` | Add `janet_cstringv` FFI declaration |
| `boot.janet` | Add `dump-docs` function (doc generation logic), eval hook for `--eval` |
| `justfile` | Add `doc-janet` recipe |
| `doc/` (new) | Generated Markdown and HTML output |
