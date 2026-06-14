#![allow(dead_code)]

use std::cell::Cell;
use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::mpsc;

use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontFamily, FontId};

use egui_code_editor::{CodeEditor, ColorTheme, Syntax, Token, TokenType};

use crate::types::{REPL_PANEL_WIDTH, SHOW_REPL_PANEL, ShapeId};

/// A single entry in the REPL output history.
pub struct HistoryEntry {
    /// Syntax-coloured LayoutJob for the code display.
    pub code_job: LayoutJob,
    pub result: String,
    pub kind: String,
    pub shape_id: Option<ShapeId>,
}

/// The REPL panel widget.
pub struct ReplPanel {
    pub input: String,
    pub history: Vec<HistoryEntry>,
    pending: Option<PendingEval>,
    pending_highlight_id: u64,
    next_id: u64,
    gui_req_tx: mpsc::Sender<String>,
    fn_names: HashSet<String>,
    ctrl_enter_pending: bool,
}

struct PendingEval {
    id: u64,
    code: String,
}

struct JsonValParser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

#[derive(Debug)]
enum JVal {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<JVal>),
    Obj(Vec<(String, JVal)>),
}

impl<'a> JsonValParser<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            bytes: s.as_bytes(),
            pos: 0,
        }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> u8 {
        self.bytes.get(self.pos).copied().unwrap_or(0)
    }

    fn expect(&mut self, b: u8) -> bool {
        self.skip_ws();
        if self.peek() == b {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn val(&mut self) -> Option<JVal> {
        self.skip_ws();
        match self.peek() {
            b'n' => self.lit("null", JVal::Null),
            b't' => self.lit("true", JVal::Bool(true)),
            b'f' => self.lit("false", JVal::Bool(false)),
            b'"' => self.parse_str().map(JVal::Str),
            b'[' => self.parse_arr(),
            b'{' => self.parse_obj(),
            b'-' | b'0'..=b'9' => self.parse_num().map(JVal::Num),
            _ => None,
        }
    }

    fn lit(&mut self, exp: &str, v: JVal) -> Option<JVal> {
        if self.bytes[self.pos..].starts_with(exp.as_bytes()) {
            self.pos += exp.len();
            Some(v)
        } else {
            None
        }
    }

    fn parse_str(&mut self) -> Option<String> {
        if !self.expect(b'"') {
            return None;
        }
        let mut s = String::new();
        loop {
            match self.bytes.get(self.pos) {
                None => return None,
                Some(b'"') => {
                    self.pos += 1;
                    return Some(s);
                }
                Some(b'\\') => {
                    self.pos += 1;
                    match self.bytes.get(self.pos) {
                        Some(b'"') => {
                            s.push('"');
                            self.pos += 1;
                        }
                        Some(b'\\') => {
                            s.push('\\');
                            self.pos += 1;
                        }
                        Some(b'n') => {
                            s.push('\n');
                            self.pos += 1;
                        }
                        Some(b'r') => {
                            s.push('\r');
                            self.pos += 1;
                        }
                        Some(b't') => {
                            s.push('\t');
                            self.pos += 1;
                        }
                        Some(b'u') => {
                            let hex = &self.bytes[self.pos + 1..self.pos + 5];
                            let code =
                                u32::from_str_radix(std::str::from_utf8(hex).unwrap_or("0"), 16)
                                    .unwrap_or(0);
                            s.push(char::from_u32(code).unwrap_or('?'));
                            self.pos += 5;
                        }
                        _ => return None,
                    }
                }
                Some(&b) => {
                    s.push(b as char);
                    self.pos += 1;
                }
            }
        }
    }

    fn parse_num(&mut self) -> Option<f64> {
        let start = self.pos;
        if self.peek() == b'-' {
            self.pos += 1;
        }
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        if self.peek() == b'.' {
            self.pos += 1;
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }
        std::str::from_utf8(&self.bytes[start..self.pos])
            .ok()?
            .parse()
            .ok()
    }

    fn parse_arr(&mut self) -> Option<JVal> {
        if !self.expect(b'[') {
            return None;
        }
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == b']' {
                self.pos += 1;
                return Some(JVal::Arr(items));
            }
            if !items.is_empty() {
                self.expect(b',');
            }
            items.push(self.val()?);
        }
    }

    fn parse_obj(&mut self) -> Option<JVal> {
        if !self.expect(b'{') {
            return None;
        }
        let mut pairs = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == b'}' {
                self.pos += 1;
                return Some(JVal::Obj(pairs));
            }
            if !pairs.is_empty() {
                self.expect(b',');
            }
            let key = self.parse_str()?;
            self.expect(b':');
            let v = self.val()?;
            pairs.push((key, v));
        }
    }

    fn obj_str(obj: &[(String, JVal)], key: &str) -> Option<String> {
        obj.iter().find(|(k, _)| k == key).and_then(|(_, v)| {
            if let JVal::Str(s) = v {
                Some(s.clone())
            } else {
                None
            }
        })
    }

    fn obj_num(obj: &[(String, JVal)], key: &str) -> Option<f64> {
        obj.iter()
            .find(|(k, _)| k == key)
            .and_then(|(_, v)| if let JVal::Num(n) = v { Some(*n) } else { None })
    }

    fn obj_arr<'b>(obj: &'b [(String, JVal)], key: &str) -> Option<&'b Vec<JVal>> {
        obj.iter()
            .find(|(k, _)| k == key)
            .and_then(|(_, v)| if let JVal::Arr(a) = v { Some(a) } else { None })
    }
}

