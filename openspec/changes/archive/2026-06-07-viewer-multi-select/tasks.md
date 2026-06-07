## 1. Viewer state: multi-select storage

- [x] 1.1 Change `selected_id: Option<ShapeId>` to `selected_ids: HashSet<ShapeId>` in `ViewerState` (`app.rs:790`)
- [x] 1.2 Add `click_start_pos: PhysicalPosition<f64>` to `ViewerState` (`app.rs`)
- [x] 1.3 Update `ViewerState` initialization: `selected_ids: HashSet::new()` and `click_start_pos` (`app.rs:977`)
- [x] 1.4 Add import for `rustc_hash::FxHashSet` or `std::collections::HashSet`

## 2. Mouse handling: wait-for-release click detection

- [x] 2.1 Store click start position on `MouseInput { state: Pressed }` for left button
- [x] 2.2 Move `handle_click` call from `Pressed` to `Released` with drag-distance threshold (3px)
- [x] 2.3 Add `CLICK_THRESHOLD` constant and distance check on release

## 3. Click handler: modifier dispatch

- [x] 3.1 Read `state.modifiers` in `handle_click` to detect Ctrl / Shift / no-modifier
- [x] 3.2 Implement plain-click path: replace selection with clicked shape
- [x] 3.3 Implement Shift+click path: add clicked shape to selection set
- [x] 3.4 Implement Ctrl+click path: toggle clicked shape in/out of selection set
- [x] 3.5 Implement click-on-nothing: plain clears, Ctrl/Shift no-op
- [x] 3.6 Emit appropriate selection events to Janet bridge (toggled_on, toggled_off, cleared)

## 4. Rendering: multi-shape highlight

- [x] 4.1 Update `SurfaceDrawer::render()` parameter: `selected_id: Option<u64>` → `selected_ids: &HashSet<u64>` (`app.rs:399`)
- [x] 4.2 Change `is_selected` check from `==` to `.contains()` (`app.rs:402`)
- [x] 4.3 Update edge-instance building to use `.contains()` instead of `==` (`app.rs:1296-1316`)
- [x] 4.4 Update the `render()` call site to pass `&state.selected_ids` (`app.rs:1394`)

## 5. Janet bridge: enhanced selection events

- [x] 5.1 Add `LAST_SELECTION_ACTION: AtomicU8` alongside `LAST_SELECTION` in `src/types.rs`
- [x] 5.2 Update `rust_poll_selection()` in `src/main.rs` to take `action` out-parameter (C ABI: return id as `u64`, action as `u8` pointer)
- [x] 5.3 Update `cad_poll_selection` in `bridge.c` to handle action types and return proper Janet values (toggled_on → number, toggled_off → `[:deselected id]`, cleared → `:deselected` keyword)
- [x] 5.4 Fix existing `:deselected` return bug (currently returns `nil` instead of keyword `:deselected` for `u64::MAX`)
- [x] 5.5 Update `boot.janet` polling loop to handle the new `[:deselected id]` tuple format

## 6. ViewerToRepl enum update

- [x] 6.1 Change `ViewerToRepl::ShapeSelected` and `ShapeDeselected` to `SelectionChanged { ids: Vec<ShapeId> }` in `src/viewer/mod.rs:15-18`
- [x] 6.2 Update `handle_click` send calls to use the new enum variant

## 7. Verification (requires display environment)

- [x] 7.1 Build and check for compilation errors (`just check`)
- [x] 7.2 Run unit tests (`just test-unit`)
- [x] 7.3 Run lint (`just lint`)
- [x] 7.4 Manual verification: plain click selects single shape (requires display)
- [x] 7.5 Manual verification: Shift+click adds to selection (requires display)
- [x] 7.6 Manual verification: Ctrl+click toggles selection (requires display)
- [x] 7.7 Manual verification: drag does not trigger selection change (requires display)
- [x] 7.8 Manual verification: click on empty space clears (plain) / no-op (Ctrl/Shift) (requires display)
