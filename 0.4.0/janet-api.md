# rojcad Janet API Reference — v0.4.0

## Operations

### `extrude`

**Usage:** `(extrude shape &keys :h :x :y :z :dir :both :eager :hide)`

Extrude a shape or wire along a direction.
Keywords: :h (height), :x/:y/:z (axis shortcuts),
         :dir [dx dy dz] (direction vector),
         :both (extrude both sides),
         :eager, :hide

**Examples:**
```janet
(extrude wire :h 10)
(extrude wire :h 10 :x)
(extrude wire :h 10 :dir [0 0 1] :both)
```

**Returns a rojcad/shape abstract value.**

### `extrude-polygon`

**Usage:** `(extrude-polygon points height &keys :plane :at :eager :hide)`

Extrude a polygon defined by a list of 2D points.
Keywords: :plane (keyword, default :xy), :at [x y z],
         :eager, :hide

**Examples:**
```janet
(extrude-polygon [[0 0] [10 0] [10 10] [0 10]] 5)
(extrude-polygon [[0 0] [10 0] [5 10]] 5 :eager)
```

**Returns a rojcad/shape abstract value.**

### `revolve`

**Usage:** `(revolve shape &keys :a :ar :c :dir :eager :hide)`

Revolve a shape around an axis to create a solid.
Keywords: :a (angle in degrees), :ar (angle in radians),
         :c [x y z] (axis origin), :dir [dx dy dz] (axis direction),
         :eager, :hide

**Examples:**
```janet
(revolve shape :a 90)
(revolve shape :ar math/pi)
(revolve shape :c [0 0 0] :dir [0 1 0] :a 180)
```

**Returns a rojcad/shape abstract value.**

## Selection

### `on-select`

**Usage:** `(on-select callback)`

Register a function to be called on selection events.
Pass nil to unregister.

**Examples:**
```janet
(on-select (fn [s] (print "selected: " s)))
(on-select nil)  # unregister
```

**Returns nil.**

### `poll-selection`

**Usage:** `(poll-selection)`

This is called internally by the event loop.

**Examples:**
```janet
(poll-selection)  # returns shape, keyword, tuple, or nil
```

**Poll for a selection event. Returns a shape if one is selected,
:deselected if all deselected, [:deselected id] if a specific shape
was deselected, or nil if no event.**

## I/O

### `read-step`

**Usage:** `(read-step path &keys :eager :hide)`

Read a STEP file from disk and return a shape.

**Examples:**
```janet
(read-step "/tmp/model.step")       -- load from file
(read-step "/tmp/model.step" :eager) -- load and tessellate
(read-step "/tmp/model.step" :hide)  -- load without showing
```

**Returns a rojcad/shape abstract value. Signals an error on failure.**

### `write-step`

**Usage:** `(write-step path & shapes)`



**Examples:**
```janet
(write-step "/tmp/model.step")                          # all visible
(write-step "/tmp/model.step" my-shape)                  # single shape
(write-step "/tmp/model.step" box-a sphere-b cylinder-c) # multiple shapes
```

**Export one or more shapes to a STEP file at the given path.
With no shape arguments, exports all currently visible shapes.
Returns nil on success, signals an error on failure.**

### `write-stl`

**Usage:** `(write-stl shape path)`



**Examples:**
```janet
(write-stl my-shape "/tmp/model.stl")
```

**Export a shape to an STL file at the given path.
Returns nil on success, signals an error on failure.**

## View

### `projection-perspective`

**Usage:** `(projection-perspective &opt value)`

Get or set perspective projection mode.
Call with no arg to query, with true/false to set.

Example: (projection-perspective true)

### `projection-toggle`

**Usage:** `(projection-toggle)`

Toggle between orthographic and perspective projection.

Example: (projection-toggle)

### `quit-requested`

**Usage:** `(quit-requested)`



**Examples:**
```janet
(quit-requested)  # returns true or nil
```

**Returns boolean or nil.**

### `stats-overlay`

**Usage:** `(stats-overlay &opt value)`

Get or set the stats-for-nerds overlay.
Call with no arg to query, with true/false to toggle.

Example: (stats-overlay true)

### `view-angle`

**Usage:** `(view-angle yaw pitch &opt distance)`

