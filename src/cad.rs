//! CAD operations — Rust wrappers around opencascade-rs.
//!
//! Implements: box, sphere, cut, common, shape type, export.

use std::f64::consts::TAU;

use glam::DVec3;
use opencascade::primitives::{Shape, ShapeType};

use crate::types::{MeshData, ShapeData, global_shape_registry};

/// Extract tessellated mesh data from an OCCT shape.
pub fn extract_mesh(shape: &Shape) -> Option<MeshData> {
    match shape.mesh() {
        Ok(occt_mesh) => {
            let vertices: Vec<[f32; 3]> = occt_mesh
                .vertices
                .iter()
                .map(|v| [v.x as f32, v.y as f32, v.z as f32])
                .collect();
            let normals: Vec<[f32; 3]> = occt_mesh
                .normals
                .iter()
                .map(|n| [n.x as f32, n.y as f32, n.z as f32])
                .collect();
            let indices: Vec<u32> = occt_mesh.indices.iter().map(|i| *i as u32).collect();
            Some(MeshData {
                vertices,
                normals,
                indices,
            })
        }
        Err(_) => None,
    }
}

/// Extract edge polylines from an OCCT shape by sampling each topological edge.
pub fn extract_edge_polylines(shape: &Shape) -> Vec<Vec<[f64; 3]>> {
    let mut polylines = Vec::new();
    for edge in shape.edges() {
        let points: Vec<[f64; 3]> = edge
            .approximation_segments()
            .map(|p| [p.x, p.y, p.z])
            .collect();
        if points.len() >= 2 {
            polylines.push(points);
        }
    }
    polylines
}

/// Generate a minimal synthetic wireframe for curved shapes:
/// an equator circle (horizontal) and a meridian circle (vertical),
/// computed from the mesh bounding sphere.
/// Uses a fixed 32-segment circle for smooth appearance.
pub fn generate_synthetic_wireframe(mesh: &MeshData) -> Vec<Vec<[f64; 3]>> {
    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];
    for v in &mesh.vertices {
        for i in 0..3 {
            min[i] = min[i].min(v[i] as f64);
            max[i] = max[i].max(v[i] as f64);
        }
    }
    let cx = (min[0] + max[0]) / 2.0;
    let cy = (min[1] + max[1]) / 2.0;
    let cz = (min[2] + max[2]) / 2.0;
    let rx = (max[0] - min[0]) / 2.0;
    let ry = (max[1] - min[1]) / 2.0;
    let rz = (max[2] - min[2]) / 2.0;

    let segments = 32;
    let mut polylines = Vec::new();

    // Horizontal circle (equator) — in XZ plane at center Y
    let mut equator: Vec<[f64; 3]> = Vec::with_capacity(segments + 1);
    for i in 0..=segments {
        let theta = (i as f64 / segments as f64) * TAU;
        equator.push([cx + rx * theta.cos(), cy, cz + rz * theta.sin()]);
    }
    polylines.push(equator);

    // Vertical circle (meridian) — in XY plane at center Z
    let mut meridian: Vec<[f64; 3]> = Vec::with_capacity(segments + 1);
    for i in 0..=segments {
        let theta = (i as f64 / segments as f64) * TAU;
        meridian.push([cx + rx * theta.cos(), cy + ry * theta.sin(), cz]);
    }
    polylines.push(meridian);

    polylines
}

/// Number of topological edges below which we add synthetic wireframe edges.
/// Polyhedral shapes (box has 12 edges) keep only topological edges;
/// curved shapes (sphere has 1 seam edge) get the synthetic fallback.
const SYNTHETIC_WIREFRAME_THRESHOLD: usize = 8;

/// Tessellate a shape and update its entry in the global registry.
/// For shapes with few topological edges (curved shapes), adds a clean
/// synthetic wireframe (equator + meridian circles) instead of dense mesh edges.
pub fn tessellate_and_update(shape_id: u64, shape: &Shape) {
    let mesh = extract_mesh(shape);
    let mut edge_polylines = extract_edge_polylines(shape);
    if edge_polylines.len() < SYNTHETIC_WIREFRAME_THRESHOLD {
        if let Some(ref m) = mesh {
            edge_polylines.extend(generate_synthetic_wireframe(m));
        }
    }
    global_shape_registry().update(shape_id, mesh, edge_polylines);
}

