use glam::DVec3;

use crate::types::MeshData;

/// Result of a ray-shape intersection test.
#[derive(Debug, Clone)]
pub struct PickResult {
    pub shape_id: u64,
    pub distance: f64,
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
