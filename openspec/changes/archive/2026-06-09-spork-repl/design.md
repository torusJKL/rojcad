## Context

The current REPL server (boot.janet) runs a single raw-text TCP server on port 9365. It reads raw bytes via `net/read`, evaluates them as Janet, and returns results. No line editing, history, completion, or protocol framing.

Spork is the standard library for Janet's TCP REPL protocol (`spork/netrepl.janet`). Clients like Conjure (Neovim), the spork CLI client, and Freja-netrepl speak this protocol and default to port 9365.

The project uses a bootstrapped Janet build (`JANET_BOOTSTRAP=1`) where external modules aren't auto-discoverable — they must be explicitly loaded at boot time via `janet_dostring`.

## Goals / Non-Goals

**Goals:**
- Add a spork netrepl server running alongside the existing raw REPL
- Spork server on port 9365 (default for all spork clients)
- Raw REPL on port 9364 (moved from 9365)
- CLI flags `--spork-port` and `--raw-port` for overriding ports (replaces old `--port`)
- Show version from Cargo.toml in startup banner (with `-dirty` if uncommitted changes)
- Vendor/embed spork source files needed for netrepl
- Match raw REPL shape lifecycle: shapes bound via `def`/`set` are auto-shown (including through threading macros)
- Propagate `(quit)` signal: `run-context` exits when `(quit)` is called

**Non-Goals:**
- Not replacing the raw REPL entirely (preserve `nc` compatibility)
- Not adding completion/docs functionality to the raw REPL
- Not building a custom REPL client

## Decisions

### Decision 1: Two separate servers, not mixed protocol

**Choice:** Run two independent TCP servers on two ports.

**Rationale:** The spork netrepl protocol's conversation pattern (client identification → prompts → outbuf capture → auto-flush) is fundamentally different from the raw REPL's fire-and-forget eval loop. Detecting the protocol on connect is possible but adds complexity for no benefit when separate ports are cleaner.

### Decision 2: Concatenate vendored spork source via `concat!` + `include_str!` + `janet_dostring`

**Choice:** Three Janet files are vendored (`msg.janet`, `ev-utils.janet`, `netrepl-server.janet`) and concatenated into one source via Rust's `concat!` macro, then loaded via a single `janet_dostring` call.

**Rationale:** The project uses `JANET_BOOTSTRAP=1` which disables Janet's native module loader (jpm). Spork's server code uses `(use ./msg)` and `(use ./ev-utils)` relative imports which fail under `janet_dostring` (no module resolution). Concatenating the files sidesteps this — all definitions are available in one evaluation.

