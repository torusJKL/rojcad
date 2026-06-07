use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};

use glam::DVec3;

use wgpu::{self, util::DeviceExt};
#[cfg(target_os = "linux")]
use winit::platform::x11::EventLoopBuilderExtX11;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, ModifiersState, NamedKey},
    window::{Window, WindowId},
};

use crate::types::{
    ACTIVE_EDGE_COLOR, EDGE_THICKNESS, INACTIVE_EDGE_COLOR, LAST_SELECTION, LAST_SELECTION_ACTION,
    MeshData, PROJECTION_PERSPECTIVE, QUIT_REQUESTED, REGISTRY_GENERATION, ReplToViewer,
    SHOW_ACTIVE_EDGES, SHOW_BACK_EDGES, SHOW_HELP_OVERLAY, SHOW_INACTIVE_EDGES, SHOW_STATS_OVERLAY,
    ShapeId, global_shape_registry, unpack_color,
};

use super::camera::OrbitCamera;
use super::gizmo::GizmoRenderer;
use super::pick::pick_shape;
use super::help::Help;
use super::stats::Stats;

use super::ViewerToRepl;

const GIZMO_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const CLICK_THRESHOLD: f64 = 3.0;

// ── Uniform buffers ───────────────────────────────────────────────────────

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ViewUniforms {
    view_proj: [[f32; 4]; 4],
}

/// Edge drawer uniforms — view-proj matrix + runtime-tunable colors + thickness.
/// Must match `Uniforms` struct in edge.wgsl.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct EdgeUniforms {
    view_proj: [[f32; 4]; 4],
    inactive_color: [f32; 4],
    active_color: [f32; 4],
    thickness: f32,
    _pad: [f32; 3],
}

// ── CameraAnimation ─────────────────────────────────────────────────────────

const ANIMATION_DURATION: f64 = 0.5;

struct CameraAnimation {
    active: bool,
    start_yaw: f64,
    start_pitch: f64,
    target_yaw: f64,
    target_pitch: f64,
    elapsed: f64,
}

