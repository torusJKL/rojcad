# rojcad Janet API Reference — 94b0b3f-dirty

## Operations

### `extrude`

**Usage:** `(extrude shape &keys :h :z :x :y :dir :both :eager :hide)`

Extrude a Face into a Solid.

Keywords: :h (height, required), :z/:x/:y (cardinal axis),
         :dir [dx dy dz] (custom direction),
         :both (extrude both sides),
         :eager (tessellate immediately), :hide (skip auto-show).

Default direction is the face normal.

**Examples:**
```janet
(extrude face :h 20)               — along face normal
(extrude face :h 20 :z)            — along Z axis
(extrude face :h 10 :both)         — both sides
(extrude face :h 5 :dir [0 0 -1])  — custom direction
```

**Returns a rojcad/shape abstract value (SOLID).**

### `extrude-polygon`

**Usage:** `(extrude-polygon points height &keys :h :plane :at :eager :hide)`

Create a Solid by extruding a polygon from points.

Positional: (extrude-polygon points height)
Points is an array of [x y] tuples.
Keywords: :h (height), :plane (workplane, default :xy),
         :at (position [x y z]),
         :eager (tessellate immediately), :hide (skip auto-show).

**Examples:**
```janet
(extrude-polygon [[0 0][10 0][10 10][0 10]] 20)
(extrude-polygon [[0 0][10 0][10 10]] :h 5)
```

**Returns a rojcad/shape abstract value (SOLID).**

### `revolve`

**Usage:** `(revolve shape &keys :a :ar :c :dir :eager :hide)`

Revolve a Face into a Solid.

Angle via :a (degrees) or :ar (radians).
Axis via :c (point [x y z], default [0 0 0]) and :dir (direction, default [0 0 1]).
Keywords: :eager (tessellate immediately), :hide (skip auto-show).

**Examples:**
```janet
(revolve face :a 360)                     — full revolution about Z
(revolve face :a 180)                     — half revolution
(revolve face :a 180 :c [0 0 0] :dir [0 1 0]) — about Y axis
```

**Returns a rojcad/shape abstract value (SOLID).**

## Selection

### `on-select`

**Usage:** `(on-select callback)`

Register a Janet function to be called when a shape is selected in the viewer. The function receives the same value as (poll-selection): a shape ID (integer), :deselected, or [:deselected id]. Pass nil to unregister the callback.

Example:
  (on-select (fn [event] (print "selection: " event)))

### `poll-selection`

**Usage:** `(poll-selection)`

Check for a pending selection event from the viewer.

If a callback was registered via (on-select), it will be invoked automatically with the result.

**Returns nil if no event, the shape ID (integer) if a shape was selected, a tuple [:deselected id] if a shape was toggled off,
 or :deselected if the entire selection was cleared.**

## I/O

### `read-step`

**Usage:** `(read-step path &keys :eager :hide)`

Read a STEP file from disk and return a shape.

Example:
  (read-step "/tmp/model.step")       — load from file
  (read-step "/tmp/model.step" :eager) — load and tessellate

**Returns a rojcad/shape abstract value. Signals an error on failure.**

### `write-step`

**Usage:** `(write-step shape path)`



**Export a shape to a STEP file at the given path. Returns nil on success, signals an error on failure.**

### `write-stl`

**Usage:** `(write-stl shape path)`



**Export a shape to an STL file at the given path. Returns nil on success, signals an error on failure.**

## View

### `projection-perspective`

**Usage:** `(projection-perspective &opt value)`

Get or set the camera projection mode.

Called with no arguments, returns true if perspective mode is active, false if orthographic.
Called with one boolean argument, sets the mode.

Example: (projection-perspective)        — query
         (projection-perspective true)    — perspective
         (projection-perspective false)   — orthographic

### `projection-toggle`

**Usage:** `(projection-toggle)`

Example: (projection-toggle)

**Toggle camera projection between perspective and orthographic. Returns true if now in perspective mode, false if orthographic.**

### `quit-requested`

**Usage:** `(quit-requested)`

Check if the application should quit.

This is a one-shot check -- returns true only once per quit request.

**Returns true if Ctrl+Q was pressed or the window was closed.
Used by boot.janet to exit the event loop.**