// ── Primitives ────────────────────────────────────────────────────────────────

/// Create a box with the given dimensions.
///
/// Width = X, Depth = Y, Height = Z, with one corner at (0,0,0).
/// If `center` is provided, the box is translated so its geometric center
/// is at that point.
pub fn make_box(
    width: f64,
    depth: f64,
    height: f64,
    center: Option<(f64, f64, f64)>,
) -> ShapeData {
    assert_valid_dimension(width, "width");
    assert_valid_dimension(depth, "depth");
    assert_valid_dimension(height, "height");

    let mut shape = Shape::box_with_dimensions(width, depth, height);
    if let Some((cx, cy, cz)) = center {
        // box_with_dimensions puts one corner at (0,0,0).
        // To center at (cx, cy, cz), translate by (cx - w/2, cy - d/2, cz - h/2).
        translate_shape(
            &mut shape,
            cx - width / 2.0,
            cy - depth / 2.0,
            cz - height / 2.0,
        );
    }
    let sd = ShapeData::new(shape);
    tessellate_and_update(sd.shape_id, &sd.shape);
    sd
}

/// Create a sphere with the given radius.
///
/// The sphere is centered at (0,0,0) by default.
/// If `center` is provided, the sphere is centered at that point.
pub fn make_sphere(radius: f64, center: Option<(f64, f64, f64)>) -> ShapeData {
    assert_valid_dimension(radius, "radius");

    let mut shape = Shape::sphere(radius).build();
    if let Some((cx, cy, cz)) = center {
        translate_shape(&mut shape, cx, cy, cz);
    }
    let sd = ShapeData::new(shape);
    tessellate_and_update(sd.shape_id, &sd.shape);
    sd
}

// ── Boolean Operations ────────────────────────────────────────────────────────

/// Subtract shape `b` from shape `a`.
///
/// OCCT boolean operations may return the result as a `COMPOUND`.
/// Returns a `ShapeData` wrapping the resulting shape.
/// Panics if the result is a null/empty shape (`ShapeType::Shape`).
pub fn cut(a: &ShapeData, b: &ShapeData) -> ShapeData {
    let result = a.shape.subtract(&b.shape);
    let shape = result.shape;
    if shape.shape_type() == ShapeType::Shape {
        panic!("cut: shapes do not intersect or produced an empty result");
    }
    let sd = ShapeData::new(shape);
    tessellate_and_update(sd.shape_id, &sd.shape);
    sd
}

/// Intersect shape `a` with shape `b`.
///
/// OCCT boolean operations may return the result as a `COMPOUND`.
/// Returns a `ShapeData` wrapping the resulting shape.
/// Panics if the result is a null/empty shape (`ShapeType::Shape`).
pub fn common(a: &ShapeData, b: &ShapeData) -> ShapeData {
    let result = a.shape.intersect(&b.shape);
    let shape = result.shape;
    if shape.shape_type() == ShapeType::Shape {
        panic!("common: shapes do not intersect or produced an empty result");
    }
    let sd = ShapeData::new(shape);
    tessellate_and_update(sd.shape_id, &sd.shape);
    sd
}

// ── Shape Translation ─────────────────────────────────────────────────────────

/// Translate a shape by the given offset.
pub fn translate_shape(shape: &mut Shape, dx: f64, dy: f64, dz: f64) {
    shape.set_global_translation(DVec3::new(dx, dy, dz));
}

// ── Export ────────────────────────────────────────────────────────────────────

/// Write a shape to a STEP file.
pub fn write_step(data: &ShapeData, path: &str) -> Result<(), String> {
    data.shape
        .write_step(path)
        .map_err(|e| format!("STEP export failed: {}", e))
}

