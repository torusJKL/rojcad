## Context

The Janet REPL (`boot.janet:306-321`) evaluates forms via `my-eval`, which compiles them against `core-env` and resumes a fiber. The existing `auto-show-on-def` change intercepts `(def name expr)` forms in `my-eval` and auto-shows the result if it's a shape. However, when a symbol is *re*directed to a new shape, the old shape's `ShapeData` remains registered in the viewer's `ShapeRegistry` — continuing to render every frame — until Janet GC happens to collect it.

The `purge` function (C-level `_purge`, wrapped as variadic `purge`) removes a shape from `ShapeRegistry` and marks its `ShapeData` as purged. But there is no automatic purging on symbol redefinition — the user must manually `(purge old-name)` before or after `(def name new-shape)`.

Key architectural fact: the Janet symbol table (`core-env`) and the Rust `ShapeRegistry` are completely decoupled. There is no Rust-side mapping from symbol names to shape IDs. All shape lifecycle management happens either via explicit user calls (`show`, `hide`, `purge`) or via `ShapeData::drop` on GC.

### Affected files

| File | Role |
|---|---|
| `boot.janet` | `my-eval` function — the single interception point for `def`/`set` evaluation |

### Unchanged files

`bridge/bridge.c`, `src/cad.rs`, `src/main.rs`, `src/types.rs`, `src/bridge.rs` — zero Rust/C changes. This is a pure Janet change in the REPL eval layer, identical in scope to the `auto-show-on-def` change.

## Goals / Non-Goals

**Goals:**
- Auto-purge old shapes when their binding symbol is redefined via top-level `def` or `set` in the REPL
- Extend auto-show to also apply to `set` (for consistency with `def`)
- Avoid purge-then-panic when old and new shapes are the same object (`(def obj obj)`)
- Zero Rust/C changes
- Always-on behavior — no opt-in keyword needed

**Non-Goals:**
- Not modifying `ShapeData`, `ShapeRegistry`, or the viewer thread
- Not handling `(def obj nil)` — the old shape persists in registry until GC (user can `(purge obj)` before `(def obj nil)` if desired)
- Not calling `(gc)` after purge — natural collection; users can call `(gc)` explicitly
- Not adding a reverse symbol→shape_id mapping in Rust
- Not changing variadic wrappers or C bridge registration

## Decisions

### Decision 1: Capture-and-purge in `my_eval` (Janet-only)

**Chosen** over Rust-level symbol registry.

`my-eval` already intercepts `def` forms for auto-show. The pattern extends naturally:

```
Pre-eval:  capture old binding from core-env
Eval:      compile + resume (does the def/set)
Post-eval: if old binding was a shape → _purge it
           if new result is a shape → auto-show it
```

**Alternatives considered:**
- **Rust-level symbol registry** (`HashMap<String, ShapeId>`) — more robust tracking but requires new bridge functions, Rust↔Janet communication, and complicates the architecture. Not justified for this behavior.
- **Replace `def` special form** in `core-env` — fragile in bootstrap mode. Janet's compiler specially recognizes `def`; replacing the env binding doesn't reliably intercept compilation.
- **Janet GC hook** (`:ogc` on abstract type) — would fire too late (only on collection) and can't distinguish "symbol redefined" from "last reference dropped".

### Decision 2: `purge` semantics (not `hide`)

When a symbol is redefined, the old shape is **purged** (removed from `ShapeRegistry`, marked as purged) rather than just **hidden** (visible=false, stays in registry).

Purge frees GPU mesh buffers immediately and removes the entry from the viewer's visible-shapes snapshot. This matches the user's intuition: "I redefined the symbol, the old shape should be gone."

Trade-off: If another Janet variable still references the same `ShapeData`, that variable now points to a purged shape — `(show other-var)` panics. This is identical to the existing behavior of `(purge name)` when the shape is shared, so it's a known and accepted limitation.

### Decision 3: Both `def` and `set` handled

`(def name val)` creates or rebinds; `(set name val)` rebinds an existing binding. Both can replace a shape, so both trigger the auto-purge. The existing auto-show condition (line 315 of `boot.janet`) is extended from `(= f0 'def)` to `(or (= f0 'def) (= f0 'set))`.

`do` forms like `(do (def b (box 10)) (fn b))` are NOT intercepted — the top-level form is `do`, not `def`/`set`. This is the same design choice as auto-show-on-def.

### Decision 4: Identity check to prevent purge-then-panic

`(def obj obj)` redefines a symbol to its current value. Without a guard: capture old=box, resume (def obj obj → same box), purge old=box (marks it purged), auto-show result=box → **panic** (shape already purged).

The fix: `(not= old-val result)` before purging. For abstract types, Janet's `=` compares by identity (pointer equality), so the same `ShapeData` is detected correctly.

### Decision 5: Use `_purge` (not variadic `purge` wrapper)

`boot.janet` captures the raw C function as `_purge` (line 31) before wrapping it. Using `_purge` directly avoids the overhead of the variadic wrapper (which constructs an array and loops). This is a minor optimization — `purge` with a single argument would also work correctly.

## Edge Cases

| Case | Behavior | Mechanism |
|---|---|---|
| `(def obj (box 10))` — first def | No purge, auto-show | `old-val` is nil, skip |
| `(def obj (sphere 20))` — redef | Purge box, auto-show sphere | old-val captured, purged |
| `(def obj obj)` — same shape | No purge, no re-show | `(= old-val result)` → skip both |
| `(set obj (sphere 20))` — set | Purge old, auto-show new | Captured in `def`/`set` path |
| `(def obj nil)` — unbind | Skip (as today) | `(not= f2 nil)` check unchanged |
| `(def a (box 10)) (def b a) (def a ...)` | Shared ref → `b` now dead | Same as manual `(purge a)` |
| Error in expression | Old value preserved in env | Captured before eval, never reaches purge |
| Non-shape value | Skip purge, skip auto-show | `(type old-val)` and `(type result)` checks |

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| Shared references die silently | Same as `(purge name)` today — documented behavior. User's responsibility to avoid sharing if they plan to redefine. |
| `(def obj obj)` panic without identity guard | Identity check (`not= old-val result`) prevents this. |
| Non-top-level `def` inside `do` won't auto-purge | Rare in REPL usage. User writes `(purge old)` explicitly. Same limitation as auto-show-on-def. |
| Captured `old-val` extends shape lifetime | `old-val` is a function-local var — released when `my-eval` returns. Shape becomes unreachable and GC collects it normally. |
