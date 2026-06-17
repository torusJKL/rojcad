## ADDED Requirements

### Requirement: Three-layer degenerate geometry protection

The system SHALL protect against degenerate geometry at three levels:

**Layer 1 — Pre-validation** (before OCCT call):
A `validate_edge_dimension(shape, value, edge_indices, name)` helper SHALL:
1. Iterate all affected edges
2. Compute approximate edge length via `start_point()` / `end_point()` (safe OCCT queries)
3. Reject if `value * 2 >= edge_length` (distance/radius must be strictly less than half the edge length)
4. Return a descriptive error with the max allowed value

This check SHALL be applied to `fillet` and `chamfer` before any OCCT chamfer/fillet call.

**Layer 2 — Post-check** (after OCCT returns):
A `validate_has_geometry(shape, op)` helper SHALL:
1. Tessellate the shape via `extract_mesh`
2. Extract edge polylines via `extract_edge_polylines`
3. Return an error if both mesh vertices AND edge polylines are empty
4. Accept wire shapes (edges without mesh as valid — wires render as polylines only)

This check SHALL be applied to operations that can produce degenerate results even with seemingly valid parameters:
- `fillet` / `chamfer`
- `wire-fillet` / `wire-chamfer` / `wire-offset`
- `extrude` / `revolve`

This check SHALL NOT be applied to:
- Boolean operations (`cut`, `common`) — empty results are valid (no intersection)
- Primitive creation (`box`, `sphere`, etc.) — always produces valid geometry
- Transformation operations (`translate`, `rotate`, `scale`) — preserves input geometry
- Import operations (`read-step`) — external files can legitimately be empty

**Layer 3 — Viewer guard** (rendering safety net):
The viewer SHALL handle shapes with no renderable geometry gracefully:
- `tessellate_if_needed()` SHALL set `mesh = None` if tessellation produces empty vertex/index buffers
- The viewer SHALL skip rendering entries where `mesh` is `None`
- The viewer SHALL skip rendering entries where mesh vertices or indices are empty

This ensures that even if a shape with no geometry bypasses layers 1 and 2, the viewer never crashes.

#### Scenario: pre-validation rejects too-large chamfer
- **WHEN** user calls `(chamfer (box 10 10 3) :d 1.5)`
- **THEN** the system signals an error before any OCCT computation

#### Scenario: pre-validation rejects too-large fillet
- **WHEN** user calls `(fillet (box 10 10 2) :r 5)`
- **THEN** the system signals an error before any OCCT computation

#### Scenario: post-check catches degenerate extrude
- **WHEN** extrude produces a shape with no mesh vertices and no edge polylines
- **THEN** the system signals an error

#### Scenario: viewer guard skips empty mesh
- **WHEN** a shape with `mesh = None` is in the visible list
- **THEN** the viewer skips surface and edge rendering without crashing

#### Scenario: wire shapes pass post-check
- **WHEN** a wire shape (edges only, no mesh) is produced by `wire-fillet`
- **THEN** `validate_has_geometry` returns Ok (wires have edge polylines)