Set the 3D viewport camera angle.

yaw (radians), pitch (radians), distance (optional, default 100).

**Examples:**
```janet
(view-angle math/pi 0)        # looking along -Z
(view-angle math/pi 0 200)    # further back
(view-angle 0 math/pi 2)      # top-down view
```

**Returns nil.**

### `view-back`

**Usage:** `(view-view-back ; distance)`

Set camera to back view (looking along -Z toward origin).
Yaw=-π/2, Pitch=0. Animates over 0.5s.
Optional distance sets zoom level; omitted preserves current.

**Examples:**
```janet
(view-back)
(view-back 200)
```

### `view-bottom`

**Usage:** `(view-view-bottom ; distance)`

Set camera to bottom view (looking along -Y toward origin).
Yaw=0, Pitch=-π/2. Animates over 0.5s.
Optional distance sets zoom level; omitted preserves current.

**Examples:**
```janet
(view-bottom)
(view-bottom 200)
```

### `view-fit`

**Usage:** `(view-fit & shapes)`

Fit the camera to frame one or more shapes.

**Examples:**
```janet
(view-fit my-shape)
(view-fit shape-a shape-b)
```

**Returns nil.**

### `view-fit-all`

**Usage:** `(view-fit-all &keys :hidden :reset)`

Fit the camera to frame all shapes in the scene.
Keywords: :hidden (include hidden shapes), :reset (reset orientation)

**Examples:**
```janet
(view-fit-all)
(view-fit-all :hidden)
```

**Returns nil.**

### `view-front`

**Usage:** `(view-view-front ; distance)`

Set camera to front view (looking along +Z toward origin).
Yaw=π/2, Pitch=0. Animates over 0.5s.
Optional distance sets zoom level; omitted preserves current.

**Examples:**
```janet
(view-front)
(view-front 200)
```

### `view-iso`

**Usage:** `(view-view-iso ; distance)`

Set camera to isometric view (looking from (1,1,1) direction).
Yaw=π/4, Pitch=asin(1/√3) ≈ 0.615 rad. Animates over 0.5s.
Optional distance sets zoom level; omitted preserves current.

**Examples:**
```janet
(view-iso)
(view-iso 150)
```

### `view-left`

**Usage:** `(view-view-left ; distance)`

Set camera to left view (looking along -X toward origin).
Yaw=π, Pitch=0. Animates over 0.5s.
Optional distance sets zoom level; omitted preserves current.

**Examples:**
```janet
(view-left)
(view-left 200)
```

### `view-right`

**Usage:** `(view-view-right ; distance)`

Set camera to right view (looking along +X toward origin).
Yaw=0, Pitch=0. Animates over 0.5s.
Optional distance sets zoom level; omitted preserves current.

**Examples:**
```janet
(view-right)
(view-right 200)
```

### `view-top`

**Usage:** `(view-view-top ; distance)`

Set camera to top view (looking along +Y toward origin).
Yaw=0, Pitch=π/2. Animates over 0.5s.
Optional distance sets zoom level; omitted preserves current.

**Examples:**
```janet
(view-top)
(view-top 200)
```

### `window-fullscreen`

**Usage:** `(window-fullscreen value)`

Set fullscreen mode. Pass true to enter, false to exit.

Example: (window-fullscreen true)

### `window-fullscreen?`

**Usage:** `(window-fullscreen?)`

Return true if the window is in fullscreen mode.

Example: (window-fullscreen?)

### `window-help-show`

**Usage:** `(window-help-show &opt value)`

Get or set help window visibility.
Call with no arg to query, with true/false to show/hide.

**Examples:**
```janet
(window-help-show true)
(window-help-show)   # query
```

### `window-help-show?`

**Usage:** `(window-help-show?)`

Return true if the help window is currently visible.

Example: (window-help-show?)

### `window-help-toggle`

**Usage:** `(window-help-toggle)`

Toggle the floating help window.

Example: (window-help-toggle)

### `window-maximized`

**Usage:** `(window-maximized value)`

Set maximized state. Pass true to maximize, false to restore.

Example: (window-maximized true)

### `window-maximized?`

**Usage:** `(window-maximized?)`

Return true if the window is maximized.

