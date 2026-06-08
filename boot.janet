# boot.janet — TCP REPL server for rojcad

(def try-catch (fn [body err-handler]
  (def f (fiber/new body :e))
  (def result (resume f))
  (if (= (fiber/status f) :error)
    (err-handler result)
    result)))

(def core-env (fiber/getenv (fiber/current)))

# ── Variadic wrappers ───────────────────────────────────────────────────────
# Wraps C functions to accept multiple shapes. Uses table mutation to
# preserve metadata (doc, source, category) for discovery tools.

# ── Side-effects ──

(def t (get core-env 'hide))
(def _hide (t :value))
(put t :value (fn [& shapes]
  (var i 0) (def n (length shapes))
  (while (< i n) (_hide (shapes i)) (set i (+ i 1)))))

(def t (get core-env 'show))
(def _show (t :value))
(put t :value (fn [& shapes]
  (var i 0) (def n (length shapes))
  (while (< i n) (_show (shapes i)) (set i (+ i 1)))))

(def t (get core-env 'purge))
(def _purge (t :value))
(put t :value (fn [& shapes]
  (var i 0) (def n (length shapes))
  (while (< i n) (_purge (shapes i)) (set i (+ i 1)))))

(def t (get core-env 'registry-remove))
(def _registry-remove (t :value))
(put t :value (fn [& shapes]
  (var i 0) (def n (length shapes))
  (while (< i n) (_registry-remove (shapes i)) (set i (+ i 1)))))

# ── Queries ──

(def t (get core-env 'shape-type))
(def _shape-type (t :value))
(put t :value (fn [& shapes]
  (def results @[])
  (var i 0) (def n (length shapes))
  (while (< i n) (array/push results (_shape-type (shapes i))) (set i (+ i 1)))
  results))

(def t (get core-env 'visible?))
(def _visible (t :value))
(put t :value (fn [& shapes]
  (def results @[])
  (var i 0) (def n (length shapes))
  (while (< i n) (array/push results (_visible (shapes i))) (set i (+ i 1)))
  results))

(def t (get core-env 'wire?))
(def _wire (t :value))
(put t :value (fn [& shapes]
  (def results @[])
  (var i 0) (def n (length shapes))
  (while (< i n) (array/push results (_wire (shapes i))) (set i (+ i 1)))
  results))

(def t (get core-env 'face?))
(def _face (t :value))
(put t :value (fn [& shapes]
  (def results @[])
  (var i 0) (def n (length shapes))
  (while (< i n) (array/push results (_face (shapes i))) (set i (+ i 1)))
  results))

(def t (get core-env 'solid?))
(def _solid (t :value))
(put t :value (fn [& shapes]
  (def results @[])
  (var i 0) (def n (length shapes))
  (while (< i n) (array/push results (_solid (shapes i))) (set i (+ i 1)))
  results))

# ── Booleans (chain + keyword routing) ──

(def t (get core-env 'cut))
(def _cut (t :value))
(put t :value (fn [tool & rest]
  (var result tool)
  (var shapes @[])
  (var eager? false)
  (var j 0) (def m (length rest))
  (while (< j m)
    (def x (rest j))
    (if (= :keyword (type x))
      (if (= x :eager) (set eager? true))
      (array/push shapes x))
    (set j (+ j 1)))
  (def n (length shapes))
  (if (> n 0)
    (do
      (var k 0) (def l (- n 1))
      (while (< k l)
        (set result (_cut result (shapes k)))
        (set k (+ k 1)))
      (def last-b (shapes (- n 1)))
      (if eager?
        (set result (_cut result last-b :eager))
        (set result (_cut result last-b)))))
  result))

(def t (get core-env 'common))
(def _common (t :value))
(put t :value (fn [first & rest]
  (var result first)
  (var shapes @[])
  (var eager? false)
  (var j 0) (def m (length rest))
  (while (< j m)
    (def x (rest j))
    (if (= :keyword (type x))
      (if (= x :eager) (set eager? true))
      (array/push shapes x))
    (set j (+ j 1)))
  (def n (length shapes))
  (if (> n 0)
    (do
      (var k 0) (def l (- n 1))
      (while (< k l)
        (set result (_common result (shapes k)))
        (set k (+ k 1)))
      (def last-b (shapes (- n 1)))
      (if eager?
        (set result (_common result last-b :eager))
        (set result (_common result last-b)))))
  result))

(def t (get core-env 'fuse))
(def _fuse (t :value))
(put t :value (fn [first & rest]
  (var result first)
  (var shapes @[])
  (var eager? false)
  (var j 0) (def m (length rest))
  (while (< j m)
    (def x (rest j))
    (if (= :keyword (type x))
      (if (= x :eager) (set eager? true))
      (array/push shapes x))
    (set j (+ j 1)))
  (def n (length shapes))
  (if (> n 0)
    (do
      (var k 0) (def l (- n 1))
      (while (< k l)
        (set result (_fuse result (shapes k)))
        (set k (+ k 1)))
      (def last-b (shapes (- n 1)))
      (if eager?
        (set result (_fuse result last-b :eager))
        (set result (_fuse result last-b)))))
  result))

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
    (if (= :tuple t)
      (if (= 0 (length x))
        "()"
        (do
          (def parts @[])
          (var i 0)
          (def n (length x))
          (while (< i n)
            (array/push parts (string (get x i)))
            (set i (+ i 1)))
          (string/join parts "\n")))
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
        (string x))))))

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
      (array/push fns k)
      (if (= "rojcad" (get v :source))
        (array/push fns k)))
    (set k (next core-env k)))
  (sort-syms fns)))

(def apropos (fn [pat]
  (def fns @[])
  (var k (next core-env nil))
  (while k
    (def v (get core-env k))
    (if (= :cfunction (type (get v :value)))
      (if (string/find pat (string k))
        (array/push fns k))
      (if (= "rojcad" (get v :source))
        (if (string/find pat (string k))
          (array/push fns k))))
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
   "edge-styling" "Edge Styling"
   "view" "View"
   "2d-primitives" "2D Primitives"
   "operations" "Operations"
   "wire-operations" "Wire Operations"
   "sketch" "Sketch"
   "text" "Text"})

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

(def shape-bindings @{})

(def my-eval (fn [form _env]
  (def compiled (compile form core-env))
  (if (= (type compiled) :function)
    (do
      (def f0 (if (= (type form) :tuple) (get form 0) nil))
      (def f2 (if (= (type form) :tuple) (get form 2) nil))
      (def old-val
        (if (= f0 'def)
          (get shape-bindings (get form 1))
          (if (= f0 'set)
            (get shape-bindings (get form 1))
            nil)))
      (def result (resume (fiber/new compiled)))
      (if (not= old-val nil)
        (if (= (type old-val) :rojcad/shape)
          (if (not= old-val result)
            (_purge old-val))))
      (if (not= f0 nil)
        (if (= f0 'def)
          (do
            (if (= (type result) :rojcad/shape)
              (put shape-bindings (get form 1) result)
              (put shape-bindings (get form 1) nil)))
          (if (= f0 'set)
            (do
              (if (= (type result) :rojcad/shape)
                (put shape-bindings (get form 1) result)
                (put shape-bindings (get form 1) nil))))))
      (if (= f0 'def)
        (if (not= f2 nil)
          (if (= (type result) :rojcad/shape)
            (if (visible? result)
              (show result))))
        (if (= f0 'set)
          (if (not= f2 nil)
            (if (= (type result) :rojcad/shape)
              (if (visible? result)
                (show result))))))
      result)
    (string "compile error: " (get compiled :error) " line:" (get compiled :line)))))

(def port (if (dyn '*netrepl-port*) (dyn '*netrepl-port*) 9365))

# ── Doc generation ────────────────────────────────────────────────────────

(def html-escape (fn [s]
  (string/replace "&" "&amp;" (string/replace "<" "&lt;" (string/replace ">" "&gt;" s)))))

(def bool-and (fn [a b] (if a b false)))
(def bool-or (fn [a b] (if a true b)))

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

(def split-docstring (fn [doc]
  (def parts (string/split "\n\n" doc))
  (def usage (string/trim (get parts 0)))
  (def body @[])
  (var examples nil)
  (var returns nil)
  (var in-examples false)
  (var pi 1)
  (def np (length parts))
  (while (< pi np)
    (def p (get parts pi))
    (def t (string/trim p))
    (if (not= nil (string/find "Examples:" t))
      (do
        (set in-examples true)
        (def ex-lines (string/split "\n" p))
        (def ex-code @[])
        (var li 0)
        (def nl (length ex-lines))
        (while (< li nl)
          (def tl (string/trim (get ex-lines li)))
          (if (bool-and (> (length tl) 0) (not= tl "Examples:"))
            (array/push ex-code tl))
          (set li (+ li 1)))
        (set examples (string/join ex-code "\n")))
      (if (not= nil (string/find "Returns" t))
        (do
          (set returns t)
          (set in-examples false))
        (if in-examples
          (do
            (def ex-lines (string/split "\n" p))
            (var li 0)
            (def nl (length ex-lines))
            (while (< li nl)
              (def tl (string/trim (get ex-lines li)))
              (if (> (length tl) 0)
                (set examples (string examples "\n" tl)))
              (set li (+ li 1))))
          (array/push body p))))
    (set pi (+ pi 1)))
  (array usage (string/join body "\n\n") examples returns)))

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

(def dump-docs (fn [&opt path version]
  (def dir (if path path "doc"))
  (try-catch (fn [] (os/mkdir dir)) (fn [e] nil))
  (def md-path (string dir "/janet-api.md"))
  (def html-path (string dir "/janet-api.html"))
  (gen-markdown md-path version)
  (gen-html html-path version)
  (string "Documentation written to " dir "/")))
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

# ── View angle presets ──────────────────────────────────────

(def view-front
  (fn [&opt distance]
    (if distance
      (view-angle (/ math/pi 2) 0 distance)
      (view-angle (/ math/pi 2) 0))))

(def view-back
  (fn [&opt distance]
    (if distance
      (view-angle (- (/ math/pi 2)) 0 distance)
      (view-angle (- (/ math/pi 2)) 0))))

(def view-right
  (fn [&opt distance]
    (if distance
      (view-angle 0 0 distance)
      (view-angle 0 0))))

(def view-left
  (fn [&opt distance]
    (if distance
      (view-angle math/pi 0 distance)
      (view-angle math/pi 0))))

(def view-top
  (fn [&opt distance]
    (if distance
      (view-angle 0 (/ math/pi 2) distance)
      (view-angle 0 (/ math/pi 2)))))

(def view-bottom
  (fn [&opt distance]
    (if distance
      (view-angle 0 (- (/ math/pi 2)) distance)
      (view-angle 0 (- (/ math/pi 2))))))

(def view-iso
  (fn [&opt distance]
    (if distance
      (view-angle (/ math/pi 4) (math/asin (/ 1 (math/sqrt 3))) distance)
      (view-angle (/ math/pi 4) (math/asin (/ 1 (math/sqrt 3)))))))

# Set metadata and docstrings for discoverability
(put (get core-env 'view-angle) :source "rojcad")
(put (get core-env 'view-angle) :category "view")
(put (get core-env 'view-front) :source "rojcad")
(put (get core-env 'view-front) :category "view")
(put (get core-env 'view-front) :doc
  (string "(view-front ; distance)\n\n"
          "Set camera to front view (looking along +Z toward origin).\n"
          "Yaw=π/2, Pitch=0. Animates over 0.5s.\n"
          "Optional distance sets zoom level; omitted preserves current.\n\n"
          "Examples:\n"
          "  (view-front)\n"
          "  (view-front 200)"))
(put (get core-env 'view-back) :source "rojcad")
(put (get core-env 'view-back) :category "view")
(put (get core-env 'view-back) :doc
  (string "(view-back ; distance)\n\n"
          "Set camera to back view (looking along -Z toward origin).\n"
          "Yaw=-π/2, Pitch=0. Animates over 0.5s.\n"
          "Optional distance sets zoom level; omitted preserves current.\n\n"
          "Examples:\n"
          "  (view-back)\n"
          "  (view-back 200)"))
(put (get core-env 'view-right) :source "rojcad")
(put (get core-env 'view-right) :category "view")
(put (get core-env 'view-right) :doc
  (string "(view-right ; distance)\n\n"
          "Set camera to right view (looking along +X toward origin).\n"
          "Yaw=0, Pitch=0. Animates over 0.5s.\n"
          "Optional distance sets zoom level; omitted preserves current.\n\n"
          "Examples:\n"
          "  (view-right)\n"
          "  (view-right 200)"))
(put (get core-env 'view-left) :source "rojcad")
(put (get core-env 'view-left) :category "view")
(put (get core-env 'view-left) :doc
  (string "(view-left ; distance)\n\n"
          "Set camera to left view (looking along -X toward origin).\n"
          "Yaw=π, Pitch=0. Animates over 0.5s.\n"
          "Optional distance sets zoom level; omitted preserves current.\n\n"
          "Examples:\n"
          "  (view-left)\n"
          "  (view-left 200)"))
(put (get core-env 'view-top) :source "rojcad")
(put (get core-env 'view-top) :category "view")
(put (get core-env 'view-top) :doc
  (string "(view-top ; distance)\n\n"
          "Set camera to top view (looking along +Y toward origin).\n"
          "Yaw=0, Pitch=π/2. Animates over 0.5s.\n"
          "Optional distance sets zoom level; omitted preserves current.\n\n"
          "Examples:\n"
          "  (view-top)\n"
          "  (view-top 200)"))
(put (get core-env 'view-bottom) :source "rojcad")
(put (get core-env 'view-bottom) :category "view")
(put (get core-env 'view-bottom) :doc
  (string "(view-bottom ; distance)\n\n"
          "Set camera to bottom view (looking along -Y toward origin).\n"
          "Yaw=0, Pitch=-π/2. Animates over 0.5s.\n"
          "Optional distance sets zoom level; omitted preserves current.\n\n"
          "Examples:\n"
          "  (view-bottom)\n"
          "  (view-bottom 200)"))
(put (get core-env 'view-iso) :source "rojcad")
(put (get core-env 'view-iso) :category "view")
(put (get core-env 'view-iso) :doc
  (string "(view-iso ; distance)\n\n"
          "Set camera to isometric view (looking from (1,1,1) direction).\n"
          "Yaw=π/4, Pitch=asin(1/√3) ≈ 0.615 rad. Animates over 0.5s.\n"
          "Optional distance sets zoom level; omitted preserves current.\n\n"
          "Examples:\n"
          "  (view-iso)\n"
          "  (view-iso 150)"))

(def poll-viewer (fn []
  (while true
    (if (quit-requested) (os/exit 0))
    (def event (poll-selection))
    (if (not= event nil)
      (if (= :tuple (type event))
        (eprint "■ deselected: " (in event 1))
        (if (= event :deselected)
          (eprint "■ deselected all")
          (eprint "■ selected: " event))))
    (ev/sleep 0.1))))

(ev/go (fiber/new poll-viewer))

(def accept-loop (fn []
  (while true
    (def conn (net/accept listen))
    (ev/go (fn [] (connect-handler conn))))))

(ev/go (fiber/new accept-loop))
