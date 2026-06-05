//! CAD operations — Rust wrappers around opencascade-rs.
//!
//! Implements: box, sphere, cylinder, cone, torus, cut, common, shape type, export.

use std::f64::consts::TAU;

use glam::DVec3;
use opencascade::primitives::{Shape, ShapeType};

use crate::types::{MeshData, ShapeData};

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
pub const SYNTHETIC_WIREFRAME_THRESHOLD: usize = 8;



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
    eager: bool,
) -> ShapeData {
    assert_valid_dimension(width, "width");
    assert_valid_dimension(depth, "depth");
    assert_valid_dimension(height, "height");

    let mut shape = Shape::box_with_dimensions(width, depth, height);
    if let Some((cx, cy, cz)) = center {
        translate_shape(
            &mut shape,
            cx - width / 2.0,
            cy - depth / 2.0,
            cz - height / 2.0,
        );
    }
    let mut sd = ShapeData::new(shape);
    if eager { sd.tessellate_if_needed(); }
    sd
}

/// Create a cube with the given side length.
///
/// One corner at (0,0,0) by default.
/// If `center` is provided, the cube is centered at that point.
pub fn make_cube(size: f64, center: Option<(f64, f64, f64)>, eager: bool) -> ShapeData {
    assert_valid_dimension(size, "size");
    let mut shape = Shape::cube(size);
    if let Some((cx, cy, cz)) = center {
        translate_shape(&mut shape, cx, cy, cz);
    }
    let mut sd = ShapeData::new(shape);
    if eager { sd.tessellate_if_needed(); }
    sd
}

/// Create a box from two opposite corners.
pub fn make_box_from_corners(
    corner1: (f64, f64, f64),
    corner2: (f64, f64, f64),
    eager: bool,
) -> ShapeData {
    let c1 = DVec3::new(corner1.0, corner1.1, corner1.2);
    let c2 = DVec3::new(corner2.0, corner2.1, corner2.2);
    let shape = Shape::box_from_corners(c1, c2);
    let mut sd = ShapeData::new(shape);
    if eager { sd.tessellate_if_needed(); }
    sd
}

/// Create a sphere with the given radius.
///
/// The sphere is centered at (0,0,0) by default.
/// If `center` is provided, the sphere is centered at that point.
/// If `angle` is provided, creates a partial sphere (e.g., hemisphere).
pub fn make_sphere(
    radius: f64,
    center: Option<(f64, f64, f64)>,
    angle: Option<f64>,
    eager: bool,
) -> ShapeData {
    assert_valid_dimension(radius, "radius");

    let mut builder = Shape::sphere(radius);
    if let Some(a) = angle {
        builder = builder.z_angle(a);
    }
    let mut shape = builder.build();
    if let Some((cx, cy, cz)) = center {
        translate_shape(&mut shape, cx, cy, cz);
    }
    let mut sd = ShapeData::new(shape);
    if eager { sd.tessellate_if_needed(); }
    sd
}

/// Create a cylinder with the given radius and height along the Z axis.
///
/// The base is at Z=0 by default.
/// If `center` is provided, the cylinder is centered at that point.
pub fn make_cylinder(
    radius: f64,
    height: f64,
    center: Option<(f64, f64, f64)>,
    eager: bool,
) -> ShapeData {
    assert_valid_dimension(radius, "radius");
    assert_valid_dimension(height, "height");

    let shape = if let Some((cx, cy, cz)) = center {
        Shape::cylinder_centered(DVec3::new(cx, cy, cz), radius, DVec3::Z, height)
    } else {
        Shape::cylinder_radius_height(radius, height)
    };
    let mut sd = ShapeData::new(shape);
    if eager { sd.tessellate_if_needed(); }
    sd
}

/// Create a cylinder between two points with the given radius.
pub fn make_cylinder_from_points(
    p1: (f64, f64, f64),
    p2: (f64, f64, f64),
    radius: f64,
    eager: bool,
) -> ShapeData {
    assert_valid_dimension(radius, "radius");
    let shape = Shape::cylinder_from_points(
        DVec3::new(p1.0, p1.1, p1.2),
        DVec3::new(p2.0, p2.1, p2.2),
        radius,
    );
    let mut sd = ShapeData::new(shape);
    if eager { sd.tessellate_if_needed(); }
    sd
}