// ── LayoutJob builder ──────────────────────────────────────────────────

fn janet_syntax() -> Syntax {
    Syntax::new("janet")
        .with_case_sensitive(true)
        .with_comment("#")
        .with_quotes(['"'])
        .with_keywords([
            "defn",
            "def",
            "var",
            "set",
            "varfn",
            "if",
            "when",
            "while",
            "each",
            "for",
            "do",
            "fn",
            "let",
            "case",
            "cond",
            "match",
            "import",
            "use",
            "return",
            "break",
            "continue",
            "not",
            "and",
            "or",
            "true",
            "false",
            "nil",
            "eval",
            "compile",
            "quote",
            "quasiquote",
            "unquote",
            "splice",
            "apply",
            "->",
            "|>",
        ])
        .with_types([
            "box",
            "sphere",
            "cylinder",
            "cone",
            "torus",
            "cut",
            "common",
            "fuse",
            "compound",
            "extrude",
            "revolve",
            "translate",
            "rotate",
            "scale",
            "mirror",
            "rect",
            "circle",
            "polygon",
            "sketch",
            "line-to",
            "move-to",
            "arc-to",
            "close-sketch",
            "build-wire",
            "wire-to-face",
            "wire-fillet",
            "wire-chamfer",
            "wire-offset",
            "text",
            "text3d",
            "read-step",
            "write-step",
            "write-stl",
            "hide",
            "show",
            "purge",
            "color",
            "set-color",
            "get-color",
            "shape-type",
            "visible?",
            "solid?",
            "face?",
            "wire?",
            "list-shapes",
            "selected-shapes",
        ])
}

/// Re-usable custom theme matching our history token colors.
const ROJCAD_THEME: ColorTheme = ColorTheme {
    name: "rojcad",
    dark: true,
    bg: "1E1E1E",
    cursor: "FFFFFF",
    selection: "264F78",
    comments: "808080",
    functions: "50C8FF",
    keywords: "C878C8",
    literals: "FFD700",
    numerics: "FFB450",
    punctuation: "FFFFFF",
    strs: "4EC9B0",
    types: "50C8FF",
    special: "C878C8",
};

fn hex_to_color32(hex: &str) -> Color32 {
    let bytes = hex.as_bytes();
    let r = (char::from(bytes[0]).to_digit(16).unwrap_or(0) * 16
        + char::from(bytes[1]).to_digit(16).unwrap_or(0)) as u8;
    let g = (char::from(bytes[2]).to_digit(16).unwrap_or(0) * 16
        + char::from(bytes[3]).to_digit(16).unwrap_or(0)) as u8;
    let b = (char::from(bytes[4]).to_digit(16).unwrap_or(0) * 16
        + char::from(bytes[5]).to_digit(16).unwrap_or(0)) as u8;
    Color32::from_rgb(r, g, b)
}

