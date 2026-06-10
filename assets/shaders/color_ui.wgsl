// Material
@group(0) @binding(0) var<uniform> color: vec3<f32>;

// Camera
@group(1) @binding(0) var<uniform> cam_uniform: CameraUniform;

var<push_constant> transform: mat4x4<f32>;

struct CameraUniform {
    position: vec3<f32>,
    view_proj: mat4x4<f32>,
}

struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) frag_pos: vec3<f32>,
};

@vertex
fn vertex(
    input: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.frag_pos = vec3<f32>(input.position, 0.0);
    out.clip_position = cam_uniform.view_proj * transform * vec4<f32>(vec3<f32>(input.position, 0.0), 1.0);

    return out;
}

@fragment
fn fragment(output: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(color, 1.0);
}