/// Create a cylinder at a given point, extending in the given direction.
pub fn make_cylinder_point_dir(
    point: (f64, f64, f64),
    radius: f64,
    dir: (f64, f64, f64),
    height: f64,
    eager: bool,
) -> ShapeData {
    assert_valid_dimension(radius, "radius");
    assert_valid_dimension(height, "height");
    let shape = Shape::cylinder(
        DVec3::new(point.0, point.1, point.2),
        radius,
        DVec3::new(dir.0, dir.1, dir.2),
        height,
    );
    let mut sd = ShapeData::new(shape);
    if eager { sd.tessellate_if_needed(); }
    sd
}

/// Create a cone with the given bottom radius, top radius, and height.
///
/// A full cone has top_radius = 0. A truncated cone has top_radius > 0.
/// If `center` is provided, the cone is centered at that point.
/// If `angle` is provided, creates a partial cone.
pub fn make_cone(
    bottom_radius: f64,
    top_radius: f64,
    height: f64,
    center: Option<(f64, f64, f64)>,
    angle: Option<f64>,
    eager: bool,
) -> ShapeData {
    assert_valid_dimension(bottom_radius, "bottom_radius");
    assert_valid_dimension(height, "height");

    let mut builder = Shape::cone()
        .bottom_radius(bottom_radius)
        .top_radius(top_radius)
        .height(height);
    if let Some(a) = angle {
        builder = builder.z_angle(a);
    }
    let mut shape = builder.build();
    if let Some((cx, cy, cz)) = center {
        translate_shape(&mut shape, cx, cy, cz);
    }
    let mut sd = ShapeData::new(shape);
    if eager { sd.tessellate_if_needed(); }
    sd
}

/// Create a torus with the given ring radius and tube radius.
///
/// If `center` is provided, the torus is centered at that point.
/// If `z_axis` is provided, sets the torus orientation.
/// If `angle` is provided, creates a partial torus (rotation sweep).
/// If `angle_start` / `angle_end` are provided, limits the torus arc.
pub fn make_torus(
    ring_radius: f64,
    tube_radius: f64,
    center: Option<(f64, f64, f64)>,
    z_axis: Option<(f64, f64, f64)>,
    angle: Option<f64>,
    angle_start: Option<f64>,
    angle_end: Option<f64>,
    eager: bool,
) -> ShapeData {
    assert_valid_dimension(ring_radius, "ring_radius");
    assert_valid_dimension(tube_radius, "tube_radius");

    let mut builder = Shape::torus()
        .radius_1(ring_radius)
        .radius_2(tube_radius);
    if let Some((x, y, z)) = z_axis {
        builder = builder.z_axis(DVec3::new(x, y, z));
    }
    if let Some(a) = angle_start {
        builder = builder.angle_1(a);
    }
    if let Some(a) = angle_end {
        builder = builder.angle_2(a);
    }
    if let Some(a) = angle {
        builder = builder.z_angle(a);
    }
    let mut shape = builder.build();
    if let Some((cx, cy, cz)) = center {
        translate_shape(&mut shape, cx, cy, cz);
    }
    let mut sd = ShapeData::new(shape);
    if eager { sd.tessellate_if_needed(); }
    sd
}

// ── Boolean Operations ────────────────────────────────────────────────────────

/// Subtract shape `b` from shape `a`.
///
/// OCCT boolean operations may return the result as a `COMPOUND`.
/// Returns a `ShapeData` wrapping the resulting shape.
/// Panics if the result is a null/empty shape (`ShapeType::Shape`).
pub fn cut(a: &ShapeData, b: &ShapeData, eager: bool) -> ShapeData {
    let result = a.shape.subtract(&b.shape);
    let shape = result.shape;
    if shape.shape_type() == ShapeType::Shape {
        panic!("cut: shapes do not intersect or produced an empty result");
    }
    let mut sd = ShapeData::new(shape);
    if eager { sd.tessellate_if_needed(); }
    sd
}

/// Intersect shape `a` with shape `b`.
///
/// OCCT boolean operations may return the result as a `COMPOUND`.
/// Returns a `ShapeData` wrapping the resulting shape.
/// Panics if the result is a null/empty shape (`ShapeType::Shape`).
pub fn common(a: &ShapeData, b: &ShapeData, eager: bool) -> ShapeData {
    let result = a.shape.intersect(&b.shape);
    let shape = result.shape;
    if shape.shape_type() == ShapeType::Shape {
        panic!("common: shapes do not intersect or produced an empty result");
    }
    let mut sd = ShapeData::new(shape);
    if eager { sd.tessellate_if_needed(); }
    sd
}

