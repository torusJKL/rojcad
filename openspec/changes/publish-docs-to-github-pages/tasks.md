## 1. Workflow Changes

- [x] 1.1 Add `Extract version from tag` step after `just tarball` in `release.yml`
- [x] 1.2 Add `Generate Rust API docs` step (`just doc`) in `release.yml`
- [x] 1.3 Add `Prepare docs for deployment` step creating `_site/` with versioned + latest structure
- [x] 1.4 Add `Deploy to GitHub Pages` step using `peaceiris/actions-gh-pages@v4` with `keep_files: true`

## 2. Repository Configuration

- [ ] 2.1 Enable GitHub Pages in repo settings, configured to serve from the `gh-pages` branch
- [ ] 2.2 Push a `v*` tag to verify the workflow runs end-to-end

## 3. Verification

- [ ] 3.1 Confirm `/latest/janet-api.html` and `/latest/rust/` are accessible after deploy
- [ ] 3.2 Confirm `/<version>/` directory is created and persists after a subsequent release
- [ ] 3.3 Confirm the Release workflow still completes without errors (appimage, tarball, release notes)
