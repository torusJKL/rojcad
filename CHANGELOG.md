# Changelog

## 0.2.0 - 2026-06-07

### Added

- Text shape creation from TrueType/OpenType fonts (`text`, `text3d`, `list-fonts`)
- `view-fit` and `view-fit-all` — frame camera on shape bounding boxes
- View-angle presets (`front`, `back`, `left`, `right`, `top`, `bottom`, `isometric`) and `view-angle` for arbitrary yaw/pitch/distance
- Floating help window (toggle with `H`, dismiss with Escape; Janet: `window-help-toggle`, `window-help-show?`, `window-help-show`)
- Stats-for-nerds overlay (Ctrl+Shift+Alt+S; Janet: `stats-overlay`)
- Hidden edge controls (`edge-hidden-toggle`, `edge-hidden-show?`, `edge-hidden`)
- Projection controls (`projection-toggle`, `projection-perspective`)
- `selected-shapes` and `list-shapes` — query shape state from Janet
- Multi-shape selection in viewer (Ctrl+click toggle, Shift+click add)
- Variadic CAD functions — `hide`, `show`, `purge`, `registry-remove`, `shape-type`, `visible?`, `wire?`, `face?`, `solid?`, `cut`, `common`, `fuse` accept multiple shapes
- Window CLI flags (`--width`, `--height`) and full Janet window API (`window-size`, `window-fullscreen`, `window-maximized`)
- Ctrl+Q to quit the entire application
- Auto-purge old shape on symbol redefinition
- Package as AppImage (`just appimage`) and tarball (`just tarball`) with CI release workflow
- Publish Janet API and Rust docs to GitHub Pages on tagged releases
- Provide version parameter to dump-docs with -dirty suffix

### Changed

- Viewer starts maximized by default (`--width`/`--height` implies windowed)
- ESC no longer closes viewer — falls through to egui help dialog
- `shape-type` returns a tuple (`@[:solid]`) instead of a single keyword (breaking)
- `poll-selection` returns `:deselected` keyword on cleared selection
- Hidden edges default to off
- Auto-show-on-def also fires on `set` forms

## 0.1.0 - 2026-06-06

### Initial Release

rojcad is a parametric CAD system with an embedded Janet DSL, built on OpenCASCADE with an optional wgpu-based 3D viewer.

#### CLI
- `--port <PORT>` / `--port=<PORT>` — set TCP REPL port (default: 9365)
- `--headless` — disable the 3D viewer
- `--eval <EXPR>` / `--eval=<EXPR>` — evaluate Janet code after boot, then exit

#### TCP REPL Server
- TCP socket-based REPL on port 9365 with multiple concurrent client support
- Embedded `boot.janet` runs on startup
- Auto-`show` on `def` (opt-out with `:hide`)
- Janet 1.41.2 vendored and statically compiled from C source

#### 3D Viewer (wgpu + winit)
- wgpu-based rendering (Vulkan/Metal/DX12) on a background thread
- Orbit camera (rotate, pan, zoom)
- Perspective/Orthographic toggle (`P` / `O`)
- Snap views via Ctrl+1, Ctrl+3, Ctrl+7 with animated transitions
- Dark theme, dynamic window resize, dirty-tracking for GPU data
- Grid (XZ plane with axes) and gizmo (axis tripod with labels)

#### Viewer Rendering
- Lambertian diffuse shading with highlight shader for selection
- Instanced screen-space quad line rendering for edges
- Edges: solid/dashed styles, configurable color and thickness, independent inactive/active toggles

#### Shape Picking
- Ray-cast picking via Möller-Trumbore algorithm on left click
- Selection event propagation to Janet via `poll-selection` and `on-select` callback

#### CAD Primitives (3D)
- `box` — box/cube with center or corner positioning
- `sphere` — sphere with partial angle support
- `cylinder` — cylinder with direction and point-to-point construction
- `cone` — full and truncated cone with partial angle support
- `torus` — torus with sweep angle and start/end angle support

#### CAD Primitives (2D)
- `rect` — rectangle as Face or Wire with workplane and position offset
- `circle` — circle as Face or Wire
- `polygon` — polygon from point pairs

#### Boolean Operations
- `cut` — subtract shape b from shape a
- `common` — intersect shapes
- `fuse` — union of shapes

#### Shape Transformations
- `translate` — translation by delta
- `rotate` — rotation by angle around axis
- `scale` — uniform scale about optional center
- `mirror` — mirror about an axis

#### Extrusion & Revolution
- `extrude` — extrude a Face to a Solid with configurable direction and both-sides support
- `revolve` — revolve a Face to a Solid
- `extrude-polygon` — one-shot polygon extrusion

#### Wire Operations
- `wire-to-face` — convert Wire to Face
- `wire-fillet` — round all vertices of a closed Wire
- `wire-chamfer` — bevel all vertices of a closed Wire
- `wire-offset` — parallel offset of a closed Wire

#### Sketching
- Functional/immutable 2D sketch system with workplane support
- `move-to`, `line-to`, `line-dx`, `line-dy`, `line-dx-dy`, `arc-to`
- `close-sketch` and `build-wire`

#### Shape Inspection
- `shape-type`, `visible?`, `wire?`, `face?`, `solid?`

#### Shape Visibility & Registry
- `show`, `hide`, `purge`, `registry-remove`
- Lazy tessellation with `:eager` opt-in

#### File I/O
- `write-step` / `write-stl` — export to STEP and STL
- `read-step` — import from STEP

#### Edge Styling (Runtime Configurable)
- `edge-toggle-inactive`, `edge-toggle-active`
- `edge-inactive-show?`, `edge-active-show?`
- `edge-thickness`, `edge-color-inactive`, `edge-color-active`

#### REPL Discoverability
- `all-fns`, `apropos`, `doc`, `cad-fns`
- `group` — list functions by category
- `dump-docs` — generate Markdown + HTML API documentation
- `display-val` — array/table-aware value display

#### Internal Architecture
- Janet GC integration with `rojcad/shape` and `rojcad/sketch` abstract types
- Thread-safe ShapeRegistry (RwLock + atomic generation counter) shared between REPL and viewer
- On-demand tessellation with synthetic wireframe generation for curved shapes
- Automatic mesh baking and resource cleanup on Drop
- Unit tests for all primitives, booleans, transforms, I/O, 2D, extrusion, revolution, and wire ops
