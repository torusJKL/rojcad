## Why

New users opening rojcad for the first time see the help window but have no obvious "what do I type first?" guidance. A concrete example — create a box, export it — bridges the gap between reading about functions and actually using them.

## What Changes

- Add a "Quick Example" section to the help window showing `(def mybox (box 10))` then `(write-step /tmp/model.step)`
- The example expression is registered from Janet at boot time, not hardcoded in Rust
- The path in the example differs per platform: `/tmp/model.step` on Unix, `C:\temp\model.step` on Windows
- Add a new `(help-set-example)` Janet function under the `"view"` group that stores the example string in shared state
- The section only renders if Janet has registered content — no crash or empty section if registration fails

## Capabilities

### New Capabilities

None — this is an enhancement to the existing help-window capability.

### Modified Capabilities

- `help-window`: The help window SHALL display a Quick Example section when content has been registered from Janet, showing a complete workflow (create shape + export to STEP file).

## Impact

- `src/types.rs` — new `OnceLock<String>` static for the example text
- `src/main.rs` — new FFI bridge function to receive the string from C
- `bridge/bridge.c` — new JANET_FN registration + extern declaration
- `src/viewer/help.rs` — new UI section gated on the OnceLock
- `boot.janet` — one-line OS check + registration call at end of file
- `openspec/specs/help-window/spec.md` — new requirement for the Quick Example section
