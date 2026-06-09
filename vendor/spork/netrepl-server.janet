(def default-host
  "Default host to run server on and connect to."
  "127.0.0.1")

(def default-port
  "Default port to run the net repl."
  "9365")

(defn- coerce-to-env
  "Get an environment for the repl."
  [env name stream]
  (cond
    (function? env) (env name stream)
    (not= nil env) env
    (let [e (make-env)]
      (put e :pretty-format "%.20Q"))))

(defn- serve-and-wait
  "Alternative to net/server that suspends the fiber until the server is closed."
  [host port handler]
  (with [s (net/listen host port)]
    (net/accept-loop s handler))
  nil)

(def- cmd-peg
  (peg/compile
    ~{:main (* :command (any (* :space :argument)))
      :space (some (set " \t"))
      :identifier (some :S)
      :command (/ ':identifier ,keyword)
      :argument (/ '(+ :quoted-arg :bare-arg) ,parse)
      :bare-arg :identifier
      :quoted-arg (* `"` (any (+ (* `\` 1) (if-not `"` 1))) `"`)}))

(defn- make-onsignal
  "Make an onsignal handler for debugging."
  [getter env e level]
  (defn enter-debugger
    [f x]
    (def nextenv (make-env env))
    (put nextenv :fiber f)
    (put nextenv :debug-level level)
    (put nextenv :signal x)
    (merge-into nextenv debugger-env)
    (debug/stacktrace f x "")
    (eflush)
    (defn debugger-chunks [buf p]
      (def status (parser/state p :delimiters))
      (def c ((parser/where p) 0))
      (def prpt (string "debug[" level "]:" c ":" status "> "))
      (getter prpt buf))
    (print "entering debug[" level "] - (quit) to exit")
    (flush)
    (repl debugger-chunks (make-onsignal getter env nextenv (+ 1 level)) nextenv)
    (print "exiting debug[" level "]")
    (flush)
    (nextenv :resume-value))
  (fn on-signal [f x]
    (case (fiber/status f)
      :dead (do (put e '_ @{:value x}) (pp x))
      (if (e :debug)
        (enter-debugger f x)
        (do (debug/stacktrace f x "") (eflush))))))

(defn- server-impl
  [server-ctor &opt host port env cleanup welcome-msg]
  (default host default-host)
  (default port default-port)
  (def main-env (curenv))
  (eprint "Starting netrepl server on " host ", port " port "...")
  (def efile (dyn *err* stderr))
  (def name-set @{})
  (def all-connections @{})

  (defn set-title
    []
    (put main-env :title (string/format "%d connections" (length all-connections))))
  (set-title)

  (defn disconnect-stream
    [stream]
    (def name (get all-connections stream))
    (unless name (break))
    (put all-connections stream nil)
    (set-title)
    (protect (:write stream ""))
    (protect (:close stream))
    (unless (= name stream)
      (put name-set name nil))
    (protect (xprint efile "closing client " name))
    (when cleanup (cleanup stream)))

  (defn disconnect-all
    []
    (eachk stream all-connections
      (disconnect-stream stream)))

  (defn repl-handler [stream]
    (var name "<unknown>")
    (put all-connections stream name)
    (set-title)
    (var last-flush 0)
    (def outbuf @"")
    (def nurse (nursery))
    (defn wrapio [f] (fn [& a] (with-dyns [:out outbuf :err outbuf] (f ;a))))
    (def recv (make-recv stream))
    (def send (make-send stream))
    (var auto-flush false)
    (var is-first true)
    (var keep-flushing false)

    (defn flush1
      []
      (def now (os/clock))
      (when (or (next outbuf) (< (+ 2 last-flush) now))
        (def msg (string "\xFF" outbuf))
        (buffer/clear outbuf)
        (send msg)
        (set last-flush now)))

    (defn flusher
      []
      (ev/sleep 0)
      (while keep-flushing
        (flush1)
        (ev/sleep 0.1)))

    (defn get-name
      []
      (def msg (recv))
      (def leader (get msg 0))
      (if (= 0xFF leader)
        (let [opts (-> msg (slice 1) parse)]
          (set auto-flush (get opts :auto-flush))
          (set name (get opts :name)))
        (set name msg)))

    (defn getline-async
      [prmpt buf]
      (if auto-flush
        (flush1)
        (if is-first
          (set is-first false)
          (let [b (get outbuf 0)]
            (when (or (= b 0xFF) (= b 0xFE))
              (buffer/blit outbuf outbuf 1 0 -1)
              (put outbuf 0 0xFE))
            (send outbuf)
            (buffer/clear outbuf))))
      (send prmpt)
      (var ret nil)
      (while (def msg (recv))
        (cond
          (= 0xFF (in msg 0))
          (send (string/format "%j" (-> msg (slice 1) parse eval protect)))
          (= 0xFE (in msg 0))
          (do
            (def cmd (peg/match cmd-peg msg 1))
            (if (one? (length cmd))
              (set ret (first cmd))
              (set ret cmd))
            (break))
          (do (buffer/push-string buf msg) (break))))
      ret)

    (defn chunk
      [buf p]
      (def delim (parser/state p :delimiters))
      (def lno ((parser/where p) 0))
      (getline-async (string name ":" lno ":" delim " ") buf))

    (spawn-nursery
      nurse
      (set name (or (get-name) (break)))
      (put all-connections stream name)
      (while (get name-set name)
        (set name (string name (gensym))))
      (put name-set name true)
      (xprint efile "client " name " connected")
      (def e
        (try (coerce-to-env env name stream)
          ([err fib]
            (xprint efile err)
            (debug/stacktrace fib "coerce-to-env failed" ""))))
      (def p (parser/new))

      (when (and welcome-msg auto-flush)
        (def msg
          (if (bytes? welcome-msg)
            welcome-msg
            (welcome-msg name)))
        (when msg
          (send (string/format "\xFF%s" msg))))

      (->
        (run-context
          {:env e
           :chunks chunk
           :on-status (make-onsignal getline-async e e 1)
           :on-compile-error (wrapio bad-compile)
           :on-parse-error (wrapio bad-parse)
           :evaluator
           (fn evaluate-wrapped [x source &]
              (setdyn :out outbuf)
              (setdyn :err outbuf)
              (def result
                (if auto-flush
                  (do
                    (set keep-flushing true)
                    (go-nursery nurse flusher)
                    (edefer (set keep-flushing false)
                      (def r (x))
                      (set keep-flushing false)
                      (flush1)
                      r))
                  (x)))
              (if (dyn :exit) (put e :exit true))
              (def check-source (try (macex source) ([e] source)))
              (when (and (= (type result) :rojcad/shape)
                         (= (type check-source) :tuple)
                         (>= (length check-source) 3)
                         (or (= (check-source 0) 'def) (= (check-source 0) 'set)))
                (show result))
              result)
            :source "repl"
            :parser p})
        coro
        (fiber/setenv (table/setproto @{:out outbuf :err outbuf :parser p} e))
        resume))

    (protect (join-nursery nurse))
    (disconnect-stream stream))

  (defer (disconnect-all) (server-ctor host port repl-handler)))

(defn- server-single-impl
  "Short-hand for serving up a repl that has a single environment table."
  [server-ctor &opt host port env cleanup welcome-msg]
  (def client-table @{})
  (def inverse-client-table @{})
  (let [e (coerce-to-env (or env (make-env)) nil nil)]
    (defn env-factory [name stream]
      (put client-table name stream)
      (put inverse-client-table stream name)
      e)
    (defn cleanup2 [stream]
      (when cleanup (cleanup stream))
      (def name (get inverse-client-table stream))
      (put client-table name nil)
      (put inverse-client-table stream nil))
    (put e :pretty-format "%.20Q")
    (put e :clients client-table)
    (server-impl server-ctor host port env-factory cleanup2 welcome-msg)))

(defn server
  "Start a netrepl server. The default host is \"127.0.0.1\" and the default port
  is \"9365\". Uses net/server for the accept loop."
  [&opt host port env cleanup welcome-msg]
  (server-impl net/server host port env cleanup welcome-msg))

(defn server-single
  "Start a netrepl server with a single environment shared across all connections."
  [&opt host port env cleanup welcome-msg]
  (server-single-impl net/server host port env cleanup welcome-msg))

(defn run-server
  "Start a netrepl server and wait until it finishes."
  [&opt host port env cleanup welcome-msg]
  (server-impl serve-and-wait host port env cleanup welcome-msg))

(defn run-server-single
  "Start a single-env netrepl server and wait until it finishes."
  [&opt host port env cleanup welcome-msg]
  (server-single-impl serve-and-wait host port env cleanup welcome-msg))
