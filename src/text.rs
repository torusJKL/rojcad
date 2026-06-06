//! Text shape creation — font loading, glyph outline extraction,
//! and conversion to OCCT shapes via ttf-parser.

use glam::DVec3;
use opencascade::primitives::{Compound, Edge, Face, Shape, Wire};
use opencascade::workplane::Workplane;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

// ── Segment types from a glyph outline ────────────────────────────────────

#[derive(Clone, Debug)]
pub enum Segment {
    Line((f64, f64), (f64, f64)),
    Quad((f64, f64), (f64, f64), (f64, f64)),
    Cubic((f64, f64), (f64, f64), (f64, f64), (f64, f64)),
}

impl Segment {
    fn control_points(&self) -> Vec<(f64, f64)> {
        match self {
            Segment::Line(p0, p1) => vec![*p0, *p1],
            Segment::Quad(p0, p1, p2) => vec![*p0, *p1, *p2],
            Segment::Cubic(p0, p1, p2, p3) => vec![*p0, *p1, *p2, *p3],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Contour {
    pub segments: Vec<Segment>,
}

impl Contour {
    fn bounds(&self) -> Option<(f64, f64, f64, f64)> {
        let mut pts = self.segments.iter().flat_map(|s| s.control_points()).fuse();
        let (x0, y0) = pts.next()?;
        let (mut x_min, mut x_max, mut y_min, mut y_max) = (x0, x0, y0, y0);
        for (x, y) in pts {
            if x < x_min {
                x_min = x;
            }
            if x > x_max {
                x_max = x;
            }
            if y < y_min {
                y_min = y;
            }
            if y > y_max {
                y_max = y;
            }
        }
        Some((x_min, y_min, x_max, y_max))
    }

    fn area(&self) -> f64 {
        self.bounds()
            .map(|(x1, y1, x2, y2)| (x2 - x1) * (y2 - y1))
            .unwrap_or(0.0)
    }
}

// ── Font loading ──────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct FontData {
    data: Vec<u8>,
    pub units_per_em: u16,
    pub ascender: i16,
    pub descender: i16,
}

impl FontData {
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, String> {
        let face = ttf_parser::Face::parse(&data, 0).map_err(|e| format!("invalid font: {e:?}"))?;
        Ok(Self {
            units_per_em: face.units_per_em(),
            ascender: face.ascender(),
            descender: face.descender(),
            data,
        })
    }

    pub fn from_path(path: &str) -> Result<Self, String> {
        // 1. Try direct path first
        if let Ok(data) = std::fs::read(path) {
            return Self::from_bytes(data);
        }

        // 2. Scan system font dirs (cached) and try to resolve
        let resolved = resolve_font_path(path);
        match resolved {
            Some(resolved_path) => {
                let data = std::fs::read(&resolved_path)
                    .map_err(|e| format!("cannot read font '{resolved_path}': {e}"))?;
                Self::from_bytes(data)
            }
            None => Err(format!(
                "cannot read font '{path}': No such file or directory"
            )),
        }
    }

    pub fn face(&self) -> ttf_parser::Face<'_> {
        ttf_parser::Face::parse(&self.data, 0).unwrap()
    }

    #[expect(dead_code)]
    pub fn font_name(&self) -> Option<String> {
        let face = self.face();
        // Prefer Windows+UnicodeBMP, name_id=1 (family)
        for n in face.names() {
            if n.name_id == 1
                && n.platform_id == ttf_parser::PlatformId::Windows
                && n.encoding_id == 1
                && let Some(s) = n.to_string()
            {
                return Some(s);
            }
        }
        // fallback: any unicode name with name_id=1
        for n in face.names() {
            if n.name_id == 1
                && let Some(s) = n.to_string()
            {
                return Some(s);
            }
        }
        None
    }
}

// ── Glyph outline extraction via OutlineBuilder ────────────────────────────

struct GlyphBuilder {
    contours: Vec<Contour>,
    current: Vec<Segment>,
    first: Option<(f64, f64)>,
    cursor: Option<(f64, f64)>,
}

impl GlyphBuilder {
    fn new() -> Self {
        Self {
            contours: Vec::new(),
            current: Vec::new(),
            first: None,
            cursor: None,
        }
    }

