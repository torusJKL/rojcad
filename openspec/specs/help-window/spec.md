## ADDED Requirements

### Requirement: Help window is visible on startup
The help window SHALL be visible when the 3D viewer first opens.

#### Scenario: First launch shows help
- **WHEN** the viewer starts
- **THEN** the help window is displayed centered in the viewport

### Requirement: Help window can be toggled with keyboard
The help window SHALL toggle visibility when the user presses `h` (or `H`). This SHALL NOT fire when egui has keyboard focus (e.g., a text field is active).

#### Scenario: Toggle help on with h key
- **WHEN** the help window is hidden and the user presses `h`
- **THEN** the help window becomes visible

#### Scenario: Toggle help off with h key
- **WHEN** the help window is visible and the user presses `h`
- **THEN** the help window becomes hidden

#### Scenario: h key ignored when egui has focus
- **WHEN** egui has keyboard focus and the user presses `h`
- **THEN** the help window visibility does not change

### Requirement: Help window closes with Escape
The help window SHALL close when the user presses Escape while it is visible. If the help window is already hidden, Escape SHALL do nothing (it does not close the application — Ctrl+Q does).

#### Scenario: Escape closes help
- **WHEN** the help window is visible and the user presses Escape
- **THEN** the help window becomes hidden

#### Scenario: Escape is no-op when help is hidden
- **WHEN** the help window is hidden and the user presses Escape
- **THEN** the help window stays hidden and the application continues running

### Requirement: Help window closes with X button
The help window SHALL close when the user clicks its title-bar close button.

#### Scenario: X button closes help
- **WHEN** the help window is visible and the user clicks the X button
- **THEN** the help window becomes hidden

### Requirement: Help window displays keyboard shortcuts
The help window SHALL display all active keyboard shortcuts with their action descriptions.

#### Scenario: All shortcuts shown
- **WHEN** the help window is visible
- **THEN** it lists: Esc, p/o, x, h, Ctrl+1, Ctrl+3, Ctrl+7, Ctrl+Shift+Alt+S, Ctrl+Q

### Requirement: Help window displays REPL documentation commands
The help window SHALL display how to get documentation from the Janet REPL.

#### Scenario: REPL commands shown
- **WHEN** the help window is visible
- **THEN** it lists: (doc ...), (apropos ...), (group), (cad-fns), (all-fns), (dump-docs)

### Requirement: Help window displays REPL connection info
The help window SHALL display how to connect to the REPL via netcat.

#### Scenario: Connection info shown
- **WHEN** the help window is visible
- **THEN** it shows: nc 127.0.0.1 9364

### Requirement: Help window displays CLI arguments
The help window SHALL display available command-line arguments.

#### Scenario: CLI args shown
- **WHEN** the help window is visible
- **THEN** it lists: --headless, --raw-port, --spork-port, --eval

### Requirement: Janet function window-help-toggle
The system SHALL provide a `window-help-toggle` Janet function that toggles help visibility and returns the new state.

#### Scenario: Toggle via Janet
- **WHEN** `(window-help-toggle)` is called in the REPL
- **THEN** the help window visibility toggles and the function returns true if now visible, false if hidden

### Requirement: Janet function window-help-show?
The system SHALL provide a `window-help-show?` Janet function that returns whether the help window is visible.

#### Scenario: Query via Janet
- **WHEN** `(window-help-show?)` is called in the REPL
- **THEN** it returns true if the help window is visible, false otherwise

### Requirement: Janet function window-help-show
The system SHALL provide a `window-help-show` Janet function that gets or sets the help window visibility.

#### Scenario: Query with no args
- **WHEN** `(window-help-show)` is called with no arguments
- **THEN** it returns true if visible, false if hidden

#### Scenario: Set visible
- **WHEN** `(window-help-show true)` is called
- **THEN** the help window becomes visible

#### Scenario: Set hidden
- **WHEN** `(window-help-show false)` is called
- **THEN** the help window becomes hidden

### Requirement: Janet functions registered in view group
All three help-related Janet functions SHALL be registered under the `"view"` category in the cad_groups table.

#### Scenario: Group listing includes help functions
- **WHEN** `(group "view")` is called
- **THEN** the listing includes window-help-toggle, window-help-show?, and window-help-show
