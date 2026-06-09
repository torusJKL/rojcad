# model.janet — Parametric model runtime for rojcad
#
# Provides:
#   defmodel  — define a parametric model (macro)
#   build     — instantiate a model (returns a shape)
#   graph     — introspect model structure
#   highlight — highlight a named part in the viewer
#   highlight-clear — remove highlighting
#
# CAD functions are wrapped at load time for automatic shape tracking
# during model builds via the *model-context* dynamic variable.

(def core-env (fiber/getenv (fiber/current)))

# ── Shape tracking context ──────────────────────────────────────────────

(var *model-context* nil)

# ── CAD functions that return shapes ────────────────────────────────────

(def cad-shape-fns
  '[box sphere cylinder cone torus
    cut common fuse compound
    translate rotate scale mirror
    extrude revolve extrude-polygon
    rect circle polygon
    wire-to-face wire-fillet wire-chamfer wire-offset
    close-sketch build-wire
    text text3d
    read-step])

# ── Wrap CAD functions for model tracking ──────────────────────────────

(each fn-name cad-shape-fns
  (def binding (get core-env fn-name))
  (when (and binding (= :function (type (binding :value))))
    (let [orig-fn (binding :value)]
      (put (get core-env fn-name) :value (fn [& args]
        (def result (apply orig-fn args))
        (when *model-context*
          (when (= :rojcad/shape (type result))
            (array/push (*model-context* :shapes) result))
          (when (= :array (type result))
            (each s result
              (when (= :rojcad/shape (type s))
                (array/push (*model-context* :shapes) s)))))
        result)))))

# ── Helpers ─────────────────────────────────────────────────────────────

(defn- model-error [msg]
  (error (string "defmodel: " msg)))

(defn- build-error [msg]
  (error (string "build: " msg)))

# Record part shapes from the parts table into the model's shape-map.
# Helper to get a C function from core-env.
(defn get-c-fn [name]
  ((get core-env name) :value))

(defn- record-shape-map [parts model]
  (var k (next parts nil))
  (while k
    (put (model :shape-map) k (get parts k))
    (set k (next parts k))))

# ── defmodel macro ─────────────────────────────────────────────────────
#
# (defmodel name [params...]
#   [:parts {:part-name expr ...}]
#   [:result expr]
#   body...)