fn ease_in_out(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

impl CameraAnimation {
    fn new() -> Self {
        Self {
            active: false,
            start_yaw: 0.0,
            start_pitch: 0.0,
            target_yaw: 0.0,
            target_pitch: 0.0,
            elapsed: 0.0,
        }
    }

    fn start(&mut self, camera: &OrbitCamera, target_yaw: f64, target_pitch: f64) {
        self.active = true;
        self.start_yaw = camera.yaw;
        self.start_pitch = camera.pitch;
        self.target_yaw = target_yaw;
        self.target_pitch = target_pitch;
        self.elapsed = 0.0;
    }

    fn update(&mut self, camera: &mut OrbitCamera, dt: f64) {
        if !self.active {
            return;
        }
        self.elapsed += dt;
        let t = (self.elapsed / ANIMATION_DURATION).clamp(0.0, 1.0);
        let e = ease_in_out(t);
        camera.yaw = self.start_yaw + (self.target_yaw - self.start_yaw) * e;
        camera.pitch = self.start_pitch + (self.target_pitch - self.start_pitch) * e;
        if t >= 1.0 {
            camera.yaw = self.target_yaw;
            camera.pitch = self.target_pitch;
            self.active = false;
            camera.perspective = false;
        }
    }

    fn stop(&mut self) {
        self.active = false;
    }
}

// ── FitAnimation ─────────────────────────────────────────────────────────────

/// Smoothly animates camera target, radius, yaw, and pitch for fit-to-shape.
struct FitAnimation {
    start_target: DVec3,
    end_target: DVec3,
    start_radius: f64,
    end_radius: f64,
    start_yaw: f64,
    end_yaw: f64,
    start_pitch: f64,
    end_pitch: f64,
    elapsed: f64,
    duration: f64,
}

impl FitAnimation {
    fn new(
        start_target: DVec3,
        end_target: DVec3,
        start_radius: f64,
        end_radius: f64,
        start_yaw: f64,
        end_yaw: f64,
        start_pitch: f64,
        end_pitch: f64,
    ) -> Self {
        Self {
            start_target,
            end_target,
            start_radius,
            end_radius,
            start_yaw,
            end_yaw,
            start_pitch,
            end_pitch,
            elapsed: 0.0,
            duration: ANIMATION_DURATION,
        }
    }

    fn update(&mut self, camera: &mut OrbitCamera, dt: f64) -> bool {
        self.elapsed += dt;
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        let e = ease_in_out(t);

        camera.target = self.start_target.lerp(self.end_target, e);
        camera.radius = self.start_radius + (self.end_radius - self.start_radius) * e;
        camera.yaw = self.start_yaw + (self.end_yaw - self.start_yaw) * e;
        camera.pitch = self.start_pitch + (self.end_pitch - self.start_pitch) * e;

        t >= 1.0
    }
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

        let render_pipeline = Self::build_pipeline(
            device,
            &pipeline_layout,
            surface_format,
            depth_format,
            "fs_main",
        );
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
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "shader.wgsl"
            ))),
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
                cull_mode: None,
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

    pub fn render(&self, pass: &mut wgpu::RenderPass, selected_ids: &HashSet<ShapeId>) {
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        for mesh in &self.meshes {
            let is_selected = selected_ids.contains(&mesh.shape_id);
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

/// Renders edges as screen-space quads (instanced line rendering).
pub struct EdgeDrawer {
    /// Pipeline for grid/axes (plain LineList, vs_main + fs_inactive_solid)
    pub grid_pipeline: wgpu::RenderPipeline,
    /// Edge pipelines (TriangleStrip, vs_line + per-entry-point fragment)
    pub inactive_solid_pipeline: wgpu::RenderPipeline,
    inactive_dashed_pipeline: wgpu::RenderPipeline,
    active_solid_pipeline: wgpu::RenderPipeline,
    active_dashed_pipeline: wgpu::RenderPipeline,
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
            size: std::mem::size_of::<EdgeUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("edge_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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

        // Grid pipeline: plain LineList, vs_main, plain positions, no depth bias
        let grid_pipeline = Self::build_pipeline(
            device,
            &pipeline_layout,
            surface_format,
            depth_format,
            "vs_main",
            "fs_inactive_solid",
            wgpu::PrimitiveTopology::LineList,
            &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                }],
            }],
            wgpu::CompareFunction::Less,
            wgpu::DepthBiasState::default(),
        );

        // Shared instance vertex layout for edge pipelines (TriangleStrip)
        fn instance_layout() -> wgpu::VertexBufferLayout<'static> {
            const ATTRS: [wgpu::VertexAttribute; 2] = [
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ];
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<SegmentInstance>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &ATTRS,
            }
        }

        // Edge pipelines: TriangleStrip, vs_line, instance data, depth bias toward camera
        let edge_depth_bias = wgpu::DepthBiasState {
            constant: -4,
            slope_scale: -2.0,
            clamp: 0.0,
        };
        let inactive_solid_pipeline = Self::build_pipeline(
            device,
            &pipeline_layout,
            surface_format,
            depth_format,
            "vs_line",
            "fs_inactive_solid",
            wgpu::PrimitiveTopology::TriangleStrip,
            &[instance_layout()],
            wgpu::CompareFunction::Less,
            edge_depth_bias,
        );
        let inactive_dashed_pipeline = Self::build_pipeline(
            device,
            &pipeline_layout,
            surface_format,
            depth_format,
            "vs_line",
            "fs_inactive_dashed",
            wgpu::PrimitiveTopology::TriangleStrip,
            &[instance_layout()],
            wgpu::CompareFunction::Greater,
            edge_depth_bias,
        );
        let active_solid_pipeline = Self::build_pipeline(
            device,
            &pipeline_layout,
            surface_format,
            depth_format,
            "vs_line",
            "fs_active_solid",
            wgpu::PrimitiveTopology::TriangleStrip,
            &[instance_layout()],
            wgpu::CompareFunction::Less,
            edge_depth_bias,
        );
        let active_dashed_pipeline = Self::build_pipeline(
            device,
            &pipeline_layout,
            surface_format,
            depth_format,
            "vs_line",
            "fs_active_dashed",
            wgpu::PrimitiveTopology::TriangleStrip,
            &[instance_layout()],
            wgpu::CompareFunction::Greater,
            edge_depth_bias,
        );

        EdgeDrawer {
            grid_pipeline,
            inactive_solid_pipeline,
            inactive_dashed_pipeline,
            active_solid_pipeline,
            active_dashed_pipeline,
            uniform_bind_group,
            uniform_buffer,
        }
    }

    fn build_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        surface_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        vertex_entry: &str,
        fragment_entry: &str,
        topology: wgpu::PrimitiveTopology,
        vertex_buffers: &[wgpu::VertexBufferLayout<'_>],
        depth_compare: wgpu::CompareFunction,
        depth_bias: wgpu::DepthBiasState,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("edge shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("edge.wgsl"))),
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(fragment_entry),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some(vertex_entry),
                buffers: vertex_buffers,
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
                topology,
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
                bias: depth_bias,
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

    pub fn update_uniforms(&mut self, queue: &wgpu::Queue, view_proj: &ViewUniforms) {
        let inactive = unpack_color(INACTIVE_EDGE_COLOR.load(std::sync::atomic::Ordering::Relaxed));
        let active = unpack_color(ACTIVE_EDGE_COLOR.load(std::sync::atomic::Ordering::Relaxed));
        let thickness =
            f64::from_bits(EDGE_THICKNESS.load(std::sync::atomic::Ordering::Relaxed)) as f32;
        let edge_uniforms = EdgeUniforms {
            view_proj: view_proj.view_proj,
            inactive_color: [
                inactive[0] as f32,
                inactive[1] as f32,
                inactive[2] as f32,
                1.0,
            ],
            active_color: [active[0] as f32, active[1] as f32, active[2] as f32, 1.0],
            thickness,
            _pad: [0.0; 3],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&edge_uniforms));
    }

    pub fn render(
        &self,
        pass: &mut wgpu::RenderPass,
        inactive_buffer: &wgpu::Buffer,
        inactive_num_instances: u32,
        active_buffer: &wgpu::Buffer,
        active_num_instances: u32,
        show_dashed: bool,
    ) {
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);

        let show_inactive = SHOW_INACTIVE_EDGES.load(std::sync::atomic::Ordering::Relaxed);
        let show_active = SHOW_ACTIVE_EDGES.load(std::sync::atomic::Ordering::Relaxed);

        // Inactive edges (light grey) — 4 verts per instance (TriangleStrip quad)
        if show_inactive && inactive_num_instances > 0 {
            pass.set_vertex_buffer(0, inactive_buffer.slice(..));
            if show_dashed {
                pass.set_pipeline(&self.inactive_dashed_pipeline);
                pass.draw(0..4, 0..inactive_num_instances);
            }
            pass.set_pipeline(&self.inactive_solid_pipeline);
            pass.draw(0..4, 0..inactive_num_instances);
        }

        // Active edges (light blue, rendered on top)
        if show_active && active_num_instances > 0 {
            pass.set_vertex_buffer(0, active_buffer.slice(..));
            if show_dashed {
                pass.set_pipeline(&self.active_dashed_pipeline);
                pass.draw(0..4, 0..active_num_instances);
            }
            pass.set_pipeline(&self.active_solid_pipeline);
            pass.draw(0..4, 0..active_num_instances);
        }
    }
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
    surface_drawer: SurfaceDrawer,
    edge_drawer: EdgeDrawer,
    inactive_instance_buffer: wgpu::Buffer,
    inactive_num_instances: u32,
    active_instance_buffer: wgpu::Buffer,
    active_num_instances: u32,
    grid_renderer: GridRenderer,
    gizmo_renderer: GizmoRenderer,
    gizmo_depth: wgpu::Texture,
    gizmo_depth_view: wgpu::TextureView,
    gizmo_viewport_size: u32,
    gizmo_margin: u32,
    selected_ids: HashSet<ShapeId>,
    click_start_pos: PhysicalPosition<f64>,
    mouse_pressed: [bool; 3],
    mouse_pos: PhysicalPosition<f64>,
    last_generation: u64,
    animation: CameraAnimation,
    fit_animation: Option<FitAnimation>,
    last_time: std::time::Instant,
    keyboard_view: Option<usize>,
    modifiers: ModifiersState,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    stats: Stats,
    help: Help,
}

