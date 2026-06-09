## 1. Fix `wrap-c-fn` to mutate `:value` in-place

- [x] 1.1 Change `boot.janet:14` from `(put core-env ',name @{:value (fn ,arglist ,;body)})` to `(put (get core-env ',name) :value (fn ,arglist ,;body))`

## 2. Fix manual `put` patterns to mutate `:value` in-place

- [x] 2.1 Change `compound` at `boot.janet:122` — save entry, mutate `:value`
- [x] 2.2 Change `box` at `boot.janet:228` — save entry, mutate `:value`
- [x] 2.3 Change `cylinder` at `boot.janet:276` — save entry, mutate `:value`
- [x] 2.4 Change `torus` at `boot.janet:326` — save entry, mutate `:value`

## 3. Fix `model.janet` to mutate `:value` in-place

- [x] 3.1 Change `boot/model.janet:38` from `(put core-env fn-name @{:value ...})` to `(put (get core-env fn-name) :value ...)`

## 4. Add docstrings to existing `defmeta` calls (12 functions)

- [x] 4.1 Add docstring to `defmeta compound` at `boot.janet:159`
- [x] 4.2 Add docstring to `defmeta color` at `boot.janet:160`
- [x] 4.3 Add docstring to `defmeta get-color` at `boot.janet:161`
- [x] 4.4 Add docstring to `defmeta quit-requested` at `boot.janet:753`
- [x] 4.5 Add docstring to `defmeta on-select` at `boot.janet:761`
- [x] 4.6 Add docstring to `defmeta poll-selection` at `boot.janet:777`
- [x] 4.7 Add docstring to `defmeta selected-shapes` at `boot.janet:785`
- [x] 4.8 Add docstring to `defmeta list-shapes` at `boot.janet:791`
- [x] 4.9 Add docstring to `defmeta edge-thickness` at `boot.janet:797`
- [x] 4.10 Add docstring to `defmeta edge-color-inactive` at `boot.janet:800`
- [x] 4.11 Add docstring to `defmeta edge-color-active` at `boot.janet:803`
- [x] 4.12 Add docstring to `defmeta view-angle` at `boot.janet:815`

## 5. Merge split doc puts into `defmeta` (13 sketch/wire functions)

- [x] 5.1 Merge `sketch` doc put into `defmeta`; remove doc put
- [x] 5.2 Merge `move-to` doc put into `defmeta`; remove doc put
- [x] 5.3 Merge `line-to` doc put into `defmeta`; remove doc put
- [x] 5.4 Merge `line-dx` doc put into `defmeta`; remove doc put
- [x] 5.5 Merge `line-dy` doc put into `defmeta`; remove doc put
- [x] 5.6 Merge `line-dx-dy` doc put into `defmeta`; remove doc put
- [x] 5.7 Merge `arc-to` doc put into `defmeta`; remove doc put
- [x] 5.8 Merge `close-sketch` doc put into `defmeta`; remove doc put
- [x] 5.9 Merge `build-wire` doc put into `defmeta`; remove doc put
- [x] 5.10 Merge `wire-to-face` doc put into `defmeta`; remove doc put
- [x] 5.11 Merge `wire-fillet` doc put into `defmeta`; remove doc put
- [x] 5.12 Merge `wire-chamfer` doc put into `defmeta`; remove doc put
- [x] 5.13 Merge `wire-offset` doc put into `defmeta`; remove doc put

## 6. Add `defmeta` + docstrings for Tier 1 — Core CAD (27 functions)

### 6A. Primitives (2)

- [x] 6.1 Add `defmeta sphere` after `boot.janet:187`
- [x] 6.2 Add `defmeta cone` after `boot.janet:210`

### 6B. Booleans (3)

- [x] 6.3 Add `defmeta cut` after `boot.janet:74`
- [x] 6.4 Add `defmeta common` after `boot.janet:93`
- [x] 6.5 Add `defmeta fuse` after `boot.janet:113`

### 6C. Transforms (4)

