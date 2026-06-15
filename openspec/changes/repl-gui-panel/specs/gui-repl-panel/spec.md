## ADDED Requirements

### Requirement: REPL panel

The system SHALL display a REPL panel docked to the right side of the viewer window when not in headless mode.

The panel SHALL be resizable with a default width of 350px and a minimum width of 200px.

The panel SHALL show a separator line between the 3D viewport and the panel area.

The panel SHALL be toggleable via the Ctrl+R keyboard shortcut.

The panel SHALL default to visible on startup unless `--no-repl` is passed.

A `--repl` CLI flag SHALL force the panel visible on startup.

A `--no-repl` CLI flag SHALL force the panel hidden on startup.

The current panel width SHALL be displayed in the stats overlay (Ctrl+Shift+Alt+S).

#### Scenario: Panel is visible at startup
- **WHEN** rojcad starts with the viewer enabled and without `--no-repl`
- **THEN** the REPL panel is visible on the right side of the window

#### Scenario: Panel toggles with Ctrl+R
- **WHEN** the user presses Ctrl+R
- **THEN** the REPL panel toggles between visible and hidden

#### Scenario: Panel is hidden in headless mode
- **WHEN** rojcad starts with `--headless`
- **THEN** no REPL panel is displayed (no viewer window exists)

#### Scenario: --no-repl flag hides panel
- **WHEN** rojcad starts with `--no-repl`
- **THEN** the REPL panel is hidden but can be shown with Ctrl+R

#### Scenario: Panel is resizable
- **WHEN** the user drags the left edge of the REPL panel
- **THEN** the panel width changes and the gizmo position adjusts

#### Scenario: Panel width in stats overlay
- **WHEN** the stats overlay is visible and the REPL panel is open
- **THEN** the overlay shows the current panel width

### Requirement: Gizmo position

When the REPL panel is visible, the axis gizmo SHALL be shifted left by 
`panel_phys + 175*sf + gm` physical pixels relative to its default position,
where `panel_phys` is the panel width converted from logical to physical pixels 
via the window scale factor `sf` (floating-point multiplication with `.round()`).

When the panel is hidden, the gizmo SHALL use the standard top-right position.

#### Scenario: Gizmo avoids panel
- **WHEN** the REPL panel is visible
- **THEN** the axis gizmo is positioned left of the panel with a constant gap

#### Scenario: DPI-aware positioning
- **WHEN** scale factor is non-integer (e.g., 1.5)
- **THEN** the offset is computed in f64 to avoid truncation errors

### Requirement: Code input

The system SHALL provide a multi-line code input area at the bottom of the REPL
panel, implemented via `egui_code_editor::CodeEditor`.

The input SHALL use a monospace font with syntax highlighting driven by a custom
Janet `Syntax` definition. Built-in functions (box, sphere, cut, etc.) SHALL be
classified as `Type` tokens. Keywords (def, defn, var, set, etc.) SHALL be
classified as `Keyword` tokens.

Pressing Enter SHALL insert a newline.

Pressing Ctrl+Enter SHALL submit the current input for evaluation.

The input SHALL support standard text editing operations.

#### Scenario: Multi-line input
- **WHEN** the user types across multiple lines
- **THEN** the input contains the expected newlines

#### Scenario: Ctrl+Enter submits
- **WHEN** the user types code and presses Ctrl+Enter
- **THEN** the expression is sent for evaluation

#### Scenario: Input is clearable
- **WHEN** the user presses the Clear button
- **THEN** the input buffer is cleared

### Requirement: Output log

The system SHALL display a scrollable output log above the input area showing
the history of evaluated expressions and their results.

Each entry SHALL consist of a syntax-coloured code line (tokenized via
`Token::highlight()` with a custom `Editor` implementation) followed by the
result line(s).

