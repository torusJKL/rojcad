## ADDED Requirements

### Requirement: Highlight a model part

A `highlight` function SHALL be provided that visually highlights a named part of a built model in the viewer.

`highlight` SHALL accept a model record and a part identifier (keyword or string matching a node ID from `graph`).

When called, `highlight` SHALL:
1. Look up the shape for the given part ID in the model's shape-map
2. If the part is a sub-model instance, highlight all shapes in that instance
3. Call `(show shape)` to register the shape in the viewer (it may not be visible yet — intermediate shapes are not auto-shown)
4. Track the shape in `model :_hl-shapes` for cleanup on `highlight-clear`
5. Send a highlight command to the viewer via the REPL→viewer mpsc channel
6. The viewer SHALL render the highlighted shape with:
   - Active edge rendering (existing `edge-color-active` in the active edge color)
   - A semi-transparent colored mesh overlay (e.g., blue tint at 30% opacity)

#### Scenario: Highlight a named part

- **WHEN** `(highlight bracket :base)` is called on a built model
- **THEN** the shape corresponding to `:base` SHALL be highlighted in the viewer
- **AND** other parts SHALL remain in their normal visual state

#### Scenario: Highlight the full result

- **WHEN** `(highlight bracket)` is called without a part identifier
- **THEN** the entire result shape SHALL be highlighted

#### Scenario: Highlight on unbuilt model signals error

- **WHEN** `(highlight bracket :base)` is called before any `build`
- **THEN** an error SHALL be signaled

### Requirement: Clear highlighting

A `highlight-clear` function SHALL be provided that removes highlighting and optionally hides previously shown parts.

`highlight-clear` SHALL accept optional model and part-id arguments:
- `(highlight-clear)` — clear viewer highlight state only, don't change visibility
- `(highlight-clear bracket)` — hide all parts that were shown by `highlight` on this model, then clear viewer highlight
- `(highlight-clear bracket :base)` — hide just the `:base` part, then clear viewer highlight

When called with a model, `highlight-clear` SHALL iterate the model's `:_hl-shapes` array and call `(hide shape)` on matching shapes, then clear the array.

#### Scenario: Clear highlights without model

- **WHEN** `(highlight-clear)` is called after a `highlight`
- **THEN** no shapes SHALL be highlighted in the viewer
- **AND** shape visibility SHALL be unchanged

#### Scenario: Clear highlights with model

- **WHEN** `(highlight-clear bracket)` is called after `(highlight bracket :hole)`
- **THEN** the hole shape SHALL be hidden
- **AND** the viewer highlight SHALL be cleared

#### Scenario: Clear a specific part

- **WHEN** `(highlight-clear bracket :base)` is called after `(highlight bracket :base)`
- **THEN** the base shape SHALL be hidden
- **AND** the viewer highlight SHALL be cleared
- **AND** other highlighted parts SHALL remain in `_hl-shapes`

### Requirement: Viewer highlight rendering

The viewer SHALL support rendering highlighted shapes.

Highlight rendering SHALL combine:
- Active edge rendering using the existing `edge-color-active` color
- A semi-transparent colored mesh overlay on top of the shape's normal mesh

Highlight state SHALL be tracked in the viewer and persist across frames until explicitly cleared.

#### Scenario: Highlighted shape renders with edges

- **WHEN** a shape is highlighted with `(highlight m :part)`
- **THEN** the viewer SHALL render that shape's edges in the active edge color

#### Scenario: Highlighted shape renders with tint

- **WHEN** a shape is highlighted with `(highlight m :part)`
- **THEN** the viewer SHALL render a colored overlay on that shape's mesh
- **AND** the overlay SHALL be semi-transparent
- **AND** the underlying mesh SHALL remain visible beneath the overlay

#### Scenario: Normal shapes are unaffected

- **WHEN** a shape is highlighted
- **THEN** non-highlighted shapes SHALL continue to render normally
- **AND** their edge rendering and mesh rendering SHALL be unchanged
