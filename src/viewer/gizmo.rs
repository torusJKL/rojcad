use glam::{DMat4, DVec3, DVec4, Vec3};
use wgpu::{self, util::DeviceExt};

use super::camera::OrbitCamera;

const AXIS_LEN: f32 = 1.0;
const CIRCLE_RADIUS: f32 = 0.2;
const SPHERE_LON: u32 = 14;
const SPHERE_LAT: u32 = 8;
const LINE_WIDTH: f32 = 0.055;
const HOVER_SCALE: f32 = 1.35;

const GIZMO_VIEWPORT_SIZE: f64 = 2.0;
const LETTER_DEPTH_BIAS: f32 = CIRCLE_RADIUS + 0.005;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GizmoUniforms {
    view_proj: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GizmoVertex {
    position: [f32; 3],
    color: [f32; 4],
}

fn axis_color(index: usize) -> [f32; 4] {
    match index {
        0 => [0.941, 0.216, 0.322, 1.0],
        1 => [0.463, 0.702, 0.086, 1.0],
        2 => [0.176, 0.537, 0.941, 1.0],
        _ => [0.5, 0.5, 0.5, 1.0],
    }
}

fn to_array(v: Vec3) -> [f32; 3] {
    [v.x, v.y, v.z]
}

fn generate_cylinder(
    center: Vec3,
    tip: Vec3,
    color: [f32; 4],
    radius: f32,
    segments: u32,
) -> Vec<GizmoVertex> {
    let dir = (tip - center).normalize();
    let up = if dir.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
    let right = dir.cross(up).normalize();
    let forward = right.cross(dir);

    let mut verts = Vec::with_capacity((segments * 6) as usize);

    for i in 0..segments {
        let angle0 = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let angle1 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

        let (sin0, cos0) = angle0.sin_cos();
        let (sin1, cos1) = angle1.sin_cos();

        let r0 = right * cos0 + forward * sin0;
        let r1 = right * cos1 + forward * sin1;

        let c0 = center + r0 * radius;
        let c1 = center + r1 * radius;
        let t0 = tip + r0 * radius;
        let t1 = tip + r1 * radius;

        verts.push(GizmoVertex {
            position: to_array(c0),
            color,
        });
        verts.push(GizmoVertex {
            position: to_array(t0),
            color,
        });
        verts.push(GizmoVertex {
            position: to_array(c1),
            color,
        });
        verts.push(GizmoVertex {
            position: to_array(t0),
            color,
        });
        verts.push(GizmoVertex {
            position: to_array(t1),
            color,
        });
        verts.push(GizmoVertex {
            position: to_array(c1),
            color,
        });
    }

    verts
}

fn generate_sphere(center: Vec3, color: [f32; 4], radius: f32) -> Vec<GizmoVertex> {
    let mut verts = Vec::new();

    for lat in 0..SPHERE_LAT {
        let theta0 = (lat as f32 / SPHERE_LAT as f32) * std::f32::consts::PI;
        let theta1 = ((lat + 1) as f32 / SPHERE_LAT as f32) * std::f32::consts::PI;
        let sin_t0 = theta0.sin();
        let cos_t0 = theta0.cos();
        let sin_t1 = theta1.sin();
        let cos_t1 = theta1.cos();

        for lon in 0..SPHERE_LON {
            let phi0 = (lon as f32 / SPHERE_LON as f32) * std::f32::consts::TAU;
            let phi1 = ((lon + 1) as f32 / SPHERE_LON as f32) * std::f32::consts::TAU;
            let sin_p0 = phi0.sin();
            let cos_p0 = phi0.cos();
            let sin_p1 = phi1.sin();
            let cos_p1 = phi1.cos();

            let v0 = Vec3::new(sin_t0 * cos_p0, cos_t0, sin_t0 * sin_p0) * radius + center;
            let v1 = Vec3::new(sin_t0 * cos_p1, cos_t0, sin_t0 * sin_p1) * radius + center;
            let v2 = Vec3::new(sin_t1 * cos_p0, cos_t1, sin_t1 * sin_p0) * radius + center;
            let v3 = Vec3::new(sin_t1 * cos_p1, cos_t1, sin_t1 * sin_p1) * radius + center;

            verts.push(GizmoVertex {
                position: to_array(v0),
                color,
            });
            verts.push(GizmoVertex {
                position: to_array(v2),
                color,
            });
            verts.push(GizmoVertex {
                position: to_array(v1),
                color,
            });
            verts.push(GizmoVertex {
                position: to_array(v1),
                color,
            });
            verts.push(GizmoVertex {
                position: to_array(v2),
                color,
            });
            verts.push(GizmoVertex {
                position: to_array(v3),
                color,
            });
        }
    }

    verts
}

fn build_letter_mesh_3d(
    center: Vec3,
    right: Vec3,
    up: Vec3,
    letter: u8,
    depth_bias: Vec3,
) -> Vec<GizmoVertex> {
    let hw = 0.09;
    let hh = 0.12;
    let lw = 0.03;
    let color = [0.15, 0.15, 0.15, 1.0];

    let pos = center + depth_bias;

    fn bar(center: Vec3, r: Vec3, u: Vec3, from: Vec3, to: Vec3, w: f32) -> Vec<Vec3> {
        let dir = (to - from).normalize();
        let perp = Vec3::new(-dir.y, dir.x, 0.0) * (w * 0.5);
        let a0 = from - perp;
        let a1 = from + perp;
        let b0 = to - perp;
        let b1 = to + perp;
        let to_w = |p: Vec3| center + r * p.x + u * p.y;
        vec![to_w(a0), to_w(b0), to_w(a1), to_w(b0), to_w(b1), to_w(a1)]
    }

    let bars: Vec<[Vec3; 2]> = match letter {
        b'X' => vec![
            [Vec3::new(-hw, -hh, 0.0), Vec3::new(hw, hh, 0.0)],
            [Vec3::new(hw, -hh, 0.0), Vec3::new(-hw, hh, 0.0)],
        ],
        b'Y' => vec![
            [Vec3::new(-hw, hh, 0.0), Vec3::new(0.0, 0.0, 0.0)],
            [Vec3::new(hw, hh, 0.0), Vec3::new(0.0, 0.0, 0.0)],
            [Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, -hh, 0.0)],
        ],
        b'Z' => vec![
            [Vec3::new(-hw, hh, 0.0), Vec3::new(hw, hh, 0.0)],
            [Vec3::new(hw, hh, 0.0), Vec3::new(-hw, -hh, 0.0)],
            [Vec3::new(-hw, -hh, 0.0), Vec3::new(hw, -hh, 0.0)],
        ],
        _ => vec![],
    };

    let mut verts = Vec::new();
    for [from, to] in bars {
        let pts = bar(pos, right, up, from, to, lw);
        for p in pts {
            verts.push(GizmoVertex {
                position: to_array(p),
                color,
            });
        }
    }
    verts
}

