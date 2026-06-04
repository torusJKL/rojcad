# boot.janet — TCP REPL server for rojcad

(def try-catch (fn [body err-handler]
  (def f (fiber/new body))
  (def result (resume f))
  (if (= (fiber/status f) :error)
    (err-handler result)
    result)))

(def core-env (fiber/getenv (fiber/current)))

(def my-parse (fn [str]
  (def p (parser/new))
  (parser/consume p str)
  (parser/produce p)))

(def my-eval (fn [form _env]
  (def compiled (compile form core-env))
  (if (= (type compiled) :function)
    (resume (fiber/new compiled))
    (string "compile error: " (get compiled :error) " line:" (get compiled :line)))))
(def port (if (dyn '*netrepl-port*) (dyn '*netrepl-port*) 9365))
(def addr "127.0.0.1")

(def connect-handler (fn [stream]
  (eprint "● client connected")
  (def env (fiber/getenv (fiber/current)))
  (while true
    (def line-raw (try-catch (fn [] (net/read stream 4096)) (fn [e] nil)))
    (if (= line-raw nil) (break))
    (def line (string/trim line-raw))
    (if (= line "") (break))
    (def parsed (my-parse line))
    (if (not= parsed nil)
      (do
        (def eval-result (my-eval parsed env))
        (def result-str (string eval-result))
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
    (ev/go (fiber/new (fn [] (connect-handler conn)))))))

(ev/go (fiber/new accept-loop))