/// Union shape `a` with shape `b`.
///
/// OCCT boolean operations may return the result as a `COMPOUND`.
/// Returns a `ShapeData` wrapping the resulting shape.
/// Panics if the result is a null/empty shape (`ShapeType::Shape`).
pub fn fuse(a: &ShapeData, b: &ShapeData, eager: bool) -> ShapeData {
    let result = a.shape.union(&b.shape);
    let shape = result.shape;
    if shape.shape_type() == ShapeType::Shape {
        panic!("fuse: shapes produced an empty result");
    }
    let mut sd = ShapeData::new(shape);
    if eager { sd.tessellate_if_needed(); }
    sd
}

// ── Shape Translation ─────────────────────────────────────────────────────────

/// Translate a shape by the given offset.
pub fn translate_shape(shape: &mut Shape, dx: f64, dy: f64, dz: f64) {
    shape.set_global_translation(DVec3::new(dx, dy, dz));
}

/// Create a translated copy of a shape.
pub fn translate(data: &ShapeData, dx: f64, dy: f64, dz: f64, eager: bool) -> ShapeData {
    let new_shape = data.shape.translated(DVec3::new(dx, dy, dz));
    let mut sd = ShapeData::new(new_shape);
    if eager { sd.tessellate_if_needed(); }
    sd
}

/// Create a rotated copy of a shape about an axis through the origin.
pub fn rotate(data: &ShapeData, axis: DVec3, angle: f64, eager: bool) -> ShapeData {
    let new_shape = data.shape.rotated(axis, angle);
    let mut sd = ShapeData::new(new_shape);
    if eager { sd.tessellate_if_needed(); }
    sd
}

/// Create a scaled copy of a shape about a point.
pub fn scale(data: &ShapeData, factor: f64, center: DVec3, eager: bool) -> ShapeData {
    let new_shape = data.shape.scaled(center, factor);
    let mut sd = ShapeData::new(new_shape);
    if eager { sd.tessellate_if_needed(); }
    sd
}

