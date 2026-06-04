// ── Edge line shaders ────────────────────────────────────────────────────
// Renders edges as screen-space quads (instanced line rendering) for
// controllable thickness, or as plain LineList for grid/axes.
// Thickness and colors are controlled via uniform buffer (runtime-tunable).

struct Uniforms {
    view_proj: mat4x4<f32>,
    inactive_color: vec4<f32>,
    active_color: vec4<f32>,
    thickness: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

// ── Vertex input for grid/axes (plain positions) ───────────────────────────

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

// ── Instanced line vertex shader ─────────────────────────────────────────
// Expands each segment into a screen-space quad (TriangleStrip, 4 verts per
// instance) for controllable thickness independent of LineList limitations.

struct LineVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_line(
    @builtin(vertex_index) vertex_id: u32,
    @location(1) a: vec3<f32>,
    @location(2) b: vec3<f32>,
) -> LineVertexOutput {
    let corner = i32(vertex_id);
    let t = f32(corner / 2);
    let side = f32((corner % 2) * 2 - 1);

    let a_clip = uniforms.view_proj * vec4(a, 1.0);
    let b_clip = uniforms.view_proj * vec4(b, 1.0);

    let a_ndc = a_clip.xy / a_clip.w;
    let b_ndc = b_clip.xy / b_clip.w;
    let dir = normalize(b_ndc - a_ndc);
    let perp = vec2(-dir.y, dir.x);

    let pos = mix(a, b, t);
    let clip_pos = uniforms.view_proj * vec4(pos, 1.0);
    let ndc_pos = clip_pos.xy / clip_pos.w;
    let final_ndc = ndc_pos + perp * f32(side) * uniforms.thickness;

    var out: LineVertexOutput;
    out.clip_position = vec4(final_ndc * clip_pos.w, clip_pos.z, clip_pos.w);
    out.uv = vec2(t, f32(side));
    return out;
}

// ── Fragment shaders ───────────────────────────────────────────────────────
// All use uniform-controlled colors. Inactive = uniforms.inactive_color,
// Active = uniforms.active_color.

struct DashedInput {
    @builtin(position) clip_position: vec4<f32>,
};

@fragment
fn fs_inactive_solid() -> @location(0) vec4<f32> {
    return uniforms.inactive_color;
}

@fragment
fn fs_inactive_dashed(input: DashedInput) -> @location(0) vec4<f32> {
    let pattern = sin(input.clip_position.x * 20.0 + input.clip_position.y * 20.0);
    if pattern < 0.0 {
        discard;
    }
    return uniforms.inactive_color;
}

@fragment
fn fs_active_solid() -> @location(0) vec4<f32> {
    return uniforms.active_color;
}

@fragment
fn fs_active_dashed(input: DashedInput) -> @location(0) vec4<f32> {
    let pattern = sin(input.clip_position.x * 20.0 + input.clip_position.y * 20.0);
    if pattern < 0.0 {
        discard;
    }
    return uniforms.active_color;
}
