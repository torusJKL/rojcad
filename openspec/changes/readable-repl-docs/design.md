## Context

The current `get-doc` function in `boot.janet:960` retrieves a raw doc string from a binding's `:doc` metadata and returns it. Two REPLs display this value:

- **Raw TCP REPL** (port 9364): Uses `display-val` which passes strings through unescaped — doc display works correctly.
- **Spork netrepl** (port 9365): Uses `pp` which calls `printf` with `(dyn *pretty-format* "%q")`. The `%q` format specifier escapes strings (wraps in `""`, escapes `\n`, escapes non-ASCII as `\xNN`) — doc display is broken.

The spork REPL is the more popular of the two (richer protocol, automatic prompts, async I/O). Users almost always see the broken output.

These are the same doc strings also used by the documentation generator (`dump-docs` → `gen-markdown`/`gen-html`), which calls `split-docstring` to parse the raw doc into usage/body/examples/returns sections based on `\n\n` separators.

## Goals / Non-Goals

**Goals:**
- `(doc box)` in the spork REPL shows readable output with proper line breaks and characters
- Replace Unicode em dashes (`—`) with `#` (in example comments) and `-` (in prose) in all doc strings
- Zero breakage of existing APIs (`doc`, `get-doc`, all CAD functions retain their signatures)
- `dump-docs` output continues to work correctly with the same section structure

**Non-Goals:**
- Adding no-arg or string-pattern modes to `doc` (upstream has these; not adding here)
- Using `doc-format` for REPL display — rejected because it merges single-`\n` lines into paragraphs, destroying the example and keyword section structure that `split-docstring` and the doc generators depend on
- Making the raw TCP REPL richer (it's a minimal server by design)

## Decisions

### Decision 1: Keep `get-doc` returning raw doc string

`get-doc` continues to return the raw doc string without `doc-format` processing. The doc generators (`gen-markdown`, `gen-html`) call `get-doc` and pass the result to `split-docstring`, which splits on `\n\n` to extract usage/body/examples/returns sections. Running the doc string through `doc-format` would flatten the `\n\n` structure and merge example lines into a single paragraph.

**Alternatives considered:**
- **Use `doc-format` in `get-doc`** — Merges single-`\n` lines into paragraphs. Examples and keyword continuations are flattened. Rejected: breaks `split-docstring` and HTML/Markdown output.

### Decision 2: Fix spork REPL display at the source — modify vendored `make-onsignal`

The spork REPL's `make-onsignal` function (in `vendor/spork/netrepl-server.janet`) handles successful evaluation results. It calls `pp` on the result value, which escapes strings with `%q` format. Since `make-onsignal` is a closure compiled during spork loading (before boot.janet's `pp` override takes effect), it holds a direct reference to the original `pp` function.

Fix: Change `make-onsignal` to detect string results and output them with `buffer/push-string` (raw, unescaped) instead of `pp`. Non-string results go through the original `pp` as before.

```janet
(fn on-signal [f x]
  (case (fiber/status f)
    :dead (do (put e '_ @{:value x})
              (if (= :string (type x))
                (buffer/push-string (dyn :out) x)
                (pp x)))
    ...))
```

**Alternatives considered:**
- **Override `pp` in `boot.janet`** — A `pp` override is added as a belt-and-suspenders measure, but the spork closure captures the original `pp` at load time, so the override doesn't affect the spork display path.
- **Make `doc` print directly** — Would bypass `pp` entirely but breaks when `get-doc` is called programmatically (by doc generators). Rejected.
- **Change spork display format** — Could change `*pretty-format*` dynamic binding, but `pp` uses it for all types, not just strings. Rejected.

### Decision 3: Replace `—` with `#` in examples, `-` in prose

Three categories of em dash usage in doc strings:

| Category | Example | Replacement | Count (boot.janet) | Count (bridge.c) |
|----------|---------|-------------|-------------------|------------------|
| Example comments | `(box 10) — creates a cube` | `#` | ~25 | ~7 |
| Prose | `— along Z axis` | `-` | ~1-2 | ~0 |
| Page titles | `Reference — version` | keep em dash | 2 | 0 |

Example pattern in boot.janet doc strings:
```
- "  (box 10)  — creates a cube\n"
+ "  (box 10)  # creates a cube\n"
```

Example pattern in bridge.c doc strings:
```
- "  (purge b)          — remove b from viewer\n"
+ "  (purge b)          # remove b from viewer\n"
```

Prose example (cylinder doc in boot.janet):
```
- "Positional: (cylinder radius height) — along Z axis"
+ "Positional: (cylinder radius height) - along Z axis"
```

Page titles at boot.janet:1158,1224 (`"rojcad Janet API Reference — version"`) — these are for generated HTML/Markdown output, not REPL display. Keep em dashes.

### Decision 4: Doc strings must preserve `\n\n` section separator structure

Both the doc generators and the REPL display depend on the doc string having a consistent structure:
- First `\n\n`-delimited block: usage signature
- Subsequent blocks: body text, "Examples:" block, "Returns" block
- Examples section: each example on its own line, indented 2 spaces, starting with `(`
- Body and examples separated by `\n\n` from other sections

## Risks / Trade-offs

- **Vendored spork modification**: The change to `vendor/spork/netrepl-server.janet` must be re-applied when updating spork from upstream. **Mitigation**: Added to AGENTS.md so the agent knows to re-apply this fix.
- **`make-onsignal` change affects all string results**: Any string return value in the spork REPL now displays raw (unescaped). This is consistent with the raw TCP REPL behavior and is better UX.
- **Doc string format must be maintained**: Future doc string additions must follow the `#` comment convention and preserve `\n\n` section separators. **Mitigation**: Added to AGENTS.md as a style requirement.
- **Raw TCP REPL unchanged**: The raw REPL already displays strings correctly. No regression expected.
- **`dump-docs` generated output shows `#` instead of `—`**: This is intentional — `#` is more idiomatic for Janet example code.
