## 1. Janet: Add version parameter to dump-docs, gen-markdown, gen-html

- [x] 1.1 Add version parameter to `gen-markdown` in `boot.janet`; when provided, write `"# rojcad Janet API Reference — <version>"` instead of `"# rojcad Janet API Reference"`
- [x] 1.2 Add version parameter to `gen-html` in `boot.janet`; when provided, include `" — <version>"` in `<title>` and `<h1>`
- [x] 1.3 Add version parameter to `dump-docs` in `boot.janet`; pass through to `gen-markdown` and `gen-html`
- [x] 1.4 Verify backward compatibility: `(dump-docs "doc")` without version produces identical output

## 2. justfile: Compute version and pass through --eval

- [x] 2.1 Update `doc-janet` recipe to compute version string and pass via `--eval` argument
- [x] 2.2 Update `tarball` recipe to compute version string and pass via `--eval` argument
- [x] 2.3 Append `-dirty` suffix when working tree has uncommitted changes (staged, unstaged, untracked)

## 3. CI: Pass version in release workflow

- [x] 3.1 Version computation embedded in `just tarball` recipe — CI runs `just tarball` on a tagged commit, so `git describe` resolves correctly. No CI change needed.