    fn close_contour(&mut self) {
        if !self.current.is_empty() {
            self.contours.push(Contour {
                segments: std::mem::take(&mut self.current),
            });
        }
        self.first = None;
        self.cursor = None;
    }

    fn push(&mut self, s: Segment) {
        match &s {
            Segment::Line(_, e) | Segment::Quad(_, _, e) | Segment::Cubic(_, _, _, e) => {
                self.cursor = Some(*e);
            }
        }
        self.current.push(s);
    }
}

impl ttf_parser::OutlineBuilder for GlyphBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.close_contour();
        let p = (x as f64, y as f64);
        self.first = Some(p);
        self.cursor = Some(p);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        if let Some(s) = self.cursor {
            self.push(Segment::Line(s, (x as f64, y as f64)));
        }
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        if let Some(s) = self.cursor {
            self.push(Segment::Quad(
                s,
                (x1 as f64, y1 as f64),
                (x as f64, y as f64),
            ));
        }
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        if let Some(s) = self.cursor {
            self.push(Segment::Cubic(
                s,
                (x1 as f64, y1 as f64),
                (x2 as f64, y2 as f64),
                (x as f64, y as f64),
            ));
        }
    }
    fn close(&mut self) {
        if let (Some(fp), Some(cp)) = (self.first, self.cursor)
            && ((cp.0 - fp.0).abs() > 1e-10 || (cp.1 - fp.1).abs() > 1e-10)
        {
            self.push(Segment::Line(cp, fp));
        }
        self.close_contour();
    }
}

pub fn glyph_outlines(font: &FontData, gid: ttf_parser::GlyphId) -> Option<Vec<Contour>> {
    let face = font.face();
    let mut builder = GlyphBuilder::new();
    let ok = face.outline_glyph(gid, &mut builder);
    builder.close_contour();
    ok.map(|_| builder.contours)
}

// ── Layout ─────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct PositionedGlyph {
    pub contours: Vec<Contour>,
    pub x_offset: f64,
}

pub fn layout_text(font: &FontData, text: &str, size: f64) -> Result<Vec<PositionedGlyph>, String> {
    if text.is_empty() {
        return Err("text: empty string".to_string());
    }
    let upem = font.units_per_em as f64;
    if upem == 0.0 {
        return Err("text: font has zero units-per-em".to_string());
    }
    let scale = size / upem;
    let face = font.face();
    let mut result = Vec::new();
    let mut cx = 0.0;

    for ch in text.chars() {
        let gid = face.glyph_index(ch).unwrap_or(ttf_parser::GlyphId(0));
        let advance = face.glyph_hor_advance(gid).unwrap_or(0) as f64 * scale;
        if let Some(contours) = glyph_outlines(font, gid)
            && !contours.is_empty()
        {
            result.push(PositionedGlyph {
                contours,
                x_offset: cx,
            });
        }
        cx += advance;
    }

    if result.is_empty() {
        return Err("text: no visible glyphs in string".to_string());
    }

    Ok(result)
}

// ── Contour → OCCT Edge conversion ────────────────────────────────────────

fn seg_to_edges(seg: &Segment, scale: f64, x_off: f64, y_off: f64, wp: &Workplane) -> Edge {
    let to_world =
        |x: f64, y: f64| wp.to_world_pos(DVec3::new(x * scale + x_off, y * scale + y_off, 0.0));
    match *seg {
        Segment::Line(p0, p1) => Edge::segment(to_world(p0.0, p0.1), to_world(p1.0, p1.1)),
        Segment::Quad(p0, p1, p2) => Edge::bezier([
            to_world(p0.0, p0.1),
            to_world(p1.0, p1.1),
            to_world(p2.0, p2.1),
        ]),
        Segment::Cubic(p0, p1, p2, p3) => Edge::bezier([
            to_world(p0.0, p0.1),
            to_world(p1.0, p1.1),
            to_world(p2.0, p2.1),
            to_world(p3.0, p3.1),
        ]),
    }
}