Token categories and their colors (defined in `ROJCAD_THEME`):
- comments -> gray (#808080)
- keywords -> purple (#C878C8)
- types (builtin functions) -> cyan (#50C8FF)
- strings -> green (#4EC9B0)
- numbers -> orange (#FFB450)
- literals -> yellow (#FFD700)
- punctuation -> white (#FFFFFF)
- special -> purple (#C878C8)

Brackets SHALL use rainbow coloring based on nesting depth, cycling through:
white -> yellow -> cyan -> green -> purple -> orange.

Result values SHALL be colored by type:
- Shape values -> yellow
- Errors -> red
- Numbers -> orange
- Strings -> green
- Keywords -> yellow
- nil -> gray
- Other -> white

The log SHALL auto-scroll to the bottom on new results.

The log SHALL be clearable via a Clear button.

#### Scenario: Coloured output
- **WHEN** the user evaluates `(box 10)`
- **THEN** the output log shows the code coloured and `#<Shape...>` in yellow

#### Scenario: Rainbow brackets
- **WHEN** nested forms are displayed in the history
- **THEN** brackets at different depths have different colours

#### Scenario: Log auto-scrolls
- **WHEN** a new result is added
- **THEN** the log scrolls to show the new entry

### Requirement: Code evaluation

The system SHALL evaluate Janet expressions on the REPL thread using the
existing `my-eval`/`my-parse` functions in `core-env`.

A dedicated cross-thread channel pair (mpsc) SHALL carry requests and responses.
Requests use a simple pipe-delimited format with `\x02` (STX) as separator:
- `e\x02<id>\x02<code>` — eval
- `h\x02<id>\x02<code>` — highlight (handled but deprecated in favour of local
  tokenization via egui_code_editor)

Responses use the same JSON format produced by the Janet-side `json-encode`
function, with fields `type`, `id`, `value`, `kind`, `new_bindings`.

#### Scenario: Expression is evaluated
- **WHEN** the user submits `(+ 1 2 3)`
- **THEN** the result `6` of kind number is returned

#### Scenario: Parse error is returned
- **WHEN** the user submits `(box`
- **THEN** the result contains a parse error of kind error

### Requirement: Cross-thread channel protocol

The system SHALL implement a bidirectional message channel between the viewer
thread and the REPL thread.

The channel SHALL use Rust `mpsc` for viewer-to-REPL direction and a second
`mpsc` for REPL-to-viewer direction, both carrying JSON-encoded strings.

Request messages SHALL use the `\x02`-delimited format:
`<type_byte>\x02<id_as_string>\x02<body>`

Type bytes: `e` = eval, `h` = highlight, `c` = completions.

Response messages SHALL use JSON with fields: `type`, `id`, and type-specific
data fields.

#### Scenario: Request-response flow
- **WHEN** the viewer sends `e\x021\x02(+ 1 2)` via the channel
- **THEN** the REPL thread responds with a JSON evalResult containing the value

### Requirement: Syntax consistency

The input editor and the history output log SHALL use the same tokenizer and
colour palette. Both SHALL be driven by a shared `janet_syntax()` function that
returns a `Syntax` configuration, and a shared `ROJCAD_THEME` constant.

Bracket rainbow colours only apply to the history output (input editor uses the
theme's punctuation colour for brackets).

#### Scenario: Same colours
- **WHEN** code appears in both the input editor and the history log
- **THEN** keywords, types, strings, numbers, and literals use the same colours

### Requirement: Code completion

The system SHALL provide a code completion popup that suggests function names as the user types.

The completion popup SHALL auto-trigger when the current word (the text from the last whitespace or bracket to the cursor position) is at least 2 characters long.

The completion candidates SHALL be filtered from the pre-loaded function names (`fn_names` set) by prefix matching against the current word.

The popup SHALL display up to 20 matching candidates in a scrollable list, ordered alphabetically.

The popup SHALL appear below the code input area, anchored to the left edge of the REPL panel.

Clicking a candidate SHALL insert the selected name into the input buffer, replacing the current word.

If no candidates match, the popup SHALL be hidden.

The popup SHALL be dismissible by pressing Escape or by clicking outside the popup area.

#### Scenario: Completion triggers on 2+ char prefix
- **WHEN** the user types `bo` in the code editor
- **THEN** a popup appears showing `box` as a completion candidate

#### Scenario: Completion inserts on click
- **WHEN** the user clicks `box` in the completion popup
- **THEN** the input is updated to replace `bo` with `box`

#### Scenario: Completion hides on empty prefix
- **WHEN** the user deletes characters so the current word is shorter than 2 characters
- **THEN** the completion popup is hidden

#### Scenario: Completion hides on no matches
- **WHEN** the user types `zz` which matches no function names
- **THEN** the completion popup is hidden (not shown empty)