fn result_color(kind: &str) -> Color32 {
    match kind {
        "rojcad/shape" => Color32::YELLOW,
        "error" => Color32::RED,
        "number" => Color32::from_rgb(255, 180, 80),
        "string" => Color32::GREEN,
        "keyword" => Color32::YELLOW,
        "nil" => Color32::GRAY,
        _ => Color32::WHITE,
    }
}

/// Rainbow colours for bracket pairs at increasing nesting depth.
const RAINBOW: [Color32; 6] = [
    Color32::from_rgb(255, 255, 255), // depth 0: white
    Color32::from_rgb(255, 215, 0),   // depth 1: yellow
    Color32::from_rgb(80, 200, 255),  // depth 2: cyan
    Color32::from_rgb(78, 201, 176),  // depth 3: green
    Color32::from_rgb(200, 120, 200), // depth 4: purple
    Color32::from_rgb(255, 180, 80),  // depth 5+: orange
];

/// Minimal editor stub that uses our theme colors for the highlight tokenizer,
/// with rainbow bracket coloring.
struct HistoryEditor {
    depth: Cell<i32>,
}

impl std::hash::Hash for HistoryEditor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Cell doesn't impl Hash; use the address instead (singleton context).
        std::ptr::hash(self, state);
    }
}

impl HistoryEditor {
    fn new() -> Self {
        Self {
            depth: Cell::new(0),
        }
    }
}

impl egui_code_editor::Editor for HistoryEditor {
    fn append(&self, job: &mut LayoutJob, token: &egui_code_editor::Token) {
        let buf = token.buffer();
        let color = match token.ty() {
            TokenType::Comment(_) => hex_to_color32(ROJCAD_THEME.comments),
            TokenType::Keyword => hex_to_color32(ROJCAD_THEME.keywords),
            TokenType::Type => hex_to_color32(ROJCAD_THEME.types),
            TokenType::Function => hex_to_color32(ROJCAD_THEME.functions),
            TokenType::Str(_) => hex_to_color32(ROJCAD_THEME.strs),
            TokenType::Numeric(_) => hex_to_color32(ROJCAD_THEME.numerics),
            TokenType::Literal => hex_to_color32(ROJCAD_THEME.literals),
            TokenType::Special => hex_to_color32(ROJCAD_THEME.special),
            TokenType::Punctuation(_) => {
                // Rainbow brackets: colour based on nesting depth
                let d = self.depth.get();
                if buf == "(" || buf == "[" || buf == "{" {
                    self.depth.set(d + 1);
                    RAINBOW[(d as usize).min(RAINBOW.len() - 1)]
                } else if buf == ")" || buf == "]" || buf == "}" {
                    let new_d = (d - 1).max(0);
                    self.depth.set(new_d);
                    RAINBOW[(new_d as usize).min(RAINBOW.len() - 1)]
                } else {
                    hex_to_color32(ROJCAD_THEME.punctuation)
                }
            }
            _ => hex_to_color32(ROJCAD_THEME.punctuation),
        };
        job.append(
            buf,
            0.0,
            TextFormat {
                font_id: FontId::new(14.0, FontFamily::Monospace),
                color,
                ..Default::default()
            },
        );
    }
}

/// Build a syntax-coloured LayoutJob using the same tokenizer as the input editor,
/// with rainbow brackets.
fn code_to_job(code: &str) -> LayoutJob {
    let syntax = janet_syntax();
    let editor = HistoryEditor::new();
    let (job, _links) = Token::default().highlight(&editor, code, &syntax);
    job
}

impl ReplPanel {
    pub fn new(gui_req_tx: mpsc::Sender<String>) -> Self {
        Self {
            input: String::new(),
            history: Vec::new(),
            pending: None,
            pending_highlight_id: 0,
            next_id: 1,
            gui_req_tx,
            fn_names: HashSet::new(),
            ctrl_enter_pending: false,
        }
    }

    /// Pre-load function names from Janet for call detection.
    pub fn set_fn_names(&mut self, names: Vec<String>) {
        self.fn_names = names.into_iter().collect();
    }

    /// Called from the winit keyboard handler when Ctrl+Enter is pressed.
    /// The actual submission happens on the next egui frame.
    pub fn set_ctrl_enter_pending(&mut self) {
        self.ctrl_enter_pending = true;
    }

