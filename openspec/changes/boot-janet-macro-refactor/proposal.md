## Why

`upstream.janet` now provides the full suite of standard Janet macros (`defn`, `each`, `case`, `++`, `seq`, `when`, `default`, `try`, etc.), but rojcad's `boot.janet` (1839 lines) was written before these were available and uses verbose, non-idiomatic patterns throughout — manual `while` loops, `(def name (fn [...] ...))` instead of `defn`, chains of `if` instead of `case`, `(set i (+ i 1))` instead of `++`, and 7 near-identical view-angle preset functions repeated by hand. This refactoring brings `boot.janet` in line with standard Janet idioms, cutting ~400-500 lines, improving readability, and eliminating repetitive boilerplate.

## What Changes

- **Function definitions**: All `(def name (fn [...] ...))` → `defn` (~40 definitions)
- **Iteration**: All manual `while` loops over collections → `each`, `for`, `seq` (~25 loops)
- **Keyword dispatch**: Chains of `(if (= kw :x) ...)` in wrapper functions → `case` (~17 function bodies)
- **Increment**: All `(set x (+ x 1))` → `++` (~50 occurrences)
- **Conditionals**: `(if cond (do ...))` → `when`, `(if (not cond) ...)` → `unless` (~15 places)
- **Array building**: Manual `(while ... array/push) ` → `seq` (~10 places)
- **Optional defaults**: Manual `(if (= nil x) (set x val))` → `default` (~5 places)
- **Error handling**: Custom `try-catch` function → upstream `try` macro (~4 call sites)
- **View-angle presets**: 7 manually-defined view functions → data-driven generation from a preset table (~120 lines → ~30)
- **Metadata settings**: ~40 repeated `(put (get core-env 'sym) :source "rojcad") (put (get core-env 'sym) :category "cat")` → helper macro
- **C-function wrapping pattern**: Capture-and-replace pattern (`(def _hide ((get core-env 'hide) :value))`) → helper macro

No behavioral changes. All function signatures, return values, docstrings, metadata, and error semantics are preserved.

## Capabilities

### New Capabilities
*(None — this is purely implementation refactoring. All behavior, APIs, and semantics are preserved.)*

### Modified Capabilities
*(None — no spec-level requirement changes. Existing specs at `openspec/specs/camera-view/spec.md`, `openspec/specs/variadic-side-effects/spec.md`, etc. continue to describe the same behavior.)*

## Impact

- **Code**: `boot.janet` reduced by ~400-500 lines (20-25% smaller). No other files change.
- **Performance**: Zero runtime change — all macros expand to equivalent forms. `case` expands to `if` chains, `each` to `while` loops, etc.
- **Binary size**: Negligible — the macros are already loaded from `upstream.janet`; this only changes the boot.janet source embedded via `include_str!`.
- **Risk**: Low — macro expansions preserve semantics. View-angle presets being data-driven is the most structural change, but camera-view spec (scenarios for all 7 presets) serves as validation.
- **Janet version**: No change — uses only macros already available from `upstream.janet` (Janet 1.41.2).
