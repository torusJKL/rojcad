## Why

Doc strings in the REPL are unreadable: `(doc box)` in the spork REPL (port 9365) shows escaped output with literal `\n` and `\xe2\x80\x94` instead of proper line breaks and em dashes. The raw TCP REPL (port 9364) displays strings correctly, but the heavily-used spork REPL pretty-prints all results including doc strings, escaping special characters.

Additionally, doc strings use Unicode em dashes (`—`) in example code comments (e.g., `(box 10) — creates a cube`) which should use Janet's comment character `#` instead.

## What Changes

- **`boot.janet` — `get-doc` function**: Run doc strings through upstream `doc-format` for terminal-friendly layout (line wrapping, signature extraction, optional ANSI color)
- **`boot.janet` — override `pp`**: Make the pretty-printer print string/buffer values raw (unescaped) so formatted docs display correctly in the spork REPL
- **`boot.janet` — doc string content**: Replace em dashes with `#` in example comments and `-` in prose throughout all doc strings
- **`bridge/bridge.c` — doc string content**: Same em dash replacements in C-level doc strings

## Capabilities

### New Capabilities
- `repl-doc-formatting`: Terminal-friendly doc display using upstream `doc-format` with line wrapping, signature highlighting, and ANSI color support

### Modified Capabilities

_(none — no existing spec-level behavior changes)_

## Impact

- **`boot.janet`**: Modify `get-doc`, override `pp`, replace em dashes in ~27 doc strings
- **`bridge/bridge.c`**: Replace em dashes in ~7 C doc strings
- **No external API changes** — `doc`, `get-doc`, and all CAD functions keep their signatures
- **`pp` behavior change**: String values in spork REPL display raw instead of escaped; this is consistent with the raw TCP REPL behavior
- **Doc string content change**: Users seeing doc strings in generated Markdown/HTML or in the raw REPL will also see `#` and `-` instead of `—`
