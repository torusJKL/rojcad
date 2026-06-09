## Context

The release workflow in `.github/workflows/release.yml` builds docs (Rust + Janet API), assembles them under `_site/latest/` and `_site/<version>/`, then deploys to the `gh-pages` branch via `peaceiris/actions-gh-pages`. There is no root-level `index.html`, so GitHub Pages returns a 404 when visiting `https://torusjkl.github.io/rojcad/`.

## Goals / Non-Goals

**Goals:**
- Serve a landing page at the root of the GitHub Pages site
- Redirect visitors to `latest/janet-api.html` (the Janet API reference)
- Minimal change — one line added to the workflow

**Non-Goals:**
- Changing the doc structure or generation process
- Adding a full landing page with styling or multiple links
- Modifying existing URL paths

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Redirect method | `<meta http-equiv="refresh">` | Works without JavaScript, no external deps, instant |
| Behavior | Immediate redirect to `latest/janet-api.html` | Visitors always want the latest docs; versioned docs remain available at their URLs |
| Where to add | In the "Prepare docs for deployment" step, after existing mkdir/cp | Co-located with the rest of doc preparation |

## Risks / Trade-offs

- **Meta refresh is not permanent (302-like)**: Users always go to latest, never to a specific version page. Acceptable — versioned URLs can still be accessed directly.
- **No landing page for browsing**: If users want a version picker, this doesn't provide one. Out of scope for now.
