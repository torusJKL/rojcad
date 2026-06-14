## Why

The parametric model system (`defmodel`, `build`, `graph`, `highlight`, `highlight-clear`) is fully implemented but invisible in documentation. Neither the generated API reference nor the README mention it — users who don't read `boot/model.janet` source have no way to discover it. This makes rojcad's core value proposition (parametric CAD) effectively hidden.

## What Changes

- Add `setmeta` calls in `boot/model.janet` for `defmodel`, `build`, `graph`, `highlight`, and `highlight-clear` so they appear in `doc/janet-api.md` / `.html` with proper docstrings, categories, and usage examples
- Add `"parametric-models"` category to `cad-groups` in `boot.janet` so the new functions render under their own section in generated docs
- Add a "Parametric Models" section to README.md showing the full workflow: defining a model, building it, inspecting with graph, and highlighting parts

## Capabilities

### New Capabilities
- `parametric-model-documentation`: Discovery docs (API reference + README) for the parametric model system — `defmodel`, `build`, `graph`, `highlight`, and `highlight-clear` with docstrings, examples, and a usage walkthrough

### Modified Capabilities
<!-- No existing spec-level behavior is changing — only documentation is being added -->

## Impact

- **Modified**: `boot/model.janet` — add `setmeta` calls after each function/macro definition
- **Modified**: `boot.janet` — add `"parametric-models"` category to `cad-groups` table
- **Modified**: `README.md` — add Parametric Models section with workflow example
- **Generated**: `doc/janet-api.md` and `doc/janet-api.html` will now include the model functions (picked up by `dump-docs` on next `just doc-janet`)
- No Rust, C bridge, or runtime behavior changes
