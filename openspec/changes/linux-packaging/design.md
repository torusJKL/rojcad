## Context

rojcad is a Rust binary (31 MB release) with a mixed-language build: OCCT (C++, statically linked), Janet (C, static), and a C bridge (static). The binary dynamically links against `libstdc++.so.6`, `libgcc_s.so.1`, `libm.so.6`, and `libc.so.6`. The 3D viewer (wgpu/winit) loads Vulkan, X11, and Wayland libraries at runtime via `dlopen`.

Currently there is no packaging — users must build from source, requiring Rust, CMake, C++11 compiler, and a 10–15 minute OCCT compilation.

## Goals / Non-Goals

**Goals:**
- Users can download and run rojcad without building from source or installing dependencies
- Two distribution formats: AppImage (zero-install, one file) and tarball (portable archive)
- Automated release pipeline triggered by `v*` tags on GitHub
- AppImage bundles the binary plus all shared library dependencies (including Vulkan loader)
- Tarball includes binary, Janet API docs (HTML + Markdown), and README

**Non-Goals:**
- Windows or macOS packaging (future work)
- Flatpak or Snap packages
- musl-based fully static build (OCCT C++ dependency makes this impractical)
- Package manager repositories (apt, dnf, etc.)

## Decisions

### 1. AppImage format (over Flatpak, Snap, or raw binary)
- **Chosen**: AppImage via linuxdeploy + vulkan plugin
- **Rationale**: Single-file download, no runtime dependencies, works on any Linux distro without sudo. Flatpak requires the runtime to be installed; Snap is Ubuntu-centric. AppImage is the simplest path for "download and run."
- **Alternatives considered**: Flatpak (better sandboxing but heavier setup), Snap (Ubuntu-only reach), static binary (blocked by OCCT's C++ linkage)

### 2. linuxdeploy for AppImage creation (over manual AppDir construction)
- **Chosen**: linuxdeploy with `--output appimage` (uses appimagetool internally)
- **Rationale**: Automatically discovers and bundles shared library dependencies. Handles Vulkan loader via plugin. No need to manually trace `ldd` output or track changing deps.
- **Alternatives considered**: Manual AppDir construction (brittle, maintenance burden), `cargo-appimage` (less mature, no Vulkan plugin)

### 3. Ubuntu 24.04 as the build platform (glibc floor)
- **Chosen**: Build on `ubuntu-latest` (Ubuntu 24.04, glibc 2.39)
- **Rationale**: AppImage bundles its own glibc, so the build machine's glibc doesn't affect target compatibility. For the tarball (which doesn't bundle libs), Ubuntu 24.04's glibc is the minimum requirement — users need glibc >= 2.39 (Ubuntu 24.04, Debian 12, Fedora 39+).
- **Alternatives considered**: Building in an older Docker container for broader glibc compat (adds complexity; the tarball is already a "power user" option)

### 4. Separate `appimage` and `tarball` justfile recipes
- **Chosen**: Two independent recipes, each calling `build-release` as dependency
- **Rationale**: CI builds both; developers can build just one locally. Cargo caching makes the redundant build negligible (sub-second for cached release artifacts).

### 5. Vulkan loader bundling
- **Chosen**: Bundle `libvulkan.so.1` via linuxdeploy-vulkan plugin; do NOT bundle GPU ICDs
- **Rationale**: The loader is required for wgpu to initialize. The GPU ICD (driver) stays on the host system in standard paths (`/usr/lib/x86_64-linux-gnu/GL/`, `/usr/lib/dri/`, etc.). If no GPU is available, the `--headless` flag works without Vulkan at all.

### 6. Doc generation from release binary
- **Chosen**: `tarball` recipe runs `cargo run --release -- --headless --eval '(dump-docs "doc")'` to generate docs
- **Rationale**: Keeps existing `doc-janet` recipe (debug mode) unchanged. The release binary is already built by the `build-release` dependency, so this adds minimal time.

## Risks / Trade-offs

- **[AppImage FUSE dependency]** → The justfile uses `APPIMAGE_EXTRACT_AND_RUN=1` to extract linuxdeploy before running it, bypassing FUSE entirely. The resulting AppImage can fall back to `--appimage-extract-and-run` on systems without FUSE.
- **[CI has no GPU]** → `--headless` mode works without a GPU. The vulkan plugin bundles only the loader shared library, which is harmless without a driver.
- **[Tarball glibc compat]** → Users on older distros (Ubuntu < 24.04, Debian < 12) may need to use the AppImage instead, which bundles its own glibc. Document this in the release notes.
- **[linuxdeploy download flakiness]** → justfile uses `test -f` to skip re-download if cached. CI can add `curl --retry 3` if needed.
- **[OCCT rebuilds on every release]** → The release workflow uses the same OCCT caching strategy as the test workflow (keyed on opencascade-rs git commit), so rebuilds only happen when the dependency is bumped.
