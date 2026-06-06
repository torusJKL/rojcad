#!/usr/bin/env bash
# REPL integration tests for variadic CAD function wrappers.
# Starts rojcad headless and tests via --eval (full Janet boot path).
# Each test runs as a separate invocation and prints PASS/FAIL.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PASS=0
FAIL=0
FAILED_TESTS=""

# Ensure binary is up-to-date
echo ":: Building..."
(cd "$ROOT" && HOME=/tmp GIT_CONFIG_NOSYSTEM=1 CC=clang CXX=clang++ CARGO_HOME="$ROOT/.local-cargo" RUSTFLAGS=-Clinker=clang cargo build --quiet 2>/dev/null)
echo ""

run_test() {
    local name="$1"
    local code="$2"
    local expect_pass="${3:-true}"

    # Evaluate: each test code includes a final (os/exit <code>) where
    # code=0 means PASS, non-zero means FAIL.
    local full_code="$code (os/exit 0)"
    local output
    output=$(HOME=/tmp GIT_CONFIG_NOSYSTEM=1 CC=clang CXX=clang++ CARGO_HOME="$ROOT/.local-cargo" RUSTFLAGS=-Clinker=clang \
        timeout 30 "$ROOT/target/debug/rojcad" --headless --eval="$full_code" 2>&1 || true)

    local exit_code=$?

    if [ "$exit_code" -eq 0 ]; then
        echo "  PASS: $name"
        PASS=$((PASS + 1))
    else
        # Extract the error message if any (lines with "error:")
        local err_line
        err_line=$(echo "$output" | grep "error:" | head -1 || true)
        if [ -n "$err_line" ]; then
            echo "  FAIL: $name — $err_line"
        elif [ "$exit_code" -eq 124 ]; then
            echo "  FAIL: $name — timed out"
        else
            echo "  FAIL: $name — exit code $exit_code"
        fi
        FAIL=$((FAIL + 1))
        FAILED_TESTS="$FAILED_TESTS  - $name"$'\n'
    fi
}

# ── Side-effect tests ──────────────────────────────────────────────────

echo ":: Side-effects (hide, show, purge)"

run_test "hide three shapes" '
(def a (box 10)) (def b (sphere 5)) (def c (cylinder 3 8))
(def v1 (visible? a b c))
(hide a b c)
(def v2 (visible? a b c))
(if (and (= false (get v2 0)) (= false (get v2 1)) (= false (get v2 2)))
  (print "PASS") (do (print "FAIL: " v2) (os/exit 1)))
'

run_test "show three shapes" '
(def a (box 10)) (def b (sphere 5)) (def c (cylinder 3 8))
(hide a b c)
(show a b c)
(def v (visible? a b c))
(if (and (= true (get v 0)) (= true (get v 1)) (= true (get v 2)))
  (print "PASS") (do (print "FAIL: " v) (os/exit 1)))
'

run_test "zero-arg hide returns nil" '
(def r (hide))
(if (= nil r) (print "PASS") (do (print "FAIL: " r) (os/exit 1)))
'

run_test "single-arg hide still works" '
(def a (box 10))
(hide a)
(def v (visible? a))
(if (= false (get v 0)) (print "PASS") (do (print "FAIL: " v) (os/exit 1)))
'

# ── Query tests ────────────────────────────────────────────────────────

echo ""
echo ":: Queries (shape-type, visible?, wire?, face?, solid?)"

run_test "shape-type two solids" '
(def a (box 10)) (def b (sphere 5))
(def t (shape-type a b))
(if (and (= (length t) 2) (= :solid (get t 0)) (= :solid (get t 1)))
  (print "PASS") (do (print "FAIL: " t) (os/exit 1)))
'

run_test "shape-type mixed types" '
(def wire-rect (rect 10 20 :wire))
(def face-rect (rect 10 20))
(def solid-box (box 10))
(def t (shape-type wire-rect face-rect solid-box))
(if (and (= (length t) 3) (= :wire (get t 0)) (= :face (get t 1)) (= :solid (get t 2)))
  (print "PASS") (do (print "FAIL: " t) (os/exit 1)))
