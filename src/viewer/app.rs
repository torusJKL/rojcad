use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};

use glam::DVec4;
use wgpu::{self, util::DeviceExt};
#[cfg(target_os = "linux")]
use winit::platform::x11::EventLoopBuilderExtX11;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

use crate::types::{global_shape_registry, MeshData, ShapeId, REGISTRY_GENERATION, LAST_SELECTION};

use super::camera::OrbitCamera;
use super::pick::pick_shape;
use super::ReplToViewer;
use super::ViewerToRepl;

// ── Uniform buffer ────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ViewUniforms {
    view_proj: [[f32; 4]; 4],
}

// ── CadMesh ───────────────────────────────────────────────────────────────

/// GPU-side representation of a single shape's mesh.
pub struct CadMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub shape_id: ShapeId,
}

impl CadMesh {
    pub fn new(device: &wgpu::Device, mesh: &MeshData, shape_id: ShapeId) -> Self {
        debug_assert_eq!(
            mesh.normals.len(),
            mesh.vertices.len(),
            "CadMesh::new: normals count ({}) must match vertices count ({})",
            mesh.normals.len(),
            mesh.vertices.len()
        );
        let interleaved: Vec<MeshVertex> = mesh
            .vertices
            .iter()
            .zip(mesh.normals.iter())
            .map(|(pos, norm)| MeshVertex {
                position: *pos,
                normal: *norm,
            })
            .collect();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("cad_mesh_{}_vertex", shape_id)),
            contents: bytemuck::cast_slice(&interleaved),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("cad_mesh_{}_index", shape_id)),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        CadMesh {
            vertex_buffer,
            index_buffer,
            num_indices: mesh.indices.len() as u32,
            shape_id,
        }
    }
}

// ── Instanced line segment ────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct SegmentInstance {
    pub a: [f32; 3],
    pub b: [f32; 3],
    pub cum_length: f32,
}

// ── Vertex type ───────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MeshVertex {
    position: [f32; 3],
    normal: [f32; 3],
}

fn mesh_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x3,
            },
        ],
    }
}

// ── SurfaceDrawer ─────────────────────────────────────────────────────────

/// Renders mesh surfaces.
pub struct SurfaceDrawer {
    render_pipeline: wgpu::RenderPipeline,
    highlight_pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    meshes: Vec<CadMesh>,
}

impl SurfaceDrawer {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("surface_uniform_buffer"),
            size: std::mem::size_of::<ViewUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("surface_bind_group_layout"),
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

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("surface_uniform_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("surface_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline =
            Self::build_pipeline(device, &pipeline_layout, surface_format, depth_format, "fs_main");
        let highlight_pipeline = Self::build_pipeline(
            device,
            &pipeline_layout,
            surface_format,
            depth_format,
            "fs_highlight",
        );

        SurfaceDrawer {
            render_pipeline,
            highlight_pipeline,
            uniform_bind_group,
            uniform_buffer,
            meshes: Vec::new(),
        }
    }

    fn build_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        surface_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        fragment_entry: &str,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mesh shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                include_str!("shader.wgsl"),
            )),
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(fragment_entry),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[mesh_vertex_buffer_layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some(fragment_entry),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
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
        })
    }

    pub fn update_uniforms(&mut self, queue: &wgpu::Queue, uniforms: &ViewUniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(uniforms));
    }

    pub fn set_meshes(&mut self, meshes: Vec<CadMesh>) {
        self.meshes = meshes;
    }

    pub fn render(&self, pass: &mut wgpu::RenderPass, selected_id: Option<ShapeId>) {
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        for mesh in &self.meshes {
            let is_selected = Some(mesh.shape_id) == selected_id;
            if is_selected {
                pass.set_pipeline(&self.highlight_pipeline);
            } else {
                pass.set_pipeline(&self.render_pipeline);
            }
            pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
        }
    }
}

// ── EdgeDrawer ────────────────────────────────────────────────────────────

/// Renders edges as instanced lines.
pub struct EdgeDrawer {
    pub solid_pipeline: wgpu::RenderPipeline,
    dashed_pipeline: wgpu::RenderPipeline,
    pub uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
}