- [x] 6.6 Add `defmeta translate` after `boot.janet:545`
- [x] 6.7 Add `defmeta rotate` after `boot.janet:566`
- [x] 6.8 Add `defmeta scale` after `boot.janet:581`
- [x] 6.9 Add `defmeta mirror` after `boot.janet:596`

### 6D. Operations (3)

- [x] 6.10 Add `defmeta extrude` after `boot.janet:377`
- [x] 6.11 Add `defmeta revolve` after `boot.janet:397`
- [x] 6.12 Add `defmeta extrude-polygon` after `boot.janet:419`

### 6E. 2D Primitives (3)

- [x] 6.13 Add `defmeta rect` after `boot.janet:441`
- [x] 6.14 Add `defmeta circle` after `boot.janet:463`
- [x] 6.15 Add `defmeta polygon` after `boot.janet:484`

### 6F. Text (3)

- [x] 6.16 Add `defmeta text` after `boot.janet:522`
- [x] 6.17 Add `defmeta text3d` after `boot.janet:523`
- [x] 6.18 Add `defmeta list-fonts` after `boot.janet:524`

### 6G. Registry (4)

- [x] 6.19 Add `defmeta hide` after `boot.janet:33`
- [x] 6.20 Add `defmeta show` after `boot.janet:36`
- [x] 6.21 Add `defmeta purge` after `boot.janet:39`
- [x] 6.22 Add `defmeta registry-remove` after `boot.janet:42`

### 6H. Queries (5)

- [x] 6.23 Add `defmeta shape-type` after `boot.janet:47`
- [x] 6.24 Add `defmeta visible?` after `boot.janet:50`
- [x] 6.25 Add `defmeta wire?` after `boot.janet:53`
- [x] 6.26 Add `defmeta face?` after `boot.janet:56`
- [x] 6.27 Add `defmeta solid?` after `boot.janet:59`

## 7. Add `defmeta` + docstrings for Tier 2 — Edge styling (8 functions)

- [x] 7.1 Add `defmeta edge-toggle-inactive` after line 603
- [x] 7.2 Add `defmeta edge-toggle-active` after line 604
- [x] 7.3 Add `defmeta edge-inactive-show?` after line 605
- [x] 7.4 Add `defmeta edge-active-show?` after line 606
- [x] 7.5 Add `defmeta edge-hidden-toggle` after line 607
- [x] 7.6 Add `defmeta edge-hidden-show?` after line 608
- [x] 7.7 Add `defmeta edge-hidden` after line 609

## 8. Add `defmeta` + docstrings for Tier 3 — View/Window (14 functions)

- [x] 8.1 Add `defmeta view-fit` after line 807
- [x] 8.2 Add `defmeta view-fit-all` after line 812
- [x] 8.3 Add `defmeta projection-toggle` after line 614
- [x] 8.4 Add `defmeta projection-perspective` after line 615
- [x] 8.5 Add `defmeta stats-overlay` after line 617
- [x] 8.6 Add `defmeta window-help-toggle` after line 619
- [x] 8.7 Add `defmeta window-help-show?` after line 620
- [x] 8.8 Add `defmeta window-help-show` after line 621
- [x] 8.9 Add `defmeta window-size` after line 623
- [x] 8.10 Add `defmeta window-size?` after line 624
- [x] 8.11 Add `defmeta window-fullscreen` after line 625
- [x] 8.12 Add `defmeta window-fullscreen?` after line 626
- [x] 8.13 Add `defmeta window-maximized` after line 627
- [x] 8.14 Add `defmeta window-maximized?` after line 628

## 9. Remove dead `rojcad-groups` code

- [x] 9.1 Remove `def rojcad-groups` table and `each` iteration from `boot.janet` (lines 822-848)

## 10. Verify

- [x] 10.1 Run `just fmt-check` — confirm formatting is clean
- [x] 10.2 Run `just test-unit` — confirm no regressions (90/90 pass)
- [x] 10.3 Run `cargo run -- --headless` — confirm build succeeds
- [x] 10.4 Smoke-test: verify 73 functions show 4 keys with correct metadata