Example: (window-maximized?)

### `window-size`

**Usage:** `(window-size width height)`

Set the application window size in pixels.

Example: (window-size 1024 768)

### `window-size?`

**Usage:** `(window-size?)`

Get the current window size as [width height].

Example: (window-size?)

## Primitives

### `box`

**Usage:** `(box &keys :w :d :h :c :pl :ph :eager :hide)`

Create a box or cube.

Positional: (box w d h) or (box size) for a cube.
Keywords: :w :d :h (dimensions), :c (center [x y z]),
         :pl :ph (opposite corners [x y z]).
         :eager (tessellate immediately).
         :hide (skip automatic show on def).

**Examples:**
```janet
(box 10 20 30)           # box at origin
(box 10 20 30 :c [5 5 5]) # centered box
(box 5)                  # 5x5x5 cube
(box :pl [0 0 0] :ph [10 20 30]) # from corners
(box :w 10 :d 20 :h 30) # keyword style
(box 10 :eager)          # eager tessellation
(box 10 :hide)           # create without showing
```

**Returns a rojcad/shape abstract value.**

### `cone`

**Usage:** `(cone &keys :br :tr :h :c :a :ar :eager :hide)`

Create a cone or truncated cone.
Keywords: :br (base radius), :tr (top radius, default 0),
         :h (height), :c (center [x y z]),
         :a (angle in degrees), :ar (angle in radians),
         :eager, :hide

**Examples:**
```janet
(cone 5 10)                   # cone radius 5 height 10
(cone 5 2 10)                 # truncated cone
(cone :br 5 :tr 2 :h 10)      # keyword style
(cone 5 10 :eager)            # eager tessellation
```

**Returns a rojcad/shape abstract value.**

### `cylinder`

**Usage:** `(cylinder &keys :r :h :c :dir :fp :tp :eager :hide)`

Create a cylinder.

Positional: (cylinder radius height) - along Z axis, base at Z=0
Keywords: :r (radius), :h (height), :c (center [x y z]),
         :dir (direction [dx dy dz]),
         :fp (from-point [x y z]), :tp (to-point [x y z]).
         :eager (tessellate immediately).

**Examples:**
```janet
(cylinder 5 10)                       # simple
(cylinder 5 10 :c [0 0 5])            # centered
(cylinder :fp [0 0 0] :tp [0 0 10] :r 5) # point-to-point
(cylinder :r 5 :h 10)                 # keyword style
(cylinder 5 10 :eager)                # eager tessellation
```

**Returns a rojcad/shape abstract value.**

### `sphere`

**Usage:** `(sphere &keys :r :c :a :ar :eager :hide)`

Create a sphere.
Keywords: :r (radius), :c (center [x y z]),
         :a (angle in degrees), :ar (angle in radians),
         :eager, :hide

**Examples:**
```janet
(sphere 5)                    # radius 5 at origin
(sphere :r 5 :c [1 2 3])      # centered
(sphere 5 :a 90)              # hemisphere
(sphere 5 :eager)             # eager tessellation
```

**Returns a rojcad/shape abstract value.**

### `torus`

**Usage:** `(torus &keys :rr :tr :c :a :ar :as :asr :ae :aer :dir :eager :hide)`

Create a torus.

Positional: (torus rr tr)
Keywords: :rr (ring radius), :tr (tube radius),
         :c (center [x y z]),
         :a (angle in degrees), :ar (angle in radians, partial),
         :as (start angle degrees), :asr (start angle radians),
         :ae (end angle degrees), :aer (end angle radians),
         :dir (axis direction [dx dy dz]),
         :eager (tessellate immediately).

**Examples:**
```janet
(torus 20 10)                    # full torus
(torus 20 10 :c [0 0 5])         # repositioned
(torus 20 10 :a 180)             # half torus
(torus :rr 20 :tr 10 :as 0 :ae 180) # angled range
(torus :rr 20 :tr 10 :dir [0 1 0]) # oriented
(torus 20 10 :eager)             # eager tessellation
```

**Returns a rojcad/shape abstract value.**

## Parametric Models

### `build`

**Usage:** `(build model & params)`

