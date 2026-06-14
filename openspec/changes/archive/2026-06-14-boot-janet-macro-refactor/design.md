## Context

`boot.janet` (1839 lines) was written before `upstream.janet` was available, so it avoids all standard Janet macros and uses low-level equivalents. Every pattern has a verbose form that upstream macros can now replace. The file has three structural layers:

```
Layer 1: C-function wrappers  (lines 12-667)  — capture C fns, replace with Janet wrappers
Layer 2: Thin re-exports      (lines 668-918) — metadata preservation, sketch/wire wrappers
Layer 3: Application logic    (lines 919-1839) — REPL, docs, presets, event loop
```

The refactoring touches all three layers differently: Layers 1-2 get macro substitutions (`defn`, `each`, `case`, `++`) preserving the capture-and-replace pattern; Layer 3 additionally gets structural changes (data-driven presets, `try` replacement, `seq` for collections).

## Goals / Non-Goals

**Goals:**
- Replace all `(def name (fn [...] ...))` with `defn` across the entire file
- Replace all manual collection `while` loops with `each`, `for`, or `seq`
- Replace all keyword-dispatch `if` chains with `case` in wrapper functions
- Replace all `(set x (+ x 1))` with `++`
- Replace `(if cond (do ...))` with `when` / `unless`
- Replace array-building loops with `seq` where index access is not needed
- Replace custom `try-catch` with upstream `try`
- Generate view-angle presets from a data table instead of 7 repeated function definitions
- Add helper macros for metadata setting and C-function wrapping

**Non-Goals:**
- No behavioral changes — all function signatures, return values, docstrings, metadata, error handling, and edge cases preserved identically
- No spec changes — this is pure implementation refactoring
- No structural reorganization — `boot.janet` sections remain in the same order
- No new CAD-specific macros for users — helper macros are internal boot-time only
- No changes outside `boot.janet`
- Not changing the C bridge (`bridge/bridge.c`), Rust source, or upstream.janet

## Decisions

### 1. Use `defn` over `(def name (fn [...] ...))` universally

Every function definition currently uses the verbose form. `defn` expands to exactly the same thing but is cleaner and standard Janet.

**Rationale**: Zero risk, purely syntactic. Standard Janet convention everywhere.

### 2. Use `each` for collection iteration, `for` for indexed, `seq` for accumulation

Three patterns identified:
- **Pure iteration** (no index needed): `hide`, `show`, `purge`, `registry-remove`, query functions → `each`
- **Index needed** (keyword parsers with `:key val` consumption): stays as `while` with `case` + `++`
- **Array building**: `selected-shapes`, `list-shapes`, `list-fonts`, query results → `seq`

**Rationale**: `each` is the most common Janet iteration form. `seq` eliminates manual array/push patterns. `for` would work generically but `each` is simpler for value iteration.

### 3. Use `case` for keyword dispatch in wrapper functions

Currently every wrapper (sphere, cone, box, cylinder, torus, extrude, revolve, etc.) has the same ~19-line keyword parser:

```janet
;; Before
(if (= :keyword (type (args i)))
  (do (def kw (args i))
    (if (= kw :eager) (set eager true))
    (if (= kw :hide) (set hide true))
    (if (= kw :r) (do (set radius (args (+ i 1))) (set i (+ i 1))))
    ...)
  (do (positional handling)))
(set i (+ i 1))
```

Becomes:

```janet
;; After
(if (= :keyword (type (args i)))
  (case (args i)
    :eager (set eager true)
    :hide (set hide true)
    :r (set radius (args (++ i))))
  (do (positional handling)))
(++ i)
```

**Alternative considered**: A custom `parse-kw` macro. Rejected because the keyword sets vary per function and positional-plus-keyword mixing is hard to abstract generically. `case` + `++` is clear enough.

### 4. Generate view-angle presets from data

The 7 preset functions (`view-front`, `view-back`, `view-right`, `view-left`, `view-top`, `view-bottom`, `view-iso`) are structurally identical except for yaw/pitch constants. Replace with a data table + generation loop:

