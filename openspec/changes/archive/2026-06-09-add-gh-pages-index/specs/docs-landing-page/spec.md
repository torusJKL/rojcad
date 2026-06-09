## ADDED Requirements

### Requirement: Root-level index page
When visiting the root of the GitHub Pages site (`/`), the system SHALL serve an HTML page that immediately redirects to the latest Janet API documentation at `latest/janet-api.html`.

#### Scenario: Visitor reaches root URL
- **WHEN** a browser requests `https://torusjkl.github.io/rojcad/`
- **THEN** the server returns an HTML document containing a meta-refresh redirect to `latest/janet-api.html`
- **AND** the browser navigates to `latest/janet-api.html` without additional user action

#### Scenario: Direct access to subpath still works
- **WHEN** a browser requests `https://torusjkl.github.io/rojcad/v0.2.1/janet-api.html`
- **THEN** the server returns the existing doc page directly (no redirect)