Instantiate a parametric model by executing its body with the
given parameter values. Old shapes from previous builds are
purged automatically. Uses the existing `my-eval` shape-binding
mechanism for auto-purge on re-def.

Accepts a model record followed by positional parameter values
matching the model's :params vector.

**Examples:**
```janet
(def br (build bracket 100 50))
(def br2 (build bracket 200 80))
```

**Returns a rojcad/shape abstract value. Signals an error on
parameter count mismatch or model execution failure.**

### `defmodel`

**Usage:** `(defmodel name [params...] &keys :parts :result body...)`

Define a parametric model. Binds a model record with :params, :body-fn, :source, :parts, :shapes, :shape-map, :current-params, and :result fields.

Keywords: :parts (table of named parts {:part-name expr ...}),
         :result (result expression, defaults to last body expression)

When :parts is provided, a `parts` local variable is bound to the
table of built shapes. The model's :result expression produces the
final shape from these parts.

Model records are pure Janet tables — they can be inspected,
passed to `build`, and introspected with `graph`.

**Examples:**
```janet
(defmodel bracket [w h]
:parts {:base (box w h 30) :hole (cylinder 5 30)}
:result (cut base hole))
(defmodel cube [s]
(box s s s))
```

**Returns nil (binds a model record as a side effect).**

### `graph`

**Usage:** `(graph model)`

Return the structure of a parametric model as a table.
The returned table has :name, :params, :current, :nodes,
and :shape-map fields. The :nodes array contains source-form
AST nodes with :type, :children, :form, and :id fields.
Each node in a built model maps to its shape via :shape-map.

**Examples:**
```janet
(graph bracket)
```

**Returns a table.**

### `highlight`

**Usage:** `(highlight model &opt part-id)`

Highlight a named part of a built model in the viewer.
The shape is shown (registered in the viewer) and rendered
with active edges and a tinted mesh overlay.
Without part-id, the entire result shape is highlighted.

Part-ids correspond to keys in the model's :parts table
or :result for the final output shape.

**Examples:**
```janet
(highlight bracket :base)
(highlight bracket)
```

**Returns nil. Signals an error if the model has not been built.**

### `highlight-clear`

**Usage:** `(highlight-clear &opt model part-id)`

Remove highlighting and hide previously highlighted shapes.

Call variants:
  (highlight-clear)                  — clear viewer highlight only
  (highlight-clear model)            — hide all highlighted parts
  (highlight-clear model :part-name) — hide a specific part

**Examples:**
```janet
(highlight-clear)
(highlight-clear bracket)
(highlight-clear bracket :base)
```

**Returns nil.**

## Sketch

### `arc-to`

**Usage:** `(arc-to sketch x2 y2 x3 y3)`



**Draw a circular arc through (x2, y2) to (x3, y3).
Returns a new sketch.**

### `build-wire`

**Usage:** `(build-wire sketch &keys :eager :hide)`

Return the sketch as an unclosed Wire.
Keywords: :eager, :hide

**Examples:**
```janet
(-> (sketch) (line-to 10 0) (line-to 10 10) (build-wire))
```

### `close-sketch`

**Usage:** `(close-sketch sketch &keys :eager :hide)`

Close the sketch and return a Face.
Keywords: :eager, :hide

**Examples:**
```janet
(-> (sketch) (line-to 10 0) (line-to 10 10) (close-sketch))
```

### `line-dx`

**Usage:** `(line-dx sketch dx)`



**Draw a horizontal line by dx units.
Returns a new sketch.**

### `line-dx-dy`

**Usage:** `(line-dx-dy sketch dx dy)`



**Draw a line by (dx, dy) offset.
Returns a new sketch.**

### `line-dy`

**Usage:** `(line-dy sketch dy)`



**Draw a vertical line by dy units.
Returns a new sketch.**

### `line-to`

**Usage:** `(line-to sketch x y)`



**Draw a line from current cursor to (x, y).
Returns a new sketch.**

### `move-to`

**Usage:** `(move-to sketch x y)`



**Move the sketch cursor to (x, y) without drawing.
Returns a new sketch.**

### `sketch`

**Usage:** `(sketch &keys :plane :at)`



**Examples:**
```janet
(sketch)
(sketch :plane :xz :at [10 0 5])
```