// ── ViewerApp ─────────────────────────────────────────────────────────────

struct ViewerApp {
    viewer_tx: Sender<ViewerToRepl>,
    repl_rx: Receiver<ReplToViewer>,
    running: Arc<AtomicBool>,
    state: Option<ViewerState>,
}

/// Main entry point for the viewer thread.
pub fn run_viewer(
    viewer_tx: Sender<ViewerToRepl>,
    repl_rx: Receiver<ReplToViewer>,
    running: Arc<AtomicBool>,
) {
    let mut builder = EventLoop::builder();
    #[cfg(target_os = "linux")]
    builder.with_any_thread(true);
    let event_loop = builder.build().expect("failed to create winit event loop");

    let mut app = ViewerApp {
        viewer_tx,
        repl_rx,
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
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .expect("failed to find adapter");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
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
        let mut gizmo_renderer = GizmoRenderer::new(&device, surface_format);

        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            event_loop,
            None,
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 1, false);

        // Initial empty instance buffers (populated from ShapeRegistry each frame)
        let inactive_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("inactive_edge_instances"),
            size: 1,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });
        let inactive_num_instances = 0u32;
        let active_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("active_edge_instances"),
            size: 1,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });
        let active_num_instances = 0u32;

        let sf = window.scale_factor();
        let gizmo_viewport_size = (200.0 * sf) as u32;
        let gizmo_margin = (14.0 * sf) as u32;
        gizmo_renderer.set_viewport_size(&device, gizmo_viewport_size);

        let gizmo_depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("gizmo_depth"),
            size: wgpu::Extent3d {
                width: size.width.max(1),
                height: size.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: GIZMO_DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let gizmo_depth_view = gizmo_depth.create_view(&wgpu::TextureViewDescriptor::default());

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
            surface_drawer,
            edge_drawer,
            inactive_instance_buffer,
            inactive_num_instances,
            active_instance_buffer,
            active_num_instances,
            grid_renderer,
            gizmo_renderer,
            gizmo_depth,
            gizmo_depth_view,
            gizmo_viewport_size,
            gizmo_margin,
            selected_ids: HashSet::new(),
            click_start_pos: PhysicalPosition { x: 0.0, y: 0.0 },
            mouse_pressed: [false; 3],
            mouse_pos: PhysicalPosition { x: 0.0, y: 0.0 },
            last_generation: 0,
            animation: CameraAnimation::new(),
            fit_animation: None,
            last_time: std::time::Instant::now(),
            keyboard_view: None,
            modifiers: ModifiersState::default(),
            egui_ctx,
            egui_state,
            egui_renderer,
            stats: Stats::new(),
            help: Help::new(),
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

        state.egui_state.on_window_event(&state.window, &event);

        match event {
            WindowEvent::CloseRequested => {
                QUIT_REQUESTED.store(true, Ordering::SeqCst);
                let _ = self.viewer_tx.send(ViewerToRepl::ViewerClosed);
                self.running.store(false, Ordering::SeqCst);
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                state.size = new_size;
                state.surface_config.width = new_size.width.max(1);
                state.surface_config.height = new_size.height.max(1);
                state
                    .surface
                    .configure(&state.device, &state.surface_config);
                let (tex, view) = create_depth_texture(&state.device, new_size);
                state.depth_texture = tex;
                state.depth_texture_view = view;
                let gizmo_depth = state.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("gizmo_depth"),
                    size: wgpu::Extent3d {
                        width: new_size.width.max(1),
                        height: new_size.height.max(1),
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: GIZMO_DEPTH_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                });
                state.gizmo_depth = gizmo_depth;
                state.gizmo_depth_view = state
                    .gizmo_depth
                    .create_view(&wgpu::TextureViewDescriptor::default());
            }
            WindowEvent::ModifiersChanged(m) => {
                state.modifiers = m.state();
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
                    if SHOW_HELP_OVERLAY.load(Ordering::SeqCst) {
                        SHOW_HELP_OVERLAY.store(false, Ordering::SeqCst);
                    }
                }
                Key::Character(c) if c == "p" || c == "P" || c == "o" || c == "O" => {
                    PROJECTION_PERSPECTIVE.fetch_xor(true, Ordering::SeqCst);
                }
                Key::Character(c) if c == "x" || c == "X" => {
                    SHOW_BACK_EDGES.fetch_xor(true, Ordering::SeqCst);
                }
                Key::Character(c) if state.modifiers.control_key() && c == "1" => {
                    let idx = state
                        .keyboard_view
                        .map_or(4, |v| if v == 4 { 5 } else { 4 });
                    let (yaw, pitch) = VIEW_TARGETS[idx];
                    state.animation.start(&state.camera, yaw, pitch);
                    state.keyboard_view = Some(idx);
                }
                Key::Character(c) if state.modifiers.control_key() && c == "7" => {
                    let idx = state
                        .keyboard_view
                        .map_or(2, |v| if v == 2 { 3 } else { 2 });
                    let (yaw, pitch) = VIEW_TARGETS[idx];
                    state.animation.start(&state.camera, yaw, pitch);
                    state.keyboard_view = Some(idx);
                }
                Key::Character(c) if state.modifiers.control_key() && c == "3" => {
                    let idx = state
                        .keyboard_view
                        .map_or(1, |v| if v == 1 { 0 } else { 1 });
                    let (yaw, pitch) = VIEW_TARGETS[idx];
                    state.animation.start(&state.camera, yaw, pitch);
                    state.keyboard_view = Some(idx);
                }
                Key::Character(c)
                    if state.modifiers.control_key()
                        && state.modifiers.shift_key()
                        && state.modifiers.alt_key()
                        && (c == "s" || c == "S") =>
                {
                    SHOW_STATS_OVERLAY.fetch_xor(true, Ordering::SeqCst);
                }
                Key::Character(c)
                    if !state.egui_ctx.wants_keyboard_input()
                        && (c == "h" || c == "H") =>
                {
                    SHOW_HELP_OVERLAY.fetch_xor(true, Ordering::SeqCst);
                }
                Key::Character(c) if state.modifiers.control_key() && (c == "q" || c == "Q") => {
                    QUIT_REQUESTED.store(true, Ordering::SeqCst);
                    let _ = self.viewer_tx.send(ViewerToRepl::ViewerClosed);
                    self.running.store(false, Ordering::SeqCst);
                    event_loop.exit();
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

                if button == MouseButton::Left {
                    if pressed {
                        state.click_start_pos = state.mouse_pos;
                    } else {
                        // Released — fire click only if not a drag
                        let dx = state.mouse_pos.x - state.click_start_pos.x;
                        let dy = state.mouse_pos.y - state.click_start_pos.y;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist < CLICK_THRESHOLD {
                            Self::handle_click(state, &self.viewer_tx);
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let dx = position.x - state.mouse_pos.x;
                let dy = position.y - state.mouse_pos.y;
                state.mouse_pos = position;

                if !state.egui_ctx.wants_pointer_input() {
                    if state.mouse_pressed[0] {
                        state.camera.rotate(dx, dy);
                        state.animation.stop();
                    }
                    if state.mouse_pressed[1] {
                        state.camera.pan(dx, dy);
                        state.animation.stop();
                    }
                    if state.mouse_pressed[2] {
                        state.camera.zoom(dy * 0.005);
                        state.animation.stop();
                    }
                }

                state.gizmo_renderer.set_hovered(&state.device, None);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if !state.egui_ctx.wants_pointer_input() {
                    let zoom_factor = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y as f64,
                        MouseScrollDelta::PixelDelta(pos) => pos.y * 0.01,
                    };
                    state.camera.zoom(zoom_factor * 0.1);
                }
            }
            WindowEvent::RedrawRequested => {
                Self::check_repl_commands(&self.repl_rx, state);
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

// ── Gizmo helpers ─────────────────────────────────────────────────────────

const VIEW_TARGETS: [(f64, f64); 6] = [
    (0.0, 0.0),                          // 0: +X (YZ plane from +X)
    (std::f64::consts::PI, 0.0),         // 1: -X (YZ plane from -X)
    (0.0, std::f64::consts::FRAC_PI_2),  // 2: +Y (XZ plane from above)
    (0.0, -std::f64::consts::FRAC_PI_2), // 3: -Y (XZ plane from below)
    (std::f64::consts::FRAC_PI_2, 0.0),  // 4: +Z (XY plane from +Z)
    (-std::f64::consts::FRAC_PI_2, 0.0), // 5: -Z (XY plane from -Z)
];

// ── Click handling ────────────────────────────────────────────────────────

impl ViewerApp {
    fn handle_click(state: &mut ViewerState, viewer_tx: &Sender<ViewerToRepl>) {
        // Shape picking from mouse click
        let aspect = state.size.width as f64 / state.size.height.max(1) as f64;
        let view_proj = state.camera.matrix(aspect);
        let inv_view_proj = view_proj.inverse();

        // Normalize mouse position to NDC [-1, 1]
        let ndc_x = (state.mouse_pos.x / state.size.width as f64) * 2.0 - 1.0;
        let ndc_y = 1.0 - (state.mouse_pos.y / state.size.height as f64) * 2.0;

        let near_h = inv_view_proj * glam::DVec4::new(ndc_x, ndc_y, -1.0, 1.0);
        let far_h = inv_view_proj * glam::DVec4::new(ndc_x, ndc_y, 1.0, 1.0);
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

        let ctrl = state.modifiers.control_key();
        let shift = state.modifiers.shift_key();

        if let Some(result) = pick_shape(origin, dir, &mesh_refs) {
            if ctrl {
                // Toggle clicked shape in/out of selection
                if state.selected_ids.contains(&result.shape_id) {
                    state.selected_ids.remove(&result.shape_id);
                    viewer_tx.send(ViewerToRepl::SelectionChanged).ok();
                    LAST_SELECTION.store(result.shape_id, Ordering::SeqCst);
                    LAST_SELECTION_ACTION.store(2, Ordering::SeqCst);
                } else {
                    state.selected_ids.insert(result.shape_id);
                    viewer_tx.send(ViewerToRepl::SelectionChanged).ok();
                    LAST_SELECTION.store(result.shape_id, Ordering::SeqCst);
                    LAST_SELECTION_ACTION.store(1, Ordering::SeqCst);
                }
            } else if shift {
                // Additive — insert if not already present
                if state.selected_ids.insert(result.shape_id) {
                    viewer_tx.send(ViewerToRepl::SelectionChanged).ok();
                    LAST_SELECTION.store(result.shape_id, Ordering::SeqCst);
                    LAST_SELECTION_ACTION.store(1, Ordering::SeqCst);
                }
            } else {
                // Plain click — replace selection
                state.selected_ids.clear();
                state.selected_ids.insert(result.shape_id);
                viewer_tx.send(ViewerToRepl::SelectionChanged).ok();
                LAST_SELECTION.store(result.shape_id, Ordering::SeqCst);
                LAST_SELECTION_ACTION.store(1, Ordering::SeqCst);
            }
        } else {
            // Miss — no shape under cursor
            if !ctrl && !shift {
                // Plain click on empty space: clear selection
                let was_selected = !state.selected_ids.is_empty();
                state.selected_ids.clear();
                if was_selected {
                    viewer_tx.send(ViewerToRepl::SelectionChanged).ok();
                    LAST_SELECTION.store(u64::MAX, Ordering::SeqCst);
                    LAST_SELECTION_ACTION.store(3, Ordering::SeqCst);
                }
            }
            // Ctrl/Shift click on empty space: no-op
        }
    }

    fn check_repl_commands(rx: &Receiver<ReplToViewer>, state: &mut ViewerState) {
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                ReplToViewer::FitToBounds {
                    center,
                    radius,
                    keep_angle,
                } => {
                    Self::fit_to_bounds(state, center, radius, keep_angle);
                }
            }
        }
    }

    fn fit_to_bounds(state: &mut ViewerState, center: DVec3, radius: f64, keep_angle: bool) {
        let aspect = state.size.width as f64 / state.size.height.max(1) as f64;
        let margin = 1.3;

        let target_radius = if state.camera.perspective {
            let tan_half_fov = (state.camera.fov_y * 0.5).tan();
            let d_height = radius / tan_half_fov;
            let d_width = radius / (tan_half_fov * aspect);
            d_height.max(d_width) * margin
        } else {
            let from_height = 2.0 * radius;
            let from_width = 2.0 * radius / aspect;
            from_height.max(from_width) * margin
        };

        let end_yaw = if keep_angle { state.camera.yaw } else { 0.0 };
        let end_pitch = if keep_angle { state.camera.pitch } else { 0.4 };

        state.fit_animation = Some(FitAnimation::new(
            state.camera.target,
            center,
            state.camera.radius,
            target_radius,
            state.camera.yaw,
            end_yaw,
            state.camera.pitch,
            end_pitch,
        ));
    }

    fn render(state: &mut ViewerState) {
        // Sync projection mode from atomic (controlled by Janet or keyboard)
        let target_perspective = PROJECTION_PERSPECTIVE.load(Ordering::Relaxed);
        if target_perspective != state.camera.perspective {
            state.camera.toggle_projection();
        }

        // Update camera animations
        let now = std::time::Instant::now();
        let dt = (now - state.last_time).as_secs_f64();
        state.last_time = now;
        state.animation.update(&mut state.camera, dt);
        if let Some(ref mut fit) = state.fit_animation
            && fit.update(&mut state.camera, dt)
        {
            state.fit_animation = None;
        }

        // Update camera uniforms
        let aspect = state.size.width as f64 / state.size.height.max(1) as f64;
        let view_proj = state.camera.matrix(aspect);
        let uniforms = ViewUniforms {
            view_proj: view_proj.to_cols_array_2d().map(|r| r.map(|v| v as f32)),
        };
        state
            .surface_drawer
            .update_uniforms(&state.queue, &uniforms);
        state.edge_drawer.update_uniforms(&state.queue, &uniforms);
        state
            .gizmo_renderer
            .update_uniforms(&state.queue, &state.device, &state.camera);

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

            // Build SegmentInstance arrays for instanced line rendering
            let selected_ids = &state.selected_ids;
            let mut inactive_instances: Vec<SegmentInstance> = Vec::new();
            let mut active_instances: Vec<SegmentInstance> = Vec::new();

            for entry in &visible {
                let is_active = selected_ids.contains(&entry.shape_id);
                let target = if is_active {
                    &mut active_instances
                } else {
                    &mut inactive_instances
                };
                for polyline in &entry.edge_polylines {
                    for pair in polyline.windows(2) {
                        target.push(SegmentInstance {
                            a: [pair[0][0] as f32, pair[0][1] as f32, pair[0][2] as f32],
                            b: [pair[1][0] as f32, pair[1][1] as f32, pair[1][2] as f32],
                            cum_length: 0.0,
                        });
                    }
                }
            }

            // Build inactive instance buffer
            if !inactive_instances.is_empty() {
                state.inactive_instance_buffer =
                    state
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("inactive_edge_instances"),
                            contents: bytemuck::cast_slice(&inactive_instances),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
            }
            state.inactive_num_instances = inactive_instances.len() as u32;

            // Build active instance buffer
            if !active_instances.is_empty() {
                state.active_instance_buffer =
                    state
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("active_edge_instances"),
                            contents: bytemuck::cast_slice(&active_instances),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
            }
            state.active_num_instances = active_instances.len() as u32;
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

        // Main scene render pass
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

            // Grid (no depth test, overlay style)
            pass.set_bind_group(0, &state.edge_drawer.uniform_bind_group, &[]);
            pass.set_pipeline(&state.edge_drawer.grid_pipeline);
            state.grid_renderer.render(&mut pass);

            // Mesh surfaces (depth test: Less, writes depth)
            state.surface_drawer.render(&mut pass, &state.selected_ids);

            // Shape edges (depth test: Less with negative bias toward camera,
            // rendered AFTER meshes so edges overlay mesh surfaces)
            state.edge_drawer.render(
                &mut pass,
                &state.inactive_instance_buffer,
                state.inactive_num_instances,
                &state.active_instance_buffer,
                state.active_num_instances,
                SHOW_BACK_EDGES.load(Ordering::Relaxed),
            );
        }

        // Gizmo overlay pass (no depth, alpha blending, top-right viewport)
        {
            let gs = state.gizmo_viewport_size;
            let gm = state.gizmo_margin;
            let gx = state.size.width.saturating_sub(gs + gm);
            let gy = gm;

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("gizmo pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &state.gizmo_depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_viewport(gx as f32, gy as f32, gs as f32, gs as f32, 0.0, 1.0);
            pass.set_scissor_rect(gx, gy, gs, gs);
            state.gizmo_renderer.render(&mut pass);
        }

        // Egui overlay pass
        {
            let raw_input = state.egui_state.take_egui_input(&state.window);
            let full_output = state.egui_ctx.run(raw_input, |ctx| {
                state.stats.ui(ctx, &state.camera, &state.selected_ids, dt);
                state.help.ui(ctx);
            });

            let pixels_per_point = state.window.scale_factor() as f32;
            let paint_jobs = state
                .egui_ctx
                .tessellate(full_output.shapes, pixels_per_point);

            for (id, delta) in &full_output.textures_delta.set {
                state
                    .egui_renderer
                    .update_texture(&state.device, &state.queue, *id, delta);
            }
            for id in &full_output.textures_delta.free {
                state.egui_renderer.free_texture(id);
            }

            let screen_descriptor = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [state.size.width, state.size.height],
                pixels_per_point,
            };

            let _ = state.egui_renderer.update_buffers(
                &state.device,
                &state.queue,
                &mut encoder,
                &paint_jobs,
                &screen_descriptor,
            );

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // egui-wgpu requires RenderPass<'static> but our pass borrows view
            // which is a local. The pass is consumed immediately before view drops,
            // so this is safe.
            #[allow(deprecated)]
            let pass_ref = unsafe {
                std::mem::transmute::<&mut wgpu::RenderPass<'_>, &mut wgpu::RenderPass<'static>>(
                    &mut pass,
                )
            };
            state
                .egui_renderer
                .render(pass_ref, &paint_jobs, &screen_descriptor);
        }

        state.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
