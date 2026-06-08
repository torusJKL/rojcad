// ── Mesh vertex shader ───────────────────────────────────────────────────
// Transforms vertex positions and passes normal/uv to fragment stage.

struct Uniforms {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@group(1) @binding(0) var<uniform> mesh_color: vec4<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) frag_position: vec3<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * vec4<f32>(input.position, 1.0);
    out.world_normal = input.normal;
    out.frag_position = input.position;
    return out;
}

// ── Mesh fragment shader ─────────────────────────────────────────────────
// Lambertian diffuse shading with per-mesh color.

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let material_color = mesh_color.rgb;
    let light_dir = normalize(vec3<f32>(1.0, 2.0, 1.0));
    let ambient = 0.3;
    let normal = normalize(input.world_normal);
    let diffuse = max(dot(normal, light_dir), 0.0);
    let shading = ambient + (1.0 - ambient) * diffuse;
    return vec4<f32>(material_color * shading, 1.0);
}

// ── Highlight fragment shader ────────────────────────────────────────────
// Tints the selected shape blue.

@fragment
fn fs_highlight(input: VertexOutput) -> @location(0) vec4<f32> {
    let material_color = vec3<f32>(0.3, 0.5, 1.0);
    let light_dir = normalize(vec3<f32>(1.0, 2.0, 1.0));
    let ambient = 0.4;
    let normal = normalize(input.world_normal);
    let diffuse = max(dot(normal, light_dir), 0.0);
    let shading = ambient + (1.0 - ambient) * diffuse;
    return vec4<f32>(material_color * shading, 1.0);
}