/// Create a mirrored copy of a shape about an axis.
pub fn mirror(data: &ShapeData, origin: DVec3, dir: DVec3, eager: bool) -> ShapeData {
    let new_shape = data.shape.mirrored(origin, dir);
    let mut sd = ShapeData::new(new_shape);
    if eager { sd.tessellate_if_needed(); }
    sd
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
        let sd = make_box(10.0, 20.0, 30.0, None, false);
        assert_eq!(sd.type_string(), "SOLID");
        assert!(sd.visible);
        assert!(sd.color.is_none());
    }

    #[test]
    fn test_make_box_centered() {
        let sd = make_box(10.0, 20.0, 30.0, Some((5.0, 10.0, 15.0)), false);
        assert_eq!(sd.type_string(), "SOLID");
        assert!(sd.visible);
    }

    #[test]
    fn test_make_sphere_default() {
        // 10.2: Test sphere creation via raw Rust API
        let sd = make_sphere(10.0, None, None, false);
        assert_eq!(sd.type_string(), "SOLID");
        assert!(sd.visible);
    }

    #[test]
    fn test_make_sphere_centered() {
        let sd = make_sphere(5.0, Some((1.0, 2.0, 3.0)), None, false);
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    #[should_panic(expected = "radius must be positive")]
    fn test_make_sphere_invalid_radius() {
        make_sphere(-1.0, None, None, false);
    }

    #[test]
    #[should_panic(expected = "width must be positive")]
    fn test_make_box_invalid_width() {
        // 4.6: Validate inputs — reject zero/negative dimensions
        make_box(0.0, 10.0, 10.0, None, false);
    }

    #[test]
    fn test_cut() {
        // 10.3: Test cut via raw Rust API
        let box_a = make_box(20.0, 20.0, 20.0, None, false);
        let sphere_b = make_sphere(10.0, Some((10.0, 10.0, 10.0)), None, false);
        let result = cut(&box_a, &sphere_b, false);
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
        let sphere_a = make_sphere(10.0, None, None, false);
        let box_b = make_box(10.0, 10.0, 10.0, None, false);
        let result = common(&sphere_a, &box_b, false);
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
        let box_a = make_box(10.0, 10.0, 10.0, None, false);
        let box_b = make_box(10.0, 10.0, 10.0, Some((100.0, 0.0, 0.0)), false);
        let result = cut(&box_a, &box_b, false);
        assert_ne!(
            result.type_string(),
            "SHAPE",
            "cut of non-overlapping shapes should not produce a null shape"
        );
    }

    #[test]
    fn test_write_step_roundtrip() {
        // 10.6: Test STEP export round-trip
        let sd = make_box(10.0, 20.0, 30.0, None, false);
        let path = "/tmp/test_rojcad_box.step";
        assert!(write_step(&sd, path).is_ok());
        assert!(std::path::Path::new(path).exists());
        // Clean up
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_write_stl() {
        // 10.7: Test STL export
        let sd = make_sphere(10.0, None, None, false);
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
        let mut sd = make_box(10.0, 10.0, 10.0, None, false);
        assert!(sd.visible);
        sd.visible = false;
        assert!(!sd.visible);
        sd.visible = true;
        assert!(sd.visible);
    }

    #[test]
    fn test_shape_type() {
        assert_eq!(make_box(10.0, 10.0, 10.0, None, false).type_string(), "SOLID");
        assert_eq!(make_sphere(10.0, None, None, false).type_string(), "SOLID");
    }

    // ── New Primitive Tests ─────────────────────────────────────────────────

    #[test]
    fn test_make_cube_default() {
        let sd = make_cube(5.0, None, false);
        assert_eq!(sd.type_string(), "SOLID");
        assert!(sd.visible);
    }

    #[test]
    fn test_make_cube_centered() {
        let sd = make_cube(5.0, Some((1.0, 2.0, 3.0)), false);
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_box_from_corners() {
        let sd = make_box_from_corners((0.0, 0.0, 0.0), (10.0, 20.0, 30.0), false);
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_sphere_with_angle() {
        let sd = make_sphere(10.0, None, Some(std::f64::consts::PI), false);
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_cylinder_default() {
        let sd = make_cylinder(5.0, 10.0, None, false);
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_cylinder_centered() {
        let sd = make_cylinder(5.0, 10.0, Some((0.0, 0.0, 5.0)), false);
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_cylinder_from_points() {
        let sd = make_cylinder_from_points((0.0, 0.0, 0.0), (0.0, 0.0, 10.0), 5.0, false);
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_cylinder_point_dir() {
        let sd = make_cylinder_point_dir((0.0, 0.0, 0.0), 5.0, (0.0, 0.0, 1.0), 10.0, false);
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_cone_full() {
        let sd = make_cone(5.0, 0.0, 10.0, None, None, false);
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_cone_truncated() {
        let sd = make_cone(5.0, 3.0, 10.0, None, None, false);
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_cone_with_angle() {
        let sd = make_cone(5.0, 0.0, 10.0, None, Some(std::f64::consts::PI), false);
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_torus_default() {
        let sd = make_torus(20.0, 10.0, None, None, None, None, None, false);
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_torus_centered() {
        let sd = make_torus(20.0, 10.0, Some((0.0, 0.0, 5.0)), None, None, None, None, false);
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_torus_partial() {
        let sd = make_torus(20.0, 10.0, None, None, Some(std::f64::consts::PI), None, None, false);
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    #[should_panic(expected = "size must be positive")]
    fn test_make_cube_invalid_size() {
        make_cube(0.0, None, false);
    }

    #[test]
    #[should_panic(expected = "radius must be positive")]
    fn test_make_cylinder_invalid_radius() {
        make_cylinder(0.0, 10.0, None, false);
    }

    #[test]
    #[should_panic(expected = "height must be positive")]
    fn test_make_cylinder_invalid_height() {
        make_cylinder(5.0, 0.0, None, false);
    }

    #[test]
    #[should_panic(expected = "bottom_radius must be positive")]
    fn test_make_cone_invalid_bottom_radius() {
        make_cone(0.0, 0.0, 10.0, None, None, false);
    }

    #[test]
    #[should_panic(expected = "height must be positive")]
    fn test_make_cone_invalid_height() {
        make_cone(5.0, 0.0, 0.0, None, None, false);
    }

    #[test]
    #[should_panic(expected = "ring_radius must be positive")]
    fn test_make_torus_invalid_ring_radius() {
        make_torus(0.0, 10.0, None, None, None, None, None, false);
    }

    #[test]
    #[should_panic(expected = "tube_radius must be positive")]
    fn test_make_torus_invalid_tube_radius() {
        make_torus(20.0, 0.0, None, None, None, None, None, false);
    }

    // ── Fuse Tests ───────────────────────────────────────────────────────

    #[test]
    fn test_fuse() {
        let a = make_box(10.0, 10.0, 10.0, None, false);
        let b = make_box(10.0, 10.0, 10.0, Some((5.0, 5.0, 5.0)), false);
        let result = fuse(&a, &b, false);
        assert!(
            result.type_string() == "SOLID" || result.type_string() == "COMPOUND",
            "expected SOLID or COMPOUND, got {}",
            result.type_string()
        );
        assert!(a.visible);
        assert_eq!(a.type_string(), "SOLID");
    }

    #[test]
    fn test_fuse_non_overlapping() {
        let a = make_box(10.0, 10.0, 10.0, None, false);
        let b = make_box(10.0, 10.0, 10.0, Some((100.0, 0.0, 0.0)), false);
        let result = fuse(&a, &b, false);
        assert_ne!(result.type_string(), "SHAPE", "fuse of non-overlapping shapes should not produce a null shape");
    }

    // ── Translate Tests ──────────────────────────────────────────────────

    #[test]
    fn test_translate() {
        let original = make_box(10.0, 10.0, 10.0, None, false);
        let moved = translate(&original, 5.0, 0.0, 0.0, false);
        assert!(original.visible);
        assert_eq!(original.type_string(), "SOLID");
        assert_eq!(moved.type_string(), "SOLID");
    }

    // ── Rotate Tests ─────────────────────────────────────────────────────

    #[test]
    fn test_rotate() {
        let original = make_box(10.0, 10.0, 10.0, None, false);
        let rotated = rotate(&original, DVec3::Z, std::f64::consts::FRAC_PI_2, false);
        assert!(original.visible);
        assert_eq!(rotated.type_string(), "SOLID");
    }

    // ── Scale Tests ──────────────────────────────────────────────────────

    #[test]
    fn test_scale() {
        let original = make_box(10.0, 10.0, 10.0, None, false);
        let scaled = scale(&original, 2.0, DVec3::ZERO, false);
        assert!(original.visible);
        assert_eq!(scaled.type_string(), "SOLID");
    }

    #[test]
    fn test_scale_with_center() {
        let original = make_box(10.0, 10.0, 10.0, None, false);
        let scaled = scale(&original, 2.0, DVec3::new(5.0, 5.0, 5.0), false);
        assert!(original.visible);
        assert_eq!(scaled.type_string(), "SOLID");
    }

    // ── Mirror Tests ─────────────────────────────────────────────────────

    #[test]
    fn test_mirror() {
        let original = make_box(10.0, 10.0, 10.0, None, false);
        let mirrored = mirror(&original, DVec3::ZERO, DVec3::X, false);
        assert!(original.visible);
        assert_eq!(mirrored.type_string(), "SOLID");
    }

    // ── Immutability Tests ───────────────────────────────────────────────

    #[test]
    fn test_immutability_translate() {
        let original = make_box(10.0, 10.0, 10.0, None, false);
        let original_id = original.shape_id;
        let _moved = translate(&original, 5.0, 0.0, 0.0, false);
        assert!(original.visible);
        assert_eq!(original.shape_id, original_id);
        assert_eq!(original.type_string(), "SOLID");
    }

    #[test]
    fn test_immutability_rotate() {
        let original = make_box(10.0, 10.0, 10.0, None, false);
        let original_id = original.shape_id;
        let _rotated = rotate(&original, DVec3::Z, std::f64::consts::FRAC_PI_2, false);
        assert!(original.visible);
        assert_eq!(original.shape_id, original_id);
    }

    #[test]
    fn test_immutability_scale() {
        let original = make_box(10.0, 10.0, 10.0, None, false);
        let original_id = original.shape_id;
        let _scaled = scale(&original, 2.0, DVec3::ZERO, false);
        assert!(original.visible);
        assert_eq!(original.shape_id, original_id);
    }

    #[test]
    fn test_immutability_mirror() {
        let original = make_box(10.0, 10.0, 10.0, None, false);
        let original_id = original.shape_id;
        let _mirrored = mirror(&original, DVec3::ZERO, DVec3::X, false);
        assert!(original.visible);
        assert_eq!(original.shape_id, original_id);
    }
}
