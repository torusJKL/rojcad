## MODIFIED Requirements

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
