## ADDED Requirements

### Requirement: Trigger on tagged release
The workflow SHALL be triggered when a git tag matching the pattern `v*` is pushed to the repository.

#### Scenario: Tag push triggers workflow
- **WHEN** a tag matching `v*` (e.g., `v0.2.0`, `v1.0.0`) is pushed
- **THEN** the docs-publishing workflow SHALL start

#### Scenario: Non-matching tag does not trigger
- **WHEN** a tag not matching `v*` (e.g., `0.2.0`, `v0.2.0-beta`) is pushed
- **THEN** the docs-publishing workflow SHALL NOT start

### Requirement: Generate Janet API documentation
The workflow SHALL generate the Janet API reference from the headless binary by running `(dump-docs "doc")`, producing `janet-api.md` and `janet-api.html` in the `doc/` directory.

#### Scenario: Janet docs generated from release binary
- **WHEN** the workflow runs after a successful release build
- **THEN** the `doc/janet-api.md` and `doc/janet-api.html` files SHALL exist and be non-empty

### Requirement: Generate Rust API documentation
The workflow SHALL generate Rust crate documentation by running `cargo doc --no-deps`, producing the documentation site in `target/doc/`.

#### Scenario: Rust docs generated
- **WHEN** the workflow runs after a successful release build
- **THEN** the `target/doc/rojcad/index.html` file SHALL exist

### Requirement: Deploy with versioned directory structure
The workflow SHALL deploy documentation to GitHub Pages under two directory prefixes:
- `/latest/` — overwritten on each new release
- `/<version>/` — where `<version>` is the git tag with the leading `v` stripped (e.g., tag `v0.2.0` → directory `0.2.0/`)

Janet API files (`janet-api.md`, `janet-api.html`) SHALL be placed at the root of each prefix.
Rust API documentation (`target/doc/*`) SHALL be placed under `/rust/` within each prefix.

#### Scenario: Versioned deployment structure
- **WHEN** a tag `v0.2.0` triggers the workflow
- **THEN** the deployment SHALL contain: `0.2.0/janet-api.html`, `0.2.0/janet-api.md`, `0.2.0/rust/rojcad/index.html`, `latest/janet-api.html`, `latest/janet-api.md`, `latest/rust/rojcad/index.html`

### Requirement: Preserve old version directories
Old version directories SHALL remain accessible after subsequent releases. The deployment SHALL NOT remove any existing directory under `/<version>/`.

#### Scenario: Old versions survive new release
- **WHEN** tag `v0.2.0` is pushed after `v0.1.0` was previously deployed
- **THEN** the `0.1.0/` directory and its contents SHALL still be accessible
- **AND** the `0.2.0/` directory SHALL be newly present

### Requirement: Reuse release build artifacts
The workflow SHALL reuse the release binary already built by the existing Release workflow's `just appimage` and `just tarball` steps — it SHALL NOT trigger an additional full compilation.

#### Scenario: No extra compilation
- **WHEN** the workflow adds doc deployment after the existing `just tarball` step
- **THEN** the binary from the preceding release build SHALL be used for doc generation, SHALL NOT trigger a new `cargo build` from scratch