fn contour_to_edges(
    contour: &Contour,
    scale: f64,
    x_off: f64,
    y_off: f64,
    wp: &Workplane,
) -> Vec<Edge> {
    contour
        .segments
        .iter()
        .map(|s| seg_to_edges(s, scale, x_off, y_off, wp))
        .collect()
}

/// Split contours into outer (largest area) and holes (rest).
fn split_outer_holes(contours: &[Contour]) -> (usize, Vec<usize>) {
    if contours.len() <= 1 {
        return (0, vec![]);
    }
    let outer_idx = contours
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.area().partial_cmp(&b.area()).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0);
    let holes: Vec<usize> = (0..contours.len()).filter(|i| *i != outer_idx).collect();
    (outer_idx, holes)
}

// ── Public API ────────────────────────────────────────────────────────────

/// Build a 2D text Shape (Face or Compound of Faces) from a string.
///
/// Each glyph with holes (e.g. 'A', 'B', '8') has holes punched via
/// boolean subtract. Multiple glyphs are grouped into a Compound.
pub fn text_to_shape(
    text: &str,
    font: &FontData,
    size: f64,
    wp: &Workplane,
) -> Result<Shape, String> {
    let positioned = layout_text(font, text, size)?;
    let upem = font.units_per_em as f64;
    let scale = size / upem;

    // Y-offset to center text vertically: baseline at -(descender + ascender)/2
    let asc = font.ascender as f64 * scale;
    let desc = font.descender as f64 * scale;
    let y_off = -(asc + desc) / 2.0 - desc;

    let mut glyph_shapes: Vec<Shape> = Vec::new();

    for pg in &positioned {
        let (outer_idx, hole_idxs) = split_outer_holes(&pg.contours);

        if outer_idx >= pg.contours.len() || pg.contours[outer_idx].segments.is_empty() {
            continue;
        }

        let outer_edges = contour_to_edges(&pg.contours[outer_idx], scale, pg.x_offset, y_off, wp);
        let outer_wire = Wire::from_edges(&outer_edges);
        let outer_face = Face::from_wire(&outer_wire);
        let outer_shape = Shape::from(outer_face);

        let hole_shapes: Vec<Shape> = hole_idxs
            .into_iter()
            .filter(|i| !pg.contours[*i].segments.is_empty())
            .map(|i| {
                let edges = contour_to_edges(&pg.contours[i], scale, pg.x_offset, y_off, wp);
                let wire = Wire::from_edges(&edges);
                let face = Face::from_wire(&wire);
                Shape::from(face)
            })
            .collect();

        if hole_shapes.is_empty() {
            glyph_shapes.push(outer_shape);
            continue;
        }

        let mut current = outer_shape;
        for hole in &hole_shapes {
            current = current.subtract(hole).shape;
        }
        glyph_shapes.push(current);
    }

    if glyph_shapes.is_empty() {
        return Err("text: no glyphs produced geometry".to_string());
    }

    // Combine into a single Shape
    if glyph_shapes.len() == 1 {
        Ok(glyph_shapes.into_iter().next().unwrap())
    } else {
        let compound = Compound::from_shapes(&glyph_shapes);
        Ok(Shape::from(compound))
    }
}