### `stats-overlay`

**Usage:** `(stats-overlay &opt value)`

Get or set the stats overlay visibility.

Called with no arguments, returns true if the overlay is visible, false if hidden.
Called with one boolean argument, sets the visibility.

Example: (stats-overlay)        — query
         (stats-overlay true)    — show overlay
         (stats-overlay false)   — hide overlay

The overlay can also be toggled with Ctrl+Shift+Alt+S in the viewer window.

### `view-angle`

**Usage:** `(view-angle yaw pitch ; distance)`

Set camera to arbitrary yaw/pitch angles (radians).

Animates the 3D camera over 0.5s to the given orientation.
Yaw and pitch are in radians. An optional third argument sets
the camera distance (zoom); omitted preserves the current distance.

**Examples:**
```janet
(view-angle 0 1.57)      — top view (yaw=0, pitch=~90°)
(view-angle 0 0 100)     — look along +X at distance 100
```

### `view-back`

**Usage:** `(view-back ; distance)`

Set camera to back view (looking along -Z toward origin).
Yaw=-π/2, Pitch=0. Animates over 0.5s.
Optional distance sets zoom level; omitted preserves current.

**Examples:**
```janet
(view-back)
(view-back 200)
```

### `view-bottom`

**Usage:** `(view-bottom ; distance)`

Set camera to bottom view (looking along -Y toward origin).
Yaw=0, Pitch=-π/2. Animates over 0.5s.
Optional distance sets zoom level; omitted preserves current.

**Examples:**
```janet
(view-bottom)
(view-bottom 200)
```

### `view-fit`

**Usage:** `(view-fit shape & shapes ; reset)`

Fit camera to the bounding box of one or more shapes.

Animates the 3D camera over 0.5s to frame the union bounding
box of the given shapes. The current orbit angle is preserved.

Use :reset to return to the default isometric angle
(yaw=0, pitch=0.4).

**Examples:**
```janet
(view-fit my-shape)
(view-fit box1 cylinder2)
(view-fit :reset part1 part2)
```

### `view-fit-all`

**Usage:** `(view-fit-all ; hidden ; reset)`

Fit camera to the bounding box of shapes.

By default only visible shapes are framed. Use :hidden to include hidden shapes as well.
Animates the 3D camera over 0.5s to frame the union bounding box.
The current orbit angle is preserved.
If no shapes are found, resets the camera to default position.

Keywords:
  :hidden  — include hidden shapes in the bounding box
  :reset   — return to the default isometric angle

**Examples:**
```janet
(view-fit-all)
(view-fit-all :reset)
(view-fit-all :hidden)
(view-fit-all :hidden :reset)
```

### `view-front`

**Usage:** `(view-front ; distance)`

Set camera to front view (looking along +Z toward origin).
Yaw=π/2, Pitch=0. Animates over 0.5s.
Optional distance sets zoom level; omitted preserves current.

**Examples:**
```janet
(view-front)
(view-front 200)
```

### `view-iso`

**Usage:** `(view-iso ; distance)`

Set camera to isometric view (looking from (1,1,1) direction).
Yaw=π/4, Pitch=asin(1/√3) ≈ 0.615 rad. Animates over 0.5s.
Optional distance sets zoom level; omitted preserves current.

**Examples:**
```janet
(view-iso)
(view-iso 150)
```

### `view-left`

**Usage:** `(view-left ; distance)`

Set camera to left view (looking along -X toward origin).
Yaw=π, Pitch=0. Animates over 0.5s.
Optional distance sets zoom level; omitted preserves current.

**Examples:**
```janet
(view-left)
(view-left 200)
```

### `view-right`

**Usage:** `(view-right ; distance)`

Set camera to right view (looking along +X toward origin).
Yaw=0, Pitch=0. Animates over 0.5s.
Optional distance sets zoom level; omitted preserves current.

**Examples:**
```janet
(view-right)
(view-right 200)
```

### `view-top`

**Usage:** `(view-top ; distance)`

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

Enter or exit fullscreen mode.

Pass true to enter borderless fullscreen, false to return to windowed mode.

**Examples:**
```janet
(window-fullscreen true)   ; enter fullscreen
(window-fullscreen false)  ; exit fullscreen
```