/// Write a shape to an STL file.
pub fn write_stl(data: &ShapeData, path: &str) -> Result<(), String> {
    data.shape
        .write_stl(path)
        .map_err(|e| format!("STL export failed: {}", e))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn assert_valid_dimension(value: f64, name: &str) {
    if value <= 0.0 {
        panic!("{} must be positive, got {}", name, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_box_default() {
        // 10.1: Test box creation via raw Rust API
        let sd = make_box(10.0, 20.0, 30.0, None);
        assert_eq!(sd.type_string(), "SOLID");
        assert!(sd.visible);
        assert!(sd.color.is_none());
    }

    #[test]
    fn test_make_box_centered() {
        let sd = make_box(10.0, 20.0, 30.0, Some((5.0, 10.0, 15.0)));
        assert_eq!(sd.type_string(), "SOLID");
        assert!(sd.visible);
    }

    #[test]
    fn test_make_sphere_default() {
        // 10.2: Test sphere creation via raw Rust API
        let sd = make_sphere(10.0, None);
        assert_eq!(sd.type_string(), "SOLID");
        assert!(sd.visible);
    }

    #[test]
    fn test_make_sphere_centered() {
        let sd = make_sphere(5.0, Some((1.0, 2.0, 3.0)));
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    #[should_panic(expected = "width must be positive")]
    fn test_make_box_invalid_width() {
        // 4.6: Validate inputs — reject zero/negative dimensions
        make_box(0.0, 10.0, 10.0, None);
    }

    #[test]
    #[should_panic(expected = "radius must be positive")]
    fn test_make_sphere_invalid_radius() {
        make_sphere(-1.0, None);
    }

    #[test]
    fn test_cut() {
        // 10.3: Test cut via raw Rust API
        let box_a = make_box(20.0, 20.0, 20.0, None);
        let sphere_b = make_sphere(10.0, Some((10.0, 10.0, 10.0)));
        let result = cut(&box_a, &sphere_b);
        // OCCT may return a COMPOUND or SOLID for boolean results
        assert!(
            result.type_string() == "SOLID" || result.type_string() == "COMPOUND",
            "expected SOLID or COMPOUND, got {}",
            result.type_string()
        );
        // Verify original shapes unchanged
        assert!(box_a.visible);
        assert_eq!(box_a.type_string(), "SOLID");
    }

    #[test]
    fn test_common() {
        // 10.4: Test common via raw Rust API
        let sphere_a = make_sphere(10.0, None);
        let box_b = make_box(10.0, 10.0, 10.0, None);
        let result = common(&sphere_a, &box_b);
        // Overlapping shapes should produce a valid result
        assert!(
            result.type_string() == "SOLID" || result.type_string() == "COMPOUND",
            "expected SOLID or COMPOUND, got {}",
            result.type_string()
        );
    }

    #[test]
    fn test_cut_non_overlapping() {
        // 10.5: Test cut with non-overlapping shapes
        // OCCT may return the original shape unchanged or a null shape.
        // Verify the result is not a null shape (ShapeType::Shape).
        let box_a = make_box(10.0, 10.0, 10.0, None);
        let box_b = make_box(10.0, 10.0, 10.0, Some((100.0, 0.0, 0.0)));
        let result = cut(&box_a, &box_b);
        assert_ne!(
            result.type_string(),
            "SHAPE",
            "cut of non-overlapping shapes should not produce a null shape"
        );
    }

    #[test]
    fn test_write_step_roundtrip() {
        // 10.6: Test STEP export round-trip
        let sd = make_box(10.0, 20.0, 30.0, None);
        let path = "/tmp/test_rojcad_box.step";
        assert!(write_step(&sd, path).is_ok());
        assert!(std::path::Path::new(path).exists());
        // Clean up
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_write_stl() {
        // 10.7: Test STL export
        let sd = make_sphere(10.0, None);
        let path = "/tmp/test_rojcad_sphere.stl";
        assert!(write_stl(&sd, path).is_ok());
        assert!(std::path::Path::new(path).exists());
        let metadata = std::fs::metadata(path).unwrap();
        assert!(metadata.len() > 0);
        // Clean up
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_visibility() {
        // 10.8: Test visibility flag lifecycle
        let mut sd = make_box(10.0, 10.0, 10.0, None);
        assert!(sd.visible);
        sd.visible = false;
        assert!(!sd.visible);
        sd.visible = true;
        assert!(sd.visible);
    }

    #[test]
    fn test_shape_type() {
        assert_eq!(make_box(10.0, 10.0, 10.0, None).type_string(), "SOLID");
        assert_eq!(make_sphere(10.0, None).type_string(), "SOLID");
    }
}
