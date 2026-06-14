## Context

The docs publishing system (`.github/workflows/release.yml`) deploys Janet API docs and Rust crate docs to GitHub Pages with a versioned directory structure (`/latest/`, `/<version>/`). The "Prepare docs for deployment" step copies `target/doc/*` into `_site/latest/rust/` and `_site/$VERSION/rust/`, and places `janet-api.html` at the root of each prefix.

However, `cargo doc` (rustdoc 1.96.0) does not generate a top-level `index.html` for single-crate projects. The `target/doc/` directory contains `rojcad/index.html` as the actual crate entry point, but no `index.html` at its root. Similarly, the `/latest/` and `/<version>/` directories themselves have no `index.html`.

The root of `_site/` already has a redirect (`index.html` → `latest/janet-api.html`), but subdirectory entry points (`/latest/`, `/0.4.0/`, `/latest/rust/`, `/0.4.0/rust/`) return directory listings or 404s.

Neither rustdoc nor the deployment workflow can be relied upon to generate these redirects automatically across versions — the fix must be explicit in the workflow.

## Goals / Non-Goals

**Goals:**
- Add `<meta http-equiv="refresh">` redirect `index.html` files at all doc site directory entry points
- `/latest/` → `/latest/janet-api.html`
- `/<version>/` → `/<version>/janet-api.html`
- `/latest/rust/` → `/latest/rust/rojcad/index.html`
- `/<version>/rust/` → `/<version>/rust/rojcad/index.html`
- All redirects use relative URLs so they resolve within their own directory without hardcoded version numbers

**Non-Goals:**
- Custom landing pages, navigation UI, or version picker
- Changes to existing file structure or URL paths
- Changes to the Janet API or Rust doc generation steps

## Decisions

### Decision: Use inline `<meta http-equiv="refresh">` HTML files

**Chosen**: Four `echo` lines in the "Prepare docs for deployment" step, each writing a minimal HTML redirect file.

```yaml
echo '<meta http-equiv="refresh" content="0; url=janet-api.html">'  > _site/latest/index.html
echo '<meta http-equiv="refresh" content="0; url=janet-api.html">'  > "_site/$VERSION/index.html"
echo '<meta http-equiv="refresh" content="0; url=rojcad/index.html">' > _site/latest/rust/index.html
echo '<meta http-equiv="refresh" content="0; url=rojcad/index.html">' > "_site/$VERSION/rust/index.html"
```

Relative URLs (`janet-api.html`, `rojcad/index.html`) resolve correctly because each `index.html` sits in the same directory as the target. This keeps the redirects version-agnostic — the same line works for both `/latest/` and `/<version>/`.

**Alternatives considered:**
- `_redirects` file (Netlify-style) — not supported by `peaceiris/actions-gh-pages@v4` without the `actions/deploy-pages` action
- Directory-level `.htaccess` — GitHub Pages does not support Apache config
- Rustdoc flags — no stable flag in rustdoc 1.96 to force generation of a top-level `index.html`
- Copy/symlink `rojcad/index.html` up one level — would break relative asset paths (CSS, JS, fonts) in rustdoc

### Decision: Reuse the existing meta-refresh pattern

**Chosen**: The root redirect already uses this exact technique:

```yaml
echo '<meta http-equiv="refresh" content="0; url=latest/janet-api.html">' > _site/index.html
```

Adding 4 more lines in the same step keeps the approach consistent and predictable.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| **Back-button loop** — navigating "back" from a redirected page lands on the redirect again | Same behavior as the existing root redirect. Acceptable — users expect to reach content, not browse directory listings. |
| **Crate rename** — if the Rust crate is renamed from `rojcad` to something else, the `url=rojcad/index.html` paths would break | Would need updating alongside the rename. Low risk — crate name is stable. |
| **rustdoc output structure change** — a future rustdoc might add its own top-level `index.html` | The workflow's file would be overwritten by `cp -r target/doc/*` (if rustdoc adds one), or would continue to work as a fallback. Either outcome is fine. |
| **Per-version maintenance** — each new release creates a new `/$VERSION/` directory automatically via the loop over `$VERSION` | The `$VERSION` lines already exist in the step. Adding the redirect alongside the existing `mkdir + cp` ensures every version gets it. |
