## Context

rojcad runs two threads: a REPL thread hosting the Janet interpreter (with two TCP REPL servers: raw and spork) and a viewer thread running a winit/wgpu/egui window. Currently, the only way to interact with the Janet REPL is through those external TCP ports. The viewer thread already has an egui context and renders overlay panels (Stats, Help) each frame. A REPL GUI can reuse this same egui pipeline.

Thread communication currently uses:
- `ShapeRegistry` (RwLock) — shapes pushed from REPL thread, read by viewer
- `ReplToViewer` (mpsc) — commands from REPL→viewer (fit-to-bounds, view angles, etc.)
- `ViewerToRepl` (mpsc) — commands from viewer→REPL (selection changed, viewer closed)
- Atomic statics — toggle states (edge visibility, projection, overlay toggles)

Janet is NOT thread-safe — all Janet operations must happen on the REPL thread. The Janet event loop runs cooperatively via `ev/go` fibers.

## Goals / Non-Goals

**Goals:**
- In-window REPL panel docked to the right side using egui `SidePanel`
- Send Janet code from viewer thread to REPL thread for evaluation and receive structured results
- Syntax-coloured input (via `egui_code_editor::CodeEditor`) and history output (via `LayoutJob` from same tokenizer)
- Rainbow bracket coloring in history output based on nesting depth
- Multi-line code input with Enter=newline, Ctrl+Enter=submit
- Toggle visibility with Ctrl+R
- Dynamic gizmo positioning that adjusts for panel width and DPI scaling
- Display panel width in stats overlay

**Non-Goals:**
- File persistence of REPL history (out of scope)
- Code completion popup (deferred — infrastructure exists)
- Jump-to-definition or hover documentation
- Separate REPL window (single-window panel is simpler)

## Decisions

### Decision 1: Pipe-delimited eval protocol, not JSON or Janet syntax

**Chosen**: `\x02`-delimited format: `e\x02<id>\x02<code>` (eval), `h\x02<id>\x02<code>` (highlight), etc.

**Rejected**: JSON (`:` interferes with Janet's keyword syntax), Janet-native `{:key value}` (unreliable parsing in bootstrap mode).

**Rationale**: The `\x02` (STX) separator can never appear in user-typed Janet code. Parsing is a simple `string/split "\x02"` on the Janet side, no grammar or escaping needed. The first character selects the operation type, the second field is the numeric ID, the third field is the body.

### Decision 2: Syntax highlighting via `egui_code_editor` crate, not channel-based

**Chosen**: Both input and history use the `egui_code_editor` crate's tokenizer with a custom Janet `Syntax` definition. History uses `Token::highlight()` with a minimal `HistoryEditor` implementing the `Editor` trait. The input uses `CodeEditor::show()` directly.

**Rationale**: Eliminates the highlight channel round-trip entirely. Tokenization is local to the viewer thread. The same `Syntax` and `ColorTheme` are shared between input and history, guaranteeing identical colors. Rainbow brackets are applied per-token in `HistoryEditor::append()` via `Cell<i32>` depth tracking. The input editor's brackets use the theme's punctuation color.

### Decision 3: `egui_code_editor` + egui 0.34 + wgpu 29

**Chosen**: Upgraded from egui 0.32/wgpu 25 to egui 0.34/wgpu 29 to support `egui_code_editor` 0.3.

**Rationale**: The `egui_code_editor` crate provides a complete code editor widget with syntax highlighting, line numbers, and a tokenizer API. It depends on egui 0.34 and wgpu 29. The upgrade required mechanical API changes (24 errors fixed): `PipelineLayoutDescriptor` now uses `Option<&BindGroupLayout>`, `Renderer::new` takes `RendererOptions`, `Surface::get_current_texture` returns `CurrentSurfaceTexture` enum, `depth_compare`/`depth_write_enabled` are now `Option`, `multiview` renamed to `multiview_mask`, etc.

### Decision 4: Local syntax highlighting, not channel-based

**Chosen**: History output colored via `code_to_job()` which calls `Token::highlight()` with a `HistoryEditor` stub. Input colored via `CodeEditor::show()` with the shared `ROJCAD_THEME`.

**Rejected**: The original design sent highlight requests through the channel to Janet, which ran `highlight-scan` (a character-scanner), and returned JSON token arrays. This added latency and the PEG grammar had bootstrap compatibility issues.

**Rationale**: Tokenization is a pure function of the code string and syntax rules — it doesn't need the REPL thread. The `egui_code_editor` crate provides the `Token::tokens()` and `Token::highlight()` APIs which work entirely on the viewer thread. This simplifies the architecture and removes the highlight round-trip.

### Decision 5: Gizmo positioning with DPI-aware formula

**Chosen**: `gx = width - gs - (panel_phys + extra_phys + gm)` where `panel_phys = panel_logical * sf` and `extra_phys = 175 * sf`, with all math in `f64` before rounding to `u64`.

**Rationale**: Egui uses logical pixels while wgpu uses physical pixels. The panel width `REPL_PANEL_WIDTH` is in logical units. The gizmo dimensions `gm`, `gs`, and window `width` are in physical units. At non-integer scale factors (e.g., 150%), truncation errors caused the gizmo to appear at the wrong position. Using `f64` multiplication with `.round()` ensures correct positioning at any DPI.

### Decision 6: Rainbow bracket colors (history only)

**Chosen**: A `Cell<i32>` depth tracker in `HistoryEditor` adjusts bracket color based on nesting depth. Palette: white → yellow → cyan → green → purple → orange.

**Rejected**: The input editor's bracket colors cannot be overridden per-position — the `Patch` API only supports keyword/type/special set extension, not position-based recoloring.

**Rationale**: Depth-based coloring improves code readability. The palette cycles through 6 colors and wraps at depth 6+.

### Decision 7: Single-ish spec

All REPL GUI capabilities (eval, highlight, completion, panel) are tightly coupled and share the same channel infrastructure. Splitting into multiple specs adds overhead without clarity benefit.

## Risks / Trade-offs

- **[Channel contention]** The `poll-gui-repl` fiber runs at 20ms intervals. If eval takes long, the fiber pauses but doesn't block other TCP REPL fibers — Janet's cooperative scheduling handles this.
- **[Tokenizer divergence]** The `egui_code_editor`'s built-in tokenizer doesn't recognize Janet-specific reader macros (`'`, `~`, `;`, `,`). These appear in the default punctuation color rather than being highlighted as special forms.
- **[egui_code_editor API]** The crate is version 0.3 and its API may change. The `HistoryEditor` implements the `Editor` trait which requires `Hash` — workaround uses pointer-hashing.
- **[Rainbow brackets in editor]** Not possible with current `Patch` API. Bracket highlighting in the input editor is plain white (punctuation color).

## Open Questions

- Completion popup is deferred. Trigger mechanism (auto after 2+ chars vs. Ctrl+Space only) and UI (popup positioned at cursor, etc.) remain to be designed.
