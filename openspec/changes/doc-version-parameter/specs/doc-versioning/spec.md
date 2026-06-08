# doc-versioning

## ADDED Requirements

### Requirement: dump-docs accepts optional version parameter

`dump-docs` SHALL accept an optional second argument `version`. When provided, the version string SHALL be included in the generated documentation title. When omitted, the title SHALL remain unchanged.

#### Scenario: Version provided to dump-docs

- **WHEN** `(dump-docs "doc" "v0.1.1")` is called
- **THEN** the generated Markdown and HTML files SHALL contain the title `"rojcad Janet API Reference — v0.1.1"`

#### Scenario: No version provided

- **WHEN** `(dump-docs "doc")` is called
- **THEN** the generated Markdown and HTML files SHALL contain the title `"rojcad Janet API Reference"` (unchanged from current behavior)

#### Scenario: Version provided without path

- **WHEN** `(dump-docs "v0.1.1")` is called (version provided, path omitted)
- **THEN** the default path `"doc"` SHALL be used and the title SHALL include `" — v0.1.1"`

### Requirement: Version appears in Markdown title

`gen-markdown` SHALL write the version string after the title on the `#` header line, separated by ` — ` (space, em-dash, space), when a version is provided.

#### Scenario: Markdown title with version

- **WHEN** `(gen-markdown "path/to/janet-api.md" "v0.1.1")` is called
- **THEN** the first line of the file SHALL be `# rojcad Janet API Reference — v0.1.1`

#### Scenario: Markdown title without version

- **WHEN** `(gen-markdown "path/to/janet-api.md")` is called
- **THEN** the first line of the file SHALL be `# rojcad Janet API Reference`

### Requirement: Version appears in HTML title and heading

`gen-html` SHALL include the version string in the `<title>` tag and the `<h1>` heading, separated by ` — `, when a version is provided.

#### Scenario: HTML title with version

- **WHEN** `(gen-html "path/to/janet-api.html" "a1b2c3d")` is called
- **THEN** the `<title>` tag SHALL contain `"rojcad Janet API Reference — a1b2c3d"` and the `<h1>` element SHALL contain the same text

#### Scenario: HTML title without version

- **WHEN** `(gen-html "path/to/janet-api.html")` is called
- **THEN** the `<title>` tag SHALL be `"rojcad Janet API Reference"` and the `<h1>` element SHALL match

### Requirement: just doc-janet computes and passes version

`just doc-janet` SHALL compute the version string from git and pass it to `dump-docs` via the `--eval` mechanism.

#### Scenario: Tagged commit

- **WHEN** the current commit has an exact git tag (e.g., `v0.1.1`)
- **THEN** `just doc-janet` SHALL pass `"v0.1.1"` as the version to `dump-docs`

#### Scenario: Untagged commit

- **WHEN** the current commit has no git tag
- **THEN** `just doc-janet` SHALL pass the short commit hash (e.g., `"a1b2c3d"`) as the version to `dump-docs`

#### Scenario: Outside a git repository

- **WHEN** `just doc-janet` is run outside a git repository (e.g., from a source tarball)
- **THEN** the version SHALL fall back to the `_version` from `Cargo.toml`

#### Scenario: Dirty working tree on a tag

- **WHEN** the current commit has a git tag AND the working tree has uncommitted changes
- **THEN** `just doc-janet` SHALL pass `"<tag>-dirty"` (e.g., `"v0.1.1-dirty"`) as the version to `dump-docs`

#### Scenario: Dirty working tree without a tag

- **WHEN** the current commit has no git tag AND the working tree has uncommitted changes
- **THEN** `just doc-janet` SHALL pass `"<hash>-dirty"` (e.g., `"a1b2c3d-dirty"`) as the version to `dump-docs`

### Requirement: just tarball computes and passes version

`just tarball` SHALL compute and pass the version to `dump-docs` when generating documentation for the tarball.

#### Scenario: Tarball build passes version to dump-docs

- **WHEN** `just tarball` is run
- **THEN** `dump-docs` SHALL receive the computed version string (tag, commit hash, or Cargo.toml fallback)