/// Build an extruded 3D text Shape from a string.
pub fn text_to_solid(
    text: &str,
    font: &FontData,
    size: f64,
    depth: f64,
    both: bool,
    wp: &Workplane,
) -> Result<Shape, String> {
    let positioned = layout_text(font, text, size)?;
    let upem = font.units_per_em as f64;
    let scale = size / upem;

    let asc = font.ascender as f64 * scale;
    let desc = font.descender as f64 * scale;
    let y_off = -(asc + desc) / 2.0 - desc;

    let mut solid_shapes: Vec<Shape> = Vec::new();

    for pg in &positioned {
        let (outer_idx, hole_idxs) = split_outer_holes(&pg.contours);

        if outer_idx >= pg.contours.len() || pg.contours[outer_idx].segments.is_empty() {
            continue;
        }

        let outer_edges = contour_to_edges(&pg.contours[outer_idx], scale, pg.x_offset, y_off, wp);
        let outer_wire = Wire::from_edges(&outer_edges);
        let outer_face = Face::from_wire(&outer_wire);

        let hole_shapes: Vec<Shape> = hole_idxs
            .into_iter()
            .filter(|i| !pg.contours[*i].segments.is_empty())
            .map(|i| {
                let edges = contour_to_edges(&pg.contours[i], scale, pg.x_offset, y_off, wp);
                Shape::from(Face::from_wire(&Wire::from_edges(&edges)))
            })
            .collect();

        let mut current_shape = Shape::from(&outer_face);
        for hole in &hole_shapes {
            current_shape = current_shape.subtract(hole).shape;
        }

        let dir = DVec3::new(0.0, 0.0, depth);
        let half_z = DVec3::new(0.0, 0.0, depth / 2.0);

        for face in current_shape.faces() {
            let solid = if both {
                let shifted = Shape::from(&face).translated(-half_z);
                shifted.expect_face().extrude(dir)
            } else {
                face.extrude(dir)
            };
            solid_shapes.push(Shape::from(solid));
        }
    }

    if solid_shapes.is_empty() {
        return Err("text: no glyphs produced geometry".to_string());
    }

    if solid_shapes.len() == 1 {
        Ok(solid_shapes.into_iter().next().unwrap())
    } else {
        Ok(Shape::from(Compound::from_shapes(&solid_shapes)))
    }
}

// ── System font listing ───────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum FontAspect {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

impl FontAspect {
    pub fn as_str(&self) -> &'static str {
        match self {
            FontAspect::Regular => "regular",
            FontAspect::Bold => "bold",
            FontAspect::Italic => "italic",
            FontAspect::BoldItalic => "bold-italic",
        }
    }
}

fn scan_font_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // Linux / BSD
    if cfg!(target_os = "linux") || cfg!(target_os = "freebsd") || cfg!(target_os = "netbsd") {
        dirs.push(PathBuf::from("/usr/share/fonts"));
        dirs.push(PathBuf::from("/usr/local/share/fonts"));
        if let Ok(ref home) = std::env::var("HOME") {
            dirs.push(PathBuf::from(home).join(".fonts"));
            dirs.push(PathBuf::from(home).join(".local/share/fonts"));
        }
    }

    // macOS
    if cfg!(target_os = "macos") {
        dirs.push(PathBuf::from("/Library/Fonts"));
        dirs.push(PathBuf::from("/System/Library/Fonts"));
        if let Ok(ref home) = std::env::var("HOME") {
            dirs.push(PathBuf::from(home).join("Library/Fonts"));
        }
    }

    // Windows
    if cfg!(target_os = "windows") {
        if let Ok(root) = std::env::var("WINDIR") {
            dirs.push(PathBuf::from(root).join("Fonts"));
        }
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            dirs.push(PathBuf::from(local).join("Microsoft/Windows/Fonts"));
        }
    }

    dirs
}

fn collect_font_files(dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for dir in dirs {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    files.append(&mut collect_font_files(&[path]));
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str())
                    && (ext.eq_ignore_ascii_case("ttf")
                        || ext.eq_ignore_ascii_case("otf")
                        || ext.eq_ignore_ascii_case("ttc"))
                {
                    files.push(path);
                }
            }
        }
    }
    files
}

