use crate::types::{MeshData, ShapeEntry, ShapeId};
use glam::{DMat4, DVec2, DVec3};

/// Result of a ray-shape intersection test.
#[derive(Debug, Clone)]
pub struct PickResult {
    pub shape_id: u64,
    pub distance: f64,
}

/// Result of an edge pick test.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EdgePickResult {
    pub shape_id: u64,
    pub edge_index: usize,
    pub distance: f64,
    pub hit_point: DVec3,
}

/// Möller–Trumbore ray-triangle intersection algorithm.
/// Returns `Some(t)` where `t` is the distance along the ray to the intersection point,
/// or `None` if no intersection.
pub fn ray_triangle_intersect(
    origin: DVec3,
    dir: DVec3,
    v0: DVec3,
    v1: DVec3,
    v2: DVec3,
) -> Option<f64> {
    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let h = dir.cross(edge2);
    let a = edge1.dot(h);
    if a.abs() < 1e-12 {
        return None;
    }
    let f = 1.0 / a;
    let s = origin - v0;
    let u = f * s.dot(h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(edge1);
    let v = f * dir.dot(q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = f * edge2.dot(q);
    if t > 1e-12 { Some(t) } else { None }
}

/// Cast a ray against all meshes and return the closest intersection.
pub fn pick_shape(origin: DVec3, dir: DVec3, meshes: &[(u64, &MeshData)]) -> Option<PickResult> {
    let mut closest: Option<PickResult> = None;
    for (shape_id, mesh) in meshes {
        for chunk in mesh.indices.chunks(3) {
            if chunk.len() < 3 {
                continue;
            }
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;
            if i0 >= mesh.vertices.len() || i1 >= mesh.vertices.len() || i2 >= mesh.vertices.len() {
                continue;
            }
            let v0: DVec3 = mesh.vertices[i0].map(|x| x as f64).into();
            let v1: DVec3 = mesh.vertices[i1].map(|x| x as f64).into();
            let v2: DVec3 = mesh.vertices[i2].map(|x| x as f64).into();
            if let Some(t) = ray_triangle_intersect(origin, dir, v0, v1, v2) {
                match closest.as_ref() {
                    Some(best) if t < best.distance => {
                        closest = Some(PickResult {
                            shape_id: *shape_id,
                            distance: t,
                        });
                    }
                    None => {
                        closest = Some(PickResult {
                            shape_id: *shape_id,
                            distance: t,
                        });
                    }
                    _ => {}
                }
            }
        }
    }
    closest
}

/// Check whether an edge hit point is occluded by its own shape's mesh.
/// Returns true if the point is behind the mesh surface (hidden edge).
pub fn is_edge_hidden(
    hit_point: DVec3,
    shape_id: ShapeId,
    meshes: &[(ShapeId, &MeshData)],
    origin: DVec3,
) -> bool {
    let dir = (hit_point - origin).normalize();
    let point_dist = (hit_point - origin).length();

    for &(sid, mesh) in meshes {
        if sid != shape_id {
            continue;
        }
        for chunk in mesh.indices.chunks(3) {
            if chunk.len() < 3 {
                continue;
            }
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;
            if i0 >= mesh.vertices.len() || i1 >= mesh.vertices.len() || i2 >= mesh.vertices.len() {
                continue;
            }
            let v0: DVec3 = mesh.vertices[i0].map(|x| x as f64).into();
            let v1: DVec3 = mesh.vertices[i1].map(|x| x as f64).into();
            let v2: DVec3 = mesh.vertices[i2].map(|x| x as f64).into();
            if let Some(t) = ray_triangle_intersect(origin, dir, v0, v1, v2) {
                if t < point_dist - 0.1 {
                    return true;
                }
            }
        }
    }
    false
}

/// Screen-space edge picking.
///
/// Projects all visible edge polylines to NDC and measures distance from
/// `click_ndc` to each projected segment. Returns the closest edge within
/// `threshold_px` pixels. Only real topological edges (index < topo_edge_count)
/// are considered — synthetic wireframe edges are skipped.
pub fn pick_edge(
    click_ndc: DVec2,
    view_proj: DMat4,
    window_size: (f32, f32),
    shapes: &[ShapeEntry],
    threshold_px: f32,
) -> Option<EdgePickResult> {
    let (w, h) = (window_size.0 as f64, window_size.1 as f64);
    let min_dim = w.min(h).max(1.0);
    let threshold = threshold_px as f64 / min_dim * 2.0;
    let threshold_sq = threshold * threshold;

    let mut best: Option<EdgePickResult> = None;

    for entry in shapes {
        for (edge_idx, polyline) in entry.edge_polylines.iter().enumerate() {
            if edge_idx >= entry.topo_edge_count {
                continue;
            }
            for pair in polyline.windows(2) {
                let a = DVec3::from_array(pair[0]);
                let b = DVec3::from_array(pair[1]);

                let a_h = view_proj * a.extend(1.0);
                let b_h = view_proj * b.extend(1.0);

                if a_h.w <= 0.0 || b_h.w <= 0.0 {
                    continue;
                }

                let a_ndc = DVec2::new(a_h.x / a_h.w, a_h.y / a_h.w);
                let b_ndc = DVec2::new(b_h.x / b_h.w, b_h.y / b_h.w);

                let seg = b_ndc - a_ndc;
                let seg_len_sq = seg.length_squared();
                if seg_len_sq < 1e-12 {
                    continue;
                }

                let t = (click_ndc - a_ndc).dot(seg) / seg_len_sq;
                let t_clamped = t.clamp(0.0, 1.0);
                let closest_ndc = a_ndc + seg * t_clamped;
                let dist_sq = click_ndc.distance_squared(closest_ndc);

                if dist_sq > threshold_sq {
                    continue;
                }

                let w_depth = a_h.w * (1.0 - t_clamped) + b_h.w * t_clamped;

                let is_better = match &best {
                    Some(prev) => w_depth < prev.distance,
                    None => true,
                };

                if is_better {
                    let hit_point = DVec3::from_array(pair[0]) * (1.0 - t_clamped)
                        + DVec3::from_array(pair[1]) * t_clamped;
                    best = Some(EdgePickResult {
                        shape_id: entry.shape_id,
                        edge_index: edge_idx,
                        distance: w_depth,
                        hit_point,
                    });
                }
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ShapeId;

    fn make_entry(id: ShapeId, topo_count: usize, polylines: Vec<Vec<[f64; 3]>>) -> ShapeEntry {
        ShapeEntry {
            shape_id: id,
            mesh: None,
            edge_polylines: polylines,
            topo_edge_count: topo_count,
            visible: true,
            color: None,
        }
    }

    #[test]
    fn test_pick_edge_hits_segment() {
        // A single edge: line from (-5, 0, 10) to (5, 0, 10)
        let polyline = vec![[-5.0, 0.0, 10.0], [5.0, 0.0, 10.0]];
        let shapes = vec![make_entry(1, 1, vec![polyline])];

        // Identity view-proj: looking down -Z, so Z=10 maps to NDC z near -1
        let view_proj = DMat4::IDENTITY;
        let wsize = (800.0, 600.0);

        // Click at NDC (0, 0) which corresponds to the midpoint of the line
        let result = pick_edge(DVec2::new(0.0, 0.0), view_proj, wsize, &shapes, 10.0);
        assert!(result.is_some(), "should hit the edge at NDC center");
        let hit = result.unwrap();
        assert_eq!(hit.shape_id, 1);
        assert_eq!(hit.edge_index, 0);
    }

    #[test]
    fn test_pick_edge_misses() {
        let polyline = vec![[-5.0, 0.0, 10.0], [5.0, 0.0, 10.0]];
        let shapes = vec![make_entry(1, 1, vec![polyline])];

        let view_proj = DMat4::IDENTITY;
        let wsize = (800.0, 600.0);

        // Click far from the edge — should miss
        let result = pick_edge(DVec2::new(10.0, 10.0), view_proj, wsize, &shapes, 1.0);
        assert!(result.is_none(), "should miss when far from edge");
    }

    #[test]
    fn test_pick_edge_skips_synthetic() {
        // Two edges: one real (topo), one synthetic
        let polyline = vec![[-5.0, 0.0, 10.0], [5.0, 0.0, 10.0]];
        let synthetic = vec![[-5.0, 5.0, 10.0], [5.0, 5.0, 10.0]];
        let shapes = vec![make_entry(1, 1, vec![polyline, synthetic])];

        let view_proj = DMat4::IDENTITY;
        let wsize = (800.0, 600.0);

        // Click near the synthetic edge — should miss because topo_count=1
        let result = pick_edge(DVec2::new(0.0, 5.0 / 10.0), view_proj, wsize, &shapes, 10.0);
        assert!(result.is_none(), "should not pick synthetic edges");

        // Click near the real edge — should hit
        let result = pick_edge(DVec2::new(0.0, 0.0), view_proj, wsize, &shapes, 10.0);
        assert!(result.is_some(), "should pick real edges");
    }

    #[test]
    fn test_pick_edge_with_default_camera() {
        // Match the default OrbitCamera: target=(0,0,0), radius=50, yaw=0, pitch=0.4
        let pos = DVec3::new(50.0 * 0.4f64.cos(), 50.0 * 0.4f64.sin(), 0.0);
        let view = DMat4::look_at_lh(pos, DVec3::ZERO, DVec3::Y);
        let proj =
            DMat4::perspective_lh(std::f64::consts::FRAC_PI_4, 1920.0 / 1080.0, 0.1, 10000.0);
        let view_proj = proj * view;
        let wsize = (1920.0, 1080.0);

        // Create a 10x10x10 box centered at origin, get its first edge polyline
        let occt_shape = opencascade::primitives::Shape::box_with_dimensions(10.0, 10.0, 10.0);
        let polylines = crate::cad::extract_edge_polylines(&occt_shape);
        assert!(!polylines.is_empty(), "box should have edge polylines");

        let topo_count = polylines.len();
        let shapes = vec![ShapeEntry {
            shape_id: 1,
            mesh: None,
            edge_polylines: polylines,
            topo_edge_count: topo_count,
            visible: true,
            color: None,
        }];

        // Project the first edge's midpoint to NDC
        let first_edge = &shapes[0].edge_polylines[0];
        let num_points = first_edge.len();
        let mid = [
            (first_edge[0][0] + first_edge[num_points - 1][0]) / 2.0,
            (first_edge[0][1] + first_edge[num_points - 1][1]) / 2.0,
            (first_edge[0][2] + first_edge[num_points - 1][2]) / 2.0,
        ];
        let mid_clip = view_proj * DVec3::from_array(mid).extend(1.0);
        let mid_ndc = DVec2::new(mid_clip.x / mid_clip.w, mid_clip.y / mid_clip.w);

        // Click at the edge's NDC position — should hit
        let result = pick_edge(mid_ndc, view_proj, wsize, &shapes, 12.0);
        assert!(
            result.is_some(),
            "should hit edge at its NDC position {:?} (mid={:?})",
            mid_ndc,
            mid
        );
        if let Some(ref hit) = result {
            assert_eq!(hit.shape_id, 1);
        }
    }

    #[test]
    fn test_pick_edge_with_perspective_projection() {
        let view = DMat4::look_at_lh(DVec3::new(0.0, 0.0, 50.0), DVec3::ZERO, DVec3::Y);
        let proj = DMat4::perspective_lh(std::f64::consts::FRAC_PI_4, 800.0 / 600.0, 0.1, 10000.0);
        let view_proj = proj * view;
        let wsize = (800.0, 600.0);

        // Edge from (-5, 5, 5) to (5, 5, 5), projects to NDC y ≈ 0.268
        let polyline = vec![[-5.0, 5.0, 5.0], [5.0, 5.0, 5.0]];

        // Compute where the edge midpoint projects in NDC
        let mid_clip = view_proj * DVec3::new(0.0, 5.0, 5.0).extend(1.0);
        let mid_ndc = DVec2::new(mid_clip.x / mid_clip.w, mid_clip.y / mid_clip.w);

        let shapes = vec![make_entry(1, 1, vec![polyline])];
        let result = pick_edge(mid_ndc, view_proj, wsize, &shapes, 20.0);
        assert!(
            result.is_some(),
            "should hit edge at its NDC position {:?}",
            mid_ndc
        );
        if let Some(ref hit) = result {
            assert_eq!(hit.shape_id, 1);
            assert_eq!(hit.edge_index, 0);
        }
    }

    #[test]
    fn test_pick_edge_finds_closest() {
        // Two edges at different depths
        let front = vec![[-5.0, 0.0, 5.0], [5.0, 0.0, 5.0]]; // Z=5 (closer)
        let back = vec![[-5.0, 0.0, 15.0], [5.0, 0.0, 15.0]]; // Z=15 (farther)
        let shapes = vec![make_entry(1, 1, vec![front]), make_entry(2, 1, vec![back])];

        let view_proj = DMat4::IDENTITY;
        let wsize = (800.0, 600.0);

        // Click at center — should pick front edge (shape_id=1)
        let result = pick_edge(DVec2::new(0.0, 0.0), view_proj, wsize, &shapes, 10.0);
        assert!(result.is_some());
        assert_eq!(result.unwrap().shape_id, 1, "should pick closer edge");
    }
}
