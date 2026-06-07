## ADDED Requirements

### Requirement: Unknown category fallback
The system SHALL render all registered CAD functions whose category key is not listed in the `cad-groups` display-name mapping under a single `## Other` section in generated documentation. This SHALL apply to both Markdown and HTML output.

#### Scenario: Unmapped category appears under Other
- **WHEN** a function has `:category` metadata with a key not present in `cad-groups`
- **THEN** (dump-docs) SHALL include that function in the `## Other` section

#### Scenario: Empty Other section omitted
- **WHEN** all registered CAD functions have categories listed in `cad-groups`
- **THEN** the generated docs SHALL NOT include a `## Other` section

#### Scenario: Multiple unmapped categories merge into one Other
- **WHEN** functions from two different unmapped categories exist
- **THEN** all such functions SHALL appear under a single `## Other` heading, not separate per-category headings

### Requirement: Category metadata completeness
Every function registered via `janet_cfuns` in the `cfuns` array that is intended for end-user use SHALL have `:source "rojcad"` and `:category` metadata set via entries in `cad_fn_categories`.

#### Scenario: Quit-requested categorized
- **WHEN** inspecting `(get (doc 'quit-requested) :category)`
- **THEN** it SHALL return `"view"`
