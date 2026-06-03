# boot.janet — TCP REPL server for rojcad
#
# Starts a Janet REPL on TCP port 9000 (loopback only).
# Each client gets their own REPL session via ev/spawn.
# Must be run after cad_register_functions has been called.

(def port 9000)
(def addr "127.0.0.1")

(defn connect-handler [stream]
  (eprint "● client connected")
  (defer (:close stream)
    (def env (fiber/getenv (fiber/current)))
    (while true
      (def line (string/trim (net/read stream 4096)))
      (when (or (= line nil) (= line "") (= line "")) (break))
      (def result
        (try
          (eval (parse line) env)
          ([e] (string "error: " e))))
      (when (not= result nil)
        (:write stream (string result "\n")))))
  (eprint "● client disconnected"))

# Start the server. On failure (e.g. port in use), print error and exit non-zero.
(def listen
  (try
    (net/listen addr port)
    ([e]
      (eprint "rojcad: failed to listen on " addr ":" port " — " e)
      (os/exit 1))))

(eprint "◆ rojcad ready — connect via: nc " addr " " port)

(forever
  (def conn (net/accept listen))
  (ev/spawn (connect-handler conn)))