**Create a new sketch on a workplane.
Keywords: :plane (keyword, default :xy), :at (array [x y z]).
Returns a rojcad/sketch abstract value.**

## 2D Primitives

### `circle`

**Usage:** `(circle &keys :r :wire :plane :at :eager :hide)`

Create a circle.
Keywords: :r (radius), :wire (output as wire),
         :plane (keyword, default :xy), :at [x y z],
         :eager, :hide

**Examples:**
```janet
(circle 5)
(circle :r 5 :plane :xz)
(circle 5 :wire)
```

**Returns a rojcad/shape abstract value.**

### `polygon`

**Usage:** `(polygon &keys :pts :wire :plane :at :eager :hide)`

Create a polygon from a list of 2D points.
Keywords: :pts (array of [x y] points), :wire (output as wire),
         :plane (keyword, default :xy), :at [x y z],
         :eager, :hide

**Examples:**
```janet
(polygon :pts [[0 0] [10 0] [5 10]])
(polygon :pts [[0 0] [10 0] [10 10] [0 10]] :wire)
```

**Returns a rojcad/shape abstract value.**

### `rect`

**Usage:** `(rect &keys :w :d :h :wire :plane :at :eager :hide)`

Create a rectangle.
Keywords: :w (width), :d/:h (depth), :wire (output as wire),
         :plane (keyword, default :xy), :at [x y z],
         :eager, :hide

**Examples:**
```janet
(rect 10 20)
(rect :w 10 :d 20 :plane :xz)
(rect 10 20 :wire)  # as wireframe
```

**Returns a rojcad/shape abstract value.**

## Text

### `list-fonts`

**Usage:** `(list-fonts)`



**Examples:**
```janet
(list-fonts)  # returns @[["Arial" "/path/Arial.ttf" :regular] ...]
```

**List available system fonts.
Returns an array of [name type style] tuples,
where type is the font file path and style is a keyword.**

### `text`

**Usage:** `(text str font size &keys :depth :both :plane :at :eager :hide)`

Create 2D text from a TrueType/OpenType font.
:font is a font name string, :size is the font size.
Keywords: :depth, :both (extrude both sides),
         :plane (keyword), :at [x y z],
         :eager, :hide

**Examples:**
```janet
(text "Hello" "Arial" 10)
(text "Hello" "Arial" 10 :depth 5)
```

**Returns a rojcad/shape abstract value.**

### `text3d`

**Usage:** `(text3d str font size depth &keys :both :plane :at :eager :hide)`

Create 3D text (extruded) from a TrueType/OpenType font.
:font is a font name string, :size is the font size.
Keywords: :both (extrude both sides),
         :plane (keyword), :at [x y z],
         :eager, :hide

**Examples:**
```janet
(text3d "Hello" "Arial" 10 5)
```

**Returns a rojcad/shape abstract value.**

## Registry

### `hide`

**Usage:** `(hide & shapes)`

Set shapes' visible flag to false.
Shapes stay registered but are no longer rendered.

**Examples:**
```janet
(hide my-shape)
(hide shape-a shape-b)
```

**Returns nil.**

### `purge`

**Usage:** `(purge & shapes)`

Remove shapes from the viewer registry and mark them as purged.
They will no longer be rendered. Use (def name nil) to unbind.

**Examples:**
```janet
(purge my-shape)
```

**Returns nil.**

### `registry-remove`

**Usage:** `(registry-remove & shapes)`

Immediately remove shapes from the viewer registry.
Used internally by `purge`. The underlying OCCT shape memory
is freed when Janet's GC collects the shape value.

**Returns nil.**

### `show`

**Usage:** `(show & shapes)`

Register shapes in the viewer and make them visible.
Calling show on an already-visible shape is a no-op.

**Examples:**
```janet
(show my-shape)
(show shape-a shape-b)
```

**Returns nil.**

## Wire Operations

### `wire-chamfer`

**Usage:** `(wire-chamfer wire &keys :d :eager :hide)`

Bevel all vertices of a closed Wire by distance :d.
Keywords: :d (required), :eager, :hide

**Examples:**
```janet
(wire-chamfer my-wire :d 2)
```

### `wire-fillet`

**Usage:** `(wire-fillet wire &keys :r :eager :hide)`