### `window-fullscreen?`

**Usage:** `(window-fullscreen?)`

Return true if the viewer is in fullscreen mode, false otherwise.

Example:
  (window-fullscreen?)   ; returns true or false

### `window-help-show`

**Usage:** `(window-help-show &opt value)`

Get or set the help window visibility.

Called with no arguments, returns true if visible, false if hidden.
Called with one boolean argument, sets the visibility.

Example: (window-help-show)        — query
         (window-help-show true)    — show
         (window-help-show false)   — hide

### `window-help-show?`

**Usage:** `(window-help-show?)`

Return true if the help window is currently visible, false if hidden.

Example: (window-help-show?)

### `window-help-toggle`

**Usage:** `(window-help-toggle)`

Example: (window-help-toggle)

**Toggle the help window visibility. Returns true if now visible, false if hidden.**

### `window-maximized`

**Usage:** `(window-maximized value)`

Enter or exit maximized state.

Pass true to maximize, false to restore to windowed.

**Examples:**
```janet
(window-maximized true)   ; maximize
(window-maximized false)  ; restore
```

### `window-maximized?`

**Usage:** `(window-maximized?)`

Return true if the viewer window is maximized, false otherwise.

Example:
  (window-maximized?)   ; returns true or false

### `window-size`

**Usage:** `(window-size width height)`

Resize the viewer window to the given logical pixel dimensions.

Both width and height must be positive integers.

Example:
  (window-size 800 600)    ; resize to 800x600

### `window-size?`

**Usage:** `(window-size?)`

Return the current viewer window dimensions as a tuple [width height] in logical pixels.

Example:
  (window-size?)    ; e.g., returns [1024 768]

## Primitives

### `box`

**Usage:** `(box width depth height &keys :w :d :h :c :pl :ph :eager :hide)`

Create a box or cube.

Positional: (box w d h) or (box size) for a cube.
Keywords: :w :d :h (dimensions), :c (center [x y z]),
         :pl :ph (opposite corners [x y z]).
         :eager (tessellate immediately).
         :hide (skip automatic show on def).

**Examples:**
```janet
(box 10 20 30)           — box at origin
(box 10 20 30 :c [5 5 5]) — centered box
(box 5)                  — 5x5x5 cube
(box :pl [0 0 0] :ph [10 20 30]) — from corners
(box :w 10 :d 20 :h 30) — keyword style
(box 10 :eager)          — eager tessellation
(box 10 :hide)           — create without showing
```

**Returns a rojcad/shape abstract value.**

### `cone`

**Usage:** `(cone bottom-radius height &keys :br :tr :h :c :a :ar :eager)`

Create a cone or truncated cone.

Positional: (cone br h) for full cone, (cone br tr h) for truncated.
Keywords: :br (bottom radius), :tr (top radius), :h (height),
         :c (center [x y z]),
         :a (angle in degrees), :ar (angle in radians, partial cone),
         :eager (tessellate immediately).

**Examples:**
```janet
(cone 5 10)                — full cone, br=5, h=10
(cone 5 3 10)              — truncated cone
(cone 5 10 :a 180)         — half cone
(cone :br 5 :h 10)         — keyword style
(cone 5 10 :eager)         — eager tessellation
```

**Returns a rojcad/shape abstract value.**

### `cylinder`

**Usage:** `(cylinder radius height &keys :r :h :c :dir :fp :tp :eager)`

Create a cylinder.

Positional: (cylinder radius height) — along Z axis, base at Z=0
Keywords: :r (radius), :h (height), :c (center [x y z]),
         :dir (direction [dx dy dz]),
         :fp (from-point [x y z]), :tp (to-point [x y z]).
         :eager (tessellate immediately).

**Examples:**
```janet
(cylinder 5 10)                       — simple
(cylinder 5 10 :c [0 0 5])            — centered
(cylinder :fp [0 0 0] :tp [0 0 10] :r 5) — point-to-point
(cylinder :r 5 :h 10)                 — keyword style
(cylinder 5 10 :eager)                — eager tessellation
```

**Returns a rojcad/shape abstract value.**

### `sphere`

