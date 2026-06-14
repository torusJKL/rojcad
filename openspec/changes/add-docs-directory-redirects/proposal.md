## Why

The published documentation site has no `index.html` at directory entry points (`/latest/`, `/<version>/`, `/latest/rust/`, `/<version>/rust/`). Navigating to these URLs shows a directory listing or 404 instead of landing on the intended documentation page.

## What Changes

- Add `<meta http-equiv="refresh">` redirect files at directory root paths
- `/latest/` redirects to `/latest/janet-api.html`
- `/<version>/` redirects to `/<version>/janet-api.html`
- `/latest/rust/` redirects to `/latest/rust/rojcad/index.html`
- `/<version>/rust/` redirects to `/<version>/rust/rojcad/index.html`
- All redirects use relative URLs so they resolve within their own version directory

## Capabilities

### New Capabilities
*None.*

### Modified Capabilities
- `docs-publishing`: Add directory-level navigation redirects to ensure all doc site entry points resolve to content

## Impact

- `.github/workflows/release.yml` — add 4 `echo` lines in the "Prepare docs for deployment" step to create redirect `index.html` files
- No new dependencies, no code changes
- Backward compatible — all existing URLs continue to work