Round all vertices of a closed Wire by radius :r.
Keywords: :r (required), :eager, :hide

**Examples:**
```janet
(wire-fillet my-wire :r 2)
```

### `wire-offset`

**Usage:** `(wire-offset wire &keys :d :eager :hide)`

Create a parallel offset of a closed Wire by distance :d.
Keywords: :d (required), :eager, :hide

**Examples:**
```janet
(wire-offset my-wire :d 2)
```

### `wire-to-face`

**Usage:** `(wire-to-face wire &keys :eager :hide)`

Convert a Wire into a Face by filling its boundary.
Keywords: :eager, :hide

**Examples:**
```janet
(wire-to-face my-wire)
```

## Booleans

### `common`

**Usage:** `(common first & keys shapes ; :eager ; :hide)`

Intersect shapes (boolean common / intersection).
Keywords: :eager, :hide

**Examples:**
```janet
(common box sphere)
(common box sphere :eager)
```

**Returns a rojcad/shape abstract value.**

### `cut`

**Usage:** `(cut tool & keys shapes ; :eager ; :hide)`

Subtract shapes from a tool shape (boolean cut).
Keywords: :eager, :hide

**Examples:**
```janet
(cut box sphere)
(cut box sphere :eager)
```

**Returns a rojcad/shape abstract value.**

### `fuse`

**Usage:** `(fuse first & keys shapes ; :eager ; :hide)`

Combine shapes (boolean fuse / union).
Keywords: :eager, :hide

**Examples:**
```janet
(fuse box sphere)
(fuse box sphere :eager)
```

**Returns a rojcad/shape abstract value.**

## Queries

### `face?`

**Usage:** `(face? & shapes)`

Check if one or more shapes are Faces.

**Examples:**
```janet
(face? my-shape)  # returns true or false
```

**Returns boolean or array of booleans.**

### `list-shapes`

**Usage:** `(list-shapes &keys :visible :hidden)`

List registered shapes, optionally filtered by visibility.
Keywords: :visible (show only visible shapes),
         :hidden (show only hidden shapes).

**Examples:**
```janet
(list-shapes)              # all registered shapes
(list-shapes :visible)     # only visible shapes
(list-shapes :hidden)      # only hidden shapes
```

**Returns an array of rojcad/shape values.**

### `selected-shapes`

**Usage:** `(selected-shapes)`

Return an array of currently selected shapes in the viewer.

**Examples:**
```janet
(selected-shapes)  # returns @[shape ...]
```

**Returns an array of rojcad/shape values.**

### `shape-type`

**Usage:** `(shape-type & shapes)`



**Examples:**
```janet
(shape-type my-shape)      # returns :solid
(shape-type shape-a shape-b) # returns @[:solid :face]
```

**Returns a keyword or array of keywords.**

### `solid?`

**Usage:** `(solid? & shapes)`

Check if one or more shapes are Solids.

**Examples:**
```janet
(solid? my-shape)  # returns true or false
```

**Returns boolean or array of booleans.**

### `visible?`

**Usage:** `(visible? & shapes)`

Check if one or more shapes are visible.

**Examples:**
```janet
(visible? my-shape)  # returns true or false
```

**Returns boolean or array of booleans.**

### `wire?`

**Usage:** `(wire? & shapes)`

Check if one or more shapes are Wires.

**Examples:**
```janet
(wire? my-shape)  # returns true or false
```

**Returns boolean or array of booleans.**

## Edge Styling

### `edge-active-show?`

**Usage:** `(edge-active-show?)`

Return true if edges on the selected shape are visible.

Example: (edge-active-show?)

### `edge-color-active`

**Usage:** `(edge-color-active &opt r g b)`

Get or set the color of edges on the selected shape.
RGB values in 0-1 range. Call with no args to query.

**Examples:**
```janet
(edge-color-active 1 0 0)   # red active edges
(edge-color-active)         # get current color
```

**Returns [r g b] or nil.**

### `edge-color-inactive`

**Usage:** `(edge-color-inactive &opt r g b)`

Get or set the color of edges on non-selected shapes.
RGB values in 0-1 range. Call with no args to query.

