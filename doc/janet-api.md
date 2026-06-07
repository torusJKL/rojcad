# rojcad Janet API Reference

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

