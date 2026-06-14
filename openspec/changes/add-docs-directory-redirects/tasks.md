## 1. Workflow change

- [x] 1.1 Add 4 `echo` lines to the "Prepare docs for deployment" step in `.github/workflows/release.yml` to create redirect `index.html` files at `/latest/`, `/$VERSION/`, `/latest/rust/`, and `/$VERSION/rust/`

## 2. Verification

- [x] 2.1 Confirm the redirect files are created in the correct directories with correct relative URLs by inspecting `_site/` after a dry-run or manual test
