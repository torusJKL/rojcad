## Why

The auto-generated Janet API docs (`just doc-janet`) only document 9 of 14 function categories defined in `bridge/bridge.c`. 22 functions from groups (sketch, 2d-primitives, text, wire-operations, operations) are silently dropped because their category keys are missing from the `cad-groups` display-name mapping. Additionally, 6 registered functions never get `:source "rojcad"` metadata at all, making them invisible to the doc system. As more groups get added to bridge.c, they'll also be silently excluded unless there's a fallback.

## What Changes

- Add 5 missing category entries (`sketch`, `2d-primitives`, `text`, `wire-operations`, `operations`) to `cad-groups` in `boot.janet` so their functions appear in generated docs
- Add 6 orphaned functions (`quit-requested`, `edge-hidden-toggle`, `edge-hidden-show?`, `edge-hidden`, `projection-toggle`, `projection-perspective`) to `cad_fn_categories` in `bridge/bridge.c` so they receive doc metadata
- Add a fallback in `gen-markdown` and `gen-html` that renders any category not in `cad-groups` under an "Other" section instead of silently dropping them
- Optimize doc generation to call `(group)` once instead of per-category

## Capabilities

### New Capabilities
- `doc-category-fallback`: Generates an "Other" section in API docs for any functions whose category key is not recognized in the display-name map, preventing silent drops of future groups

### Modified Capabilities
- `janet-api-docs`: Updated category completeness requirements — all categories registered in bridge.c SHALL appear in generated docs, either under their mapped name or under "Other"

## Impact

| Area | Change |
|------|--------|
| `bridge/bridge.c` | Add 6 entries to `cad_fn_categories` for orphaned functions |
| `boot.janet` | Add 5 entries to `cad-groups`; add fallback logic in `gen-markdown` and `gen-html`; optimize to single `(group)` call |