pub fn list_system_fonts() -> Vec<(String, String, FontAspect)> {
    let dirs = scan_font_dirs();
    let files = collect_font_files(&dirs);
    let mut seen = HashSet::new();
    let mut results = Vec::new();

    for path in &files {
        let path_str = path.to_string_lossy().to_string();
        // Try to parse the font and get its name
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let face = match ttf_parser::Face::parse(&data, 0) {
            Ok(f) => f,
            Err(_) => continue,
        };

        // Get font family name
        let name = get_font_name_from_face(&face);

        // Determine aspect
        let aspect = if face.is_bold() && face.is_italic() {
            FontAspect::BoldItalic
        } else if face.is_bold() {
            FontAspect::Bold
        } else if face.is_italic() {
            FontAspect::Italic
        } else {
            FontAspect::Regular
        };

        // Deduplicate by (name, aspect)
        let key = (name.clone(), aspect.as_str().to_string());
        if seen.insert(key) {
            results.push((name, path_str, aspect));
        }
    }

    results.sort_by(|a, b| a.0.cmp(&b.0));
    results
}

fn get_font_name_from_face(face: &ttf_parser::Face<'_>) -> String {
    for n in face.names() {
        if n.name_id == 1
            && n.platform_id == ttf_parser::PlatformId::Windows
            && n.encoding_id == 1
            && let Some(s) = n.to_string()
        {
            return s;
        }
    }
    for n in face.names() {
        if n.name_id == 1
            && let Some(s) = n.to_string()
        {
            return s;
        }
    }
    String::new()
}

/// Read the family name from a font file, lowercased.
fn read_font_name(path: &Path) -> Option<String> {
    let data = std::fs::read(path).ok()?;
    let face = ttf_parser::Face::parse(&data, 0).ok()?;
    let name = get_font_name_from_face(&face);
    if name.is_empty() {
        None
    } else {
        Some(name.to_lowercase())
    }
}

/// Cached font lookup: (filename→path, name→path).
fn font_lookup() -> &'static Vec<(String, String)> {
    static LOOKUP: OnceLock<Vec<(String, String)>> = OnceLock::new();
    LOOKUP.get_or_init(|| {
        let files = collect_font_files(&scan_font_dirs());
        let mut entries = Vec::new();

        for path in &files {
            let path_str = path.to_string_lossy().to_string();
            // Index by filename, e.g. "DejaVuSans.ttf"
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                entries.push((name.to_lowercase(), path_str.clone()));
            }
            // Index by font family name, e.g. "DejaVu Sans"
            if let Some(name) = read_font_name(path) {
                entries.push((name, path_str));
            }
        }

        entries
    })
}

