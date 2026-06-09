//! CAD operations — Rust wrappers around opencascade-rs.
//!
//! Implements: box, sphere, cylinder, cone, torus, cut, common, shape type, export.

use std::f64::consts::TAU;

use glam::{DQuat, DVec3};
use opencascade::angle::Angle;
use opencascade::primitives::{Compound, Face, JoinType, Shape, ShapeType, Solid, Wire};
use opencascade::workplane::Workplane;

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
) -> Result<ShapeData, String> {
    validate_dimension(width, "width")?;
    validate_dimension(depth, "depth")?;
    validate_dimension(height, "height")?;

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
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

/// Create a cube with the given side length.
///
/// One corner at (0,0,0) by default.
/// If `center` is provided, the cube is centered at that point.
pub fn make_cube(
    size: f64,
    center: Option<(f64, f64, f64)>,
    eager: bool,
) -> Result<ShapeData, String> {
    validate_dimension(size, "size")?;
    let mut shape = Shape::cube(size);
    if let Some((cx, cy, cz)) = center {
        translate_shape(&mut shape, cx, cy, cz);
    }
    let mut sd = ShapeData::new(shape);
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

/// Create a box from two opposite corners.
pub fn make_box_from_corners(
    corner1: (f64, f64, f64),
    corner2: (f64, f64, f64),
    eager: bool,
) -> Result<ShapeData, String> {
    let c1 = DVec3::new(corner1.0, corner1.1, corner1.2);
    let c2 = DVec3::new(corner2.0, corner2.1, corner2.2);
    let shape = Shape::box_from_corners(c1, c2);
    let mut sd = ShapeData::new(shape);
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
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
) -> Result<ShapeData, String> {
    validate_dimension(radius, "radius")?;

    let mut builder = Shape::sphere(radius);
    if let Some(a) = angle {
        builder = builder.z_angle(a);
    }
    let mut shape = builder.build();
    if let Some((cx, cy, cz)) = center {
        translate_shape(&mut shape, cx, cy, cz);
    }
    let mut sd = ShapeData::new(shape);
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
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
) -> Result<ShapeData, String> {
    validate_dimension(radius, "radius")?;
    validate_dimension(height, "height")?;

    let shape = if let Some((cx, cy, cz)) = center {
        Shape::cylinder_centered(DVec3::new(cx, cy, cz), radius, DVec3::Z, height)
    } else {
        Shape::cylinder_radius_height(radius, height)
    };
    let mut sd = ShapeData::new(shape);
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

/// Create a cylinder between two points with the given radius.
pub fn make_cylinder_from_points(
    p1: (f64, f64, f64),
    p2: (f64, f64, f64),
    radius: f64,
    eager: bool,
) -> Result<ShapeData, String> {
    validate_dimension(radius, "radius")?;
    let shape = Shape::cylinder_from_points(
        DVec3::new(p1.0, p1.1, p1.2),
        DVec3::new(p2.0, p2.1, p2.2),
        radius,
    );
    let mut sd = ShapeData::new(shape);
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

/// Create a cylinder at a given point, extending in the given direction.
pub fn make_cylinder_point_dir(
    point: (f64, f64, f64),
    radius: f64,
    dir: (f64, f64, f64),
    height: f64,
    eager: bool,
) -> Result<ShapeData, String> {
    validate_dimension(radius, "radius")?;
    validate_dimension(height, "height")?;
    let shape = Shape::cylinder(
        DVec3::new(point.0, point.1, point.2),
        radius,
        DVec3::new(dir.0, dir.1, dir.2),
        height,
    );
    let mut sd = ShapeData::new(shape);
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
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
) -> Result<ShapeData, String> {
    validate_dimension(bottom_radius, "bottom_radius")?;
    validate_dimension(height, "height")?;

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
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
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
) -> Result<ShapeData, String> {
    validate_dimension(ring_radius, "ring_radius")?;
    validate_dimension(tube_radius, "tube_radius")?;

    let mut builder = Shape::torus().radius_1(ring_radius).radius_2(tube_radius);
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
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

// ── Boolean Operations ────────────────────────────────────────────────────────

/// Subtract shape `b` from shape `a`.
///
/// OCCT boolean operations may return the result as a `COMPOUND`.
/// Returns a `ShapeData` wrapping the resulting shape.
/// Panics if the result is a null/empty shape (`ShapeType::Shape`).
pub fn cut(a: &ShapeData, b: &ShapeData, eager: bool) -> Result<ShapeData, String> {
    let result = a.shape.subtract(&b.shape);
    let shape = result.shape;
    if shape.shape_type() == ShapeType::Shape {
        return Err("cut: shapes do not intersect or produced an empty result".to_string());
    }
    let mut sd = ShapeData::new(shape);
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

/// Intersect shape `a` with shape `b`.
///
/// OCCT boolean operations may return the result as a `COMPOUND`.
/// Returns a `ShapeData` wrapping the resulting shape.
/// Panics if the result is a null/empty shape (`ShapeType::Shape`).
pub fn common(a: &ShapeData, b: &ShapeData, eager: bool) -> Result<ShapeData, String> {
    let result = a.shape.intersect(&b.shape);
    let shape = result.shape;
    if shape.shape_type() == ShapeType::Shape {
        return Err("common: shapes do not intersect or produced an empty result".to_string());
    }
    let mut sd = ShapeData::new(shape);
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

/// Union shape `a` with shape `b`.
///
/// OCCT boolean operations may return the result as a `COMPOUND`.
/// Returns a `ShapeData` wrapping the resulting shape.
/// Panics if the result is a null/empty shape (`ShapeType::Shape`).
pub fn fuse(a: &ShapeData, b: &ShapeData, eager: bool) -> Result<ShapeData, String> {
    let result = a.shape.union(&b.shape);
    let shape = result.shape;
    if shape.shape_type() == ShapeType::Shape {
        return Err("fuse: shapes produced an empty result".to_string());
    }
    let mut sd = ShapeData::new(shape);
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

/// Group 2+ shapes into an OCCT Compound (lightweight topological container).
pub fn make_compound(shapes: &[&ShapeData], eager: bool) -> Result<ShapeData, String> {
    if shapes.len() < 2 {
        return Err("make_compound: at least two shapes are required".to_string());
    }
    let refs: Vec<&Shape> = shapes.iter().map(|s| &s.shape).collect();
    let compound = Compound::from_shapes(&refs);
    let mut sd = ShapeData::new(Shape::from(compound));
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

/// Set a shape's render color (mutates in place).
pub fn set_color(data: &mut ShapeData, r: f64, g: f64, b: f64) {
    data.color = Some([r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)]);
    if data.registered {
        global_shape_registry().set_color(data.shape_id, data.color);
    }
}

/// Get a shape's render color, or `None` if unset.
pub fn get_color(data: &ShapeData) -> Option<[f64; 3]> {
    data.color
}

// ── Shape Translation ─────────────────────────────────────────────────────────

/// Translate a shape by the given offset.
pub fn translate_shape(shape: &mut Shape, dx: f64, dy: f64, dz: f64) {
    shape.set_global_translation(DVec3::new(dx, dy, dz));
}

/// Create a translated copy of a shape.
pub fn translate(
    data: &ShapeData,
    dx: f64,
    dy: f64,
    dz: f64,
    eager: bool,
) -> Result<ShapeData, String> {
    let new_shape = data.shape.translated(DVec3::new(dx, dy, dz));
    let mut sd = ShapeData::new(new_shape);
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

/// Create a rotated copy of a shape about an axis through the origin.
pub fn rotate(data: &ShapeData, axis: DVec3, angle: f64, eager: bool) -> Result<ShapeData, String> {
    let new_shape = data.shape.rotated(axis, angle);
    let mut sd = ShapeData::new(new_shape);
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

/// Create a scaled copy of a shape about a point.
pub fn scale(
    data: &ShapeData,
    factor: f64,
    center: DVec3,
    eager: bool,
) -> Result<ShapeData, String> {
    let new_shape = data.shape.scaled(center, factor);
    let mut sd = ShapeData::new(new_shape);
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

/// Create a mirrored copy of a shape about an axis.
pub fn mirror(
    data: &ShapeData,
    origin: DVec3,
    dir: DVec3,
    eager: bool,
) -> Result<ShapeData, String> {
    let new_shape = data.shape.mirrored(origin, dir);
    let mut sd = ShapeData::new(new_shape);
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

// ── Import ────────────────────────────────────────────────────────────────────

/// Read a shape from a STEP file.
pub fn read_step(path: &str, eager: bool) -> Result<ShapeData, String> {
    let shape = Shape::read_step(path).map_err(|e| format!("STEP import failed: {}", e))?;
    let mut sd = ShapeData::new(shape);
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

// ── Export ────────────────────────────────────────────────────────────────────

/// Write one or more shapes to a STEP file using a single writer.
pub fn write_all_step(shapes: &[&ShapeData], path: &str) -> Result<(), String> {
    if shapes.is_empty() {
        return Err("at least one shape is required to write a STEP file".to_string());
    }
    let refs: Vec<&Shape> = shapes.iter().map(|s| &s.shape).collect();
    Shape::write_all_step(&refs, path)
        .map_err(|e| format!("STEP export failed: {}", e))
}

/// Write a shape to an STL file.
pub fn write_stl(data: &ShapeData, path: &str) -> Result<(), String> {
    data.shape
        .write_stl(path)
        .map_err(|e| format!("STL export failed: {}", e))
}

// ── 2D Primitives ────────────────────────────────────────────────────────────

/// Convert a workplane keyword to a Workplane, with optional position offset.
pub fn workplane_from_keyword(plane: &str, at: Option<(f64, f64, f64)>) -> Workplane {
    let wp = match plane {
        "xy" | "" => Workplane::xy(),
        "xz" => Workplane::xz(),
        "yz" => Workplane::yz(),
        "zx" => Workplane::zx(),
        "zy" => Workplane::zy(),
        "yx" => Workplane::yx(),
        _ => Workplane::xy(),
    };
    match at {
        Some((x, y, z)) => wp.translated(DVec3::new(x, y, z)),
        None => wp,
    }
}

/// Create a rectangle.
pub fn make_rect(
    w: f64,
    d: f64,
    is_wire: bool,
    plane: &str,
    at: Option<(f64, f64, f64)>,
    eager: bool,
) -> Result<ShapeData, String> {
    validate_dimension(w, "width")?;
    validate_dimension(d, "depth")?;
    let wp = workplane_from_keyword(plane, at);
    let wire = wp.rect(w, d);
    let shape = if is_wire {
        Shape::from(wire)
    } else {
        Shape::from(Face::from_wire(&wire))
    };
    let mut sd = ShapeData::new(shape);
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

/// Create a circle.
pub fn make_circle(
    r: f64,
    is_wire: bool,
    plane: &str,
    at: Option<(f64, f64, f64)>,
    eager: bool,
) -> Result<ShapeData, String> {
    validate_dimension(r, "radius")?;
    let wp = workplane_from_keyword(plane, at);
    let wire = wp.circle(0.0, 0.0, r);
    let shape = if is_wire {
        Shape::from(wire)
    } else {
        Shape::from(Face::from_wire(&wire))
    };
    let mut sd = ShapeData::new(shape);
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

/// Create a polygon from a list of 2D points (as flattened f64 array).
pub fn make_polygon(
    pts: &[f64],
    is_wire: bool,
    plane: &str,
    at: Option<(f64, f64, f64)>,
    eager: bool,
) -> Result<ShapeData, String> {
    let wp = workplane_from_keyword(plane, at);
    let world_pts: Vec<DVec3> = pts
        .chunks(2)
        .map(|c| wp.to_world_pos(DVec3::new(c[0], c[1], 0.0)))
        .collect();
    let wire = Wire::from_ordered_points(world_pts).expect("polygon needs at least 2 points");
    let shape = if is_wire {
        Shape::from(wire)
    } else {
        Shape::from(Face::from_wire(&wire))
    };
    let mut sd = ShapeData::new(shape);
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

// ── Extrusion & Revolution ──────────────────────────────────────────────────

/// Extrude a Face to a Solid.
///
/// If `dir` is zero, extrudes along the face's normal.
pub fn extrude_shape(
    data: &ShapeData,
    height: f64,
    dir: DVec3,
    both: bool,
    eager: bool,
) -> Result<ShapeData, String> {
    let face = data.shape.expect_face();
    let extrusion_dir = if dir.length_squared() < 1e-10 {
        face.normal_at_center()
    } else {
        dir.normalize()
    };
    let full_vec = extrusion_dir * height;
    let solid = if both {
        let half_vec = extrusion_dir * (height / 2.0);
        let shifted = Shape::from(&face).translated(-half_vec);
        shifted.expect_face().extrude(full_vec)
    } else {
        face.extrude(full_vec)
    };
    let mut sd = ShapeData::new(Shape::from(solid));
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

/// Revolve a Face to a Solid by lofting through rotated wire copies.
///
/// Falls back to `face.revolve()` (OCCT MakeRevol) when the wire is
/// rotationally symmetric about the revolve axis (degenerate loft case).
pub fn revolve_shape(
    data: &ShapeData,
    angle_rad: f64,
    origin: DVec3,
    axis: DVec3,
    eager: bool,
) -> Result<ShapeData, String> {
    let face = data.shape.expect_face();
    let wire = face.outer_wire();

    let center = face.center_of_mass();
    let offset = center - origin;
    let dist_from_axis = offset.cross(axis).length();
    let is_symmetric = dist_from_axis < 1e-6;
    let is_full = (angle_rad - TAU).abs() < 1e-9;

    let face_normal = face.normal_at_center();
    let rev_axis_is_normal = face_normal.cross(axis.normalize()).length() < 1e-6;

    let solid = if is_symmetric && rev_axis_is_normal {
        if is_full {
            face.revolve(origin, axis, Some(Angle::Radians(6.265)))
        } else {
            face.revolve(origin, axis, Some(Angle::Radians(angle_rad)))
        }
    } else if !is_symmetric {
        let n = (angle_rad / TAU * 48.0).ceil().max(3.0) as u32;
        let step = angle_rad / n as f64;
        let count = if is_full { n } else { n + 1 };

        let wires: Vec<Wire> = (0..count)
            .map(|i| {
                let a = step * i as f64;
                let q = DQuat::from_axis_angle(axis, a);
                let rot_origin = q * origin;
                let tx = origin - rot_origin;
                wire.transform(tx, axis, Angle::Radians(a))
            })
            .collect();

        Solid::loft(wires.iter())
    } else {
        let n = (angle_rad / TAU * 48.0).ceil().max(3.0) as u32;
        let step = angle_rad / n as f64;
        let count = if is_full { n } else { n + 1 };

        let wires: Vec<Wire> = (0..count)
            .map(|i| {
                let a = step * i as f64;
                let q = DQuat::from_axis_angle(axis, a);
                let rot_origin = q * origin;
                let tx = origin - rot_origin;
                wire.transform(tx, axis, Angle::Radians(a))
            })
            .collect();

        Solid::loft(wires.iter())
    };

    let mut sd = ShapeData::new(Shape::from(solid));
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

/// One-shot polygon extrusion.
pub fn extrude_polygon_raw(
    pts: &[f64],
    height: f64,
    plane: &str,
    at: Option<(f64, f64, f64)>,
    eager: bool,
) -> Result<ShapeData, String> {
    let wp = workplane_from_keyword(plane, at);
    let world_pts: Vec<DVec3> = pts
        .chunks(2)
        .map(|c| wp.to_world_pos(DVec3::new(c[0], c[1], 0.0)))
        .collect();
    let wire =
        Wire::from_ordered_points(world_pts).expect("extrude-polygon needs at least 2 points");
    let face = Face::from_wire(&wire);
    let solid = face.extrude(DVec3::Z * height);
    let mut sd = ShapeData::new(Shape::from(solid));
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

// ── Wire Operations ─────────────────────────────────────────────────────────

/// Convert a Wire to a Face.
pub fn wire_to_face(data: &ShapeData, eager: bool) -> Result<ShapeData, String> {
    let wire = data.shape.expect_wire();
    let face = Face::from_wire(&wire);
    let mut sd = ShapeData::new(Shape::from(face));
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

/// Fillet a Wire.
pub fn wire_fillet(data: &ShapeData, radius: f64, eager: bool) -> Result<ShapeData, String> {
    validate_dimension(radius, "radius")?;
    let wire = data.shape.expect_wire();
    let result = wire.fillet(radius);
    let mut sd = ShapeData::new(Shape::from(result));
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

/// Chamfer a Wire.
pub fn wire_chamfer(data: &ShapeData, distance: f64, eager: bool) -> Result<ShapeData, String> {
    validate_dimension(distance, "distance")?;
    let wire = data.shape.expect_wire();
    let result = wire.chamfer(distance);
    let mut sd = ShapeData::new(Shape::from(result));
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

/// Offset a Wire.
pub fn wire_offset(data: &ShapeData, distance: f64, eager: bool) -> Result<ShapeData, String> {
    validate_dimension(distance, "distance")?;
    let wire = data.shape.expect_wire();
    let result = wire.offset(distance, JoinType::Arc);
    let mut sd = ShapeData::new(Shape::from(result));
    if eager {
        sd.tessellate_if_needed();
    }
    Ok(sd)
}

// ── Helper Queries ──────────────────────────────────────────────────────────

pub fn is_wire(data: &ShapeData) -> bool {
    data.shape.shape_type() == ShapeType::Wire
}
pub fn is_face(data: &ShapeData) -> bool {
    data.shape.shape_type() == ShapeType::Face
}
pub fn is_solid(data: &ShapeData) -> bool {
    data.shape.shape_type() == ShapeType::Solid
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn validate_dimension(value: f64, name: &str) -> Result<(), String> {
    if value <= 0.0 {
        Err(format!("{} must be positive, got {}", name, value))
    } else {
        Ok(())
    }
}

/// Compute the union bounding box center and bounding sphere radius from shape mesh data.
/// Shapes with `mesh: None` are silently skipped. Returns `None` if no mesh data found.
pub fn compute_union_bounds(shapes: &[&ShapeData]) -> Option<(DVec3, f64)> {
    let mut min = DVec3::splat(f64::MAX);
    let mut max = DVec3::splat(f64::MIN);
    let mut has_vertices = false;

    for shape in shapes {
        if let Some(ref mesh) = shape.mesh {
            for v in &mesh.vertices {
                let p = DVec3::new(v[0] as f64, v[1] as f64, v[2] as f64);
                min = min.min(p);
                max = max.max(p);
                has_vertices = true;
            }
        }
    }

    if !has_vertices {
        return None;
    }

    let center = (min + max) * 0.5;
    let radius = (max - min).length() * 0.5;

    Some((center, radius * 1.3))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helpers to unwrap Result in tests
    fn unwrap_box(w: f64, d: f64, h: f64, c: Option<(f64, f64, f64)>, e: bool) -> ShapeData {
        make_box(w, d, h, c, e).unwrap()
    }

    fn unwrap_sphere(r: f64, c: Option<(f64, f64, f64)>, a: Option<f64>, e: bool) -> ShapeData {
        make_sphere(r, c, a, e).unwrap()
    }

    #[test]
    fn test_make_box_default() {
        // 10.1: Test box creation via raw Rust API
        let sd = unwrap_box(10.0, 20.0, 30.0, None, false);
        assert_eq!(sd.type_string(), "SOLID");
        assert!(sd.visible);
        assert!(sd.color.is_none());
    }

    #[test]
    fn test_make_box_centered() {
        let sd = unwrap_box(10.0, 20.0, 30.0, Some((5.0, 10.0, 15.0)), false);
        assert_eq!(sd.type_string(), "SOLID");
        assert!(sd.visible);
    }

    #[test]
    fn test_make_sphere_default() {
        // 10.2: Test sphere creation via raw Rust API
        let sd = unwrap_sphere(10.0, None, None, false);
        assert_eq!(sd.type_string(), "SOLID");
        assert!(sd.visible);
    }

    #[test]
    fn test_make_sphere_centered() {
        let sd = unwrap_sphere(5.0, Some((1.0, 2.0, 3.0)), None, false);
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_sphere_invalid_radius() {
        match make_sphere(-1.0, None, None, false) {
            Err(msg) => assert!(msg.contains("radius must be positive"), "{}", msg),
            Ok(_) => panic!("expected Err"),
        }
    }

    #[test]
    fn test_make_box_invalid_width() {
        // 4.6: Validate inputs — reject zero/negative dimensions
        match make_box(0.0, 10.0, 10.0, None, false) {
            Err(msg) => assert!(msg.contains("width must be positive"), "{}", msg),
            Ok(_) => panic!("expected Err"),
        }
    }

    #[test]
    fn test_cut() {
        // 10.3: Test cut via raw Rust API
        let box_a = unwrap_box(20.0, 20.0, 20.0, None, false);
        let sphere_b = unwrap_sphere(10.0, Some((10.0, 10.0, 10.0)), None, false);
        let result = cut(&box_a, &sphere_b, false).unwrap();
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
        let sphere_a = unwrap_sphere(10.0, None, None, false);
        let box_b = unwrap_box(10.0, 10.0, 10.0, None, false);
        let result = common(&sphere_a, &box_b, false).unwrap();
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
        let box_a = unwrap_box(10.0, 10.0, 10.0, None, false);
        let box_b = unwrap_box(10.0, 10.0, 10.0, Some((100.0, 0.0, 0.0)), false);
        let result = cut(&box_a, &box_b, false);
        // Non-overlapping shapes OCCT may return the original shape or a null shape.
        // Accept either an error or a valid (non-shape) result.
        match result {
            Err(msg) => assert!(msg.contains("do not intersect"), "{}", msg),
            Ok(sd) => assert_ne!(sd.type_string(), "SHAPE"),
        }
    }

    #[test]
    fn test_write_all_step_single() {
        let sd = unwrap_box(10.0, 20.0, 30.0, None, false);
        let path = "/tmp/test_rojcad_box.step";
        assert!(write_all_step(&[&sd], path).is_ok());
        assert!(std::path::Path::new(path).exists());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_write_all_step_multiple() {
        let s1 = unwrap_box(10.0, 20.0, 30.0, None, false);
        let s2 = unwrap_sphere(10.0, None, None, false);
        let s3 = make_cylinder(5.0, 15.0, None, false).unwrap();
        let path = "/tmp/test_rojcad_multi.step";
        assert!(write_all_step(&[&s1, &s2, &s3], path).is_ok());
        assert!(std::path::Path::new(path).exists());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_write_all_step_empty() {
        let path = "/tmp/test_rojcad_empty.step";
        let result = write_all_step(&[], path);
        assert!(result.is_err());
        assert!(!std::path::Path::new(path).exists());
    }

    #[test]
    fn test_read_step_roundtrip() {
        let sd = unwrap_box(10.0, 20.0, 30.0, None, false);
        let path = "/tmp/test_rojcad_roundtrip.step";
        assert!(write_all_step(&[&sd], path).is_ok());
        let imported = read_step(path, false).expect("should read back STEP file");
        assert!(imported.shape_id > 0);
        assert!(imported.visible);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_read_step_file_not_found() {
        let result = read_step("/tmp/nonexistent_rojcad_file.step", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_stl() {
        // 10.7: Test STL export
        let sd = unwrap_sphere(10.0, None, None, false);
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
        let mut sd = unwrap_box(10.0, 10.0, 10.0, None, false);
        assert!(sd.visible);
        sd.visible = false;
        assert!(!sd.visible);
        sd.visible = true;
        assert!(sd.visible);
    }

    #[test]
    fn test_shape_type() {
        assert_eq!(
            unwrap_box(10.0, 10.0, 10.0, None, false).type_string(),
            "SOLID"
        );
        assert_eq!(
            unwrap_sphere(10.0, None, None, false).type_string(),
            "SOLID"
        );
    }

    // ── New Primitive Tests ─────────────────────────────────────────────────

    #[test]
    fn test_make_cube_default() {
        let sd = make_cube(5.0, None, false).unwrap();
        assert_eq!(sd.type_string(), "SOLID");
        assert!(sd.visible);
    }

    #[test]
    fn test_make_cube_centered() {
        let sd = make_cube(5.0, Some((1.0, 2.0, 3.0)), false).unwrap();
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_box_from_corners() {
        let sd = make_box_from_corners((0.0, 0.0, 0.0), (10.0, 20.0, 30.0), false).unwrap();
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_sphere_with_angle() {
        let sd = unwrap_sphere(10.0, None, Some(std::f64::consts::PI), false);
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_cylinder_default() {
        let sd = make_cylinder(5.0, 10.0, None, false).unwrap();
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_cylinder_centered() {
        let sd = make_cylinder(5.0, 10.0, Some((0.0, 0.0, 5.0)), false).unwrap();
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_cylinder_from_points() {
        let sd = make_cylinder_from_points((0.0, 0.0, 0.0), (0.0, 0.0, 10.0), 5.0, false).unwrap();
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_cylinder_point_dir() {
        let sd =
            make_cylinder_point_dir((0.0, 0.0, 0.0), 5.0, (0.0, 0.0, 1.0), 10.0, false).unwrap();
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_cone_full() {
        let sd = make_cone(5.0, 0.0, 10.0, None, None, false).unwrap();
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_cone_truncated() {
        let sd = make_cone(5.0, 3.0, 10.0, None, None, false).unwrap();
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_cone_with_angle() {
        let sd = make_cone(5.0, 0.0, 10.0, None, Some(std::f64::consts::PI), false).unwrap();
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_torus_default() {
        let sd = make_torus(20.0, 10.0, None, None, None, None, None, false).unwrap();
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_torus_centered() {
        let sd = make_torus(
            20.0,
            10.0,
            Some((0.0, 0.0, 5.0)),
            None,
            None,
            None,
            None,
            false,
        )
        .unwrap();
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_make_torus_partial() {
        let sd = make_torus(
            20.0,
            10.0,
            None,
            None,
            Some(std::f64::consts::PI),
            None,
            None,
            false,
        )
        .unwrap();
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_invalid_dimensions_return_error() {
        assert!(make_cube(0.0, None, false).is_err());
        assert!(make_cylinder(0.0, 10.0, None, false).is_err());
        assert!(make_cylinder(5.0, 0.0, None, false).is_err());
        assert!(make_cone(0.0, 0.0, 10.0, None, None, false).is_err());
        assert!(make_cone(5.0, 0.0, 0.0, None, None, false).is_err());
        assert!(make_torus(0.0, 10.0, None, None, None, None, None, false).is_err());
        assert!(make_torus(20.0, 0.0, None, None, None, None, None, false).is_err());
    }

    // ── Fuse Tests ───────────────────────────────────────────────────────

    #[test]
    fn test_fuse() {
        let a = unwrap_box(10.0, 10.0, 10.0, None, false);
        let b = unwrap_box(10.0, 10.0, 10.0, Some((5.0, 5.0, 5.0)), false);
        let result = fuse(&a, &b, false).unwrap();
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
        let a = unwrap_box(10.0, 10.0, 10.0, None, false);
        let b = unwrap_box(10.0, 10.0, 10.0, Some((100.0, 0.0, 0.0)), false);
        let result = fuse(&a, &b, false);
        // Non-overlapping fuse returns an error or produces a compound
        if let Ok(sd) = result {
            assert_ne!(sd.type_string(), "SHAPE");
        }
    }

    // ── Translate Tests ──────────────────────────────────────────────────

    #[test]
    fn test_translate() {
        let original = unwrap_box(10.0, 10.0, 10.0, None, false);
        let moved = translate(&original, 5.0, 0.0, 0.0, false).unwrap();
        assert!(original.visible);
        assert_eq!(original.type_string(), "SOLID");
        assert_eq!(moved.type_string(), "SOLID");
    }

    // ── Rotate Tests ─────────────────────────────────────────────────────

    #[test]
    fn test_rotate() {
        let original = unwrap_box(10.0, 10.0, 10.0, None, false);
        let rotated = rotate(&original, DVec3::Z, std::f64::consts::FRAC_PI_2, false).unwrap();
        assert!(original.visible);
        assert_eq!(rotated.type_string(), "SOLID");
    }

    // ── Scale Tests ──────────────────────────────────────────────────────

    #[test]
    fn test_scale() {
        let original = unwrap_box(10.0, 10.0, 10.0, None, false);
        let scaled = scale(&original, 2.0, DVec3::ZERO, false).unwrap();
        assert!(original.visible);
        assert_eq!(scaled.type_string(), "SOLID");
    }

    #[test]
    fn test_scale_with_center() {
        let original = unwrap_box(10.0, 10.0, 10.0, None, false);
        let scaled = scale(&original, 2.0, DVec3::new(5.0, 5.0, 5.0), false).unwrap();
        assert!(original.visible);
        assert_eq!(scaled.type_string(), "SOLID");
    }

    // ── Mirror Tests ─────────────────────────────────────────────────────

    #[test]
    fn test_mirror() {
        let original = unwrap_box(10.0, 10.0, 10.0, None, false);
        let mirrored = mirror(&original, DVec3::ZERO, DVec3::X, false).unwrap();
        assert!(original.visible);
        assert_eq!(mirrored.type_string(), "SOLID");
    }

    // ── Immutability Tests ───────────────────────────────────────────────

    #[test]
    fn test_immutability_translate() {
        let original = unwrap_box(10.0, 10.0, 10.0, None, false);
        let original_id = original.shape_id;
        let _moved = translate(&original, 5.0, 0.0, 0.0, false);
        assert!(original.visible);
        assert_eq!(original.shape_id, original_id);
        assert_eq!(original.type_string(), "SOLID");
    }

    #[test]
    fn test_immutability_rotate() {
        let original = unwrap_box(10.0, 10.0, 10.0, None, false);
        let original_id = original.shape_id;
        let _rotated = rotate(&original, DVec3::Z, std::f64::consts::FRAC_PI_2, false);
        assert!(original.visible);
        assert_eq!(original.shape_id, original_id);
    }

    #[test]
    fn test_immutability_scale() {
        let original = unwrap_box(10.0, 10.0, 10.0, None, false);
        let original_id = original.shape_id;
        let _scaled = scale(&original, 2.0, DVec3::ZERO, false);
        assert!(original.visible);
        assert_eq!(original.shape_id, original_id);
    }

    #[test]
    fn test_make_rect() {
        let sd = make_rect(10.0, 20.0, false, "xy", None, false).unwrap();
        assert_eq!(sd.type_string(), "FACE");
    }

    #[test]
    fn test_make_circle() {
        let sd = make_circle(5.0, false, "xy", None, false).unwrap();
        assert_eq!(sd.type_string(), "FACE");
    }

    #[test]
    fn test_make_polygon() {
        let pts = [0.0, 0.0, 10.0, 0.0, 10.0, 10.0, 0.0, 10.0];
        let sd = make_polygon(&pts, false, "xy", None, false).unwrap();
        assert_eq!(sd.type_string(), "FACE");
    }

    #[test]
    fn test_extrude_face() {
        let sd = make_rect(10.0, 20.0, false, "xy", None, false).unwrap();
        let result = extrude_shape(&sd, 5.0, DVec3::Z, false, false).unwrap();
        assert_eq!(result.type_string(), "SOLID");
    }

    #[test]
    fn test_revolve_face_z() {
        let sd = make_rect(10.0, 20.0, false, "xy", None, false).unwrap();
        let result =
            revolve_shape(&sd, std::f64::consts::TAU, DVec3::ZERO, DVec3::Z, false).unwrap();
        assert_eq!(result.type_string(), "SOLID");
    }

    #[test]
    fn test_revolve_face_y() {
        let sd = make_circle(5.0, false, "xy", None, false).unwrap();
        let result = revolve_shape(
            &sd,
            std::f64::consts::FRAC_PI_2,
            DVec3::ZERO,
            DVec3::Y,
            false,
        )
        .unwrap();
        assert_eq!(result.type_string(), "SOLID");
    }

    #[test]
    fn test_revolve_non_face() {
        let sd = unwrap_box(10.0, 10.0, 10.0, None, false);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            revolve_shape(&sd, std::f64::consts::TAU, DVec3::ZERO, DVec3::Z, false)
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_wire_to_face() {
        let sd = make_rect(10.0, 20.0, true, "xy", None, false).unwrap();
        assert_eq!(sd.type_string(), "WIRE");
        let face = wire_to_face(&sd, false).unwrap();
        assert_eq!(face.type_string(), "FACE");
    }

    #[test]
    fn test_wire_queries() {
        let wire = make_rect(10.0, 20.0, true, "xy", None, false).unwrap();
        assert!(is_wire(&wire));
        assert!(!is_face(&wire));
        assert!(!is_solid(&wire));

        let face = make_rect(10.0, 20.0, false, "xy", None, false).unwrap();
        assert!(!is_wire(&face));
        assert!(is_face(&face));
        assert!(!is_solid(&face));

        let solid = unwrap_box(10.0, 10.0, 10.0, None, false);
        assert!(!is_wire(&solid));
        assert!(!is_face(&solid));
        assert!(is_solid(&solid));
    }

    #[test]
    fn test_rect_wire() {
        let sd = make_rect(10.0, 20.0, true, "xy", None, false).unwrap();
        assert_eq!(sd.type_string(), "WIRE");
    }

    #[test]
    fn test_circle_wire() {
        let sd = make_circle(5.0, true, "xy", None, false).unwrap();
        assert_eq!(sd.type_string(), "WIRE");
    }

    #[test]
    fn test_extrude_both() {
        let face = make_rect(10.0, 20.0, false, "xy", None, false).unwrap();
        let solid = extrude_shape(&face, 5.0, DVec3::Z, true, false).unwrap();
        assert_eq!(solid.type_string(), "SOLID");
    }

    #[test]
    fn test_extrude_polygon() {
        let pts = [0.0, 0.0, 10.0, 0.0, 10.0, 10.0, 0.0, 10.0];
        let sd = extrude_polygon_raw(&pts, 5.0, "xy", None, false).unwrap();
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_wire_fillet() {
        let wire = make_rect(10.0, 20.0, true, "xy", None, false).unwrap();
        let result = wire_fillet(&wire, 2.0, false).unwrap();
        assert_eq!(result.type_string(), "WIRE");
    }

    #[test]
    fn test_wire_chamfer() {
        let wire = make_rect(10.0, 20.0, true, "xy", None, false).unwrap();
        let result = wire_chamfer(&wire, 1.0, false).unwrap();
        assert_eq!(result.type_string(), "WIRE");
    }

    #[test]
    fn test_wire_offset() {
        let wire = make_rect(10.0, 20.0, true, "xy", None, false).unwrap();
        let result = wire_offset(&wire, 2.0, false).unwrap();
        assert_eq!(result.type_string(), "WIRE");
    }

    #[test]
    fn test_extrude_face_normal() {
        let face = make_rect(10.0, 20.0, false, "xy", None, false).unwrap();
        let solid = extrude_shape(&face, 5.0, DVec3::ZERO, false, false).unwrap();
        assert_eq!(solid.type_string(), "SOLID");
    }

    #[test]
    fn test_extrude_non_face() {
        let solid = unwrap_box(10.0, 10.0, 10.0, None, false);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = extrude_shape(&solid, 5.0, DVec3::Z, false, false);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_wire_to_face_non_wire() {
        let solid = unwrap_box(10.0, 10.0, 10.0, None, false);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = wire_to_face(&solid, false);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_extrude_polygon_xz_plane() {
        let pts = [0.0, 0.0, 10.0, 0.0, 10.0, 10.0, 0.0, 10.0];
        let sd = extrude_polygon_raw(&pts, 5.0, "xz", None, false).unwrap();
        assert_eq!(sd.type_string(), "SOLID");
    }

    #[test]
    fn test_immutability_mirror() {
        let original = unwrap_box(10.0, 10.0, 10.0, None, false);
        let original_id = original.shape_id;
        let _mirrored = mirror(&original, DVec3::ZERO, DVec3::X, false);
        assert!(original.visible);
        assert_eq!(original.shape_id, original_id);
    }

    #[test]
    fn test_wire_no_validate_negative_fillet() {
        let wire = make_rect(10.0, 20.0, true, "xy", None, false).unwrap();
        let result = wire_fillet(&wire, -1.0, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_wire_no_validate_negative_chamfer() {
        let wire = make_rect(10.0, 20.0, true, "xy", None, false).unwrap();
        let result = wire_chamfer(&wire, -1.0, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_wire_no_validate_negative_offset() {
        let wire = make_rect(10.0, 20.0, true, "xy", None, false).unwrap();
        let result = wire_offset(&wire, -1.0, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_fit_bounds_single() {
        let mut sd = make_box(1.0, 1.0, 1.0, None, false).unwrap();
        sd.mesh = Some(MeshData {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [10.0, 0.0, 0.0],
                [0.0, 10.0, 0.0],
                [0.0, 0.0, 10.0],
            ],
            normals: vec![],
            indices: vec![],
        });
        let refs = [&sd];
        let (center, radius) = compute_union_bounds(&refs).unwrap();
        assert!((center - DVec3::new(5.0, 5.0, 5.0)).length() < 1e-10);
        let expected_radius = DVec3::new(10.0, 10.0, 10.0).length() * 0.5 * 1.3;
        assert!((radius - expected_radius).abs() < 1e-10);
    }

    #[test]
    fn test_fit_bounds_multiple() {
        let mut sd1 = make_box(1.0, 1.0, 1.0, None, false).unwrap();
        sd1.mesh = Some(MeshData {
            vertices: vec![[0.0, 0.0, 0.0], [10.0, 10.0, 10.0]],
            normals: vec![],
            indices: vec![],
        });
        let mut sd2 = make_box(1.0, 1.0, 1.0, None, false).unwrap();
        sd2.mesh = Some(MeshData {
            vertices: vec![[20.0, 20.0, 20.0], [30.0, 30.0, 30.0]],
            normals: vec![],
            indices: vec![],
        });
        let refs = [&sd1, &sd2];
        let (center, radius) = compute_union_bounds(&refs).unwrap();
        assert!((center - DVec3::new(15.0, 15.0, 15.0)).length() < 1e-10);
        let diagonal = DVec3::new(30.0, 30.0, 30.0).length();
        let expected_radius = diagonal * 0.5 * 1.3;
        assert!((radius - expected_radius).abs() < 1e-10);
    }

    #[test]
    fn test_fit_bounds_no_mesh() {
        let sd = make_box(1.0, 1.0, 1.0, None, false).unwrap();
        let refs = [&sd];
        assert!(compute_union_bounds(&refs).is_none());
    }

    #[test]
    fn test_fit_bounds_single_vertex() {
        let mut sd = make_box(1.0, 1.0, 1.0, None, false).unwrap();
        sd.mesh = Some(MeshData {
            vertices: vec![[42.0, 43.0, 44.0]],
            normals: vec![],
            indices: vec![],
        });
        let refs = [&sd];
        let (center, radius) = compute_union_bounds(&refs).unwrap();
        assert!((center - DVec3::new(42.0, 43.0, 44.0)).length() < 1e-10);
        assert!(radius.abs() < 1e-10);
    }

    // ── Compound Tests ────────────────────────────────────────────────────

    #[test]
    fn test_make_compound_two_shapes() {
        let a = unwrap_box(10.0, 10.0, 10.0, None, false);
        let b = unwrap_sphere(5.0, None, None, false);
        let refs = [&a, &b];
        let result = make_compound(&refs, false).unwrap();
        assert_eq!(result.type_string(), "COMPOUND");
    }

    #[test]
    fn test_make_compound_too_few() {
        let a = unwrap_box(10.0, 10.0, 10.0, None, false);
        let refs = [&a];
        let result = make_compound(&refs, false);
        match result {
            Err(msg) => assert!(msg.contains("at least two"), "{}", msg),
            Ok(_) => panic!("expected error for single shape"),
        }
    }

    #[test]
    fn test_make_compound_empty() {
        let refs: [&ShapeData; 0] = [];
        let result = make_compound(&refs, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_make_compound_three_shapes() {
        let a = unwrap_box(10.0, 10.0, 10.0, None, false);
        let b = unwrap_sphere(5.0, None, None, false);
        let c = make_cylinder(5.0, 10.0, None, false).unwrap();
        let refs = [&a, &b, &c];
        let result = make_compound(&refs, false).unwrap();
        assert_eq!(result.type_string(), "COMPOUND");
    }

    #[test]
    fn test_make_compound_eager() {
        let a = unwrap_box(10.0, 10.0, 10.0, None, false);
        let b = unwrap_sphere(5.0, None, None, false);
        let refs = [&a, &b];
        let result = make_compound(&refs, true).unwrap();
        assert_eq!(result.type_string(), "COMPOUND");
        assert!(result.mesh.is_some());
    }

    // ── Color Tests ───────────────────────────────────────────────────────

    #[test]
    fn test_set_color_and_get_color() {
        let mut sd = unwrap_box(10.0, 10.0, 10.0, None, false);
        assert!(sd.color.is_none());
        assert!(get_color(&sd).is_none());
        set_color(&mut sd, 0.8, 0.2, 0.2);
        assert_eq!(sd.color, Some([0.8, 0.2, 0.2]));
        assert_eq!(get_color(&sd), Some([0.8, 0.2, 0.2]));
    }

    #[test]
    fn test_set_color_clamping() {
        let mut sd = unwrap_box(10.0, 10.0, 10.0, None, false);
        set_color(&mut sd, 1.5, -0.5, 0.0);
        assert_eq!(sd.color, Some([1.0, 0.0, 0.0]));
    }

    #[test]
    fn test_color_default_none() {
        let sd = unwrap_box(10.0, 10.0, 10.0, None, false);
        assert!(get_color(&sd).is_none());
    }
}