```janet
(def view-presets
  {:front  (tuple (/ math/pi 2) 0 "looking along +Z toward origin")
   :back   (tuple (- (/ math/pi 2)) 0 "looking along -Z toward origin")
   :right  (tuple 0 0 "looking along +X toward origin")
   :left   (tuple math/pi 0 "looking along -X toward origin")
   :top    (tuple 0 (/ math/pi 2) "looking along +Y toward origin")
   :bottom (tuple 0 (- (/ math/pi 2)) "looking along -Y toward origin")
   :iso    (tuple (/ math/pi 4) (math/asin (/ 1 (math/sqrt 3)))
                  "looking from (1,1,1) direction")})

(each [name yaw pitch desc] view-presets
  (defn name [&opt distance] ...)
  ;; set metadata + docstring
  )
```

**Rationale**: Eliminates ~120 lines of near-identical function+docstring+metadata blocks. Single source of truth for angles. `document` preset table format is easy to extend.

### 5. Metadata helper macro

The `(put (get core-env 'sym) :source "rojcad")` + `(put (get core-env 'sym) :category cat)` + optional `(put (get core-env 'sym) :doc doc)` pattern repeats ~40 times:

```janet
(defmacro defmeta [sym cat &opt doc]
  ~(do
     (put (get core-env ',sym) :source "rojcad")
     (put (get core-env ',sym) :category ,cat)
     ,(when doc ~(put (get core-env ',sym) :doc ,doc))))
```

Usage: `(defmeta sphere "primitives" "...docstring...")` instead of 3-4 `put` lines.

### 6. C-function wrapping helper macro

The capture-and-replace pattern repeats ~30 times:

```janet
(def _hide ((get core-env 'hide) :value))
(put core-env 'hide @{:value (fn [& shapes]
  (each s shapes (_hide s)))})
```

A helper macro captures the pattern:

```janet
(defmacro wrap-c-fn [name orig & body]
  ~(do
     (def ~orig ((get core-env ',name) :value))
     (put core-env ',name @{:value (fn ,;body)})))
```

Usage: `(wrap-c-fn hide _hide [& shapes] (each s shapes (_hide s)))`

**Rationale**: The `orig` parameter (e.g., `_hide`) must be explicit because it's referenced in the body. A gensym wouldn't be referable. The macro saves 2 lines per function and makes the wrapping intent explicit.

### 7. Replace custom `try-catch` with upstream `try`

The removed lines 3-8:
```janet
(def try-catch (fn [body err-handler]
  (def f (fiber/new body :e))
  (def result (resume f))
  (if (= (fiber/status f) :error)
    (err-handler result)
    result)))
```

Replaced by the upstream `try` macro at call sites:
```janet
;; Before
(try-catch (fn [] (os/mkdir dir)) (fn [e] nil))

;; After
(try (os/mkdir dir) ([e] nil))
```

### 8. `++` everywhere

All `(set x (+ x 1))` → `(++ x)`. Also `(set pos-count (+ pos-count 1))` → `(++ pos-count)`.

### 9. Execution order

Refactoring proceeds top-to-bottom through the file. Each change is:
1. Macro substitution (mechanical)
2. Verify no whitespace/docstring changes beyond macro expansion
3. Run `just test-unit` after each batch

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| `case` in keyword parsers misses edge cases (unrecognized keyword causes nil vs if-chain which silently ignores) | `case` with unrecognized keyword returns nil — same as current behavior (the if-chain falls through). Verified by reading `case` macro expansion: unhandled cases return nil. |
| `each` loop with `break` doesn't work | Verified: `each` expands to `while` via `each-template`. `break` in Janet is a special form that works with `while`. |
| View-angle presets from data changes docstring format | Docstrings are constructed from template strings in the data table, matching current output exactly. |
| `defmeta` macro expands differently than manual `put` | Macro expansion is literal — `(put (get core-env ',sym) :source "rojcad")` expands to exactly the same form as the original code, just generated from a macro. |
| `defn` changes function name metadata (Janet stores name in function) | `defn` passes the name to `(fn name [...] ...)`, preserving function naming. Same behavior as `(def name (fn name [...] ...))` which is the actual current form (yes, Janet functions have names: `(def name (fn name [...] ...))`). |
| Line count may not reduce exactly as expected — macro expansions and formatting differences | Acceptable. The goal is readability and idiom, not strict line count targets. |
| Missing one `(set i (+ i 1))` when converting to `++` | `rg 'set i \(' boot.janet | wc -l` and `rg '\+\+ i'` after conversion should match. Same for other variables. |
