## Context

API documentation is generated entirely at runtime in Janet (`boot.janet`). Three functions are involved: `dump-docs` (orchestrator), `gen-markdown` (Markdown output), and `gen-html` (HTML output). Currently the title is hardcoded as `"rojcad Janet API Reference"` with no version information.

Version is available from git tags (`git describe --tags --exact-match`) or, failing that, the short commit hash (`git rev-parse --short HEAD`). The justfile already has a `_version` variable from `Cargo.toml` but that's not tag-aware.

## Goals / Non-Goals

**Goals:**
- Add an optional `version` parameter to `dump-docs` that flows through to both Markdown and HTML output
- When `version` is provided, append ` — <version>` to the title in both formats
- When `version` is nil/omitted, output remains identical to today
- Compute the version string in the justfile from git (tag → hash → Cargo.toml fallback)
- Pass the version through the existing `--eval` mechanism (no Rust/C changes)

**Non-Goals:**
- Adding any Rust or C bridge changes
- Runtime version detection inside Janet (no `os/shell` or `os/environ`)
- Changing the content or structure of doc entries themselves

## Decisions

**Decision: Pure parameter passing instead of environment variables or runtime git calls.**

- *Alternative considered:* Reading `os/environ` in Janet → simpler at the call site but introduces implicit state and a hidden dependency on env vars being set correctly.
- *Alternative considered:* Computing version inside Janet via `os/shell` git commands → fragile (fails without git, fails outside repo, e.g., tarball builds).
- *Chosen approach:* The justfile computes the version once in the shell and passes it as a string argument through `--eval`. This is explicit, testable in isolation, and has no side effects.

**Decision: Version string format is opaque — justfile decides.**

- The justfile uses a backtick expression (`_doc_version`) computed at load time:
  ```
  git describe --tags --exact-match --dirty  →  "v0.1.1" / "v0.1.1-dirty"
  git rev-parse --short HEAD + dirty check  →  "a1b2c3d" / "a1b2c3d-dirty"
  sed Cargo.toml                            →  "0.2.0"
  ```
- `--dirty` appends `-dirty` when the working tree has uncommitted changes (staged, unstaged, or untracked). This uses `git describe --dirty` for the tag case, and `git status --porcelain` + `grep -q .` for the hash fallback.
- The version string is injected verbatim into the title via `{{_doc_version}}` (just variable interpolation, no shell escaping needed).

**Decision: Title format changes to `"rojcad Janet API Reference — <version>"`.**

- An em-dash separates title from version. Clean, readable, consistent.
- Examples: `"rojcad Janet API Reference — v0.1.1"`, `"rojcad Janet API Reference — a1b2c3d-dirty"`

## Risks / Trade-offs

- **[None]** Shell injection: version is injected via `{{_doc_version}}` which is a just variable, not shell interpolation. No escaping issues.
- **[Low]** No validation inside `dump-docs`: the version string is used as-is. If nil/omitted, no version is shown. No type checking on the parameter.
