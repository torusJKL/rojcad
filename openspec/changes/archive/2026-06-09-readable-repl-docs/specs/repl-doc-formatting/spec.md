## ADDED Requirements

### Requirement: Doc strings display without escaping in spork REPL

The spork REPL SHALL display string result values as raw text instead of with `%q` escaping (no `""` wrapping, no `\n`/`\xNN` escape sequences).

#### Scenario: doc string displays without escaping
- **WHEN** user types `(doc box)` in the spork REPL
- **THEN** the output SHALL show proper line breaks (not literal `\n`)
- **THEN** the output SHALL show proper characters (not `\xNN` escape sequences)
- **THEN** each example expression SHALL appear on its own line
- **THEN** the "Examples:" header SHALL appear on its own line before the examples

#### Scenario: string output ends with newline before prompt
- **WHEN** a string value is displayed in the spork REPL
- **THEN** a trailing newline SHALL be appended after the string content so the next prompt appears on its own line

#### Scenario: other string values also display raw
- **WHEN** a user expression returns a string value (e.g., `(string "hello\nworld")`)
- **THEN** the spork REPL SHALL display the string content with actual newlines and characters, not escaped sequences

#### Scenario: raw TCP REPL is unaffected
- **WHEN** user types `(doc box)` in the raw TCP REPL (port 9364)
- **THEN** the output SHALL show proper line breaks and characters (same as before this change)

### Requirement: `get-doc` returns raw doc string

The `get-doc` function SHALL return the raw doc string from the binding's `:doc` metadata, without running it through `doc-format` or any other reformatting. This preserves the `\n\n` section structure that `split-docstring` and the doc generators depend on.

#### Scenario: get-doc preserves structure
- **WHEN** `gen-markdown` or `gen-html` calls `get-doc` and passes the result to `split-docstring`
- **THEN** the usage, body, examples, and returns sections SHALL be correctly identified
- **THEN** the HTML output SHALL put each example on its own line inside a `<pre><code>` block

### Requirement: Em dashes replaced in doc strings

All doc strings SHALL use `#` (hash) instead of `—` (em dash, U+2014) for example comments, and `-` (hyphen) instead of `—` for prose.

#### Scenario: example comments use #
- **WHEN** user views docs via `(doc box)` or `dump-docs`
- **THEN** example code in the doc SHALL show `#` as the comment marker, e.g., `(box 10)  # creates a cube`

#### Scenario: prose uses -
- **WHEN** user views docs that previously had prose em dashes
- **THEN** those occurrences SHALL show `-` instead of `—`

#### Scenario: page titles keep em dashes
- **WHEN** user generates documentation via `dump-docs`
- **THEN** page titles SHALL retain em dashes, e.g., `rojcad Janet API Reference — version`
