## Why

The generated API documentation (Markdown and HTML) has no version identifier. When browsing docs, users can't tell which release or commit they correspond to. This is especially confusing on GitHub Pages where multiple versions are archived under `/latest/` and `/<version>/`.

## What Changes

- `dump-docs` gains an optional `version` parameter
- When provided, the version string appears in the Markdown title and HTML `<title>`/`<h1>`
- When omitted, output is unchanged (backward compatible)
- `just doc-janet` computes the version from git (tag or short hash) and passes it through
- `just tarball` and the CI release workflow pass the version too

## Capabilities

### New Capabilities
- `doc-versioning`: Optional version parameter for dump-docs that injects a version string into generated API documentation (Markdown + HTML)

### Modified Capabilities

<!-- No existing spec requirements are changing -->

## Impact

- `boot.janet`: `dump-docs`, `gen-markdown`, `gen-html` signatures and title lines
- `justfile`: `doc-janet` and `tarball` recipes to compute and pass version
- `.github/workflows/release.yml`: pass version to tarball step