    /// Send an eval request to the REPL thread.
    pub fn submit(&mut self) {
        let code = self.input.trim().to_string();
        if code.is_empty() {
            return;
        }
        let id = self.next_id;
        self.next_id += 1;
        let msg = format!("e\x02{}\x02{}", id, &code);
        let _ = self.gui_req_tx.send(msg);
        self.pending = Some(PendingEval {
            id,
            code: self.input.clone(),
        });

        // Build a coloured LayoutJob using the same tokenizer as the input editor
        let code_job = code_to_job(&self.input);

        self.history.push(HistoryEntry {
            code_job,
            result: String::new(),
            kind: "pending".to_string(),
            shape_id: None,
        });
        self.input.clear();
    }

    /// Send a highlight request for the current input.
    pub fn request_highlight(&mut self) {
        if self.input.is_empty() {
            return;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.pending_highlight_id = id;
        let msg = format!("h\x02{}\x02{}", id, &self.input);
        let _ = self.gui_req_tx.send(msg);
    }

    /// Handle an incoming response JSON string from the REPL thread.
    pub fn handle_response(&mut self, resp: &str) {
        let mut parser = JsonValParser::new(resp);
        let val = parser.val();
        let obj = match &val {
            Some(JVal::Obj(obj)) => obj,
            _ => return,
        };
        let type_str = JsonValParser::obj_str(obj, "type").unwrap_or_default();

        match type_str.as_str() {
            "evalResult" => {
                let value = JsonValParser::obj_str(obj, "value").unwrap_or_default();
                let kind = JsonValParser::obj_str(obj, "kind").unwrap_or_default();
                self.pending = None;
                if let Some(entry) = self.history.last_mut() {
                    entry.result = value;
                    entry.kind = kind;
                }
            }
            "highlightResult" => {}
            "completionsResult" => {
                // TODO: complete
            }
            _ => {}
        }
    }

    pub fn visible(&self) -> bool {
        SHOW_REPL_PANEL.load(Ordering::Relaxed)
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("REPL")
            .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::ZERO)
            .resizable(true)
            .default_width(350.0)
            .min_width(200.0)
            .title_bar(false)
            .show(ctx, |ui| {
                // Track panel width for gizmo adjustment.
                let full_w = ctx.content_rect().width() - ui.clip_rect().left();
                REPL_PANEL_WIDTH.store(full_w as u32, Ordering::Relaxed);

                // Reserve space for input area + toolbar + separator below
                let input_area_height = 130.0;
                let avail = ui.available_height();
                let scroll_height = (avail - input_area_height).max(50.0);

                // Output log — constrained to leave room for input below
                egui::ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .stick_to_bottom(true)
                    .max_height(scroll_height)
                    .show(ui, |ui| {
                        let mono = FontId::new(14.0, FontFamily::Monospace);
                        for entry in &self.history {
                            ui.label(entry.code_job.clone());
                            if !entry.result.is_empty() {
                                let res_job = LayoutJob::simple(
                                    format!("  {}", entry.result),
                                    mono.clone(),
                                    result_color(&entry.kind),
                                    f32::INFINITY,
                                );
                                ui.label(res_job);
                            }
                        }
                    });

                ui.separator();

                // Code editor with shared Janet syntax and rainbow brackets
                let syntax = janet_syntax();
                let mut code_editor = CodeEditor::default()
                    .id_source("repl_input")
                    .with_fontsize(14.0)
                    .with_theme(ROJCAD_THEME)
                    .with_numlines(false)
                    .with_rows(6);
                let resp = code_editor.show(ui, &mut self.input, &syntax);

                // Request highlight on text change
                if resp.response.changed() {
                    self.request_highlight();
                }

                // Ctrl+Enter triggers submit (flag set by winit handler)
                if std::mem::take(&mut self.ctrl_enter_pending) {
                    self.submit();
                }

                // Toolbar
                ui.horizontal(|ui| {
                    if ui.button("\u{25B6} Run").clicked() {
                        self.submit();
                    }
                    if ui.button("\u{2715} Clear").clicked() {
                        self.history.clear();
                    }
                });
            });
    }
}
