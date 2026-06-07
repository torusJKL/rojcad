## 1. Packaging assets

- [x] 1.1 Create `packaging/rojcad.desktop` — FreeDesktop entry with Name, Comment, Exec, Terminal=true, Categories=Graphics;CAD;
- [x] 1.2 Create `packaging/rojcad.svg` — placeholder SVG icon (CAD-themed: isometric box + circle, "rc" text)

## 2. justfile recipes

- [x] 2.1 Add `_version` variable (`:=` evaluated at load) extracting version from Cargo.toml
- [x] 2.2 Add `_appimage-tools` private recipe — download linuxdeploy + appimagetool, make executable
- [x] 2.3 Add `appimage` recipe — depends on `build-release` + `_appimage-tools`, runs linuxdeploy with vulkan plugin, moves result to `dist/`
- [x] 2.4 Add `tarball` recipe — depends on `build-release`, generates Janet API docs via release binary, copies binary + docs + README, creates tar.gz in `dist/`

## 3. Gitignore and docs

- [x] 3.1 Add `dist/` and `.appimage/` to `.gitignore`
- [x] 3.2 Add `appimage` and `tarball` entries to the common commands table in `AGENTS.md`

## 4. Release workflow

- [x] 4.1 Create `.github/workflows/release.yml` — trigger on `v*` tag push and `workflow_dispatch`, with `permissions: contents: write`
- [x] 4.2 Add OCCT cache step (keyed on opencascade-rs commit, release profile paths)
- [x] 4.3 Add steps: `just appimage`, `just tarball`, `softprops/action-gh-release@v2` uploading `dist/*` with `generate_release_notes: true`