const HIT_TARGETS: [(f64, f64, f64); 3] = [(1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0)];

pub struct GizmoRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    pub hovered: Option<usize>,
    viewport_size: u32,
}

impl GizmoRenderer {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let vertices = Self::build_static_vertices(None);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("gizmo_vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let num_vertices = vertices.len() as u32;

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gizmo_uniforms"),
            size: std::mem::size_of::<GizmoUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gizmo_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gizmo_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("gizmo_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gizmo shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "gizmo.wgsl"
            ))),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("gizmo_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<GizmoVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: 12,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let (depth_texture, depth_texture_view) = Self::create_depth(device, 200);

        GizmoRenderer {
            pipeline,
            uniform_buffer,
            bind_group,
            vertex_buffer,
            num_vertices,
            depth_texture,
            depth_texture_view,
            hovered: None,
            viewport_size: 200,
        }
    }

    fn create_depth(device: &wgpu::Device, size: u32) -> (wgpu::Texture, wgpu::TextureView) {
        let size = size.max(1);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("gizmo_depth"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    fn build_static_vertices(hovered: Option<usize>) -> Vec<GizmoVertex> {
        let axis_dirs: [Vec3; 3] = [Vec3::X, Vec3::Y, Vec3::Z];
        let mut verts = Vec::new();

        for (i, &dir) in axis_dirs.iter().enumerate() {
            let tip = dir * AXIS_LEN;
            let color = axis_color(i);

            verts.extend(generate_cylinder(
                Vec3::ZERO,
                tip,
                color,
                LINE_WIDTH * 0.5,
                SPHERE_LON,
            ));

            let r = if hovered == Some(i) {
                CIRCLE_RADIUS * HOVER_SCALE
            } else {
                CIRCLE_RADIUS
            };
            verts.extend(generate_sphere(tip, color, r));
        }

        verts.extend(generate_sphere(
            Vec3::ZERO,
            [0.4, 0.4, 0.4, 1.0],
            LINE_WIDTH,
        ));

        verts
    }

    pub fn set_viewport_size(&mut self, device: &wgpu::Device, size: u32) {
        self.viewport_size = size;
        let (tex, view) = Self::create_depth(device, size);
        self.depth_texture = tex;
        self.depth_texture_view = view;
    }

    pub fn depth_view(&self) -> &wgpu::TextureView {
        &self.depth_texture_view
    }

    pub fn update_uniforms(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        main_camera: &OrbitCamera,
    ) {
        let forward = main_camera.forward();
        let eye = forward * 5.0;
        let gizmo_view = DMat4::look_at_lh(eye, DVec3::ZERO, DVec3::Y);
        let gizmo_proj = DMat4::orthographic_lh(
            -GIZMO_VIEWPORT_SIZE,
            GIZMO_VIEWPORT_SIZE,
            -GIZMO_VIEWPORT_SIZE,
            GIZMO_VIEWPORT_SIZE,
            -10.0,
            10.0,
        );
        let view_proj = gizmo_proj * gizmo_view;

        let uniforms = GizmoUniforms {
            view_proj: view_proj.to_cols_array_2d().map(|r| r.map(|v| v as f32)),
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let f = (DVec3::ZERO - eye).normalize();
        let right = f.cross(DVec3::Y).normalize();
        let up = right.cross(f).normalize();

        let right_v3 = Vec3::new(right.x as f32, right.y as f32, right.z as f32);
        let up_v3 = Vec3::new(up.x as f32, up.y as f32, up.z as f32);
        let forward_v3 = Vec3::new(f.x as f32, f.y as f32, f.z as f32);
        let depth_bias = -forward_v3 * LETTER_DEPTH_BIAS;

        let axis_dirs = [Vec3::X, Vec3::Y, Vec3::Z];
        let letters = [b'X', b'Y', b'Z'];

        let mut letter_verts = Vec::new();
        for i in 0..3 {
            let center = axis_dirs[i] * AXIS_LEN;
            let r = if i == 2 { -right_v3 } else { right_v3 };
            letter_verts.extend(build_letter_mesh_3d(
                center, r, up_v3, letters[i], depth_bias,
            ));
        }

        let all_verts: Vec<GizmoVertex> = Self::build_static_vertices(self.hovered)
            .into_iter()
            .chain(letter_verts)
            .collect();

        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("gizmo_vertices"),
            contents: bytemuck::cast_slice(&all_verts),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        self.num_vertices = all_verts.len() as u32;
    }

    pub fn hit_test(&self, mx: f64, my: f64, main_camera: &OrbitCamera) -> Option<usize> {
        let vs = self.viewport_size as f64;
        let ndc_x = (mx / vs) * 2.0 - 1.0;
        let ndc_y = 1.0 - (my / vs) * 2.0;

        let forward = main_camera.forward();
        let eye = forward * 5.0;
        let f = (DVec3::ZERO - eye).normalize();
        let right = f.cross(DVec3::Y).normalize();
        let up = right.cross(f).normalize();

        let frustum_half = GIZMO_VIEWPORT_SIZE;
        let ray_origin = eye + right * ndc_x * frustum_half + up * ndc_y * frustum_half;
        let ray_dir = f;

        let sphere_radius = CIRCLE_RADIUS as f64;

        let mut best: Option<(usize, f64)> = None;

        for i in 0..3 {
            let (dx, dy, dz) = HIT_TARGETS[i];
            let center = DVec3::new(dx, dy, dz);

            let oc = ray_origin - center;
            let a = ray_dir.dot(ray_dir);
            let b = 2.0 * oc.dot(ray_dir);
            let c = oc.dot(oc) - sphere_radius * sphere_radius;
            let disc = b * b - 4.0 * a * c;

            if disc < 0.0 {
                continue;
            }

            let t = (-b - disc.sqrt()) / (2.0 * a);
            if t > 0.0 {
                let is_closer = best.as_ref().is_none_or(|&(_, d)| t < d);
                if is_closer {
                    best = Some((i, t));
                }
            }
        }

        best.map(|(i, _)| i)
    }

    pub fn set_hovered(&mut self, device: &wgpu::Device, hovered: Option<usize>) {
        if self.hovered == hovered {
            return;
        }
        self.hovered = hovered;
    }

    pub fn render(&self, pass: &mut wgpu::RenderPass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..self.num_vertices, 0..1);
    }
}
