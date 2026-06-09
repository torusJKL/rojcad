## Context

The help window is rendered in pure Rust (`src/viewer/help.rs`) with egui. It currently has four hardcoded sections: keyboard shortcuts, REPL documentation, connection info, and CLI arguments. The content is static — nothing is pulled from Janet at runtime.

This change introduces a Janet-to-Rust communication channel for a single piece of content: a "Quick Example" expression string. Janet registers it at boot time, Rust reads it on each help-window render.

## Goals / Non-Goals

**Goals:**
- Add a "Quick Example" section to the help window showing a runnable workflow
- The example expression is defined in Janet (not Rust) for easy modification
- The example path adapts to the OS (Unix vs Windows)
- Zero new crate dependencies

**Non-Goals:**
- Not a general-purpose "help content from Janet" system — just one string for now
- Not dynamic (no live updates from the REPL after boot)
- No changes to the viewer-side event loop or mpsc channels

## Decisions

### 1. Push at boot (Janet registers into shared state) rather than request-response

**Chosen:** `OnceLock<String>` in `src/types.rs`, set once by Janet at boot time.

**Alternatives considered:**
- *mpsc channel from viewer to REPL thread*: Overkill for a static string that never changes after boot. Adds threading complexity and ordering guarantees not needed here.
- *Hardcoded in Rust*: Simpler, but the user specifically wants the content to come from Janet so it can evolve without recompiling.

### 2. `OnceLock<String>` rather than a Mutex

`OnceLock` is set-once, thread-safe, and lock-free on read. Since the string is set once during boot and never changes, this is a perfect fit. A `Mutex` would add unnecessary overhead on every help render.

### 3. OS check in Janet rather than Rust

`(os)` returns the platform name. The check is a one-liner in Janet. Alternatives:
- *Rust `#[cfg]` conditional compilation*: Would require two separate builds or a runtime check in Rust anyway.
- *Rust `std::env::temp_dir()`*: More precise but adds complexity (need another bridge function). User explicitly chose hardcoded paths for simplicity.

### 4. `ui.monospace()` for multiline display

The Janet string embeds `\n` for two lines. egui's `monospace` label renders newlines correctly — no need for a grid or separate labels.

## Risks / Trade-offs

- **[Fragile path]** `/tmp/model.step` may not exist on all Unix systems (though `/tmp` is near-universal). `C:\temp\model.step` requires the directory to exist. → Mitigation: these are standard locations on both platforms; the example is illustrative, not production-critical.
- **[String mismatch]** If `janet_getstring` returns something other than valid UTF-8, `CStr::from_ptr` + `to_str()` will panic. → Mitigation: the string is a Janet literal in boot.janet, always valid UTF-8. An `unwrap()` is safe here.
- **[No escape hatch]** If the OnceLock is never set (e.g., a future refactor removes the boot.janet call), the section simply doesn't render — no error, no crash. This is acceptable behavior.
