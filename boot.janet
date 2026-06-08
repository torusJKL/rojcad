# boot.janet — TCP REPL server for rojcad

(def core-env (fiber/getenv (fiber/current)))

# ── Helper macros ────────────────────────────────────────────────────────────

(defmacro defmeta [sym cat &opt doc]
  ~(do
     (put (get core-env ',sym) :source "rojcad")
     (put (get core-env ',sym) :category ,cat)
     ,(if doc ~(put (get core-env ',sym) :doc ,doc) nil)))

(defmacro wrap-c-fn [name orig arglist & body]
  ~(let [,orig ((get core-env ',name) :value)]
     (put core-env ',name @{:value (fn ,arglist ,;body)})))

(defn setmeta [sym cat &opt doc]
  (def t (get core-env sym))
  (when (= :table (type t))
    (put t :source "rojcad")
    (put t :category cat)
    (when doc (put t :doc doc))))


# ── Variadic wrappers ───────────────────────────────────────────────────────
# Wraps C functions to accept multiple shapes. Uses table mutation to
# preserve metadata (doc, source, category) for discovery tools.

# ── Side-effects ──

(wrap-c-fn hide _hide [& shapes]
  (each s shapes (_hide s)))

(wrap-c-fn show _show [& shapes]
  (each s shapes (_show s)))

(def _purge ((get core-env 'purge) :value))
(put core-env 'purge @{:value (fn [& shapes]
  (each s shapes (_purge s)))})

(wrap-c-fn registry-remove _registry-remove [& shapes]
  (each s shapes (_registry-remove s)))

# ── Queries ──

(wrap-c-fn shape-type _shape-type [& shapes]
  (seq [s :in shapes] (_shape-type s)))

(wrap-c-fn visible? _visible [& shapes]
  (seq [s :in shapes] (_visible s)))

(wrap-c-fn wire? _wire [& shapes]
  (seq [s :in shapes] (_wire s)))

(wrap-c-fn face? _face [& shapes]
  (seq [s :in shapes] (_face s)))

(wrap-c-fn solid? _solid [& shapes]
  (seq [s :in shapes] (_solid s)))

# ── Booleans (chain + keyword routing) ──
(wrap-c-fn cut _cut [tool & rest]
  (var result tool)
  (var shapes @[])
  (var eager? false)
  (var hide? false)
  (each x rest
    (if (= :keyword (type x))
      (case x
        :eager (set eager? true)
        :hide (set hide? true))
      (array/push shapes x)))
  (def n (length shapes))
  (when (> n 0)
    (for k 0 (- n 1)
      (set result (_cut result (shapes k) 0 0)))
    (def last-b (shapes (- n 1)))
    (set result (_cut result last-b (if eager? 1 0) (if hide? 1 0))))
  (unless hide? (show result))
  result)

(wrap-c-fn common _common [first & rest]
  (var result first)
  (var shapes @[])
  (var eager? false)
  (var hide? false)
  (each x rest
    (if (= :keyword (type x))
      (case x
        :eager (set eager? true)
        :hide (set hide? true))
      (array/push shapes x)))
  (def n (length shapes))
  (when (> n 0)
    (for k 0 (- n 1)
      (set result (_common result (shapes k) 0 0)))
    (def last-b (shapes (- n 1)))
    (set result (_common result last-b (if eager? 1 0) (if hide? 1 0))))
  (unless hide? (show result))
  result)

(wrap-c-fn fuse _fuse [first & rest]
  (var result first)
  (var shapes @[])
  (var eager? false)
  (var hide? false)
  (each x rest
    (if (= :keyword (type x))
      (case x
        :eager (set eager? true)
        :hide (set hide? true))
      (array/push shapes x)))
  (def n (length shapes))
  (when (> n 0)
    (for k 0 (- n 1)
      (set result (_fuse result (shapes k) 0 0)))
    (def last-b (shapes (- n 1)))
    (set result (_fuse result last-b (if eager? 1 0) (if hide? 1 0))))
  (unless hide? (show result))
  result)

# ── Compound and color wrappers ──────────────────────────

(def _compound-cfn ((get core-env 'compound) :value))
(put core-env 'compound @{:value (fn [& args]
  (var shapes @[])
  (var color nil)
  (var eager false)
  (var hide false)
  (var i 0)
  (while (< i (length args))
    (if (= :keyword (type (args i)))
      (case (args i)
        :color (do
          (def v (args (++ i)))
          (set color @[(v 0) (v 1) (v 2)]))
        :eager (set eager true)
        :hide (set hide true))
      (array/push shapes (args i)))
    (++ i))
  (def n (length shapes))
  (if (= n 0)
    (error "compound: at least one shape is required")
    (if (= n 1)
      (let [shape (get shapes 0)]
        (when color (set-color shape (color 0) (color 1) (color 2)))
        shape)
      (let [shape (_compound-cfn shapes (if eager 1 0) (if hide 1 0))]
        (when color (set-color shape (color 0) (color 1) (color 2)))
        (unless hide (show shape))
        shape))))})
 
# `color` uses C function `set-color`, so we can't use wrap-c-fn (name mismatch).
(def _set-color ((get core-env 'set-color) :value))
(put core-env 'color @{:value (fn [shape r g b]
  (_set-color shape r g b)
  shape)})

(wrap-c-fn get-color _get-color [shape]
  (def c (_get-color shape))
  (if (= nil c) nil c))

(defmeta compound "cad-operations")
(defmeta color "cad-operations")
(defmeta get-color "cad-operations")

# ── Medium wrappers (manual variadic) ────────────────────────────

(wrap-c-fn sphere _sphere [& args]
  (var radius nil) (var cx nil) (var cy nil) (var cz nil)
  (var angle nil) (var eager false) (var hide false)
  (var i 0)
  (while (< i (length args))
    (if (= :keyword (type (args i)))
      (case (args i)
        :eager (set eager true)
        :hide (set hide true)
        :r (set radius (args (++ i)))
        :c (do (def v (args (++ i))) (set cx (v 0)) (set cy (v 1)) (set cz (v 2)))
        :a (set angle (* (args (++ i)) (/ math/pi 180)))
        :ar (set angle (args (++ i))))
      (when (= nil radius) (set radius (args i))))
    (++ i))
  (def shape (_sphere radius
               (if (not= nil cx) cx math/nan)
               (if (not= nil cy) cy math/nan)
               (if (not= nil cz) cz math/nan)
               (if (not= nil angle) angle math/nan)
               (if eager 1 0) (if hide 1 0)))
  (unless hide (show shape))
  shape)

(wrap-c-fn cone _cone [& args]
  (var br nil) (var tr nil) (var h nil)
  (var cx nil) (var cy nil) (var cz nil)
  (var angle nil) (var eager false) (var hide false)
  (var pos-arr @[]) (var pos-count 0)
  (var i 0)
  (while (< i (length args))
    (if (= :keyword (type (args i)))
      (case (args i)
        :eager (set eager true)
        :hide (set hide true)
        :br (set br (args (++ i)))
        :tr (set tr (args (++ i)))
        :h (set h (args (++ i)))
        :c (do (def v (args (++ i))) (set cx (v 0)) (set cy (v 1)) (set cz (v 2)))
        :a (set angle (* (args (++ i)) (/ math/pi 180)))
        :ar (set angle (args (++ i))))
      (do
        (array/push pos-arr (args i))
        (++ pos-count)))
    (++ i))
  (case pos-count
    2 (do (set br (pos-arr 0)) (set tr 0) (set h (pos-arr 1)))
    3 (do (set br (pos-arr 0)) (set tr (pos-arr 1)) (set h (pos-arr 2))))
  (def shape (_cone br tr h
               (if (not= nil cx) cx math/nan)
               (if (not= nil cy) cy math/nan)
               (if (not= nil cz) cz math/nan)
               (if (not= nil angle) angle math/nan)
               (if eager 1 0) (if hide 1 0)))
  (unless hide (show shape))
  shape)

# ── Complex wrappers (box, cylinder, torus) ─────────────────────────

(var _init_box (get core-env '_init-box))
(if (= :table (type _init_box)) (set _init_box (get _init_box :value)))
(var _init_cube (get core-env '_init-cube))
(if (= :table (type _init_cube)) (set _init_cube (get _init_cube :value)))
(var _init_box_from_corners (get core-env '_init-box-from-corners))
(if (= :table (type _init_box_from_corners)) (set _init_box_from_corners (get _init_box_from_corners :value)))
(put core-env 'box @{:value (fn [& args]
  (var cx nil) (var cy nil) (var cz nil)
  (var pl nil) (var ph nil)
  (var eager false) (var hide false)
  (var pos-arr @[]) (var pos-count 0)
  (var kw-w nil) (var kw-d nil) (var kw-h nil)
  (var i 0)
  (while (< i (length args))
    (if (= :keyword (type (args i)))
      (case (args i)
        :eager (set eager true)
        :hide (set hide true)
        :c (do (def v (args (++ i))) (set cx (v 0)) (set cy (v 1)) (set cz (v 2)))
        :w (set kw-w (args (++ i)))
        :d (set kw-d (args (++ i)))
        :h (set kw-h (args (++ i)))
        :pl (set pl (args (++ i)))
        :ph (set ph (args (++ i))))
      (do
        (array/push pos-arr (args i))
        (++ pos-count)))
    (++ i))
  (def shape
    (cond
      (and pl ph) (_init_box_from_corners (pl 0) (pl 1) (pl 2) (ph 0) (ph 1) (ph 2)
                    (if eager 1 0) (if hide 1 0))
      (or pl ph) (error "box: :pl and :ph must both be provided")
      (and kw-w kw-d kw-h) (_init_box kw-w kw-d kw-h
                              (if (not= nil cx) cx math/nan) (if (not= nil cy) cy math/nan) (if (not= nil cz) cz math/nan)
                              (if eager 1 0) (if hide 1 0))
      (or kw-w kw-d kw-h) (error "box: specify :w, :d, :h together")
      (= pos-count 1) (_init_cube (pos-arr 0)
                        (if (not= nil cx) cx math/nan) (if (not= nil cy) cy math/nan) (if (not= nil cz) cz math/nan)
                        (if eager 1 0) (if hide 1 0))
      (>= pos-count 3) (_init_box (pos-arr 0) (pos-arr 1) (pos-arr 2)
                         (if (not= nil cx) cx math/nan) (if (not= nil cy) cy math/nan) (if (not= nil cz) cz math/nan)
                         (if eager 1 0) (if hide 1 0))
      (error "box: expected 1 or 3 positional args or keywords :w :d :h or :pl :ph")))
  (unless hide (show shape))
  shape)}) 
(defmeta box "primitives"
  "(box &keys :w :d :h :c :pl :ph :eager :hide)\n\nCreate a box or cube.\n\nPositional: (box w d h) or (box size) for a cube.\nKeywords: :w :d :h (dimensions), :c (center [x y z]),\n         :pl :ph (opposite corners [x y z]).\n         :eager (tessellate immediately).\n         :hide (skip automatic show on def).\n\nExamples:\n  (box 10 20 30)           — box at origin\n  (box 10 20 30 :c [5 5 5]) — centered box\n  (box 5)                  — 5x5x5 cube\n  (box :pl [0 0 0] :ph [10 20 30]) — from corners\n  (box :w 10 :d 20 :h 30) — keyword style\n  (box 10 :eager)          — eager tessellation\n  (box 10 :hide)           — create without showing\n\nReturns a rojcad/shape abstract value.")

(var _init_cylinder (get core-env 'cylinder))
(if (= :table (type _init_cylinder)) (set _init_cylinder (get _init_cylinder :value)))
(var _init_cylinder_from_points (get core-env '_init-cylinder-from-points))
(if (= :table (type _init_cylinder_from_points)) (set _init_cylinder_from_points (get _init_cylinder_from_points :value)))
(var _init_cylinder_point_dir (get core-env '_init-cylinder-point-dir))
(if (= :table (type _init_cylinder_point_dir)) (set _init_cylinder_point_dir (get _init_cylinder_point_dir :value)))
(put core-env 'cylinder @{:value (fn [& args]
  (var cx nil) (var cy nil) (var cz nil)
  (var fp nil) (var tp nil) (var dir nil)
  (var r nil) (var h nil)
  (var eager false) (var hide false)
  (var pos-arr @[]) (var pos-count 0)
  (var i 0)
  (while (< i (length args))
    (if (= :keyword (type (args i)))
      (case (args i)
        :eager (set eager true)
        :hide (set hide true)
        :c (do (def v (args (++ i))) (set cx (v 0)) (set cy (v 1)) (set cz (v 2)))
        :r (set r (args (++ i)))
        :h (set h (args (++ i)))
        :dir (set dir (args (++ i)))
        :fp (set fp (args (++ i)))
        :tp (set tp (args (++ i))))
      (do
        (array/push pos-arr (args i))
        (++ pos-count)))
    (++ i))
  (cond
    (and fp tp)
    (do
      (when (= nil r) (error "cylinder: :r (radius) is required with :fp/:tp"))
      (def shape (_init_cylinder_from_points (fp 0) (fp 1) (fp 2) (tp 0) (tp 1) (tp 2) r
                   (if eager 1 0) (if hide 1 0)))
      (unless hide (show shape))
      shape)
    (or fp tp)
    (error "cylinder: :fp and :tp must both be provided")
    (do
      (default r (if (> pos-count 0) (pos-arr 0) (error "cylinder: radius required")))
      (default h (if (> pos-count 1) (pos-arr 1) (error "cylinder: height required")))
      (def shape
        (if dir
          (let [ox (if (not= nil cx) cx 0.0)
                oy (if (not= nil cy) cy 0.0)
                oz (if (not= nil cz) cz 0.0)]
            (_init_cylinder_point_dir ox oy oz r (dir 0) (dir 1) (dir 2) h
              (if eager 1 0) (if hide 1 0)))
          (_init_cylinder r h
            (if (not= nil cx) cx math/nan) (if (not= nil cy) cy math/nan) (if (not= nil cz) cz math/nan)
            (if eager 1 0) (if hide 1 0))))
      (unless hide (show shape))
      shape)))}) 
(defmeta cylinder "primitives"
  "(cylinder &keys :r :h :c :dir :fp :tp :eager :hide)\n\nCreate a cylinder.\n\nPositional: (cylinder radius height) — along Z axis, base at Z=0\nKeywords: :r (radius), :h (height), :c (center [x y z]),\n         :dir (direction [dx dy dz]),\n         :fp (from-point [x y z]), :tp (to-point [x y z]).\n         :eager (tessellate immediately).\n\nExamples:\n  (cylinder 5 10)                       — simple\n  (cylinder 5 10 :c [0 0 5])            — centered\n  (cylinder :fp [0 0 0] :tp [0 0 10] :r 5) — point-to-point\n  (cylinder :r 5 :h 10)                 — keyword style\n  (cylinder 5 10 :eager)                — eager tessellation\n\nReturns a rojcad/shape abstract value.")

(var _init_torus (get core-env 'torus))
(if (= :table (type _init_torus)) (set _init_torus (get _init_torus :value)))
(put core-env 'torus @{:value (fn [& args]
  (var cx nil) (var cy nil) (var cz nil)
  (var dir nil) (var rr nil) (var tr nil)
  (var a nil) (var as nil) (var ae nil)
  (var eager false) (var hide false)
  (var pos-arr @[]) (var pos-count 0)
  (var i 0)
  (while (< i (length args))
    (if (= :keyword (type (args i)))
      (case (args i)
        :eager (set eager true)
        :hide (set hide true)
        :c (do (def v (args (++ i))) (set cx (v 0)) (set cy (v 1)) (set cz (v 2)))
        :rr (set rr (args (++ i)))
        :tr (set tr (args (++ i)))
        :dir (set dir (args (++ i)))
        :a (set a (* (args (++ i)) (/ math/pi 180)))
        :ar (set a (args (++ i)))
        :as (set as (* (args (++ i)) (/ math/pi 180)))
        :asr (set as (args (++ i)))
        :ae (set ae (* (args (++ i)) (/ math/pi 180)))
        :aer (set ae (args (++ i))))
      (do
        (array/push pos-arr (args i))
        (++ pos-count)))
    (++ i))
  (default rr (if (> pos-count 0) (pos-arr 0) (error "torus: ring radius required")))
  (default tr (if (> pos-count 1) (pos-arr 1) (error "torus: tube radius required")))
  (def shape (_init_torus rr tr
               (if (not= nil cx) cx math/nan) (if (not= nil cy) cy math/nan) (if (not= nil cz) cz math/nan)
               (if dir (dir 0) math/nan) (if dir (dir 1) math/nan) (if dir (dir 2) math/nan)
               (if (not= nil a) a math/nan) (if (not= nil as) as math/nan) (if (not= nil ae) ae math/nan)
               (if eager 1 0) (if hide 1 0)))
  (unless hide (show shape))
  shape)}) 
(defmeta torus "primitives"
  "(torus &keys :rr :tr :c :a :ar :as :asr :ae :aer :dir :eager :hide)\n\nCreate a torus.\n\nPositional: (torus rr tr)\nKeywords: :rr (ring radius), :tr (tube radius),\n         :c (center [x y z]),\n         :a (angle in degrees), :ar (angle in radians, partial),\n         :as (start angle degrees), :asr (start angle radians),\n         :ae (end angle degrees), :aer (end angle radians),\n         :dir (axis direction [dx dy dz]),\n         :eager (tessellate immediately).\n\nExamples:\n  (torus 20 10)                    — full torus\n  (torus 20 10 :c [0 0 5])         — repositioned\n  (torus 20 10 :a 180)             — half torus\n  (torus :rr 20 :tr 10 :as 0 :ae 180) — angled range\n  (torus :rr 20 :tr 10 :dir [0 1 0]) — oriented\n  (torus 20 10 :eager)             — eager tessellation\n\nReturns a rojcad/shape abstract value.")

(wrap-c-fn extrude _extrude [shape & args]
  (var height nil) (var dx math/nan) (var dy math/nan) (var dz math/nan)
  (var both false) (var eager false) (var hide false)
  (var i 0)
  (while (< i (length args))
    (if (= :keyword (type (args i)))
      (case (args i)
        :eager (set eager true)
        :hide (set hide true)
        :both (set both true)
        :h (set height (args (++ i)))
        :x (do (set dx 1) (set dy 0) (set dz 0))
        :y (do (set dx 0) (set dy 1) (set dz 0))
        :z (do (set dx 0) (set dy 0) (set dz 1))
        :dir (do (def v (args (++ i))) (set dx (v 0)) (set dy (v 1)) (set dz (v 2))))
      nil)
    (++ i))
  (def s (_extrude shape height dx dy dz (if both 1 0) (if eager 1 0) (if hide 1 0)))
  (unless hide (show s))
  s)

(wrap-c-fn revolve _revolve [shape & args]
  (var angle nil) (var ox math/nan) (var oy math/nan) (var oz math/nan)
  (var dx math/nan) (var dy math/nan) (var dz math/nan)
  (var eager false) (var hide false)
  (var i 0)
  (while (< i (length args))
    (if (= :keyword (type (args i)))
      (case (args i)
        :eager (set eager true)
        :hide (set hide true)
        :a (set angle (* (args (++ i)) (/ math/pi 180)))
        :ar (set angle (args (++ i)))
        :c (do (def v (args (++ i))) (set ox (v 0)) (set oy (v 1)) (set oz (v 2)))
        :dir (do (def v (args (++ i))) (set dx (v 0)) (set dy (v 1)) (set dz (v 2))))
      nil)
    (++ i))
  (default angle (* 2 math/pi))
  (def s (_revolve shape angle ox oy oz dx dy dz (if eager 1 0) (if hide 1 0)))
  (unless hide (show s))
  s)

(wrap-c-fn extrude-polygon _extrude-polygon [& args]
  (var pts nil) (var height nil)
  (var plane nil) (var ax 0) (var ay 0) (var az 0)
  (var eager false) (var hide false)
  (var pos-count 0)
  (var i 0)
  (while (< i (length args))
    (if (= :keyword (type (args i)))
      (case (args i)
        :eager (set eager true)
        :hide (set hide true)
        :h (set height (args (++ i)))
        :plane (set plane (args (++ i)))
        :at (do (def v (args (++ i))) (set ax (v 0)) (set ay (v 1)) (set az (v 2))))
      (do
        (case pos-count
          0 (set pts (args i))
          1 (set height (args i)))
        (++ pos-count)))
    (++ i))
  (def s (_extrude-polygon pts height plane ax ay az (if eager 1 0) (if hide 1 0)))
  (unless hide (show s))
  s)

(wrap-c-fn rect _rect [& args]
  (var w nil) (var d nil)
  (var is-wire false) (var plane :xy) (var ax 0) (var ay 0) (var az 0)
  (var eager false) (var hide false)
  (var pos-count 0)
  (var i 0)
  (while (< i (length args))
    (if (= :keyword (type (args i)))
      (case (args i)
        :eager (set eager true)
        :hide (set hide true)
        :wire (set is-wire true)
        :w (set w (args (++ i)))
        :d (set d (args (++ i)))
        :h (set d (args (++ i)))
        :plane (set plane (args (++ i)))
        :at (do (def v (args (++ i))) (set ax (v 0)) (set ay (v 1)) (set az (v 2))))
       (do
         (case pos-count
           0 (set w (args i))
           1 (set d (args i)))
         (++ pos-count)))
     (++ i))
  (def s (_rect w d (if is-wire 1 0) plane ax ay az (if eager 1 0) (if hide 1 0)))
  (unless hide (show s))
  s)

(wrap-c-fn circle _circle [& args]
  (var r nil)
  (var is-wire false) (var plane :xy) (var ax 0) (var ay 0) (var az 0)
  (var eager false) (var hide false)
  (var i 0)
  (while (< i (length args))
    (if (= :keyword (type (args i)))
      (case (args i)
        :eager (set eager true)
        :hide (set hide true)
        :wire (set is-wire true)
        :r (set r (args (++ i)))
        :plane (set plane (args (++ i)))
        :at (do (def v (args (++ i))) (set ax (v 0)) (set ay (v 1)) (set az (v 2))))
      (when (= nil r) (set r (args i))))
    (++ i))
  (def s (_circle r (if is-wire 1 0) plane ax ay az (if eager 1 0) (if hide 1 0)))
  (unless hide (show s))
  s)

(wrap-c-fn polygon _polygon [& args]
  (var pts nil)
  (var is-wire false) (var plane :xy) (var ax 0) (var ay 0) (var az 0)
  (var eager false) (var hide false)
  (var i 0)
  (while (< i (length args))
    (if (= :keyword (type (args i)))
      (case (args i)
        :eager (set eager true)
        :hide (set hide true)
        :wire (set is-wire true)
        :pts (set pts (args (++ i)))
        :plane (set plane (args (++ i)))
        :at (do (def v (args (++ i))) (set ax (v 0)) (set ay (v 1)) (set az (v 2))))
      nil)
    (++ i))
  (def s (_polygon pts (if is-wire 1 0) plane ax ay az (if eager 1 0) (if hide 1 0)))
  (unless hide (show s))
  s)

(wrap-c-fn text _text [str font size & args]
  (var depth 0) (var plane nil) (var ax 0) (var ay 0) (var az 0)
  (var both false) (var eager false) (var hide false)
  (var i 0)
  (while (< i (length args))
    (if (= :keyword (type (args i)))
      (case (args i)
        :eager (set eager true)
        :hide (set hide true)
        :both (set both true)
        :depth (set depth (args (++ i)))
        :plane (set plane (args (++ i)))
        :at (do (def v (args (++ i))) (set ax (v 0)) (set ay (v 1)) (set az (v 2))))
      nil)
    (++ i))
  (def s (_text str font size depth (if both 1 0) plane ax ay az (if eager 1 0) (if hide 1 0)))
  (unless hide (show s))
  s)

(wrap-c-fn text3d _text3d [str font size depth & args]
  (var plane nil) (var ax 0) (var ay 0) (var az 0)
  (var both false) (var eager false) (var hide false)
  (var i 0)
  (while (< i (length args))
    (if (= :keyword (type (args i)))
      (case (args i)
        :eager (set eager true)
        :hide (set hide true)
        :both (set both true)
        :plane (set plane (args (++ i)))
        :at (do (def v (args (++ i))) (set ax (v 0)) (set ay (v 1)) (set az (v 2))))
      nil)
    (++ i))
  (def s (_text3d str font size depth (if both 1 0) plane ax ay az (if eager 1 0) (if hide 1 0)))
  (unless hide (show s))
  s)

(wrap-c-fn list-fonts _list-fonts []
  (def raw (_list-fonts))
  (seq [entry :in raw]
    (def parts (string/split "|" entry))
    (tuple (parts 0) (parts 1) (keyword (parts 2)))))

(wrap-c-fn translate _translate [shape & args]
  (var dx nil) (var dy nil) (var dz nil)
  (var eager false) (var hide false)
  (var pos-count 0)
  (var i 0)
  (while (< i (length args))
    (if (= :keyword (type (args i)))
      (case (args i)
        :eager (set eager true)
        :hide (set hide true)
        :t (do (def v (args (++ i))) (set dx (v 0)) (set dy (v 1)) (set dz (v 2))))
       (do
         (case pos-count
           0 (set dx (args i))
           1 (set dy (args i))
           2 (set dz (args i)))
         (++ pos-count)))
     (++ i))
  (def s (_translate shape dx dy dz (if eager 1 0) (if hide 1 0)))
  (unless hide (show s))
  s)

(wrap-c-fn rotate _rotate [shape & args]
  (var angle nil) (var ax 0) (var ay 0) (var az 0)
  (var eager false) (var hide false)
  (var i 0)
  (while (< i (length args))
    (if (= :keyword (type (args i)))
      (case (args i)
        :eager (set eager true)
        :hide (set hide true)
        :a (set angle (* (args (++ i)) (/ math/pi 180)))
        :ar (set angle (args (++ i)))
        :x (do (set ax 1) (set ay 0) (set az 0))
        :y (do (set ax 0) (set ay 1) (set az 0))
        :z (do (set ax 0) (set ay 0) (set az 1))
        :r (do (def v (args (++ i))) (set ax (v 0)) (set ay (v 1)) (set az (v 2))))
      nil)
    (++ i))
  (def s (_rotate shape ax ay az angle (if eager 1 0) (if hide 1 0)))
  (unless hide (show s))
  s)

(wrap-c-fn scale _scale [shape factor & args]
  (var ox math/nan) (var oy math/nan) (var oz math/nan)
  (var eager false) (var hide false)
  (var i 0)
  (while (< i (length args))
    (if (= :keyword (type (args i)))
      (case (args i)
        :eager (set eager true)
        :hide (set hide true)
        :o (do (def v (args (++ i))) (set ox (v 0)) (set oy (v 1)) (set oz (v 2))))
      nil)
    (++ i))
  (def s (_scale shape factor ox oy oz (if eager 1 0) (if hide 1 0)))
  (unless hide (show s))
  s)

(wrap-c-fn mirror _mirror [shape ox oy oz dx dy dz & args]
  (var eager false) (var hide false)
  (each arg args
    (case arg
      :eager (set eager true)
      :hide (set hide true)))
  (def s (_mirror shape ox oy oz dx dy dz (if eager 1 0) (if hide 1 0)))
  (unless hide (show s))
  s)

# ── Thin C-primitive wrappers ───────────────────────────────────────────────
# Each wrapper saves the C JANET_FN then replaces the binding with a Janet
# function. This preserves the C docstring/metadata while keeping the
# binding ready for future Janet-level overrides.

# Edge visibility toggles and queries

(wrap-c-fn edge-toggle-inactive _edge-toggle-inactive [] (_edge-toggle-inactive))
(wrap-c-fn edge-toggle-active _edge-toggle-active [] (_edge-toggle-active))
(wrap-c-fn edge-inactive-show? _edge-inactive-show? [] (_edge-inactive-show?))
(wrap-c-fn edge-active-show? _edge-active-show? [] (_edge-active-show?))
(wrap-c-fn edge-hidden-toggle _edge-hidden-toggle [] (_edge-hidden-toggle))
(wrap-c-fn edge-hidden-show? _edge-hidden-show? [] (_edge-hidden-show?))
(wrap-c-fn edge-hidden _edge-hidden [&opt value]
  (if (not= nil value) (_edge-hidden value) (_edge-hidden)))

# Projection and overlay toggles

(wrap-c-fn projection-toggle _projection-toggle [] (_projection-toggle))
(wrap-c-fn projection-perspective _projection-perspective [&opt value]
  (if (not= nil value) (_projection-perspective value) (_projection-perspective)))
(wrap-c-fn stats-overlay _stats-overlay [&opt value]
  (if (not= nil value) (_stats-overlay value) (_stats-overlay)))
(wrap-c-fn window-help-toggle _window-help-toggle [] (_window-help-toggle))
(wrap-c-fn window-help-show? _window-help-show? [] (_window-help-show?))
(wrap-c-fn window-help-show _window-help-show [&opt value]
  (if (not= nil value) (_window-help-show value) (_window-help-show)))
(wrap-c-fn window-size _window-size [width height] (_window-size width height))
(wrap-c-fn window-size? _window-size? [] (_window-size?))
(wrap-c-fn window-fullscreen _window-fullscreen [value] (_window-fullscreen value))
(wrap-c-fn window-fullscreen? _window-fullscreen? [] (_window-fullscreen?))
(wrap-c-fn window-maximized _window-maximized [value] (_window-maximized value))
(wrap-c-fn window-maximized? _window-maximized? [] (_window-maximized?))

# ── Sketch wrappers ──────────────────────────────────────────────────────────
# Replace C user-facing names with Janet wrappers for docstrings.
# The C thin primitives are registered under both the user-facing name
# and the _-prefixed name for internal use.

(wrap-c-fn sketch _sketch [&keys {:plane plane :at at}]
  (def args @[])
  (when plane (array/push args :plane) (array/push args plane))
  (when at (array/push args :at) (array/push args at))
  (apply _sketch args))
(put (get core-env 'sketch) :doc
  "(sketch &keys :plane :at)\n\nCreate a new sketch on a workplane.\nKeywords: :plane (keyword, default :xy), :at (array [x y z]).\nReturns a rojcad/sketch abstract value.\n\nExamples:\n  (sketch)\n  (sketch :plane :xz :at [10 0 5])")

(wrap-c-fn move-to _move-to [sketch x y] (_move-to sketch x y))
(put (get core-env 'move-to) :doc
  "(move-to sketch x y)\n\nMove the sketch cursor to (x, y) without drawing.\nReturns a new sketch.")

(wrap-c-fn line-to _line-to [sketch x y] (_line-to sketch x y))
(put (get core-env 'line-to) :doc
  "(line-to sketch x y)\n\nDraw a line from current cursor to (x, y).\nReturns a new sketch.")

(wrap-c-fn line-dx _line-dx [sketch dx] (_line-dx sketch dx))
(put (get core-env 'line-dx) :doc
  "(line-dx sketch dx)\n\nDraw a horizontal line by dx units.\nReturns a new sketch.")

(wrap-c-fn line-dy _line-dy [sketch dy] (_line-dy sketch dy))
(put (get core-env 'line-dy) :doc
  "(line-dy sketch dy)\n\nDraw a vertical line by dy units.\nReturns a new sketch.")

(wrap-c-fn line-dx-dy _line-dx-dy [sketch dx dy] (_line-dx-dy sketch dx dy))
(put (get core-env 'line-dx-dy) :doc
  "(line-dx-dy sketch dx dy)\n\nDraw a line by (dx, dy) offset.\nReturns a new sketch.")

(wrap-c-fn arc-to _arc-to [sketch x2 y2 x3 y3] (_arc-to sketch x2 y2 x3 y3))
(put (get core-env 'arc-to) :doc
  "(arc-to sketch x2 y2 x3 y3)\n\nDraw a circular arc through (x2, y2) to (x3, y3).\nReturns a new sketch.")

(wrap-c-fn close-sketch _close-sketch [sketch &keys {:eager eager :hide hide}]
  (def args @[sketch])
  (when eager (array/push args :eager))
  (when hide (array/push args :hide))
  (def s (apply _close-sketch args))
  (when hide (hide s))
  s)
(put (get core-env 'close-sketch) :doc
  "(close-sketch sketch &keys :eager :hide)\n\nClose the sketch and return a Face.\nKeywords: :eager, :hide\n\nExamples:\n  (-> (sketch) (line-to 10 0) (line-to 10 10) (close-sketch))")

(wrap-c-fn build-wire _build-wire [sketch &keys {:eager eager :hide hide}]
  (def args @[sketch])
  (when eager (array/push args :eager))
  (when hide (array/push args :hide))
  (def s (apply _build-wire args))
  (when hide (hide s))
  s)
(put (get core-env 'build-wire) :doc
  "(build-wire sketch &keys :eager :hide)\n\nReturn the sketch as an unclosed Wire.\nKeywords: :eager, :hide\n\nExamples:\n  (-> (sketch) (line-to 10 0) (line-to 10 10) (build-wire))")

# ── Wire operation wrappers ──────────────────────────────────────────────────

(wrap-c-fn wire-to-face _wire-to-face [wire &keys {:eager eager :hide hide}]
  (def args @[wire])
  (when eager (array/push args :eager))
  (when hide (array/push args :hide))
  (def s (apply _wire-to-face args))
  (when hide (hide s))
  s)
(put (get core-env 'wire-to-face) :doc
  "(wire-to-face wire &keys :eager :hide)\n\nConvert a Wire into a Face by filling its boundary.\nKeywords: :eager, :hide\n\nExamples:\n  (wire-to-face my-wire)")

(wrap-c-fn wire-fillet _wire-fillet [wire &keys {:r r :eager eager :hide hide}]
  (def args @[wire :r (if r r 0)])
  (when eager (array/push args :eager))
  (when hide (array/push args :hide))
  (def s (apply _wire-fillet args))
  (when hide (hide s))
  s)
(put (get core-env 'wire-fillet) :doc
  "(wire-fillet wire &keys :r :eager :hide)\n\nRound all vertices of a closed Wire by radius :r.\nKeywords: :r (required), :eager, :hide\n\nExamples:\n  (wire-fillet my-wire :r 2)")

(wrap-c-fn wire-chamfer _wire-chamfer [wire &keys {:d d :eager eager :hide hide}]
  (def args @[wire :d (if d d 0)])
  (when eager (array/push args :eager))
  (when hide (array/push args :hide))
  (def s (apply _wire-chamfer args))
  (when hide (hide s))
  s)
(put (get core-env 'wire-chamfer) :doc
  "(wire-chamfer wire &keys :d :eager :hide)\n\nBevel all vertices of a closed Wire by distance :d.\nKeywords: :d (required), :eager, :hide\n\nExamples:\n  (wire-chamfer my-wire :d 2)")

(wrap-c-fn wire-offset _wire-offset [wire &keys {:d d :eager eager :hide hide}]
  (def args @[wire :d (if d d 0)])
  (when eager (array/push args :eager))
  (when hide (array/push args :hide))
  (def s (apply _wire-offset args))
  (when hide (hide s))
  s)
(put (get core-env 'wire-offset) :doc
  "(wire-offset wire &keys :d :eager :hide)\n\nCreate a parallel offset of a closed Wire by distance :d.\nKeywords: :d (required), :eager, :hide\n\nExamples:\n  (wire-offset my-wire :d 2)")

# ── I/O wrappers ────────────────────────────────────────────────────────────

(wrap-c-fn write-step _write-step [shape path] (_write-step shape path))
(defmeta write-step "io"
  "(write-step shape path)\n\nExport a shape to a STEP file at the given path.\nReturns nil on success, signals an error on failure.\n\nExamples:\n  (write-step my-shape \"/tmp/model.step\")")

(wrap-c-fn write-stl _write-stl [shape path] (_write-stl shape path))
(defmeta write-stl "io"
  "(write-stl shape path)\n\nExport a shape to an STL file at the given path.\nReturns nil on success, signals an error on failure.\n\nExamples:\n  (write-stl my-shape \"/tmp/model.stl\")")

(wrap-c-fn read-step _read-step [path &keys {:eager eager :hide hide}]
  (def s (_read-step path (if eager true false)))
  (when hide (hide s))
  s)
(defmeta read-step "io"
  "(read-step path &keys :eager :hide)\n\nRead a STEP file from disk and return a shape.\n\nExamples:\n  (read-step \"/tmp/model.step\")       -- load from file\n  (read-step \"/tmp/model.step\" :eager) -- load and tessellate\n  (read-step \"/tmp/model.step\" :hide)  -- load without showing\n\nReturns a rojcad/shape abstract value. Signals an error on failure.")

# ── Selection callback storage ──────────────────────────────────────────────

(var *on-select-callback* nil)

# ── Quit & Selection wrappers ────────────────────────────────────────────────

(wrap-c-fn quit-requested _quit-requested [] (_quit-requested))
(defmeta quit-requested "view")

(put core-env 'on-select @{:value (fn [callback]
    (cond
      (= nil callback) (set *on-select-callback* nil)
      (= :function (type callback)) (set *on-select-callback* callback)
      (error "on-select expects a function or nil"))
    nil)})
(defmeta on-select "selection")

(def _poll-selection-raw ((get core-env '_poll-selection-raw) :value))
(put core-env 'poll-selection @{:value (fn []
    (when _poll-selection-raw
      (def raw (_poll-selection-raw))
      (when raw
        (def action (in raw 0))
        (def id (in raw 1))
        (def event
          (case action
            3 :deselected
            2 [:deselected id]
            id))
        (when *on-select-callback* (*on-select-callback* event))
        event)))}) 
(defmeta poll-selection "selection")

# ── Shape query wrappers ─────────────────────────────────────────────────────

(def _get-selected-ids ((get core-env '_get-selected-ids) :value))
(def _get-shape ((get core-env '_get-shape) :value))
(put core-env 'selected-shapes @{:value (fn []
    (tuple/slice (seq [id :in (_get-selected-ids)] (_get-shape id))))})
(defmeta selected-shapes "queries")

(def _get-registered-ids ((get core-env '_get-registered-ids) :value))
(put core-env 'list-shapes @{:value (fn [&keys {:visible visible :hidden hidden}]
    (def filter (if hidden 2 (if visible 1 0)))
    (tuple/slice (seq [id :in (_get-registered-ids filter)] (_get-shape id))))})
(defmeta list-shapes "queries")

# ── Edge styling wrappers ────────────────────────────────────────────────────

(wrap-c-fn edge-thickness _edge-thickness [&opt value]
  (if (not= nil value) (_edge-thickness value) (_edge-thickness)))
(defmeta edge-thickness "edge-styling")
(wrap-c-fn edge-color-inactive _edge-color-inactive [&opt r g b]
  (if r (_edge-color-inactive r g b) (_edge-color-inactive)))
(defmeta edge-color-inactive "edge-styling")
(wrap-c-fn edge-color-active _edge-color-active [&opt r g b]
  (if r (_edge-color-active r g b) (_edge-color-active)))
(defmeta edge-color-active "edge-styling")

# ── View control wrappers ────────────────────────────────────────────────────

(wrap-c-fn view-fit _view-fit [& args] (apply _view-fit args))
(wrap-c-fn view-fit-all _view-fit-all [&keys {:hidden hidden :reset reset}]
  (def args @[])
  (when hidden (array/push args :hidden))
  (when reset (array/push args :reset))
  (apply _view-fit-all args))
(wrap-c-fn view-angle _view-angle [yaw pitch &opt distance]
  (if (not= nil distance) (_view-angle yaw pitch distance) (_view-angle yaw pitch)))
(defmeta view-angle "view")

# view-angle metadata is set below with the presets

# ── rojcad metadata ───────────────────────────────────────────────────────
# Replaces removed C cad_fn_categories table + adds metadata for wrappers.

(def rojcad-groups
  {"primitives" [sphere cone cylinder torus]
   "booleans" [cut common fuse]
   "transforms" [translate rotate scale mirror]
   "2d-primitives" [rect circle polygon]
   "operations" [extrude revolve extrude-polygon]
   "text" [text text3d list-fonts]
   "wire-operations" [wire-to-face wire-fillet wire-chamfer wire-offset]
   "sketch" [sketch move-to line-to line-dx line-dy
             line-dx-dy arc-to close-sketch build-wire]
   "view" [view-fit view-fit-all
           projection-toggle projection-perspective stats-overlay
           window-help-toggle window-help-show? window-help-show
           window-size window-size? window-fullscreen
           window-fullscreen? window-maximized window-maximized?]
   "queries" [shape-type visible? wire? face? solid?]
   "registry" [purge hide show registry-remove]
   "edge-styling" [edge-toggle-inactive edge-toggle-active
                   edge-inactive-show? edge-active-show?
                   edge-hidden-toggle edge-hidden-show? edge-hidden]})

(each [cat syms] (pairs rojcad-groups)
  (each sym syms
    (def t (get core-env sym))
    (when (= :table (type t))
      (put t :source "rojcad")
      (put t :category cat))))

# ── Display helper (array-aware string conversion) ─────────────────────────

(defn display-val [x]
  (case (type x)
    :array (string/join (seq [v :in x] (string v)) "\n")
    :tuple (if (empty? x) "()"
            (string/join (seq [v :in x] (string v)) "\n"))
    :table (do
             (def lines @[])
             (var k (next x nil))
             (while k
               (def val (get x k))
               (if (= :array (type val))
                 (do
                   (array/push lines (string k ":"))
                   (each v val (array/push lines (string "  " v))))
                 (array/push lines (string k " → " val)))
               (set k (next x k)))
             (string/join lines "\n"))
    (string x)))

# ── REPL discoverability helpers ────────────────────────────────────────────

(defn sort-syms [arr]
  (for i 0 (length arr)
    (for j (+ i 1) (length arr)
      (def a (string (arr i)))
      (def b (string (arr j)))
      (when (> a b)
        (def tmp (arr i))
        (put arr i (arr j))
        (put arr j tmp))))
  arr)

(defn all-fns []
  (def fns @[])
  (var k (next core-env nil))
  (while k
    (def v (get core-env k))
    (when (or (= :cfunction (type (get v :value)))
              (= "rojcad" (get v :source)))
      (array/push fns k))
    (set k (next core-env k)))
  (sort-syms fns))

(defn apropos [pat]
  (def fns @[])
  (var k (next core-env nil))
  (while k
    (def v (get core-env k))
    (when (and (string/find pat (string k))
               (or (= :cfunction (type (get v :value)))
                   (= "rojcad" (get v :source))))
      (array/push fns k))
    (set k (next core-env k)))
  (sort-syms fns))

(defn doc [sym]
  (def binding (get core-env sym))
  (if (= :table (type binding))
    (do
      (def docs (get binding :doc))
      (if docs (string docs) (string "No documentation for " sym)))
    (string "No documentation for " sym)))

(defn cad-fns []
  (def fns @[])
  (var k (next core-env nil))
  (while k
    (def v (get core-env k))
    (when (= (get v :source) "rojcad")
      (array/push fns k))
    (set k (next core-env k)))
  (sort-syms fns))

(def cad-groups
  {"primitives" "Primitives"
   "booleans" "Booleans"
   "transforms" "Transforms"
   "queries" "Queries"
   "registry" "Registry"
   "io" "I/O"
   "selection" "Selection"
   "edge-styling" "Edge Styling"
   "view" "View"
   "2d-primitives" "2D Primitives"
   "operations" "Operations"
   "wire-operations" "Wire Operations"
   "sketch" "Sketch"
   "text" "Text"})

(defn group [&opt category]
  (if category
    (do
      (def fns @[])
      (var k (next core-env nil))
      (while k
        (def v (get core-env k))
        (when (= (get v :category) category)
          (array/push fns k))
        (set k (next core-env k)))
      (sort-syms fns))
    (do
      (def result @{})
      (var k (next core-env nil))
      (while k
        (def v (get core-env k))
        (when (= (get v :source) "rojcad")
          (def cat (or (get v :category) "other"))
          (def arr (get result cat))
          (if arr
            (array/push arr k)
            (put result cat (array k))))
        (set k (next core-env k)))
      (var gk (next result nil))
      (while gk
        (put result gk (sort-syms (get result gk)))
        (set gk (next result gk)))
      result)))

(defn my-parse [str]
  (def p (parser/new))
  (parser/consume p str)
  (parser/produce p))

(def shape-bindings @{})

(defn my-eval [form _env]
  (def compiled (compile form core-env))
  (if (= (type compiled) :function)
    (do
      (def f0 (if (= (type form) :tuple) (get form 0) nil))
      (def f2 (if (= (type form) :tuple) (get form 2) nil))
      (def old-val
        (case f0
          'def (get shape-bindings (get form 1))
          'set (get shape-bindings (get form 1))
          nil))
      (def result (resume (fiber/new compiled)))
      (when (and old-val (= (type old-val) :rojcad/shape) (not= old-val result))
        (_purge old-val))
      (when (and f0 (or (= f0 'def) (= f0 'set)))
        (put shape-bindings (get form 1)
          (if (= (type result) :rojcad/shape) result nil)))
      (when (and f0 (or (= f0 'def) (= f0 'set)) f2
                 (= (type result) :rojcad/shape) (visible? result))
        (show result))
      result)
    (string "compile error: " (get compiled :error) " line:" (get compiled :line))))

(def port (if (dyn '*netrepl-port*) (dyn '*netrepl-port*) 9365))

# ── Doc generation ────────────────────────────────────────────────────────

(defn html-escape [s]
  (string/replace "&" "&amp;" (string/replace "<" "&lt;" (string/replace ">" "&gt;" s))))

(defn bool-and [a b] (if a b false))
(defn bool-or [a b] (if a true b))

(def special-forms
  @{:def :sp :fn :sp :if :sp :do :sp :while :sp :var :sp :set :sp
    :break :sp :not :sp :in :sp :get :sp :put :sp
    :next :sp :true :sp :false :sp :nil :sp})

(def tokenize-janet (fn [code]
  (def out @[])
  (def len (length code))
  (var i 0)
  (while (< i len)
    (def b (get code i))
    (if (= 59 b)
      (do
        (def start i)
        (while (bool-and (< i len) (bool-and (not= 10 (get code i)) (not= 13 (get code i))))
          (set i (+ i 1)))
        (array/push out (string "<span class=\"tok-c\">" (html-escape (string/slice code start i)) "</span>")))
      (if (= 34 b)
        (do
          (def start i)
          (set i (+ i 1))
          (while (bool-and (< i len) (not= 34 (get code i)))
            (if (= 92 (get code i)) (set i (+ i 1)))
            (set i (+ i 1)))
          (if (< i len) (set i (+ i 1)))
          (array/push out (string "<span class=\"tok-s\">" (html-escape (string/slice code start i)) "</span>")))
        (if (= 58 b)
          (do
            (def start i)
            (set i (+ i 1))
            (while (bool-and (< i len) (bool-or (bool-or (bool-or (bool-or (<= 97 (get code i) 122) (<= 65 (get code i) 90)) (<= 48 (get code i) 57)) (= 63 (get code i))) (= 45 (get code i))))
              (set i (+ i 1)))
            (array/push out (string "<span class=\"tok-k\">" (html-escape (string/slice code start i)) "</span>")))
          (if (bool-or (<= 48 b 57) (bool-and (= 45 b) (bool-and (< (+ i 1) len) (<= 48 (get code (+ i 1)) 57))))
            (do
              (def start i)
              (if (= 45 b) (set i (+ i 1)))
              (while (bool-and (< i len) (bool-or (<= 48 (get code i) 57) (= 46 (get code i))))
                (set i (+ i 1)))
              (array/push out (string "<span class=\"tok-n\">" (html-escape (string/slice code start i)) "</span>")))
            (if (bool-or (bool-or (bool-or (bool-or (<= 97 b 122) (<= 65 b 90)) (= 63 b)) (= 33 b)) (= 95 b))
              (do
                (def start i)
                (set i (+ i 1))
                (while (bool-and (< i len) (bool-or (bool-or (bool-or (bool-or (bool-or (bool-or (<= 97 (get code i) 122) (<= 65 (get code i) 90)) (<= 48 (get code i) 57)) (= 63 (get code i))) (= 33 (get code i))) (= 45 (get code i))) (= 95 (get code i))))
                  (set i (+ i 1)))
                (def sym (string/slice code start i))
                (if (get special-forms (keyword sym))
                  (array/push out (string "<span class=\"tok-sp\">" (html-escape sym) "</span>"))
                  (array/push out (html-escape sym))))
              (do
                (array/push out (html-escape (string/from-bytes b)))
                (set i (+ i 1)))))))))
  (string/join out "")))

(defn split-docstring [doc]
  (def parts (string/split "\n\n" doc))
  (def usage (string/trim (parts 0)))
  (def body @[])
  (var examples nil)
  (var returns nil)
  (var in-examples false)
  (for pi 1 (length parts)
    (def p (parts pi))
    (def t (string/trim p))
    (cond
      (string/find "Examples:" t)
      (do
        (set in-examples true)
        (def ex-code (seq [line :in (string/split "\n" p)
                           :let [tl (string/trim line)]
                           :when (and (> (length tl) 0) (not= tl "Examples:"))]
                       tl))
        (set examples (string/join ex-code "\n")))
      (string/find "Returns" t)
      (do
        (set returns t)
        (set in-examples false))
      in-examples
      (each line (string/split "\n" p)
        (def tl (string/trim line))
        (when (> (length tl) 0)
          (set examples (string examples "\n" tl))))
      (array/push body p)))
  (array usage (string/join body "\n\n") examples returns))

(def gen-markdown (fn [path &opt version]
  (def f (file/open path :wn))
  (if (= nil f)
    (print "dump-docs: failed to open " path)
    (do
      (def title (string "rojcad Janet API Reference" (if version (string " — " version) "")))
      (file/write f (string "# " title "\n\n"))
      (def all-groups (group))
      (var cat-k (next cad-groups nil))
      (while cat-k
        (def fns (get all-groups cat-k))
        (if fns
          (do
            (file/write f (string "## " (get cad-groups cat-k) "\n\n"))
            (var fi 0)
            (while (< fi (length fns))
              (def fn-name (get fns fi))
              (def fn-doc (doc fn-name))
              (def doc-arr (split-docstring fn-doc))
              (def usage (get doc-arr 0))
              (def body-text (get doc-arr 1))
              (def examples-text (get doc-arr 2))
              (def returns-text (get doc-arr 3))
              (file/write f (string "### `" fn-name "`\n\n"))
              (file/write f (string "**Usage:** `" usage "`\n\n"))
              (file/write f (string body-text "\n\n"))
              (if (not= nil examples-text)
                (do
                  (file/write f "**Examples:**\n```janet\n")
                  (file/write f examples-text)
                  (file/write f "\n```\n\n")))
              (if (not= nil returns-text)
                (file/write f (string "**" returns-text "**\n\n")))
              (set fi (+ fi 1)))))
        (set cat-k (next cad-groups cat-k)))
      (def other-fns @[])
      (var gk (next all-groups nil))
      (while gk
        (if (= nil (get cad-groups gk))
          (array/concat other-fns (get all-groups gk)))
        (set gk (next all-groups gk)))
      (if (> (length other-fns) 0)
        (do
          (file/write f "## Other\n\n")
          (var fi 0)
          (while (< fi (length other-fns))
            (def fn-name (get other-fns fi))
            (def fn-doc (doc fn-name))
            (def doc-arr (split-docstring fn-doc))
            (def usage (get doc-arr 0))
            (def body-text (get doc-arr 1))
            (def examples-text (get doc-arr 2))
            (def returns-text (get doc-arr 3))
            (file/write f (string "### `" fn-name "`\n\n"))
            (file/write f (string "**Usage:** `" usage "`\n\n"))
            (file/write f (string body-text "\n\n"))
            (if (not= nil examples-text)
              (do
                (file/write f "**Examples:**\n```janet\n")
                (file/write f examples-text)
                (file/write f "\n```\n\n")))
            (if (not= nil returns-text)
              (file/write f (string "**" returns-text "**\n\n")))
            (set fi (+ fi 1))))
      (file/close f))))))

(def gen-html (fn [path &opt version]
  (def f (file/open path :wn))
  (if (= nil f)
    (print "dump-docs: failed to open " path)
    (do
      (def title (string "rojcad Janet API Reference" (if version (string " — " version) "")))
      (file/write f
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n"
        "<meta charset=\"UTF-8\">\n"
        "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n"
        (string "<title>" title "</title>\n")
        "<style>\n"
        "*,*::before,*::after{margin:0;padding:0;box-sizing:border-box}\n"
        "html,body{height:100%;overflow:hidden}\n"
        "body{font-family:sans-serif;color:#222;background:#fff;line-height:1.6;display:flex;flex-direction:column}\n"
        "#search{width:100%;padding:12px 20px;font-size:16px;border:none;border-bottom:1px solid #ddd;outline:none;flex-shrink:0}\n"
        "#search:focus{border-bottom-color:#06c}\n"
        "#layout{display:flex;flex:1;overflow:hidden}\n"
        "#sidebar{width:240px;flex-shrink:0;border-right:1px solid #eee;padding:20px;overflow-y:auto;background:#fafafa}\n"
        "#sidebar h2{font-size:11px;text-transform:uppercase;color:#999;letter-spacing:1px;margin-bottom:12px}\n"
        "#sidebar ul{list-style:none;margin-bottom:20px}\n"
        "#sidebar li{margin-bottom:4px}\n"
        "#sidebar a{color:#555;text-decoration:none;font-size:14px}\n"
        "#sidebar a:hover{color:#06c}\n"
        "#sidebar .cat{margin-top:4px;font-weight:600;color:#333}\n"
        "#sidebar .cat a{color:#333}\n"
        "#sidebar .fn{padding-left:14px;font-size:13px}\n"
        "main{flex:1;padding:32px 48px;overflow-y:auto}\n"
        "main h1{font-size:28px;margin-bottom:8px;color:#111}\n"
        "main h2{font-size:20px;margin-top:40px;margin-bottom:16px;padding-bottom:6px;border-bottom:1px solid #eee;color:#222}\n"
        "article{margin-bottom:32px;padding:20px 0 0 0}\n"
        "article h3{font-size:18px;margin-bottom:8px;font-family:monospace;color:#06c}\n"
        ".usage{font-family:monospace;font-size:13px;background:#f5f5f5;padding:8px 12px;border-radius:4px;margin-bottom:12px;color:#555;overflow-x:auto}\n"
        ".desc p{margin-bottom:8px;font-size:14px}\n"
        ".examples{margin:12px 0}\n"
        ".examples pre{background:#f8f8f8;border:1px solid #eee;border-radius:4px;padding:12px;overflow-x:auto;font-family:monospace;font-size:13px;line-height:1.5}\n"
        ".returns{margin-top:8px;font-size:13px;color:#666}\n"
        ".tok-c{color:#888}\n.tok-s{color:#a31515}\n.tok-k{color:#00c}\n.tok-n{color:#098658}\n.tok-sp{color:#795e26}\n"
        "#top-btn{position:fixed;bottom:24px;right:24px;width:40px;height:40px;border-radius:8px;border:1px solid #ddd;background:#fff;color:#555;font-size:20px;cursor:pointer;display:flex;align-items:center;justify-content:center;box-shadow:0 2px 8px rgba(0,0,0,.08);transition:background .15s,color .15s;z-index:100}\n"
        "#top-btn:hover{background:#06c;color:#fff;border-color:#06c}\n"
        "</style>\n</head>\n<body>\n"
        "<input id=\"search\" type=\"text\" placeholder=\"Search functions... (Ctrl+K)\">\n"
        "<div id=\"layout\">\n"
        "<nav id=\"sidebar\">\n<h2>Categories</h2>\n<ul>\n")
      (def all-groups (group))
      (var cat-k (next cad-groups nil))
      (while cat-k
        (def fns (get all-groups cat-k))
        (if fns
          (do
            (def cat-disp (get cad-groups cat-k))
            (file/write f (string "<li class=\"cat\"><a href=\"#" cat-k "\">" cat-disp "</a></li>\n"))
            (var fi 0)
            (while (< fi (length fns))
              (def fn-name (get fns fi))
              (file/write f (string "<li class=\"fn\"><a href=\"#" fn-name "\">" fn-name "</a></li>\n"))
              (set fi (+ fi 1)))))
        (set cat-k (next cad-groups cat-k)))
      (def other-fns @[])
      (var gk (next all-groups nil))
      (while gk
        (if (= nil (get cad-groups gk))
          (array/concat other-fns (get all-groups gk)))
        (set gk (next all-groups gk)))
      (if (> (length other-fns) 0)
        (do
          (file/write f "<li class=\"cat\"><a href=\"#other\">Other</a></li>\n")
          (var fi 0)
          (while (< fi (length other-fns))
            (def fn-name (get other-fns fi))
            (file/write f (string "<li class=\"fn\"><a href=\"#" fn-name "\">" fn-name "</a></li>\n"))
            (set fi (+ fi 1)))))
      (file/write f (string "</ul>\n</nav>\n<main>\n<h1>" title "</h1>\n"))
      (set cat-k (next cad-groups nil))
      (while cat-k
        (def fns (get all-groups cat-k))
        (if fns
          (do
            (def cat-disp (get cad-groups cat-k))
            (file/write f (string "<section id=\"" cat-k "\">\n<h2>" cat-disp "</h2>\n"))
            (var fi 0)
            (while (< fi (length fns))
              (def fn-name (get fns fi))
              (def fn-doc (doc fn-name))
              (def doc-arr (split-docstring fn-doc))
              (def usage (get doc-arr 0))
              (def body-text (get doc-arr 1))
              (def examples-text (get doc-arr 2))
              (def returns-text (get doc-arr 3))
              (file/write f (string "<article id=\"" fn-name "\">\n"))
              (file/write f (string "<h3>" fn-name "</h3>\n"))
              (file/write f (string "<div class=\"usage\">" (html-escape usage) "</div>\n"))
              (def body-paras (string/split "\n\n" body-text))
              (file/write f "<div class=\"desc\">\n")
              (var bi 0)
              (def bn (length body-paras))
              (while (< bi bn)
                (def p (get body-paras bi))
                (if (> (length (string/trim p)) 0)
                  (file/write f (string "<p>" (html-escape p) "</p>\n")))
                (set bi (+ bi 1)))
              (file/write f "</div>\n")
              (if (not= nil examples-text)
                (do
                  (file/write f "<div class=\"examples\">\n<pre><code>")
                  (file/write f (tokenize-janet examples-text))
                  (file/write f "</code></pre>\n</div>\n")))
              (if (not= nil returns-text)
                (file/write f (string "<div class=\"returns\">" (html-escape returns-text) "</div>\n")))
              (file/write f "</article>\n")
              (set fi (+ fi 1)))
            (file/write f "</section>\n")))
        (set cat-k (next cad-groups cat-k)))
      (if (> (length other-fns) 0)
        (do
          (file/write f "<section id=\"other\">\n<h2>Other</h2>\n")
          (var fi 0)
          (while (< fi (length other-fns))
            (def fn-name (get other-fns fi))
            (def fn-doc (doc fn-name))
            (def doc-arr (split-docstring fn-doc))
            (def usage (get doc-arr 0))
            (def body-text (get doc-arr 1))
            (def examples-text (get doc-arr 2))
            (def returns-text (get doc-arr 3))
            (file/write f (string "<article id=\"" fn-name "\">\n"))
            (file/write f (string "<h3>" fn-name "</h3>\n"))
            (file/write f (string "<div class=\"usage\">" (html-escape usage) "</div>\n"))
            (def body-paras (string/split "\n\n" body-text))
            (file/write f "<div class=\"desc\">\n")
            (var bi 0)
            (def bn (length body-paras))
            (while (< bi bn)
              (def p (get body-paras bi))
              (if (> (length (string/trim p)) 0)
                (file/write f (string "<p>" (html-escape p) "</p>\n")))
              (set bi (+ bi 1)))
            (file/write f "</div>\n")
            (if (not= nil examples-text)
              (do
                (file/write f "<div class=\"examples\">\n<pre><code>")
                (file/write f (tokenize-janet examples-text))
                (file/write f "</code></pre>\n</div>\n")))
            (if (not= nil returns-text)
              (file/write f (string "<div class=\"returns\">" (html-escape returns-text) "</div>\n")))
            (file/write f "</article>\n")
            (set fi (+ fi 1)))
          (file/write f "</section>\n")))
      (file/write f
        "</main>\n</div>\n"
        "<button id=\"top-btn\" onclick=\"document.querySelector('main').scrollTo({top:0,behavior:'smooth'})\" aria-label=\"Back to top\">↑</button>\n"
        "<script>\n"
        "document.addEventListener('keydown',function(e){if(e.ctrlKey&&e.key==='k'){e.preventDefault();document.getElementById('search').focus()}});\n"
        "document.getElementById('search').addEventListener('input',function(){var q=this.value.toLowerCase();document.querySelectorAll('article').forEach(function(a){a.style.display=a.textContent.toLowerCase().includes(q)?'':'none'})});\n"
        "</script>\n</body>\n</html>\n")
      (file/close f)))))

(defn dump-docs [&opt path version]
  (def dir (if path path "doc"))
  (try (os/mkdir dir) ([e] nil))
  (def md-path (string dir "/janet-api.md"))
  (def html-path (string dir "/janet-api.html"))
  (gen-markdown md-path version)
  (gen-html html-path version)
  (string "Documentation written to " dir "/"))
(def addr "127.0.0.1")

(defn connect-handler [stream]
  (eprint "● client connected")
  (fiber/setenv (fiber/current) core-env)
  (def env core-env)
  (while true
    (def line-raw (try (net/read stream 4096) ([e] nil)))
    (if (= line-raw nil) (break))
    (def line (string/trim line-raw))
    (if (= line "") (break))
    (def parsed (my-parse line))
    (if (not= parsed nil)
      (do
        (def eval-result (try (my-eval parsed env) ([e] e)))
        (def result-str (display-val eval-result))
        (:write stream result-str)
        (:write stream "\n"))
      (:write stream "parse error\n")))
  (:close stream)
  (eprint "● client disconnected"))

(def listen
  (do
    (def listen-fiber (fiber/new (fn [] (net/listen addr port))))
    (def listen-val (resume listen-fiber))
    (def listen-status (fiber/status listen-fiber))
    (if (= listen-status :dead) listen-val
      (do (eprint "rojcad: failed to listen on " addr ":" port) (os/exit 1)))))

(eprint "◆ rojcad ready — connect via: nc " addr " " port)

# ── View angle presets (data-driven) ─────────────────────────

(def view-presets
  {:view-front  [(/ math/pi 2) 0 "Set camera to front view (looking along +Z toward origin)."
                 "Yaw=π/2, Pitch=0." "  (view-front)\n  (view-front 200)"]
   :view-back   [(- (/ math/pi 2)) 0 "Set camera to back view (looking along -Z toward origin)."
                 "Yaw=-π/2, Pitch=0." "  (view-back)\n  (view-back 200)"]
   :view-right  [0 0 "Set camera to right view (looking along +X toward origin)."
                 "Yaw=0, Pitch=0." "  (view-right)\n  (view-right 200)"]
   :view-left   [math/pi 0 "Set camera to left view (looking along -X toward origin)."
                 "Yaw=π, Pitch=0." "  (view-left)\n  (view-left 200)"]
   :view-top    [0 (/ math/pi 2) "Set camera to top view (looking along +Y toward origin)."
                 "Yaw=0, Pitch=π/2." "  (view-top)\n  (view-top 200)"]
   :view-bottom [0 (- (/ math/pi 2)) "Set camera to bottom view (looking along -Y toward origin)."
                 "Yaw=0, Pitch=-π/2." "  (view-bottom)\n  (view-bottom 200)"]
   :view-iso    [(/ math/pi 4) (math/asin (/ 1 (math/sqrt 3)))
                 "Set camera to isometric view (looking from (1,1,1) direction)."
                 "Yaw=π/4, Pitch=asin(1/√3) ≈ 0.615 rad." "  (view-iso)\n  (view-iso 150)"]})

(defn set-viewmeta [name cat &opt doc]
  (def t (get core-env name))
  (when (= :table (type t))
    (put t :source "rojcad")
    (put t :category cat)
    (when doc (put t :doc doc))))

(setmeta view-angle "view")
(var vk (next view-presets nil))
(while vk
  (def v (view-presets vk))
  (def name (symbol vk))
  (put core-env name @{:value
    (fn [&opt distance]
      (if distance
        (view-angle (v 0) (v 1) distance)
        (view-angle (v 0) (v 1))))})
  (set-viewmeta name "view"
    (string "(view-" name " ; distance)\n\n"
            (v 2) "\n"
            (v 3) " Animates over 0.5s.\n"
            "Optional distance sets zoom level; omitted preserves current.\n\n"
            "Examples:\n"
            (v 4)))
  (set vk (next view-presets vk)))

(defmeta sketch "sketch")
(defmeta move-to "sketch")
(defmeta line-to "sketch")
(defmeta line-dx "sketch")
(defmeta line-dy "sketch")
(defmeta line-dx-dy "sketch")
(defmeta arc-to "sketch")
(defmeta close-sketch "sketch")
(defmeta build-wire "sketch")
(defmeta wire-to-face "wire-operations")
(defmeta wire-fillet "wire-operations")
(defmeta wire-chamfer "wire-operations")
(defmeta wire-offset "wire-operations")

(defn poll-viewer []
  (while true
    (when (quit-requested) (os/exit 0))
    (def event (poll-selection))
    (when event
      (case (type event)
        :tuple (eprint "■ deselected: " (in event 1))
        (case event
          :deselected (eprint "■ deselected all")
          (eprint "■ selected: " event))))
    (ev/sleep 0.1)))

(ev/go (fiber/new poll-viewer))

(defn accept-loop []
  (while true
    (def conn (net/accept listen))
    (ev/go (fn [] (connect-handler conn)))))

(ev/go (fiber/new accept-loop))