**Usage:** `(sphere radius &keys :r :c :a :ar :eager)`

Create a sphere.

Positional: (sphere radius)
Keywords: :r (radius), :c (center [x y z]),
         :a (angle in degrees), :ar (angle in radians),
         :eager (tessellate immediately).

**Examples:**
```janet
(sphere 10)               — full sphere at origin
(sphere 10 :c [1 2 3])    — repositioned
(sphere 10 :a 180)        — hemisphere
(sphere :r 10)            — keyword style
(sphere 10 :eager)        — eager tessellation
```

**Returns a rojcad/shape abstract value.**

### `torus`

**Usage:** `(torus ring-radius tube-radius &keys :rr :tr :c :a :ar :as :asr :ae :aer :dir :eager)`

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
(torus 20 10)                    — full torus
(torus 20 10 :c [0 0 5])         — repositioned
(torus 20 10 :a 180)             — half torus
(torus :rr 20 :tr 10 :as 0 :ae 180) — angled range
(torus :rr 20 :tr 10 :dir [0 1 0]) — oriented
(torus 20 10 :eager)             — eager tessellation
```

**Returns a rojcad/shape abstract value.**

## Sketch

### `arc-to`

**Usage:** `(arc-to sketch x2 y2 x3 y3)`



**Draw a circular arc from current cursor through (x2, y2) to (x3, y3). Returns a new sketch.**

### `build-wire`

**Usage:** `(build-wire sketch &keys :eager :hide)`

Return the sketch as an unclosed Wire. Does not close the loop.

Keywords: :eager, :hide

**Returns a rojcad/shape abstract value (WIRE).**

### `close-sketch`

**Usage:** `(close-sketch sketch &keys :eager :hide)`

Close the sketch and return a Face. Adds a closing edge if needed.

Keywords: :eager, :hide

**Returns a rojcad/shape abstract value (FACE).**

### `line-dx`

**Usage:** `(line-dx sketch dx)`



**Draw a horizontal line by dx units. Returns a new sketch.**

### `line-dx-dy`

**Usage:** `(line-dx-dy sketch dx dy)`



**Draw a line by (dx, dy) offset. Returns a new sketch.**

### `line-dy`

**Usage:** `(line-dy sketch dy)`



**Draw a vertical line by dy units. Returns a new sketch.**

### `line-to`

**Usage:** `(line-to sketch x y)`



**Draw a line from the current cursor to (x, y). Returns a new sketch.**

### `move-to`

**Usage:** `(move-to sketch x y)`



**Move the sketch cursor to (x, y) without drawing. Returns a new sketch.**

### `sketch`

**Usage:** `(sketch &keys :plane :at)`

Create a new sketch on a workplane.

Keywords: :plane (workplane, default :xy), :at (position [x y z]).

**Examples:**
```janet
(sketch)                              — XY plane at origin
(sketch :plane :xz :at [10 0 5])      — XZ plane at [10, 0, 5]
Combine with -> for threading:
(-> (sketch) (line-to 10 0) (line-to 10 10) (close-sketch))
```

**Returns a rojcad/sketch abstract value. Each sketch operation returns
a new sketch — no mutation.**

## 2D Primitives

### `circle`

**Usage:** `(circle radius &keys :r :wire :plane :at :eager :hide)`

Create a circle.

Positional: (circle radius)
Keywords: :r (radius), :wire (return Wire instead of Face),
         :plane (workplane, default :xy), :at (position [x y z]),
         :eager (tessellate immediately), :hide (skip auto-show).

**Examples:**
```janet
(circle 5)                       — on XY plane
(circle :r 5 :wire)              — circle wire
(circle :r 5 :plane :xz)         — on XZ plane
```

**Returns a rojcad/shape abstract value.**

### `polygon`

**Usage:** `(polygon &keys :pts :wire :plane :at :eager :hide)`

Create a polygon from a list of 2D points.

Keywords: :pts (array of [x y] tuples), :wire (return Wire instead of Face),
         :plane (workplane, default :xy), :at (position [x y z]),
         :eager (tessellate immediately), :hide (skip auto-show).

**Examples:**
```janet
(polygon :pts [[0 0] [10 0] [10 10] [0 10]])  — square on XY
(polygon :pts [[0 0] [10 0] [10 10]] :wire)    — L-shaped wire
```

**Returns a rojcad/shape abstract value.**

### `rect`

**Usage:** `(rect width depth &keys :w :d :h :wire :plane :at :eager :hide)`

Create a rectangle.

Positional: (rect w d)
Keywords: :w :d or :h (dimensions), :wire (return Wire instead of Face),
         :plane (workplane, default :xy), :at (position [x y z]),
         :eager (tessellate immediately), :hide (skip auto-show).

**Examples:**
```janet
(rect 10 20)                     — on XY plane
(rect :w 10 :d 20 :wire)         — rect wire
(rect :w 10 :h 20)               — :h alias for :d
(rect :w 10 :d 20 :plane :xz :at [5 0 0]) — on XZ plane
```

**Returns a rojcad/shape abstract value (FACE by default, WIRE with :wire).**

## Text

### `list-fonts`

**Usage:** `(list-fonts)`

List available system fonts.

Scans standard OS font directories and returns an array of
[name path aspect] tuples for each discovered TTF/OTF font.
Aspect is :regular, :bold, :italic, or :bold-italic.

**Examples:**
```janet
(list-fonts)  — all system fonts
(keep [name path] (list-fonts)) — just names and paths
```

**Returns an array of tuples.**

### `text`

**Usage:** `(text string font-path size &keys :depth :plane :at :eager :hide)`

Create a 2D or 3D text shape from a TrueType/OpenType font.

Positional: (text "Hello" "DejaVuSans.ttf" 10)
Keywords: :depth (extrude 3D text), :plane ("xy"/"xz"/"yz"...),
         :at (position [x y z]), :eager, :hide.

Without :depth returns a Face. With :depth returns an extruded Solid.

**Examples:**
```janet
(text "Hi" "font.ttf" 10)              — 2D text face
(text "Hi" "font.ttf" 10 :depth 5)     — 3D extruded text
(text "Hi" "font.ttf" 10 :plane "xz") — on XZ plane
(text "Hi" "font.ttf" 10 :at [5 0 0])  — positioned
```

**Returns a rojcad/shape abstract value.**

### `text3d`

**Usage:** `(text3d string font-path size depth &keys :plane :at :both :eager :hide)`

Create a 3D extruded text shape.

Positional: (text3d "Hello" "DejaVuSans.ttf" 10 5)
Keywords: :plane ("xy"/"xz"/"yz"...), :at (position [x y z]),
         :both (extrude symetrically), :eager, :hide.

Equivalent to (text ... :depth depth).

**Returns a rojcad/shape abstract value.**

## Registry

### `hide`

**Usage:** `(hide shape)`

Set a shape's visible flag to false. The shape stays registered in the viewer but is no longer rendered.

**Examples:**
```janet
(hide b)         — shape disappears from viewer
(show b)         — reappears without re-tessellating
```

**Returns nil.**

### `purge`

**Usage:** `(purge shape)`

Remove a shape from the viewer registry and mark it as purged.
The shape will no longer be rendered. To also unbind the Janet variable,
use (purge shape) followed by (def name nil).

**Examples:**
```janet
(purge b)          — remove b from viewer
(purge b) (def b nil) (gc)  — full cleanup
```

**Returns nil.**

### `registry-remove`

**Usage:** `(registry-remove shape)`

Immediately remove a shape from the viewer registry and mark it as purged.
The shape will no longer be rendered. The underlying OCCT shape memory
is freed when Janet's GC collects the shape value.

This is used internally by the `purge` macro.

**Returns nil.**

### `show`

**Usage:** `(show shape)`

Register a shape in the viewer and make it visible.

If the shape has not been tessellated, tessellation happens automatically.
Calling show on an already-visible shape is a no-op.

**Examples:**
```janet
(def b (box 10))
(show b)         — tessellates if needed, registers, makes visible
(show b)         — second call is a no-op (already visible)
```

**Returns nil.**

## Wire Operations

### `wire-chamfer`

**Usage:** `(wire-chamfer wire &keys :d :eager :hide)`

Bevel all vertices of a closed Wire by a distance.

Keywords: :d (distance, required), :eager, :hide

**Returns a rojcad/shape abstract value (WIRE).**

### `wire-fillet`

**Usage:** `(wire-fillet wire &keys :r :eager :hide)`

Round all vertices of a closed Wire by a radius.

Keywords: :r (radius, required), :eager, :hide

**Returns a rojcad/shape abstract value (WIRE).**

### `wire-offset`

**Usage:** `(wire-offset wire &keys :d :eager :hide)`

Create a parallel offset of a closed Wire by a distance.

Keywords: :d (distance, required), :eager, :hide

**Returns a rojcad/shape abstract value (WIRE).**

### `wire-to-face`

**Usage:** `(wire-to-face wire &keys :eager :hide)`

Convert a Wire shape into a Face by filling its boundary.

Keywords: :eager, :hide

**Returns a rojcad/shape abstract value (FACE).**

## Booleans

### `common`

**Usage:** `(common shape-a shape-b &keys :eager)`

Signals an error if the shapes do not intersect.
Keywords: :eager (tessellate immediately).

**Intersect shape-a with shape-b. Returns a new rojcad/shape representing the shared volume.**

### `cut`

**Usage:** `(cut shape-a shape-b &keys :eager)`

Signals an error if the shapes do not intersect or produce an empty result.
Keywords: :eager (tessellate immediately).

**Subtract shape-b from shape-a. Returns a new rojcad/shape representing the resulting solid.**

### `fuse`

**Usage:** `(fuse shape-a shape-b &keys :eager)`

Signals an error if the operation produces an empty result.
Keywords: :eager (tessellate immediately).

**Combine shape-a and shape-b into a single solid. Returns a new rojcad/shape representing the union of both shapes.**

## Queries

### `face?`

**Usage:** `(face? shape)`

Return true if the shape is a Face.

### `list-shapes`

**Usage:** `(list-shapes &keys :visible :hidden)`

Return a tuple of all registered ShapeData abstract values, optionally filtered by visibility.

With no arguments, returns all registered shapes.
With :visible, returns only visible shapes.
With :hidden, returns only hidden shapes.
If both :visible and :hidden are given, :hidden takes precedence.

Only shapes that have been shown (registered in the viewer) are included.

**Examples:**
```janet
(list-shapes)                    — all registered shapes
(list-shapes :visible)           — visible shapes only
(list-shapes :hidden)            — hidden shapes only
(each s (list-shapes) (print s)) — print all shapes
```

**Returns a tuple of rojcad/shape abstract values.**

### `selected-shapes`

**Usage:** `(selected-shapes)`

Return a tuple of ShapeData abstract values currently selected in the 3D viewer.

**Examples:**
```janet
(selected-shapes)                     — get selected shapes
(each s (selected-shapes) (hide s))   — hide all selected
```

**Returns a tuple of rojcad/shape abstract values.**

### `shape-type`

**Usage:** `(shape-type shape)`



**Return the OCCT topological type of a shape as a keyword. Returns :solid, :face, :edge, :wire, :shell, :vertex, :compound, :compound-solid, or :shape.**

### `solid?`

**Usage:** `(solid? shape)`

Return true if the shape is a Solid.

### `visible?`

**Usage:** `(visible? shape)`

Return true if the shape's visible flag is set, false otherwise.

### `wire?`

**Usage:** `(wire? shape)`

Return true if the shape is a Wire.

## Edge Styling

### `edge-active-show?`

**Usage:** `(edge-active-show?)`

Return true if edges on the selected shape are currently visible, false if hidden.

### `edge-color-active`

**Usage:** `(edge-color-active &opt r g b)`

Get or set the active (selected) edge color as RGB values in [0, 1].

Called with no arguments, returns the current color as a tuple '(r g b).
Called with three numeric arguments (r g b), sets the color.

Example: (edge-color-active 0.3 0.5 1.0)  — light blue
         (edge-color-active)               — query

### `edge-color-inactive`

**Usage:** `(edge-color-inactive &opt r g b)`

Get or set the inactive edge color as RGB values in [0, 1].

Called with no arguments, returns the current color as a tuple '(r g b).
Called with three numeric arguments (r g b), sets the color.

Example: (edge-color-inactive 0.8 0.8 0.8)  — light grey
         (edge-color-inactive)               — query

### `edge-hidden`

**Usage:** `(edge-hidden &opt value)`

Get or set visibility of hidden (occluded) edges.

Called with no arguments, returns true if hidden edges are shown, false if hidden.
Called with one boolean argument, sets the visibility.

Example: (edge-hidden)        — query
         (edge-hidden true)    — show hidden edges
         (edge-hidden false)   — hide hidden edges

### `edge-hidden-show?`

**Usage:** `(edge-hidden-show?)`

Return true if hidden (occluded) edges are currently visible, false if hidden.

### `edge-hidden-toggle`

**Usage:** `(edge-hidden-toggle)`

Example: (edge-hidden-toggle)

**Toggle visibility of hidden (occluded) edges. Returns true if hidden edges are now visible, false if hidden.**

### `edge-inactive-show?`

**Usage:** `(edge-inactive-show?)`

Return true if edges on non-selected shapes are currently visible, false if hidden.

### `edge-thickness`

**Usage:** `(edge-thickness &opt value)`

Get or set the edge line thickness in NDC units.

Called with no arguments, returns the current thickness.
Called with one numeric argument, sets the thickness and returns it.

Example: (edge-thickness 0.008) — thicker lines
         (edge-thickness)      — query

### `edge-toggle-active`

**Usage:** `(edge-toggle-active)`

Example: (edge-toggle-active)

**Toggle visibility of edges on the selected shape. Returns true if active edges are now visible, false if hidden.**

### `edge-toggle-inactive`

**Usage:** `(edge-toggle-inactive)`

Example: (edge-toggle-inactive)

**Toggle visibility of edges on non-selected shapes. Returns true if inactive edges are now visible, false if hidden.**

## Transforms

### `mirror`

**Usage:** `(mirror shape ox oy oz dx dy dz &keys :eager)`

Create a mirrored copy of shape about an axis.

Positional: (mirror shape ox oy oz dx dy dz)
Where (ox, oy, oz) is a point on the axis and (dx, dy, dz) is the axis direction.
Keywords: :eager (tessellate immediately).

**Examples:**
```janet
(mirror box 0 0 0 1 0 0)       — mirror across X axis through origin
(mirror box 5 0 0 0 1 0)       — mirror across Y axis through (5,0,0)
(mirror box 0 0 0 1 0 0 :eager) — eager tessellation
```

**Returns a new rojcad/shape abstract value. The original shape is unchanged.**

### `rotate`

**Usage:** `(rotate shape &keys :a :ar :x :y :z :r :eager)`

Create a rotated copy of shape.

Angle is specified via :a (degrees) or :ar (radians).
Axis is specified via :x, :y, :z (cardinal), or :r [dx dy dz] (custom).
:eager (tessellate immediately).

**Examples:**
```janet
(rotate box :a 45 :z)           — 45 degrees about Z
(rotate box :ar 1.5708 :x)      — pi/2 radians about X
(rotate box :a 90 :r [1 1 0])   — 90 degrees about custom axis
(rotate box :a 90 :z :eager)    — eager tessellation
```

**Returns a new rojcad/shape abstract value. The original shape is unchanged.**

### `scale`

**Usage:** `(scale shape factor &keys :o :eager)`

Create a uniformly scaled copy of shape.

Positional: (scale shape factor)
Keywords: :o [x y z] (center point, defaults to origin),
         :eager (tessellate immediately).

**Examples:**
```janet
(scale box 2.0)                — 2x about origin
(scale box 2.0 :o [5 5 5])     — 2x about custom point
(scale box 2.0 :eager)         — eager tessellation
```

**Returns a new rojcad/shape abstract value. The original shape is unchanged.**

### `translate`

**Usage:** `(translate shape dx dy dz &keys :t :eager)`

Create a translated copy of shape.

Positional: (translate shape dx dy dz)
Keywords: :t [dx dy dz], :eager (tessellate immediately).

**Examples:**
```janet
(translate box 5 0 0)               — move 5 units in X
(translate box :t [1 2 3])          — keyword style
(translate box 5 0 0 :eager)        — eager tessellation
```

**Returns a new rojcad/shape abstract value. The original shape is unchanged.**

