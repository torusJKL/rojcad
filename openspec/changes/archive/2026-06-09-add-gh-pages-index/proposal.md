## Why

GitHub Pages is configured to serve the `gh-pages` branch from root (`/`), but the deployed docs only exist under `/latest/` and versioned subdirectories. There is no `index.html` at the root, so visitors see a 404 instead of landing on the documentation.

## What Changes

- Add an `index.html` redirect page at the root of the published `_site/` directory in the release workflow
- The redirect will point to `latest/janet-api.html` (the Janet API reference)
- No changes to existing doc files, directory structure, or deployment mechanism

## Capabilities

### New Capabilities
- `docs-landing-page`: A root-level index page for the GitHub Pages site that redirects visitors to the latest Janet API documentation

### Modified Capabilities
*(none)*

## Impact

- **Single file**: `.github/workflows/release.yml` — one additional line in the "Prepare docs for deployment" step
- No change to existing doc generation, file structure, or deployment logic
- Existing URL paths (`/latest/*`, `/<version>/*`) remain unchanged