impl EdgeDrawer {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("edge_uniform_buffer"),
            size: std::mem::size_of::<ViewUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("edge_bind_group_layout"),
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

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("edge_uniform_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("edge_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let solid_pipeline = Self::build_pipeline(
            device,
            &pipeline_layout,
            surface_format,
            depth_format,
            "fs_solid",
            wgpu::CompareFunction::Less,
        );
        let dashed_pipeline = Self::build_pipeline(
            device,
            &pipeline_layout,
            surface_format,
            depth_format,
            "fs_dashed",
            wgpu::CompareFunction::Greater,
        );

        EdgeDrawer {
            solid_pipeline,
            dashed_pipeline,
            uniform_bind_group,
            uniform_buffer,
        }
    }

    fn build_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        surface_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        fragment_entry: &str,
        depth_compare: wgpu::CompareFunction,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("edge shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "edge.wgsl"
            ))),
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(fragment_entry),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x3,
                    }],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some(fragment_entry),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: false,
                depth_compare,
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
        })
    }

    pub fn update_uniforms(&mut self, queue: &wgpu::Queue, uniforms: &ViewUniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(uniforms));
    }

    pub fn render(
        &self,
        pass: &mut wgpu::RenderPass,
        edge_buffer: &wgpu::Buffer,
        num_vertices: u32,
        show_dashed: bool,
    ) {
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_vertex_buffer(0, edge_buffer.slice(..));

        // Pass 1: dashed back-edges (depth Greater)
        if show_dashed {
            pass.set_pipeline(&self.dashed_pipeline);
            pass.draw(0..num_vertices, 0..1);
        }

        // Pass 2: solid front-edges (depth Less)
        pass.set_pipeline(&self.solid_pipeline);
        pass.draw(0..num_vertices, 0..1);
    }
}

// ── Cone tip helpers ─────────────────────────────────────────────────────

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ConeVertex {
    position: [f32; 3],
}

fn cone_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<ConeVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x3,
        }],
    }
}

