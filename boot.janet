# boot.janet — TCP REPL server for rojcad

(def try-catch (fn [body err-handler]
  (def f (fiber/new body :e))
  (def result (resume f))
  (if (= (fiber/status f) :error)
    (err-handler result)
    result)))

(def core-env (fiber/getenv (fiber/current)))

# ── Display helper (array-aware string conversion) ─────────────────────────

(def display-val (fn [x]
  (def t (type x))
  (if (= :array t)
    (do
      (def parts @[])
      (var i 0)
      (def n (length x))
      (while (< i n)
        (array/push parts (string (get x i)))
        (set i (+ i 1)))
      (string/join parts "\n"))
    (if (= :table t)
      (do
        (def lines @[])
        (var k (next x nil))
        (while k
          (def val (get x k))
          (if (= :array (type val))
            (do
              (array/push lines (string k ":"))
              (var j 0)
              (def m (length val))
              (while (< j m)
                (array/push lines (string "  " (get val j)))
                (set j (+ j 1))))
            (array/push lines (string k " → " val)))
          (set k (next x k)))
        (string/join lines "\n"))
      (string x)))))

# ── REPL discoverability helpers ────────────────────────────────────────────

(def sort-syms (fn [arr]
  (def n (length arr))
  (var i 0)
  (while (< i n)
    (var j (+ i 1))
    (while (< j n)
      (def a (string (get arr i)))
      (def b (string (get arr j)))
      (if (> a b)
        (do
          (def tmp (get arr i))
          (put arr i (get arr j))
          (put arr j tmp)))
      (set j (+ j 1)))
    (set i (+ i 1)))
  arr))

(def all-fns (fn []
  (def fns @[])
  (var k (next core-env nil))
  (while k
    (def v (get core-env k))
    (if (= :cfunction (type (get v :value)))
      (array/push fns k))
    (set k (next core-env k)))
  (sort-syms fns)))

(def apropos (fn [pat]
  (def fns @[])
  (var k (next core-env nil))
  (while k
    (def v (get core-env k))
    (if (= :cfunction (type (get v :value)))
      (if (string/find pat (string k))
        (array/push fns k)))
    (set k (next core-env k)))
  (sort-syms fns)))

(def doc (fn [sym]
  (def binding (get core-env sym))
  (if (= :table (type binding))
    (do
      (def docs (get binding :doc))
      (if docs (string docs)
               (string "No documentation for " sym)))
    (string "No documentation for " sym))))

(def cad-fns (fn []
  (def fns @[])
  (var k (next core-env nil))
  (while k
    (def v (get core-env k))
    (if (= (get v :source) "rojcad")
      (array/push fns k))
    (set k (next core-env k)))
  (sort-syms fns)))

(def cad-groups
  {"primitives" "Primitives"
   "booleans" "Booleans"
   "transforms" "Transforms"
   "queries" "Queries"
   "registry" "Registry"
   "io" "I/O"
   "selection" "Selection"
   "edge-styling" "Edge Styling"})

(def group (fn [&opt category]
  (if category
    (do
      (def fns @[])
      (var k (next core-env nil))
      (while k
        (def v (get core-env k))
        (if (= (get v :category) category)
          (array/push fns k))
        (set k (next core-env k)))
      (sort-syms fns))
    (do
      (def result @{})
      (var k (next core-env nil))
      (while k
        (def v (get core-env k))
        (if (= (get v :source) "rojcad")
          (do
            (def raw-cat (get v :category))
(def cat (if raw-cat raw-cat "other"))
            (def arr (get result cat))
            (if arr
              (array/push arr k)
              (put result cat (array k)))))
        (set k (next core-env k)))
      (var gk (next result nil))
      (while gk
        (put result gk (sort-syms (get result gk)))
        (set gk (next result gk)))
      result))))

(def my-parse (fn [str]
  (def p (parser/new))
  (parser/consume p str)
  (parser/produce p)))

(def my-eval (fn [form _env]
  (def compiled (compile form core-env))
  (if (= (type compiled) :function)
    (do
      (def result (resume (fiber/new compiled)))
      (if (= (type form) :tuple)
        (do
          (def f0 (get form 0))
          (def f2 (get form 2))
          (if (= f0 'def)
            (if (not= f2 nil)
              (if (= (type result) :rojcad/shape)
                (if (visible? result)
                  (show result)))))))
      result)
    (string "compile error: " (get compiled :error) " line:" (get compiled :line)))))
(def port (if (dyn '*netrepl-port*) (dyn '*netrepl-port*) 9365))
(def addr "127.0.0.1")

(def connect-handler (fn [stream]
  (eprint "● client connected")
  (fiber/setenv (fiber/current) core-env)
  (def env core-env)
  (while true
    (def line-raw (try-catch (fn [] (net/read stream 4096)) (fn [e] nil)))
    (if (= line-raw nil) (break))
    (def line (string/trim line-raw))
    (if (= line "") (break))
    (def parsed (my-parse line))
    (if (not= parsed nil)
      (do
        (def eval-result (try-catch (fn [] (my-eval parsed env)) (fn [e] e)))
        (def result-str (display-val eval-result))
        (:write stream result-str)
        (:write stream "\n"))
      (:write stream "parse error\n")))
  (:close stream)
  (eprint "● client disconnected")))

(def listen
  (do
    (def listen-fiber (fiber/new (fn [] (net/listen addr port))))
    (def listen-val (resume listen-fiber))
    (def listen-status (fiber/status listen-fiber))
    (if (= listen-status :dead) listen-val
      (do (eprint "rojcad: failed to listen on " addr ":" port) (os/exit 1)))))

(eprint "◆ rojcad ready — connect via: nc " addr " " port)

(def poll-viewer (fn []
  (while true
    (def event (poll-selection))
    (if (not= event nil)
      (if (not= event :deselected)
        (eprint "■ selected: " event)
        (eprint "■ deselected")))
    (ev/sleep 0.1))))

(ev/go (fiber/new poll-viewer))

(def accept-loop (fn []
  (while true
    (def conn (net/accept listen))
    (ev/go (fn [] (connect-handler conn))))))

(ev/go (fiber/new accept-loop))
