## 1. Shared state — types.rs

- [x] 1.1 Add `static QUIT_REQUESTED: AtomicBool` to `src/types.rs`

## 2. Viewer key handling — app.rs

- [x] 2.1 Import `QUIT_REQUESTED` from `crate::types` in `src/viewer/app.rs`
- [x] 2.2 Remove the `Escape` match arm from the `KeyboardInput` handler
- [x] 2.3 Add Ctrl+Q match arm that sets `QUIT_REQUESTED`, sends `ViewerClosed`, stops the event loop
- [x] 2.4 Update `CloseRequested` handler to set `QUIT_REQUESTED` before closing

## 3. Rust FFI — main.rs

- [x] 3.1 Import `QUIT_REQUESTED` from `crate::types`
- [x] 3.2 Add `rust_quit_requested()` C function that returns the flag via `swap(false)`

## 4. C bridge — bridge.c

- [x] 4.1 Add `extern int rust_quit_requested(void);` declaration
- [x] 4.2 Add `cad_quit_requested` JANET_FN with docstring
- [x] 4.3 Register `{"quit-requested", cad_quit_requested, cad_quit_requested_docstring_}` in `cfuns[]`

## 5. Janet boot script — boot.janet

- [x] 5.1 Add `(if (quit-requested) (os/exit 0))` check at the top of `poll-viewer` loop

## 6. Verification

- [x] 6.1 Build the project with `just build` and verify no compile errors
- [x] 6.2 Run `just fmt` to ensure formatting is consistent