(defmacro defmodel [name params & more]
  (var parts-expr nil)
  (var result-expr nil)
  (var body-exprs @[])

  (var i 0)
  (while (< i (length more))
    (def item (more i))
    (if (= :keyword (type item))
      (case item
        :parts (do
                 (set parts-expr (more (+ i 1)))
                 (set i (+ i 2)))
        :result (do
                  (set result-expr (more (+ i 1)))
                  (set i (+ i 2)))
        (do
          (array/push body-exprs item)
          (set i (+ i 1))))
      (do
        (array/push body-exprs item)
        (set i (+ i 1)))))

  (when (and (not= nil parts-expr) (= nil result-expr))
    (model-error ":parts requires :result"))

  # Determine the effective result expression
  (def effective-result
    (if (not= nil result-expr)
      result-expr
      (do
        (when (empty? body-exprs)
          (model-error "body cannot be empty"))
        (let [last-expr (last body-exprs)]
          (set body-exprs (tuple/slice body-exprs 0 (- (length body-exprs) 1)))
          last-expr))))

  # Build the fn body expressions
  (def body-forms @[])
  (when (not= nil parts-expr)
    (array/push body-forms (tuple 'def 'parts parts-expr))
    # Record part shapes in the model's shape-map for introspection
    (array/push body-forms (tuple 'record-shape-map 'parts '*model-context*)))
  (each expr body-exprs
    (array/push body-forms expr))
  (array/push body-forms effective-result)

  # Source form (preserved for introspection)
  (def source-form (apply tuple 'fn params (tuple/slice body-forms 0)))

  # Emit a bare top-level def (not wrapped in do, so the compiler
  # can see the binding for subsequent forms).
  (def emit-parts (if (not= nil parts-expr) (tuple 'quote parts-expr) nil))

  ~(def ,name
     (table :params ',params
            :source ',source-form
            :body-fn (fn ,params ,;body-forms)
            :parts ,emit-parts
            :shapes (array)
            :shape-map (table)
            :current-params nil
            :result nil)))

# ── Build function ─────────────────────────────────────────────────────

(defn build [model & params]
  (when (not= :table (type model))
    (build-error "expected a model record"))
  (def body-fn (model :body-fn))
  (when (not= :function (type body-fn))
    (build-error "model has no :body-fn"))

  (def expected-count (length (model :params)))
  (when (not= expected-count (length params))
    (build-error (string "expected " expected-count " parameters, got " (length params))))

  # Purge old shapes
  (def purge-fn (get-c-fn 'purge))
  (each s (model :shapes)
    (when (= :rojcad/shape (type s))
      (purge-fn s)))
  (put model :shapes @[])
  (put model :shape-map @{})
  (put model :result nil)

  # Execute body-fn with shape tracking
  (def old-context *model-context*)
  (set *model-context* model)
  (def build-fiber (fiber/new (fn [] (apply body-fn params))))
  (def status (resume build-fiber))
  (set *model-context* old-context)

  (def payload (fiber/last-value build-fiber))
  (if (= :dead (fiber/status build-fiber))
    (do
      (put model :current-params params)
      (put model :result payload)
      (put (model :shape-map) :result payload)
      # Re-show the result shape (the wrapper hid it along with intermediates)
      (show payload)
      # If we're inside a parent context, register our result for purge tracking
      (when old-context
        (array/push (old-context :shapes) payload)
        (unless (get (old-context :shape-map) :_sub-instances)
          (put (old-context :shape-map) :_sub-instances @[]))
        (array/push (old-context :shape-map :_sub-instances)
          {:model model :result payload}))
      payload)
    (do
      # Purge any shapes that were created before the error
      (def purge-fn (get-c-fn 'purge))
      (each s (model :shapes)
        (when (= :rojcad/shape (type s))
          (purge-fn s)))
      (put model :shapes @[])
      (error (string "build error: " payload)))))

# ── Graph introspection ────────────────────────────────────────────────

(defn- walk-source [form]
  (def node-type
    (if (= :tuple (type form))
      (keyword (string (first form)))
      nil))
  (def children @[])
  (when (= :tuple (type form))
    (for i 1 (length form)
      (def child (form i))
      (if (= :tuple (type child))
        (array/push children (walk-source child))
        (array/push children @{:type :atom :value child}))))
  (if node-type
    @{:type node-type :children children :form form :id (gensym)}
    @{:type :atom :value form}))

(defn- flatten-nodes [tree]
  (def nodes @[])
  (defn walk [node parent-path]
    (def my-path @[])
    (each p parent-path (array/push my-path p))
    (array/push my-path (length nodes))
    (put node :path my-path)
    (array/push nodes node)
    (each child (node :children)
      (when (child :children)
        (walk child my-path)))
    nil)
  (walk tree @[])
  nodes)

(defn graph [model]
  (when (not= :table (type model))
    (error "graph: expected a model record"))
  (def source (model :source))
  (def tree (walk-source source))
  (def nodes (flatten-nodes tree))

  @{:name (first source)
    :params (model :params)
    :current (model :current-params)
    :nodes nodes
    :shape-map (model :shape-map)})

# ── Highlight API ─────────────────────────────────────────────────────
# Track all shapes shown by highlight() so highlight-clear() can hide them.

(var hl-shapes @[])

(defn- hl-register [model shape]
  (show shape)
  (unless (get model :_hl-shapes)
    (put model :_hl-shapes @[]))
  (array/push (model :_hl-shapes) shape)
  (array/push hl-shapes shape)
  ((get-c-fn '_highlight-shape) shape))

(defn highlight [model &opt part-id]
  (when (not= :table (type model))
    (error "highlight: expected a model record"))
  (def shape-map (model :shape-map))
  (when (empty? shape-map)
    (error "highlight: model has not been built yet, call (build ...) first"))
  (if (nil? part-id)
    (when (model :result)
      (hl-register model (model :result)))
    (do
      (def shape (get shape-map part-id))
      (if shape
        (hl-register model shape)
        (error (string "highlight: part " part-id " not found in model"))))))

(defn highlight-clear [&opt model part-id]
  (cond
    (= nil model)
    (do
      (each s hl-shapes (hide s))
      (array/clear hl-shapes))
    (= nil part-id)
    (do
      (each s (get model :_hl-shapes) (hide s))
      (put model :_hl-shapes @[])
      (array/clear hl-shapes))
    (do
      (def shape (get (model :shape-map) part-id))
      (when shape
        (hide shape)
        # Remove from per-model list
        (def new-shapes @[])
        (each s (get model :_hl-shapes)
          (when (not= s shape) (array/push new-shapes s)))
        (put model :_hl-shapes new-shapes)
        # Remove from global list
        (def new-global @[])
        (each s hl-shapes
          (when (not= s shape) (array/push new-global s)))
        (set hl-shapes new-global))))
  ((get-c-fn '_highlight-clear)))

# ── Documentation metadata ──────────────────────────────────────────
# Tag each function for the doc generation pipeline.

(setmeta 'defmodel "parametric-models"
  "(defmodel name [params...] &keys :parts :result body...)\n\nDefine a parametric model. Binds a model record with :params, :body-fn, :source, :parts, :shapes, :shape-map, :current-params, and :result fields.\n\nKeywords: :parts (table of named parts {:part-name expr ...}),\n         :result (result expression, defaults to last body expression)\n\nWhen :parts is provided, a `parts` local variable is bound to the\ntable of built shapes. The model's :result expression produces the\nfinal shape from these parts.\n\nModel records are pure Janet tables — they can be inspected,\npassed to `build`, and introspected with `graph`.\n\nExamples:\n  (defmodel bracket [w h]\n    :parts {:base (box w h 30) :hole (cylinder 5 30)}\n    :result (cut base hole))\n  (defmodel cube [s]\n    (box s s s))\n\nReturns nil (binds a model record as a side effect).")

(setmeta 'build "parametric-models"
  "(build model & params)\n\nInstantiate a parametric model by executing its body with the\ngiven parameter values. Old shapes from previous builds are\npurged automatically. Uses the existing `my-eval` shape-binding\nmechanism for auto-purge on re-def.\n\nAccepts a model record followed by positional parameter values\nmatching the model's :params vector.\n\nExamples:\n  (def br (build bracket 100 50))\n  (def br2 (build bracket 200 80))\n\nReturns a rojcad/shape abstract value. Signals an error on\nparameter count mismatch or model execution failure.")

(setmeta 'graph "parametric-models"
  "(graph model)\n\nReturn the structure of a parametric model as a table.\nThe returned table has :name, :params, :current, :nodes,\nand :shape-map fields. The :nodes array contains source-form\nAST nodes with :type, :children, :form, and :id fields.\nEach node in a built model maps to its shape via :shape-map.\n\nExamples:\n  (graph bracket)\n\nReturns a table.")

(setmeta 'highlight "parametric-models"
  "(highlight model &opt part-id)\n\nHighlight a named part of a built model in the viewer.\nThe shape is shown (registered in the viewer) and rendered\nwith active edges and a tinted mesh overlay.\nWithout part-id, the entire result shape is highlighted.\n\nPart-ids correspond to keys in the model's :parts table\nor :result for the final output shape.\n\nExamples:\n  (highlight bracket :base)\n  (highlight bracket)\n\nReturns nil. Signals an error if the model has not been built.")

(setmeta 'highlight-clear "parametric-models"
  "(highlight-clear &opt model part-id)\n\nRemove highlighting and hide previously highlighted shapes.\n\nCall variants:\n  (highlight-clear)                  — clear viewer highlight only\n  (highlight-clear model)            — hide all highlighted parts\n  (highlight-clear model :part-name) — hide a specific part\n\nExamples:\n  (highlight-clear)\n  (highlight-clear bracket)\n  (highlight-clear bracket :base)\n\nReturns nil.")
