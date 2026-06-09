## 1. Fix pan() in OrbitCamera

- [x] 1.1 Replace the body of `OrbitCamera::pan()` in `src/viewer/camera.rs` to compute both `right` and `up` vectors from `self.forward()`, producing an orthogonal screen-space pair
- [x] 1.2 Verify that `right()` and `up()` public methods are untouched and `pan()` is the only changed function
- [x] 1.3 Add a doc-comment to `right()` and `up()` clarifying they return world-aligned vectors (not screen-space), per design risk mitigation

## 2. Manual verification

- [x] 2.1 Build and run: `just build && just run` (or `just run -- --headless` to check compilation)
- [ ] 2.2 At default pitch (0.4 rad), middle-mouse-drag left/right — confirm scene follows cursor cleanly
- [ ] 2.3 At pitch=0 (looking straight ahead from +X), confirm pan still works as before
- [ ] 2.4 At pitch ≈ 1.0 rad (steep downward), confirm left/right pan stays screen-aligned
- [ ] 2.5 Confirm Shift+left-drag panning behaves identically to middle-mouse-drag