/// Build cone tip meshes (8 triangles each) for the three axis endpoints.
/// Returns (vertex_buffer, index_buffer, num_indices).
fn build_axis_cones(
    device: &wgpu::Device,
) -> (Option<wgpu::Buffer>, Option<wgpu::Buffer>, u32) {
    let segments = 8;
    let height = 1.5;
    let radius = 0.5;

    // Each cone: base ring (segments) + tip (1), triangles = segments
    // Three axes: X(red), Y(green), Z(blue)
    let axis_dirs: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let axis_len = 15.0f32;

    let mut vertices: Vec<ConeVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for dir in &axis_dirs {
        let tip = [dir[0] * axis_len, dir[1] * axis_len, dir[2] * axis_len];
        let base_center = [
            dir[0] * (axis_len - height),
            dir[1] * (axis_len - height),
            dir[2] * (axis_len - height),
        ];

        // Compute perpendicular vectors for the base ring
        let abs_x = dir[0].abs();
        let up = if abs_x < 0.9 {
            [1.0f32, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let u = [
            up[1] * dir[2] - up[2] * dir[1],
            up[2] * dir[0] - up[0] * dir[2],
            up[0] * dir[1] - up[1] * dir[0],
        ];
        let ulen = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
        let u = [u[0] / ulen, u[1] / ulen, u[2] / ulen];
        let v = [
            dir[1] * u[2] - dir[2] * u[1],
            dir[2] * u[0] - dir[0] * u[2],
            dir[0] * u[1] - dir[1] * u[0],
        ];

        let base_start = vertices.len() as u32;

        // Base ring vertices
        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let cx = angle.cos();
            let sx = angle.sin();
            vertices.push(ConeVertex {
                position: [
                    base_center[0] + (u[0] * cx + v[0] * sx) * radius,
                    base_center[1] + (u[1] * cx + v[1] * sx) * radius,
                    base_center[2] + (u[2] * cx + v[2] * sx) * radius,
                ],
            });
        }

        let tip_index = vertices.len() as u32;
        vertices.push(ConeVertex { position: tip });

        // Triangles from base to tip
        for i in 0..segments {
            let i0 = base_start + i as u32;
            let i1 = base_start + ((i + 1) % segments) as u32;
            indices.push(i0);
            indices.push(i1);
            indices.push(tip_index);
        }
    }

    if vertices.is_empty() || indices.is_empty() {
        return (None, None, 0);
    }

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("axis_cone_vertices"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("axis_cone_indices"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    (Some(vertex_buffer), Some(index_buffer), indices.len() as u32)
}

fn build_cone_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
) -> Option<wgpu::RenderPipeline> {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("cone shader"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
            "struct VertexInput { @location(0) position: vec3<f32>, };\
             struct VertexOutput { @builtin(position) clip_position: vec4<f32>, };\
             @vertex fn vs_main(input: VertexInput) -> VertexOutput {\
               var out: VertexOutput;\
               out.clip_position = vec4f(input.position, 1.0);\
               return out;\
             }\
             @fragment fn fs_main() -> @location(0) vec4<f32> {\
               return vec4f(0.8, 0.8, 0.8, 1.0);\
             }",
        )),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("cone_pipeline_layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("cone_pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[cone_vertex_buffer_layout()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::REPLACE),
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

    Some(pipeline)
}

// ── Line vertex buffer helpers ────────────────────────────────────────────

/// Flatten a slice of segment endpoint pairs into a vertex buffer.
fn segments_to_vertices(segments: &[[[f32; 3]; 2]]) -> Vec<[f32; 3]> {
    let mut verts = Vec::with_capacity(segments.len() * 2);
    for seg in segments {
        verts.push(seg[0]);
        verts.push(seg[1]);
    }
    verts
}

/// Build a GPU vertex buffer from segment endpoint pairs.
fn create_line_buffer(device: &wgpu::Device, label: &str, segments: &[[[f32; 3]; 2]]) -> wgpu::Buffer {
    let verts = segments_to_vertices(segments);
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(&verts),
        usage: wgpu::BufferUsages::VERTEX,
    })
}

// ── GridRenderer ──────────────────────────────────────────────────────────

/// Grid line instances.
pub struct GridRenderer {
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
}

impl GridRenderer {
    pub fn new(device: &wgpu::Device) -> Self {
        let segments = generate_grid_segments();
        let verts = segments_to_vertices(&segments);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("grid_lines"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsages::VERTEX,
        });
        GridRenderer {
            vertex_buffer,
            num_vertices: verts.len() as u32,
        }
    }

    pub fn render(&self, pass: &mut wgpu::RenderPass) {
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..self.num_vertices, 0..1);
    }
}

fn generate_grid_segments() -> Vec<[[f32; 3]; 2]> {
    let mut segments = Vec::new();
    let half_size = 100.0;
    let major_step = 10.0;

    // Central axes (X and Z through origin)
    segments.push([[-half_size, 0.0, 0.0], [half_size, 0.0, 0.0]]);
    segments.push([[0.0, 0.0, -half_size], [0.0, 0.0, half_size]]);

    // Major and minor grid lines on XZ plane
    let mut z = major_step;
    while z <= half_size {
        // Lines parallel to X axis
        segments.push([[-half_size, 0.0, z], [half_size, 0.0, z]]);
        segments.push([[-half_size, 0.0, -z], [half_size, 0.0, -z]]);
        // Lines parallel to Z axis
        segments.push([[z, 0.0, -half_size], [z, 0.0, half_size]]);
        segments.push([[-z, 0.0, -half_size], [-z, 0.0, half_size]]);
        z += major_step;
    }

    segments
}

// ── AxisRenderer ──────────────────────────────────────────────────────────

/// Axis indicator (RGB = XYZ).
pub struct AxisRenderer {
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
}

