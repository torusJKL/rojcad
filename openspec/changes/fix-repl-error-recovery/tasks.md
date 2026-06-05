## 1. Fix `try-catch` error protection

- [x] 1.1 Change `(fiber/new body)` to `(fiber/new body :e)` in the `try-catch` function definition (boot.janet:3)

- [x] 2.1 Replace the naked `(def eval-result (my-eval parsed env))` call on line 48 with `(def eval-result (try-catch (fn [] (my-eval parsed env)) (fn [e] e)))` so evaluation errors are caught and returned to the client

- [x] 3.1 Change `(ev/go (fiber/new (fn [] (connect-handler conn))))` on line 80 to `(ev/go (fn [] (connect-handler conn)))` so `ev/go` applies error masking automatically

## 4. Verify

- [x] 4.1 Rebuild the project with `just build`
- [x] 4.2 Test that an arity error (e.g., `(show)`) returns `"arity mismatch, expected at least 1, got 0"` and subsequent commands (`(+ 3 4)`) return correct results (`7`)
- [x] 4.3 Test that `def` and evaluation work after a prior error: `(show)` → error, `(+ 1 2)` → `3`, `(def x 42)` → `42`
- [x] 4.4 Test that sending an empty line disconnects cleanly: server stderr shows `"● client disconnected"`
