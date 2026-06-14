## Why

rojcad has no hosted documentation site. The Janet API reference (`.md` + `.html`) and Rust crate docs exist only locally via `just doc-janet` and `just doc`. Users must build from source to see them. Publishing to GitHub Pages on each tagged release gives users an always-available, versioned documentation site without extra infrastructure.

## What Changes

- Extend the existing `release.yml` workflow (triggered on `v*` tags) to deploy docs to GitHub Pages
- Publish the Janet API reference (`janet-api.md`, `janet-api.html`) at the site root
- Publish Rust crate docs (`cargo doc --no-deps` output) under a `/rust/` subdirectory
- Each release produces a versioned directory (`/0.1.0/`, `/0.2.0/`, …) that persists across releases
- A `/latest/` directory is overwritten on each new release
- Old version directories are preserved automatically

## Capabilities

### New Capabilities
- `docs-publishing`: Automatically generate and deploy Janet API and Rust documentation to GitHub Pages on tagged releases, with versioned archives and a latest-alias

### Modified Capabilities
*None.*

## Impact

- `.github/workflows/release.yml` — 4 new steps added after the existing build steps
- Doc generation to `doc/` (Janet) and `target/doc/` (Rust) already exists; no new dependencies
- Single new GitHub Actions dependency: `peaceiris/actions-gh-pages@v4`
- No changes to application code, build system, or developer workflow
