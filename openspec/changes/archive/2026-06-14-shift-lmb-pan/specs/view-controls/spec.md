## ADDED Requirements

### Requirement: View control bindings are documented

The view-controls spec SHALL reference the camera-pan spec for panning controls. The mapping of mouse buttons and keyboard modifiers to camera operations SHALL be:

| Gesture | Action | Defined in |
|---|---|---|
| LMB drag (no modifier) | Orbit / rotate | Existing |
| Shift + LMB drag | Pan | `camera-pan` spec |
| MMB drag | Pan (alternative) | `camera-pan` spec |
| RMB drag | Zoom | Existing |
| Scroll wheel | Zoom | Existing |

#### Scenario: Pan action is delegated to camera-pan spec

- **WHEN** a user performs a Shift+LMB drag or MMB drag
- **THEN** the pan behavior SHALL follow the requirements defined in the `camera-pan` spec