impl AxisRenderer {
    pub fn new(device: &wgpu::Device) -> Self {
        let segments: Vec<[[f32; 3]; 2]> = vec![
            // X axis (red)
            [[0.0, 0.0, 0.0], [15.0, 0.0, 0.0]],
            // Y axis (green)
            [[0.0, 0.0, 0.0], [0.0, 15.0, 0.0]],
            // Z axis (blue)
            [[0.0, 0.0, 0.0], [0.0, 0.0, 15.0]],
        ];
        let verts = segments_to_vertices(&segments);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("axis_lines"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsages::VERTEX,
        });
        AxisRenderer {
            vertex_buffer,
            num_vertices: verts.len() as u32,
        }
    }

    pub fn render(&self, pass: &mut wgpu::RenderPass) {
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..self.num_vertices, 0..1);
    }
}

// ── ViewerState ───────────────────────────────────────────────────────────

pub struct ViewerState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    window: Arc<Window>,
    camera: OrbitCamera,
    size: PhysicalSize<u32>,
    surface_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    surface_drawer: SurfaceDrawer,
    edge_drawer: EdgeDrawer,
    edge_vertex_buffer: wgpu::Buffer,
    edge_num_vertices: u32,
    grid_renderer: GridRenderer,
    axis_renderer: AxisRenderer,
    cone_pipeline: Option<wgpu::RenderPipeline>,
    axis_cone_buffer: Option<wgpu::Buffer>,
    axis_cone_index_buffer: Option<wgpu::Buffer>,
    axis_cone_num_indices: u32,
    selected_id: Option<ShapeId>,
    show_back_edges: bool,
    mouse_pressed: [bool; 3],
    mouse_pos: PhysicalPosition<f64>,
    last_generation: u64,
}

// ── ViewerApp ─────────────────────────────────────────────────────────────

struct ViewerApp {
    repl_rx: Receiver<ReplToViewer>,
    viewer_tx: Sender<ViewerToRepl>,
    running: Arc<AtomicBool>,
    state: Option<ViewerState>,
}

/// Main entry point for the viewer thread.
pub fn run_viewer(
    repl_rx: Receiver<ReplToViewer>,
    viewer_tx: Sender<ViewerToRepl>,
    running: Arc<AtomicBool>,
) {
    let mut builder = EventLoop::builder();
    #[cfg(target_os = "linux")]
    builder.with_any_thread(true);
    let event_loop = builder.build().expect("failed to create winit event loop");

    let mut app = ViewerApp {
        repl_rx,
        viewer_tx,
        running,
        state: None,
    };

    event_loop.run_app(&mut app).expect("event loop failed");
}

// ── Depth texture helpers ─────────────────────────────────────────────────

