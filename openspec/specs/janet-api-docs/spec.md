## ADDED Requirements

### Requirement: `--eval` CLI flag
The system SHALL accept a `--eval <expr>` CLI argument that evaluates a Janet expression at startup after all CAD functions and boot.janet helpers are registered. The system SHALL use `my-parse` + `my-eval` (the same evaluation path as the TCP REPL) so that `(def b (box 10))` triggers auto-show in the viewer. The system SHALL NOT exit after evaluating — the expression controls exit via `(os/exit 0)`. The system SHALL accept multiple `--eval` flags and evaluate them in order.

#### Scenario: Eval expression runs at startup
- **WHEN** rojcad is started with `--eval '(+ 1 2)'`
- **THEN** the expression `(+ 1 2)` SHALL be evaluated and the result (`3`) SHALL be printed to stderr

#### Scenario: Def shape with viewer show
- **WHEN** rojcad is started (without `--headless`) with `--eval '(def b (box 10))'`
- **THEN** a box SHALL be created, registered, and shown in the 3D viewer

#### Scenario: Multiple eval expressions
- **WHEN** rojcad is started with `--eval '(def b (box 10))' --eval '(def s (sphere 5))'`
- **THEN** both expressions SHALL be evaluated in order

#### Scenario: Eval with exit
- **WHEN** rojcad is started with `--eval '(do (print "done") (os/exit 0))'`
- **THEN** the process SHALL exit after evaluating, with "done" printed

### Requirement: `dump-docs` function
The system SHALL provide a `(dump-docs &opt path)` Janet function callable from the REPL or via `--eval`. The function SHALL iterate all registered CAD functions via `(group)`, retrieve each function's docstring via `(doc 'fn)`, and generate documentation files. The optional `path` argument SHALL specify the output directory (default: `"doc"`).

#### Scenario: Generate docs to default directory
- **WHEN** `(dump-docs)` is called
- **THEN** files `doc/janet-api.md` and `doc/janet-api.html` SHALL be created

#### Scenario: Generate docs to custom directory
- **WHEN** `(dump-docs "output")` is called
- **THEN** files `output/janet-api.md` and `output/janet-api.html` SHALL be created

#### Scenario: Error handling on write failure
- **WHEN** `(dump-docs "/nonexistent/deep/path")` is called
- **THEN** the function SHALL print an error and return nil without crashing

### Requirement: Markdown output
The Markdown file SHALL contain one `#` title, one `##` section per category, and one `###` subsection per function. Each function subsection SHALL include **Usage:**, the description body, **Examples:** in a fenced code block with `janet` language tag, and **Returns:**. The structure SHALL be flat (single column, no sidebar).

Categories SHALL be rendered from the `cad-groups` display-name mapping. Any function registered in the Janet environment with `:source "rojcad"` whose category key is NOT present in `cad-groups` SHALL be rendered under a single `## Other` section at the end of the file.

#### Scenario: Category grouping
- **WHEN** inspecting `doc/janet-api.md`
- **THEN** functions SHALL be grouped under `## CategoryName` headings matching the `cad-groups` display names, with any unmapped categories appearing under `## Other`

#### Scenario: Function entry structure
- **WHEN** inspecting `doc/janet-api.md`
- **THEN** each function SHALL have the format `### \`<name>\`` followed by **Usage:**, description, **Examples:**, and **Returns:**

#### Scenario: Code block language tag
- **WHEN** inspecting example sections in `doc/janet-api.md`
- **THEN** fenced code blocks SHALL use the `janet` language tag (````janet`)

### Requirement: HTML output
The HTML file SHALL be a single self-contained page with inline CSS and JS. It SHALL include: a search input with Ctrl+K keyboard shortcut, a sidebar with collapsible category links, a main content area with category sections and function cards, and syntax-highlighted code examples. The page SHALL use a light theme (white background, `#222` text). Body text SHALL use system sans-serif. Code blocks SHALL use system monospace. The layout SHALL be fixed: the search bar and sidebar SHALL remain visible at all times; only the `<main>` content area SHALL scroll (`overflow-y:auto`). The sidebar MAY scroll independently if its content exceeds the viewport height.

The sidebar SHALL include a link for the `## Other` section if any unmapped categories exist.

#### Scenario: Sidebar includes Other link
- **WHEN** `(dump-docs)` generates docs with unmapped categories
- **THEN** the HTML sidebar SHALL contain an "Other" link in the category list

#### Scenario: Search filters function cards
- **WHEN** user types in the search input or presses Ctrl+K to focus it
- **THEN** function cards SHALL be filtered to those whose text content matches the query

#### Scenario: Sidebar navigation
- **WHEN** user clicks a category or function link in the sidebar
- **THEN** the page SHALL scroll to the corresponding `##` or `###` anchor

#### Scenario: Syntax highlighting in examples
- **WHEN** viewing example code blocks in the HTML
- **THEN** comments SHALL be colored `#888888` (gray), strings `#a31515` (red), keywords `#0000ff` (blue), numbers `#098658` (green), and special forms (`def`, `fn`, `if`, `do`, `while`, `var`, `set`, `break`) `#795e26` (brown)

#### Scenario: Back-to-top button
- **WHEN** user clicks the "↑" button in the bottom-right corner
- **THEN** the `<main>` content area SHALL scroll to the top with smooth animation

#### Scenario: Light theme defaults
- **WHEN** the HTML is opened without any user preferences
- **THEN** the background SHALL be white, text SHALL be `#222`

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

### Requirement: `just doc-janet` recipe
The justfile SHALL provide a `doc-janet` recipe that builds rojcad and runs it with `--headless --eval '(do (dump-docs "doc") (os/exit 0))'`.

#### Scenario: Running doc-janet
- **WHEN** `just doc-janet` is run
- **THEN** rojcad SHALL start headless, generate documentation files, and exit
