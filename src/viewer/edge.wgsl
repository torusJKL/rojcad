// ── Edge line shaders ────────────────────────────────────────────────────
// Renders edges as thin lines using LineList topology.

struct Uniforms {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * vec4<f32>(input.position, 1.0);
    return out;
}

// ── Solid edges (front) ────────────────────────────────────────────────────

@fragment
fn fs_solid() -> @location(0) vec4<f32> {
    return vec4<f32>(0.05, 0.05, 0.05, 1.0);
}

// ── Dashed edges (back) ────────────────────────────────────────────────────

struct DashedInput {
    @builtin(position) clip_position: vec4<f32>,
};

@fragment
fn fs_dashed(input: DashedInput) -> @location(0) vec4<f32> {
    // Simple screen-space dashing based on clip-space position
    let dash_length = 0.04;
    let pattern = sin(input.clip_position.x * 20.0 + input.clip_position.y * 20.0);
    if pattern < 0.0 {
        discard;
    }
    return vec4<f32>(0.05, 0.05, 0.05, 1.0);
}
