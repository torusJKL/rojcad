## Why

rojcad currently requires building from source: Rust toolchain, CMake, C++ compiler, and a 10–15 minute OCCT compilation. Users who lack these prerequisites or technical confidence cannot try the app. A downloadable, runnable binary removes this barrier.

## What Changes

- Add `appimage` recipe to justfile — produces a single-file AppImage bundling the binary plus all shared library dependencies (glibc, libstdc++, libvulkan loader)
- Add `tarball` recipe to justfile — produces a tar.gz containing the binary, Janet API docs, and README
- Add `.github/workflows/release.yml` — triggers on `v*` tag push, builds both artifacts, creates a GitHub Release with auto-generated release notes
- Add `packaging/rojcad.desktop` and `packaging/rojcad.svg` for FreeDesktop integration
- Add `dist/` and `.appimage/` to `.gitignore`
- Document new recipes in `AGENTS.md`

## Capabilities

### New Capabilities
- `linux-distribution`: Build and distribute rojcad as ready-to-run Linux packages (AppImage and tarball) via automated release pipeline

### Modified Capabilities
- (none)

## Impact

- **Build system**: justfile gains three recipes (`appimage`, `tarball`, `_appimage-tools`)
- **New files**: `packaging/rojcad.desktop`, `packaging/rojcad.svg`, `.github/workflows/release.yml`
- **Modified files**: `justfile`, `.gitignore`, `AGENTS.md`
- **CI**: New release workflow on tag push; existing test/lint workflows unchanged
- **Dependencies**: At build time, linuxdeploy + vulkan plugin (downloaded by justfile), curl. No new runtime dependencies.
