## ADDED Requirements

### Requirement: Default window size

The viewer window SHALL default to 1024×768 logical pixels when no `--width` or `--height` CLI flags are provided (the window will also be maximized, but these dimensions serve as the un-maximized fallback size).

#### Scenario: Default window size

- **WHEN** the application starts without `--width` or `--height` flags
- **THEN** the viewer window SHALL be created with dimensions 1024×768 logical pixels (as fallback)

### Requirement: CLI size override

The application SHALL accept `--width <PX>` and `--height <PX>` CLI flags to set initial window dimensions. Each flag SHALL default to 1024 (width) or 768 (height) when only one is provided. Providing either flag SHALL disable maximized mode.

#### Scenario: Both dimensions specified

- **WHEN** the application starts with `--width 800 --height 600`
- **THEN** the viewer window SHALL be created at 800×600 logical pixels in windowed mode

#### Scenario: Only width specified

- **WHEN** the application starts with `--width 1280`
- **THEN** the viewer window SHALL be created at 1280×768 logical pixels in windowed mode

#### Scenario: Only height specified

- **WHEN** the application starts with `--height 720`
- **THEN** the viewer window SHALL be created at 1024×720 logical pixels in windowed mode

### Requirement: Runtime resize via Janet

The system SHALL expose `(window-size width height)` to resize the viewer window at runtime from the Janet REPL. The function SHALL accept exactly two integer arguments representing logical pixel dimensions.

#### Scenario: Resize to different dimensions

- **WHEN** a user calls `(window-size 1920 1080)` from the Janet REPL
- **THEN** the viewer window SHALL resize to 1920×1080 logical pixels

### Requirement: Runtime window size query

The system SHALL expose `(window-size?)` to query the current window dimensions from the Janet REPL. The function SHALL return a tuple of two integers `[width height]` representing the current logical pixel dimensions.

#### Scenario: Query window size

- **WHEN** a user calls `(window-size?)` from the Janet REPL
- **THEN** the function SHALL return the current window dimensions as a tuple `[width height]`

#### Scenario: Size matches after resize

- **WHEN** a user calls `(window-size 800 600)` then calls `(window-size?)`
- **THEN** the result SHALL be `[800 600]`

### Requirement: Size returns correct value after OS resize

The `(window-size?)` SHALL return the actual current window size after any resize, including OS window manager drags or fullscreen transitions.

#### Scenario: Size after manual drag

- **WHEN** a user drags the window edge to resize it, then calls `(window-size?)`
- **THEN** the result SHALL match the new OS-reported window size
