## Context

The help window (`src/viewer/help.rs`) was created before the spork-repl change split the single REPL server into two (raw TCP on 9364, spork netrepl on 9365). It was never updated to reflect the new port layout, CLI flags, or correct Janet doc syntax. The integration test (`tests/test-variadic.sh`) also targets the wrong port. The existing spec (`openspec/specs/help-window/spec.md`) documents the stale connection info and CLI args.

This is a documentation-only fix — no runtime behavior changes, no API changes, no new dependencies.

## Goals / Non-Goals

**Goals:**
- Update help window connection info to show correct raw REPL port (9364)
- Update help window CLI args section to show `--raw-port`/`--spork-port` instead of removed `--port`
- Fix `(doc 'sym)` → `(doc sym)` to match actual Janet convention
- Fix integration test port to target raw REPL (9364)
- Update spec to reflect correct connection and CLI args requirements

**Non-Goals:**
- Not adding spork REPL connection info to the help window (show only `nc`)
- Not restructuring the help window or adding new sections
- Not changing any server-side port logic
- Not adding width/height CLI args to the spec scenario (out of scope)

## Decisions

### Decision 1: Show only raw REPL in help window

**Choice:** The connecting section in help.rs will describe only the raw TCP REPL and its `nc` command.

**Rationale:** The spork REPL is for Conjure/spork CLI users who don't need `nc`. The help window's connection section is aimed at someone who wants to connect immediately with a simple tool — that's `nc` to the raw port.

### Decision 2: Add both CLI flags, don't subtract

**Choice:** Replace the single `--port` entry with both `--raw-port` and `--spork-port` entries in the CLI args grid.

**Rationale:** Users who need to change ports for either server need to know both flags exist. Showing only one would be misleading.

### Decision 3: Fix spec speculatively in this change

**Choice:** Update the `help-window/spec.md` alongside the code, rather than as a separate change.

**Rationale:** The spec is wrong, and fixing it without the corresponding code fix would leave the system in an inconsistent state. The spec describes what the help window should show — both must match.

## Risks / Trade-offs

- **Spec still lists fewer CLI args than the help window**: The spec scenario for CLI args only mentions `--headless, --raw-port, --spork-port, --eval`. The help window also shows `--width` and `--height`. Adding them to the spec is out of scope for this change — they'd need a separate update.
- **Test timing**: The integration test starts rojcad headless and connects after 3 seconds. Changing the port shouldn't affect timing, but if the spork server startup delays the raw server, the test might need a longer sleep.
