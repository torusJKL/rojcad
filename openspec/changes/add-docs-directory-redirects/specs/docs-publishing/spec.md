## ADDED Requirements

### Requirement: Directory entry points redirect to content
The deployment SHALL include `<meta http-equiv="refresh">` redirect `index.html` files at each directory-level entry point, so that navigating to a directory path resolves to its primary documentation file.

The following redirects SHALL be present in every version prefix (`/latest/` and `/<version>/`):

| Directory | Redirect target |
|-----------|----------------|
| `/<prefix>/` | `/<prefix>/janet-api.html` |
| `/<prefix>/rust/` | `/<prefix>/rust/rojcad/index.html` |

All URLs in redirect files SHALL be relative paths, resolving within the same directory as the `index.html` file.

#### Scenario: Latest directory redirects to Janet API
- **WHEN** a user navigates to `/latest/`
- **THEN** the browser SHALL be redirected to `/latest/janet-api.html`

#### Scenario: Version directory redirects to that version's Janet API
- **WHEN** a user navigates to `/0.4.0/`
- **THEN** the browser SHALL be redirected to `/0.4.0/janet-api.html`

#### Scenario: Latest rust directory redirects to crate index
- **WHEN** a user navigates to `/latest/rust/`
- **THEN** the browser SHALL be redirected to `/latest/rust/rojcad/index.html`

#### Scenario: Version rust directory redirects to that version's crate index
- **WHEN** a user navigates to `/0.4.0/rust/`
- **THEN** the browser SHALL be redirected to `/0.4.0/rust/rojcad/index.html`

#### Scenario: Existing file URLs unaffected
- **WHEN** a user navigates to `/latest/janet-api.html` or `/0.4.0/rust/rojcad/fn.main.html`
- **THEN** the file SHALL be served directly without redirect