/// Resolve a font path by trying: direct, filename lookup, font name lookup.
fn resolve_font_path(input: &str) -> Option<String> {
    let lower = input.to_lowercase();
    for (key, path) in font_lookup() {
        if *key == lower {
            return Some(path.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_text_errors() {
        let data = vec![0u8; 4];
        assert!(FontData::from_bytes(data).is_err());
    }

    #[test]
    fn test_layout_text_empty_string() {
        let data = std::fs::read("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
        if let Some(font_data) = data.ok().and_then(|d| FontData::from_bytes(d).ok()) {
            let result = layout_text(&font_data, "", 10.0);
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("empty"));
        }
    }

    #[test]
    fn test_font_from_invalid_path() {
        let result = FontData::from_path("/nonexistent/font.ttf");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot read"));
    }

    #[test]
    fn test_glyph_outlines_valid_char() {
        let data = std::fs::read("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
        if let Some(font_data) = data.ok().and_then(|d| FontData::from_bytes(d).ok()) {
            let face = font_data.face();
            if let Some(gid) = face.glyph_index('A') {
                let outlines = glyph_outlines(&font_data, gid);
                assert!(outlines.is_some());
                let contours = outlines.unwrap();
                assert!(
                    contours.len() >= 2,
                    "expected 'A' to have >=2 contours, got {}",
                    contours.len()
                );
            }
        }
    }

    #[test]
    fn test_glyph_outlines_invalid_gid() {
        let data = std::fs::read("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
        if let Some(font_data) = data.ok().and_then(|d| FontData::from_bytes(d).ok()) {
            let outlines = glyph_outlines(&font_data, ttf_parser::GlyphId(u16::MAX));
            assert!(outlines.is_none());
        }
    }

    #[test]
    fn test_hole_detection_outer_largest() {
        let data = std::fs::read("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
        if let Some(font_data) = data.ok().and_then(|d| FontData::from_bytes(d).ok()) {
            let face = font_data.face();
            if let Some(gid) = face.glyph_index('A') {
                if let Some(contours) = glyph_outlines(&font_data, gid) {
                    let (outer_idx, hole_idxs) = split_outer_holes(&contours);
                    assert!(outer_idx < contours.len());
                    assert!(!hole_idxs.is_empty(), "'A' should have hole contours");
                    let outer_area = contours[outer_idx].area();
                    for &hi in &hole_idxs {
                        assert!(
                            outer_area > contours[hi].area(),
                            "outer {outer_area} should be > hole area {}",
                            contours[hi].area()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_text_to_shape_basic() {
        let data = std::fs::read("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
        if let Some(font_data) = data.ok().and_then(|d| FontData::from_bytes(d).ok()) {
            let wp = Workplane::xy();
            let result = text_to_shape("Hi", &font_data, 10.0, &wp);
            assert!(result.is_ok(), "text_to_shape failed: {:?}", result.err());
        }
    }

    #[test]
    fn test_text_to_solid_basic() {
        let data = std::fs::read("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
        if let Some(font_data) = data.ok().and_then(|d| FontData::from_bytes(d).ok()) {
            let wp = Workplane::xy();
            let result = text_to_solid("Hi", &font_data, 10.0, 5.0, false, &wp);
            assert!(result.is_ok(), "text_to_solid failed: {:?}", result.err());
        }
    }

    #[test]
    fn test_text_to_solid_both() {
        let data = std::fs::read("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
        if let Some(font_data) = data.ok().and_then(|d| FontData::from_bytes(d).ok()) {
            let wp = Workplane::xy();
            let result = text_to_solid("Hi", &font_data, 10.0, 5.0, true, &wp);
            assert!(
                result.is_ok(),
                "text_to_solid(both) failed: {:?}",
                result.err()
            );
        }
    }

    #[test]
    fn test_text_with_hole_glyph() {
        let data = std::fs::read("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
        if let Some(font_data) = data.ok().and_then(|d| FontData::from_bytes(d).ok()) {
            let wp = Workplane::xy();
            let result = text_to_shape("AO", &font_data, 10.0, &wp);
            assert!(result.is_ok(), "text with holes failed: {:?}", result.err());
        }
    }

    #[test]
    fn test_list_fonts_at_least_returns_something() {
        let fonts = list_system_fonts();
        for (_name, path, aspect) in &fonts {
            assert!(!path.is_empty(), "font entry has empty path");
            assert!(
                std::path::Path::new(path).exists(),
                "font path doesn't exist: {path}"
            );
            match aspect.as_str() {
                "regular" | "bold" | "italic" | "bold-italic" => {}
                other => panic!("unexpected font aspect: {other}"),
            }
        }
    }

    #[test]
    fn test_font_resolve_by_filename() {
        // "DejaVuSans.ttf" is a bare filename — should resolve via system font scan
        let result = FontData::from_path("DejaVuSans.ttf");
        assert!(
            result.is_ok(),
            "bare filename 'DejaVuSans.ttf' should resolve: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_font_resolve_by_name() {
        // "DejaVu Sans" is a font family name — should resolve via font name lookup
        let result = FontData::from_path("DejaVu Sans");
        assert!(
            result.is_ok(),
            "font name 'DejaVu Sans' should resolve: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_font_resolve_bogus_fails() {
        let result = FontData::from_path("QuxZotFakeFont.xyz");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No such file or directory"));
    }
}