**Examples:**
```janet
(edge-color-inactive 0.5 0.5 0.5)  # grey inactive edges
(edge-color-inactive)              # get current color
```

**Returns [r g b] or nil.**

### `edge-hidden`

**Usage:** `(edge-hidden &opt value)`

Get or set hidden edge visibility.
Call with no arg to query, with true/false to set.

**Examples:**
```janet
(edge-hidden true)   # show hidden edges
(edge-hidden)        # query
```

**Returns boolean.**

### `edge-hidden-show?`

**Usage:** `(edge-hidden-show?)`

Return true if hidden edges are visible.

Example: (edge-hidden-show?)

### `edge-hidden-toggle`

**Usage:** `(edge-hidden-toggle)`

Toggle visibility of hidden edges (edges occluded by the shape).

Example: (edge-hidden-toggle)

### `edge-inactive-show?`

**Usage:** `(edge-inactive-show?)`

Return true if edges on non-selected shapes are visible.

Example: (edge-inactive-show?)

### `edge-thickness`

**Usage:** `(edge-thickness &opt value)`

Get or set the edge line thickness.
Call with no arguments to query current thickness.

**Examples:**
```janet
(edge-thickness 2)  # set thickness
(edge-thickness)    # get current thickness
```

**Returns the thickness value.**

### `edge-toggle-active`

**Usage:** `(edge-toggle-active)`

Example: (edge-toggle-active)

**Toggle visibility of edges on the selected shape.
Returns true if active edges are now visible, false if hidden.**

### `edge-toggle-inactive`

**Usage:** `(edge-toggle-inactive)`

Example: (edge-toggle-inactive)

**Toggle visibility of edges on non-selected shapes.
Returns true if inactive edges are now visible, false if hidden.**

## Transforms

### `mirror`

**Usage:** `(mirror shape ox oy oz dx dy dz &keys :eager :hide)`

Mirror a shape across a plane defined by a point and normal.
:ox oy oz is a point on the plane.
:dx dy dz is the plane normal.
Keywords: :eager, :hide

**Examples:**
```janet
(mirror box 0 0 0 1 0 0)  # mirror across YZ plane
```

**Returns a rojcad/shape abstract value.**

### `rotate`

**Usage:** `(rotate shape &keys :a :ar :x :y :z :r :eager :hide)`

Rotate a shape around an axis.
Keywords: :a (angle in degrees), :ar (angle in radians),
         :x/:y/:z (axis shortcuts),
         :r [dx dy dz] (rotation axis vector),
         :eager, :hide

**Examples:**
```janet
(rotate box :z :a 90)
(rotate box :x :ar math/pi)
(rotate box :r [0 1 0] :a 45)
```

**Returns a rojcad/shape abstract value.**

### `scale`

**Usage:** `(scale shape factor &keys :o :eager :hide)`

Uniformly scale a shape by a factor.
Keywords: :o [x y z] (origin for scaling),
         :eager, :hide

**Examples:**
```janet
(scale box 2)                # double size
(scale box 2 :o [5 5 5])     # scale from center
```

**Returns a rojcad/shape abstract value.**

### `translate`

**Usage:** `(translate shape dx dy dz &keys :t :eager :hide)`

Translate (move) a shape by a vector.
Positional: (translate shape dx dy dz)
Keywords: :t [dx dy dz], :eager, :hide

**Examples:**
```janet
(translate box 10 0 0)
(translate box :t [10 0 0])
```

**Returns a rojcad/shape abstract value.**

## Other

### `color`

**Usage:** `(color shape r g b)`



**Examples:**
```janet
(color my-shape 1 0 0)    # red
(-> (box 10) (color 0 1 0))  # green box
```

**Returns the shape.**

### `compound`

**Usage:** `(compound & shapes &keys :color :eager :hide)`

Combine multiple shapes into a single compound shape.
Keywords: :color [r g b], :eager, :hide

**Examples:**
```janet
(compound sphere cone)
(compound box sphere :color [1 0 0])
```

**Returns a rojcad/shape abstract value.**

### `get-color`

**Usage:** `(get-color shape)`

Get a shape's render color as an array [r g b], or nil if unset.

**Examples:**
```janet
(get-color my-shape)  # returns [r g b] or nil
```

**Returns an array or nil.**

