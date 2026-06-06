## MODIFIED Requirements

### Requirement: Edge wireframe overlay

**Change**: Edge color and selection-aware rendering.

The viewer SHALL render topological edges of each shape as smooth polylines overlaid on the mesh surface. Non-selected shape edges SHALL be drawn in light grey (~0.7, 0.7, 0.7). Selected shape edges SHALL be drawn in light blue (~0.4, 0.6, 1.0). The toggle state of `(edge-toggle-inactive)` and `(edge-toggle-active)` SHALL be respected.

#### Scenario: Edges visible on shape
- **WHEN** a shape is displayed
- **THEN** its topological edges are drawn as thin light grey lines on top of the mesh surface

#### Scenario: Selected shape edges are blue
- **WHEN** a shape is selected in the viewer
- **THEN** its topological edges are drawn as thin light blue lines (solid front, dashed back) instead of grey

#### Scenario: Back-edge toggle
- **WHEN** the user presses `X`
- **THEN** dashed back-edge rendering for both active and inactive edges is toggled on/off

#### Scenario: Hidden inactive edges
- **WHEN** `(edge-toggle-inactive)` has been called to hide inactive edges
- **THEN** no grey edges are rendered; only the selected shape's blue edges (if any) and mesh surfaces are shown

#### Scenario: Hidden active edges
- **WHEN** `(edge-toggle-active)` has been called to hide active edges
- **THEN** no blue edges are rendered for the selected shape; only grey edges (if inactive edges are visible) and mesh surfaces are shown
