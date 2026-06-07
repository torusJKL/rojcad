## ADDED Requirements

### Requirement: AppImage package
The system SHALL produce a single-file AppImage distribution that bundles the rojcad binary with all shared library dependencies.

#### Scenario: AppImage is buildable
- **WHEN** `just appimage` is run on a Linux x86_64 system
- **THEN** a file matching `dist/rojcad-*-x86_64.AppImage` is created

#### Scenario: AppImage runs without FUSE
- **WHEN** the AppImage is executed with `--appimage-extract-and-run` on a system without FUSE
- **THEN** rojcad starts successfully in headless mode

#### Scenario: AppImage bundles dynamic libraries
- **WHEN** the AppImage is inspected
- **THEN** it contains `libstdc++.so.6` and `libvulkan.so.1` (the Vulkan loader)

#### Scenario: AppImage includes FreeDesktop metadata
- **WHEN** the AppImage is inspected
- **THEN** it contains a `.desktop` file with `Name=rojcad` and an SVG icon at standard paths

### Requirement: Tarball package
The system SHALL produce a compressed tar archive with the rojcad binary, Janet API documentation, and README.

#### Scenario: Tarball is buildable
- **WHEN** `just tarball` is run on a Linux x86_64 system
- **THEN** a file matching `dist/rojcad-*-x86_64.tar.gz` is created

#### Scenario: Tarball contains required files
- **WHEN** the tarball is extracted
- **THEN** it contains `rojcad`, `doc/janet-api.md`, `doc/janet-api.html`, and `README.md`

### Requirement: Automated release pipeline
The system SHALL publish AppImage and tarball artifacts automatically when a version tag is pushed.

#### Scenario: Release triggers on tag push
- **WHEN** a tag matching `v*` is pushed to GitHub
- **THEN** a GitHub Actions workflow builds both AppImage and tarball artifacts

#### Scenario: Release artifacts are published
- **WHEN** the release workflow completes
- **THEN** a GitHub Release is created with the AppImage and tarball as downloadable assets, with auto-generated release notes
