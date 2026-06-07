## ADDED Requirements

### Requirement: Ctrl+Q quits the entire application
The system SHALL close the entire application (viewer + REPL process) when the user presses Ctrl+Q while the 3D viewer window is focused.

#### Scenario: Ctrl+Q with viewer focused
- **WHEN** the 3D viewer window has focus AND the user presses Ctrl+Q
- **THEN** the viewer window closes AND the REPL process exits AND the application terminates

#### Scenario: Ctrl+Q without control modifier
- **WHEN** the user presses "q" without holding Control
- **THEN** no quit action occurs

#### Scenario: Ctrl+Q with caps lock active
- **WHEN** the user presses Ctrl+Q while Caps Lock is on
- **THEN** the application quits (case-insensitive match)

### Requirement: ESC does not close the viewer
The system SHALL NOT close the viewer window when the user presses ESC. ESC SHALL propagate to egui for its own handling (closing help dialogs, etc.).

#### Scenario: ESC in viewer
- **WHEN** the 3D viewer window has focus AND the user presses ESC
- **THEN** the viewer window remains open AND no quit action occurs

#### Scenario: ESC while egui help dialog is open
- **WHEN** an egui help dialog is open AND the user presses ESC
- **THEN** egui closes the help dialog AND the viewer window remains open

### Requirement: Window close button quits the entire application
The system SHALL close the entire application when the user clicks the window close button (X) on the 3D viewer window title bar.

#### Scenario: Close button clicked
- **WHEN** the user clicks the window close button (X)
- **THEN** the viewer window closes AND the REPL process exits AND the application terminates

### Requirement: Quit is one-shot
The quit request SHALL be delivered exactly once. After processing, subsequent checks SHALL return false until a new quit is requested.

#### Scenario: Polling after quit
- **WHEN** `(quit-requested)` returns true AND the caller polls again without a new quit request
- **THEN** the second poll returns false

### Requirement: Headless mode is unaffected
The system SHALL NOT introduce any quit mechanism in headless mode. Quitting a headless session is the user's responsibility (SIGINT, etc.).

#### Scenario: Ctrl+Q in headless mode
- **WHEN** the application runs with `--headless` AND no viewer exists
- **THEN** no Ctrl+Q handler exists AND no quit mechanism is available
