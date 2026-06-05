## Context

rojcad embeds 30+ CAD functions as C `JANET_FN` macros in `bridge/bridge.c`, each with a usage string and docstring body. At runtime, these are stored in the Janet environment and accessible via `(doc 'fn-name)`. The `boot.janet` script provides helpers (`cad-fns`, `group`, `apropos`, `doc`) that query this metadata. Currently, there is no way to extract this information into external files.

The design adds a `--eval` CLI flag for one-shot Janet evaluation, an eval hook in boot.janet, and a `dump-docs` function that generates Markdown and HTML documentation using Janet's own runtime metadata.

## Goals / Non-Goals

**Goals:**
- Provide `--eval <expr>` CLI flag that evaluates Janet expressions at startup
- `--eval` uses the same eval path (`my-parse` + `my-eval`) as the TCP REPL, so `(def b (box 10))` auto-shows shapes
- `--eval` does not exit — users opt in with `(os/exit 0)` inside the expression
- Generate `doc/janet-api.md` — structured Markdown for AI agents (flat, parseable)
- Generate `doc/janet-api.html` — single-file HTML for humans (sidebar, search, syntax highlighting)
- Provide `just doc-janet` recipe to build and generate

**Non-Goals:**
- Not modifying the docstring format in `bridge/bridge.c`
- Not adding external dependencies (no JS libraries, no Python scripts, no Janet packages)
- Not modifying the existing `janet-doc` behavior — new recipe is separate
- Not generating docs for non-CAD Janet functions (core library functions)

## Decisions

### 1. Runtime extraction over source parsing
**Chosen:** Extract docs at runtime via Janet's `(doc 'fn)` function.
**Alternatives considered:** Parsing `bridge/bridge.c` with regex or a Python script.
**Rationale:** Runtime extraction guarantees docstrings match what the REPL shows. The metadata helpers (`cad-fns`, `group`, `doc`) already exist in boot.janet and provide structured access. No fragile C parsing, no external script language.

### 2. `--eval` via source appending
**Chosen:** Append the `--eval` expression as raw Janet code at the end of boot.janet's source before passing it to `janet_dostring`.
**Alternatives considered:** Dynamic variable hook (`janet_setdyn` + `(dyn ...)`), evaluating before boot.janet.
**Rationale:** The dyn var approach doesn't work because `janet_setdyn` (C API) stores by keyword while `(dyn ...)` (Janet) looks up by symbol — different key types. Source appending is simpler: the expression runs at the end of boot.janet where all helpers are defined. CAD functions auto-show via `maybe_hide` in C, so `(def b (box 10))` appears in the viewer. Users opt into exit with `(os/exit 0)`.

### 3. Single-file HTML over template engine
**Chosen:** `dump-docs` generates a self-contained HTML file with inline CSS and JS.
**Alternatives considered:** Using Jinja2, mdBook, or another template system.
**Rationale:** Zero dependencies. The page is simple enough (30 functions, 8 categories) that inline everything is reasonable. GitHub Pages friendly — one file to publish.

### 4. Pre-colored syntax highlighting over JS-based
**Chosen:** The Janet `dump-docs` function tokenizes example code and emits `<span class="token-*">` tags at generation time.
**Alternatives considered:** Highlight.js on page load.
**Rationale:** No flash of unstyled code. No extra JS. Smaller runtime. The Janet syntax is simple enough to tokenize in Janet.

### 5. Fixed layout with scrollable main content
**Chosen:** `html,body` use `overflow:hidden` + `height:100%`. Body is a flex column. The search bar is pinned at top via `flex-shrink:0`. The layout area is `flex:1; overflow:hidden`. Sidebar scrolls independently only if needed (`overflow-y:auto`). Only the `<main>` content area scrolls (`overflow-y:auto`).
**Alternatives considered:** Page-level scrolling (default browser behavior).
**Rationale:** Keeps search and category navigation always visible — critical for a 30+ function API reference. Users can search or jump categories without losing their scroll position.

### 6. System font stack over web font
**Chosen:** `font-family: sans-serif` for body, `font-family: monospace` for code blocks.
**Alternatives considered:** Google Fonts Cousine.
**Rationale:** Cousine requires an external request and breaks offline viewing. System fonts are faster and work everywhere.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Docstring format changes in C (`JANET_FN`) could produce malformed output | `dump-docs` uses the canonical `(doc 'fn)` string — it will match whatever the REPL shows. If docstrings change, docs will reflect it automatically. |
| Color rendering varies by OS/browser for "gray" comments | Use a hex value (`#888`) — consistent across platforms. |
| `--eval` expression fails silently | The eval hook uses `try-catch` (already defined in boot.janet) so errors are printed but don't crash the server. |
| File write fails (permissions, missing directory) | `dump-docs` tries to create the output directory. On failure, it prints the error and continues. |
