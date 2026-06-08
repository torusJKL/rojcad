## 1. bridge/bridge.c — Tag orphaned functions

- [x] 1.1 Add `quit-requested`, `edge-hidden-toggle`, `edge-hidden-show?`, `edge-hidden`, `projection-toggle`, `projection-perspective` entries to `cad_fn_categories` table (assign to `"view"` or `"edge-styling"` categories)

## 2. boot.janet — Add missing category display names

- [x] 2.1 Add `"2d-primitives"`, `"operations"`, `"wire-operations"`, `"sketch"`, `"text"` entries to the `cad-groups` dictionary with appropriate display names

## 3. boot.janet — Add "Other" fallback to `gen-markdown`

- [x] 3.1 Call `(group)` once at the start of `gen-markdown` and use the result table for both known and unknown categories
- [x] 3.2 After iterating known `cad-groups` keys, collect functions from categories not in `cad-groups` and render them under `## Other`

## 4. boot.janet — Add "Other" fallback to `gen-html`

- [x] 4.1 Same pattern as markdown: call `(group)` once, use result table, add `## Other` fallback section
- [x] 4.2 Add sidebar link for the "Other" category when it has functions

## 5. Verification

- [x] 5.1 Run `just doc-janet` and verify all 5 missing groups appear with correct display names
- [x] 5.2 Verify orphaned functions appear under their assigned categories
- [x] 5.3 Verify generated Markdown and HTML contain 14 category sections (no silent drops)