'

run_test "shape-type single returns tuple" '
(def a (box 10))
(def t (shape-type a))
(if (and (= (length t) 1) (= :solid (get t 0)))
  (print "PASS") (do (print "FAIL: " t) (os/exit 1)))
'

run_test "shape-type zero args returns empty" '
(def t (shape-type))
(if (= (length t) 0) (print "PASS") (do (print "FAIL: length " (length t)) (os/exit 1)))
'

run_test "visible? two shapes" '
(def a (box 10)) (def b (sphere 5)) (hide b)
(def v (visible? a b))
(if (and (= true (get v 0)) (= false (get v 1)))
  (print "PASS") (do (print "FAIL: " v) (os/exit 1)))
'

run_test "wire? face? solid? mixed" '
(def w (rect 10 20 :wire))
(def f (rect 10 20))
(def s (box 10))
(def wr (wire? w f s)) (def fr (face? w f s)) (def sr (solid? w f s))
(if (and (= true (get wr 0)) (= false (get wr 1)) (= false (get wr 2))
         (= false (get fr 0)) (= true (get fr 1)) (= false (get fr 2))
         (= false (get sr 0)) (= false (get sr 1)) (= true (get sr 2)))
  (print "PASS") (do (print "FAIL") (os/exit 1)))
'

# ── Boolean tests ──────────────────────────────────────────────────────

echo ""
echo ":: Booleans (cut, common, fuse)"

run_test "cut two operands" '
(def tool (box 30))
(def a (sphere 10 :c [15 15 15]))
(def b (sphere 8 :c [15 15 15]))
(def result (cut tool a b))
(def t (shape-type result))
(if (= :compound (get t 0)) (print "PASS") (do (print "FAIL: " t) (os/exit 1)))
'

run_test "cut single operand (backward compat)" '
(def tool (box 30))
(def a (sphere 10 :c [15 15 15]))
(def result (cut tool a))
(def t (shape-type result))
(if (= :compound (get t 0)) (print "PASS") (do (print "FAIL: " t) (os/exit 1)))
'

run_test "cut zero operands returns tool" '
(def tool (box 10))
(def result (cut tool))
(if (= :rojcad/shape (type result))
  (print "PASS") (do (print "FAIL: type " (type result)) (os/exit 1)))
'

run_test "fuse three shapes" '
(def a (box 10)) (def b (box 10 :c [5 5 5])) (def c (box 10 :c [10 10 10]))
(def result (fuse a b c))
(def t (shape-type result))
(if (= :compound (get t 0)) (print "PASS") (do (print "FAIL: " t) (os/exit 1)))
'

run_test "common three shapes" '
(def a (box 10)) (def b (box 10 :c [5 5 5])) (def c (box 10 :c [10 10 10]))
(def result (common a b c))
(if (= :rojcad/shape (type result))
  (print "PASS") (do (print "FAIL: type " (type result)) (os/exit 1)))
'

run_test "fuse single returns shape" '
(def a (box 10))
(def result (fuse a))
(if (= (type result) (type a)) (print "PASS") (do (print "FAIL") (os/exit 1)))
'

# ── Discovery tool tests ───────────────────────────────────────────────

echo ""
echo ":: Discovery tools (doc, all-fns, apropos, cad-fns, group)"

run_test "doc hide returns string" '
(def d (doc "hide"))
(if (= :string (type d)) (print "PASS") (do (print "FAIL: type " (type d)) (os/exit 1)))
'

run_test "all-fns includes hide" '
(def cf (all-fns)) (var found false) (var i 0)
(while (< i (length cf))
  (if (= "hide" (string (get cf i))) (set found true)) (set i (+ i 1)))
(if found (print "PASS") (do (print "FAIL") (os/exit 1)))
'

run_test "all-fns includes cut" '
(def cf (all-fns)) (var found false) (var i 0)
(while (< i (length cf))
  (if (= "cut" (string (get cf i))) (set found true)) (set i (+ i 1)))
