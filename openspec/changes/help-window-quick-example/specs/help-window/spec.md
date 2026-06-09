## ADDED Requirements

### Requirement: Help window displays a Quick Example section

The help window SHALL display a "Quick Example" section showing a complete, runnable Janet workflow. The section SHALL only appear when Janet has registered content via the `help-set-example` function. The example SHALL show a shape creation followed by a STEP file export.

#### Scenario: Quick Example shown when content registered
- **WHEN** Janet registers an example string via `(help-set-example ...)`
- **THEN** the help window displays a "Quick Example" section with the registered expression rendered in monospace font, followed by the description "Export all visible shapes to a STEP file"

#### Scenario: Quick Example absent when no content registered
- **WHEN** Janet has NOT called `(help-set-example ...)`
- **THEN** no "Quick Example" section appears in the help window

#### Scenario: Example shows box creation and export on Unix
- **WHEN** the OS is Unix (Linux, macOS, BSD) and the viewer starts
- **THEN** the Quick Example section shows:
  ```
  (def mybox (box 10))
  (write-step "/tmp/model.step")
  ```

#### Scenario: Example shows box creation and export on Windows
- **WHEN** the OS is Windows and the viewer starts
- **THEN** the Quick Example section shows:
  ```
  (def mybox (box 10))
  (write-step "C:\temp\model.step")
  ```

### Requirement: Janet function help-set-example

The system SHALL provide a `help-set-example` Janet function that registers the example expression string displayed in the help window's Quick Example section.

#### Scenario: Register example via Janet
- **WHEN** `(help-set-example "(def mybox (box 10))\n(write-step \"/tmp/model.step\")")` is called
- **THEN** the help window displays the expression in the Quick Example section

#### Scenario: Function registered in view group
- **WHEN** `(group "view")` is called
- **THEN** the listing includes `help-set-example`