fn create_depth_texture(
    device: &wgpu::Device,
    size: PhysicalSize<u32>,
) -> (wgpu::Texture, wgpu::TextureView) {
    let depth_format = wgpu::TextureFormat::Depth32Float;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth_texture"),
        size: wgpu::Extent3d {
            width: size.width.max(1),
            height: size.height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: depth_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

// ── ApplicationHandler implementation ─────────────────────────────────────

impl ApplicationHandler for ViewerApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let window_attrs = Window::default_attributes()
            .with_title("rojcad — 3D Viewer")
            .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0));
        let window = Arc::new(
            event_loop
                .create_window(window_attrs)
                .expect("failed to create window"),
        );

        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window.clone())
            .expect("failed to create surface");
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            }))
            .expect("failed to find adapter");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
                .expect("failed to create device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let depth_format = wgpu::TextureFormat::Depth32Float;
        let (depth_texture, depth_texture_view) = create_depth_texture(&device, size);

        let camera = OrbitCamera::new();
        let surface_drawer = SurfaceDrawer::new(&device, surface_format, depth_format);
        let edge_drawer = EdgeDrawer::new(&device, surface_format, depth_format);
        let grid_renderer = GridRenderer::new(&device);
        let axis_renderer = AxisRenderer::new(&device);

        // Initial empty edge buffer (populated from ShapeRegistry each frame)
        let edge_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("edge_lines"),
            size: 1,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });
        let edge_num_vertices = 0u32;

        let (axis_cone_buffer, axis_cone_index_buffer, axis_cone_num_indices) =
            build_axis_cones(&device);

        let cone_pipeline = build_cone_pipeline(&device, surface_format);

        self.state = Some(ViewerState {
            device,
            queue,
            surface,
            surface_config,
            depth_texture,
            depth_texture_view,
            window,
            camera,
            size,
            surface_format,
            depth_format,
            surface_drawer,
            edge_drawer,
            edge_vertex_buffer,
            edge_num_vertices,
            grid_renderer,
            axis_renderer,
            cone_pipeline,
            axis_cone_buffer,
            axis_cone_index_buffer,
            axis_cone_num_indices,
            selected_id: None,
            show_back_edges: true,
            mouse_pressed: [false; 3],
            mouse_pos: PhysicalPosition { x: 0.0, y: 0.0 },
            last_generation: 0,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(s) => s,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                let _ = self.viewer_tx.send(ViewerToRepl::ViewerClosed);
                self.running.store(false, Ordering::SeqCst);
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                state.size = new_size;
                state.surface_config.width = new_size.width.max(1);
                state.surface_config.height = new_size.height.max(1);
                state.surface.configure(&state.device, &state.surface_config);
                let (tex, view) = create_depth_texture(&state.device, new_size);
                state.depth_texture = tex;
                state.depth_texture_view = view;
            }
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        logical_key: key,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match key {
                Key::Named(NamedKey::Escape) => {
                    let _ = self.viewer_tx.send(ViewerToRepl::ViewerClosed);
                    self.running.store(false, Ordering::SeqCst);
                    event_loop.exit();
                }
                Key::Character(c) if c == "p" || c == "P" || c == "o" || c == "O" => {
                    state.camera.toggle_projection();
                }
                Key::Character(c) if c == "x" || c == "X" => {
                    state.show_back_edges = !state.show_back_edges;
                }
                _ => {}
            },
            WindowEvent::MouseInput {
                button,
                state: button_state,
                ..
            } => {
                let idx = match button {
                    MouseButton::Left => 0,
                    MouseButton::Middle => 1,
                    MouseButton::Right => 2,
                    _ => return,
                };
                let pressed = button_state == ElementState::Pressed;
                state.mouse_pressed[idx] = pressed;

                if pressed && button == MouseButton::Left {
                    Self::handle_click(state, &self.viewer_tx);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let dx = position.x - state.mouse_pos.x;
                let dy = position.y - state.mouse_pos.y;
                state.mouse_pos = position;

                if state.mouse_pressed[0] {
                    state.camera.rotate(dx, dy);
                }
                if state.mouse_pressed[1] {
                    state.camera.pan(dx, dy);
                }
                if state.mouse_pressed[2] {
                    state.camera.zoom(dy * 0.005);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let zoom_factor = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y as f64,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f64 * 0.01,
                };
                state.camera.zoom(zoom_factor * 0.1);
            }
            WindowEvent::RedrawRequested => {
                Self::render(state);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &mut self.state {
            state.window.request_redraw();
        }
    }
}

// ── Click handling ────────────────────────────────────────────────────────

impl ViewerApp {
    fn handle_click(state: &mut ViewerState, viewer_tx: &Sender<ViewerToRepl>) {
        let aspect = state.size.width as f64 / state.size.height.max(1) as f64;
        let view_proj = state.camera.matrix(aspect);
        let inv_view_proj = view_proj.inverse();

        // Normalize mouse position to NDC [-1, 1]
        let ndc_x = (state.mouse_pos.x / state.size.width as f64) * 2.0 - 1.0;
        let ndc_y = 1.0 - (state.mouse_pos.y / state.size.height as f64) * 2.0;

        let near_h = inv_view_proj * DVec4::new(ndc_x, ndc_y, -1.0, 1.0);
        let far_h = inv_view_proj * DVec4::new(ndc_x, ndc_y, 1.0, 1.0);
        let near = near_h.truncate() / near_h.w;
        let far = far_h.truncate() / far_h.w;

        let origin = near;
        let dir = (far - near).normalize();


        // Collect visible mesh data for picking
        let registry = global_shape_registry();
        let visible = registry.visible_shapes();
        let mesh_refs: Vec<(u64, &MeshData)> = visible
            .iter()
            .filter_map(|e| e.mesh.as_ref().map(|m| (e.shape_id, m)))
            .collect();

        if let Some(result) = pick_shape(origin, dir, &mesh_refs) {
            state.selected_id = Some(result.shape_id);
            let _ = viewer_tx.send(ViewerToRepl::ShapeSelected(result.shape_id));
            LAST_SELECTION.store(result.shape_id, std::sync::atomic::Ordering::SeqCst);
        } else {
            let was_selected = state.selected_id.is_some();
            state.selected_id = None;
            if was_selected {
                let _ = viewer_tx.send(ViewerToRepl::ShapeDeselected);
                LAST_SELECTION.store(u64::MAX, std::sync::atomic::Ordering::SeqCst);
            }
        }
    }

    fn render(state: &mut ViewerState) {
        // Update camera uniforms
        let aspect = state.size.width as f64 / state.size.height.max(1) as f64;
        let view_proj = state.camera.matrix(aspect);
        let uniforms = ViewUniforms {
            view_proj: view_proj.to_cols_array_2d().map(|r| r.map(|v| v as f32)),
        };
        state.surface_drawer.update_uniforms(&state.queue, &uniforms);
        state.edge_drawer.update_uniforms(&state.queue, &uniforms);

        // Rebuild GPU data only if registry has changed (dirty tracking)
        let current_gen = REGISTRY_GENERATION.load(std::sync::atomic::Ordering::Relaxed);
        if current_gen != state.last_generation {
            state.last_generation = current_gen;

            let registry = global_shape_registry();
            let visible = registry.visible_shapes();

            // Rebuild surface meshes
            let meshes: Vec<CadMesh> = visible
                .iter()
                .filter_map(|entry| {
                    entry
                        .mesh
                        .as_ref()
                        .map(|m| CadMesh::new(&state.device, m, entry.shape_id))
                })
                .collect();
            state.surface_drawer.set_meshes(meshes);

            // Rebuild edge vertex buffer from shape edge polylines
            let mut edge_segments: Vec<[[f32; 3]; 2]> = Vec::new();
            for entry in &visible {
                for polyline in &entry.edge_polylines {
                    for pair in polyline.windows(2) {
                        edge_segments.push([
                            [pair[0][0] as f32, pair[0][1] as f32, pair[0][2] as f32],
                            [pair[1][0] as f32, pair[1][1] as f32, pair[1][2] as f32],
                        ]);
                    }
                }
            }
            let edge_verts = segments_to_vertices(&edge_segments);
            if !edge_verts.is_empty() {
                state.edge_vertex_buffer =
                    state
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("edge_lines"),
                            contents: bytemuck::cast_slice(&edge_verts),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
            }
            state.edge_num_vertices = edge_verts.len() as u32;
        }

        let frame = match state.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => return,
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.15,
                            g: 0.15,
                            b: 0.15,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &state.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Grid and axes (no depth test, overlay style)
            pass.set_bind_group(0, &state.edge_drawer.uniform_bind_group, &[]);
            pass.set_pipeline(&state.edge_drawer.solid_pipeline);
            state.grid_renderer.render(&mut pass);
            state.axis_renderer.render(&mut pass);

            // Axis cone tips (solid meshes)
            if let (Some(cone_pipeline), Some(cone_vb), Some(cone_ib)) = (
                &state.cone_pipeline,
                &state.axis_cone_buffer,
                &state.axis_cone_index_buffer,
            ) {
                if state.axis_cone_num_indices > 0 {
                    pass.set_pipeline(cone_pipeline);
                    pass.set_vertex_buffer(0, cone_vb.slice(..));
                    pass.set_index_buffer(cone_ib.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..state.axis_cone_num_indices, 0, 0..1);
                }
            }

            // Shape edges (with depth testing, two-pass for back edges)
            if state.edge_num_vertices > 0 {
                state
                    .edge_drawer
                    .render(&mut pass, &state.edge_vertex_buffer, state.edge_num_vertices, state.show_back_edges);
            }

            // Mesh surfaces
            state.surface_drawer.render(&mut pass, state.selected_id);
        }

        state.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