A server-only subset of spork is used (`netrepl-server.janet`) — the `getline.janet`, `rawterm.c`, and `generators.janet` parts are excluded (they're only needed by the CLI client, not the server). This avoids a C compile dependency on `spork/rawterm`.

**Alternatives considered:**
- Loading as separate `janet_dostring` calls: `(use ./msg)` fails due to module resolution in bootstrap mode
- Using a Git submodule: adds build complexity, spork's C extensions are not needed for netrepl
- Implementing netrepl protocol from scratch in boot.janet: unnecessary duplication when spork works
- Full spork with rawterm C module: avoids strip-down but adds a C compile dependency

### Decision 3: New CLI flags `--spork-port` and `--raw-port`

**Choice:** Replace the single `--port` flag with `--spork-port <PORT>` and `--raw-port <PORT>` flags.

**Rationale:** Two independent ports need two independent flags. The old `--port` flag mapped to a single port and made no sense when there are two servers. Removing it eliminates ambiguity.

### Decision 4: Share the same core environment between both servers

**Choice:** Both REPL servers evaluate against the same `core-env` (the current boot-time environment).

**Rationale:** Consistency — users expect the same bindings regardless of which port they connect to. The raw REPL passes `core-env` to `my-eval`, and the spork server can use the same env via `netrepl/server`'s env argument.

### Decision 5: Inject port values as Janet globals at boot time

**Choice:** Port values are prefixed as `(def *raw-repl-port* N)` and `(def *spork-repl-port* N)` before boot.janet, using Rust string formatting in `src/main.rs`.

**Rationale:** Initial approach used `janet_setdyn` (Janet C API for dynamic variables), but `(dyn '*raw-repl-port*)` returned nil in boot.janet because `janet_dostring` creates new fibers that don't inherit dynamic bindings set from C. Injecting as literal `def` forms into boot code ensures the values are available as regular globals before any Janet code runs.

### Decision 6: Shape lifecycle hooks in spork evaluator

**Choice:** The spork evaluator (`evaluate-wrapped` in `netrepl-server.janet`) adds post-evaluation checks: if the result is a `:rojcad/shape` from a form whose macro-expanded head is `def`/`set`, call `(show result)`. Uses `(macex source)` to expand macros (handles `->`, `->>`, etc.).

**Rationale:** The spork REPL uses `run-context` which has its own evaluation pipeline. The raw REPL has custom `my-eval` with shape lifecycle hooks. Without replicating these hooks in the spork server's evaluator, shapes created via `def`/`set` would never be sent to the viewer.

### Decision 7: Quit propagation via `(dyn :exit)` → `(put e :exit true)`

**Choice:** After each evaluation, the spork evaluator checks `(dyn :exit)`. If set (by the `quit` function), it propagates to `(put e :exit true)` in the environment table. `run-context` checks `(env :exit)` to break its main loop.

**Rationale:** The `quit` function does `(setdyn :exit true)` which sets a dynamic binding, but `run-context` checks `(env :exit)` (environment table lookup). These are separate mechanisms. The evaluator bridges them by copying the dynamic binding to the env table after each evaluation.

### Decision 8: Run spork server via `netrepl/run-server` from boot.janet

**Choice:** After setting up the raw REPL server, call `(netrepl/run-server addr spork-port core-env)` in a separate fiber.

**Rationale:** Uses `serve-and-wait` internally which blocks the fiber until shutdown, keeping the `disconnect-all` defer from running prematurely. `netrepl/server` returns immediately, causing cleanup to fire right away.

### Decision 9: Version banner with dirty-suffix detection

**Choice:** The startup banner reads the version from `Cargo.toml` via `env!("CARGO_PKG_VERSION")` and appends `-dirty` if `git status --porcelain` reports any uncommitted or untracked files at startup.

**Rationale:** The version is injected as `*rojcad-version*` into the boot prefix alongside ports. The `-dirty` suffix makes it obvious when running from a modified working tree. The git check runs at startup, reflecting the current state (not build-time).

**Out-of-scope for v1:** Full debugger support, client tracking with name display, `:auto-flush` from boot — these are built into spork but don't need configuration.

## Architecture

```
┌─────────────────────────────────────────────────┐
│                  rojcad process                  │
│                                                  │
│  ┌──────────────┐     ┌──────────────────────┐  │
│  │ Raw REPL     │     │ Spork netrepl        │  │
│  │ (port 9364)  │     │ (port 9365)          │  │
│  │              │     │                      │  │
│  │ net/listen   │     │ netrepl/server       │  │
│  │ accept-loop  │     │ (runs in its own     │  │
│  │ connect-hdlr │     │  fiber)              │  │
│  └──────┬───────┘     └──────────┬───────────┘  │
│         │                        │              │
│         └────────┬───────────────┘              │
│                  │ shared core-env              │
│          ┌───────▼───────┐                      │
│          │   core-env    │                      │
│          │ (CAD fns +    │                      │
│          │  Janet stdlib)│                      │
│          └───────────────┘                      │
└─────────────────────────────────────────────────┘
```

**Startup sequence:**

1. Parse CLI args (`--spork-port`, `--raw-port`) via macro-generated Rust functions
2. Init Janet VM, register core libs and CAD functions
3. Load upstream.janet (standard macros, including `macex`, `->`, `->>`, etc.)
4. **NEW:** Load concatenated spork source files via `janet_dostring` (msg.janet + ev-utils.janet + netrepl-server.janet + namespace aliases)
5. Load boot.janet (prefixed with `def *rojcad-version*`, `def *raw-repl-port*`, `def *spork-repl-port*`)
6. Control passes to event loop

**boot.janet changes:**

```janet
; Ports injected as globals by Rust boot prefix
(def raw-port *raw-repl-port*)     ; 9364 by default
(def spork-port *spork-repl-port*) ; 9365 by default

; Raw REPL server (unchanged logic, new port)
(def raw-listen (net/listen addr raw-port))
(ev/go (fiber/new (fn [] (net/accept-loop raw-listen connect-handler))))

; Spork REPL server — uses run-server (blocks until done)
(ev/go (fiber/new (fn []
  (def [ok val] (protect (netrepl/run-server addr spork-port core-env)))
  (when (not ok)
    (eprint "rojcad: spork server on " addr ":" spork-port " failed: " val)))))

(eprint "◆ rojcad ready - (" *rojcad-version* ")")
(eprint "◆ raw REPL (nc): " addr " " raw-port)
(eprint "◆ spork REPL: " addr " " spork-port)
```

## Files Changed

| File | Change |
|------|--------|
| `src/main.rs` | Replace `--port` with `--spork-port`/`--raw-port` CLI flags (via macro); inject version (from `env!("CARGO_PKG_VERSION")` + git dirty check) and port values as `def` globals in boot prefix; load concatenated spork source via `concat!`+`janet_dostring` between upstream and boot |
| `boot.janet` | Add spork server (`netrepl/run-server`) alongside raw server; update port vars to globals; dual banner |
| `AGENTS.md` | Update default port (9364 raw, 9365 spork); document vendored spork subset |
| `README.md` | Update connection instructions for both ports |
| `vendor/spork/msg.janet` (new) | Length-prefixed message framing (pure Janet, from upstream spork) |
| `vendor/spork/ev-utils.janet` (new) | Fiber nursery utilities (pure Janet, from upstream spork) |
| `vendor/spork/netrepl-server.janet` (new) | Server-only netrepl with evaluator hooks for shape lifecycle and quit propagation |
| `vendor/spork/LICENSE` (new) | MIT license attribution |

## Risks / Trade-offs

- **Spork version drift**: Vendored source will diverge from upstream. Mitigation: document the version and a refresh process in AGENTS.md.
- **Both servers fail**: If port binding fails for one server, the other still works (separate fibers). The error message identifies which port failed.
- **Increased startup time**: Loading 4 extra Janet files adds negligible time (~1ms).
- **Conjure connection timing**: If Conjure connects before the spork server starts, the connection is refused. Mitigation: the spork server fiber starts synchronously inside boot.janet before the event loop accepts connections, so it's ready before any client can connect.