(if found (print "PASS") (do (print "FAIL") (os/exit 1)))
'

run_test "all-fns includes shape-type" '
(def cf (all-fns)) (var found false) (var i 0)
(while (< i (length cf))
  (if (= "shape-type" (string (get cf i))) (set found true)) (set i (+ i 1)))
(if found (print "PASS") (do (print "FAIL") (os/exit 1)))
'

run_test "all-fns includes visible?" '
(def cf (all-fns)) (var found false) (var i 0)
(while (< i (length cf))
  (if (= "visible?" (string (get cf i))) (set found true)) (set i (+ i 1)))
(if found (print "PASS") (do (print "FAIL") (os/exit 1)))
'

run_test "apropos finds cut" '
(def ap (apropos "cut")) (var found false) (var i 0)
(while (< i (length ap))
  (if (= "cut" (string (get ap i))) (set found true)) (set i (+ i 1)))
(if found (print "PASS") (do (print "FAIL") (os/exit 1)))
'

run_test "cad-fns includes hide" '
(def cf (cad-fns)) (var found false) (var i 0)
(while (< i (length cf))
  (if (= "hide" (string (get cf i))) (set found true)) (set i (+ i 1)))
(if found (print "PASS") (do (print "FAIL") (os/exit 1)))
'

run_test "cad-fns includes cut" '
(def cf (cad-fns)) (var found false) (var i 0)
(while (< i (length cf))
  (if (= "cut" (string (get cf i))) (set found true)) (set i (+ i 1)))
(if found (print "PASS") (do (print "FAIL") (os/exit 1)))
'

run_test "group booleans includes cut common fuse" '
(def g (group "booleans")) (var found-cut false) (var found-common false) (var found-fuse false) (var i 0)
(while (< i (length g))
  (def s (string (get g i)))
  (if (= s "cut") (set found-cut true))
  (if (= s "common") (set found-common true))
  (if (= s "fuse") (set found-fuse true))
  (set i (+ i 1)))
(if (and found-cut found-common found-fuse) (print "PASS") (do (print "FAIL") (os/exit 1)))
'

run_test "group registry includes hide show purge" '
(def g (group "registry")) (var found-hide false) (var found-show false) (var found-purge false) (var i 0)
(while (< i (length g))
  (def s (string (get g i)))
  (if (= s "hide") (set found-hide true))
  (if (= s "show") (set found-show true))
  (if (= s "purge") (set found-purge true))
  (set i (+ i 1)))
(if (and found-hide found-show found-purge)
  (print "PASS") (do (print "FAIL: " g) (os/exit 1)))
'

# ── TCP REPL test ──────────────────────────────────────────────────────

echo ""
echo ":: TCP REPL interaction"

# Start server on default port (--port flag has a pre-existing dyn timing bug)
PORT=9365
"$ROOT/target/debug/rojcad" --headless &
SERVER_PID=$!
sleep 3

RESP=$(echo "(+ 1 2)" | timeout 3 nc -w 2 127.0.0.1 "$PORT" 2>/dev/null || true)
kill "$SERVER_PID" 2>/dev/null || true
wait "$SERVER_PID" 2>/dev/null || true

if [ "$RESP" = "3" ]; then
    echo "  PASS: REPL returns 3 for (+ 1 2)"
    PASS=$((PASS + 1))
else
    echo "  FAIL: REPL expected '3', got '$RESP'"
    FAIL=$((FAIL + 1))
    FAILED_TESTS="$FAILED_TESTS  - REPL returns 3 for (+ 1 2)"$'\n'
fi

# ── Results ────────────────────────────────────────────────────────────

echo ""
echo "═══════════════════════════════════════════"
echo "  Results: $PASS passed, $FAIL failed"
echo "═══════════════════════════════════════════"

if [ "$FAIL" -gt 0 ]; then
    echo ""
    echo "Failed tests:"
    echo "$FAILED_TESTS"
    exit 1
fi
