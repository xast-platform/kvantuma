// Camera
@group(1) @binding(0) var<uniform> cam_uniform: CameraUniform;

var<push_constant> transform: mat4x4<f32>;

struct CameraUniform {
    position: vec3<f32>,
    view_proj: mat4x4<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vertex(
    input: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.color = input.color;
    out.clip_position = cam_uniform.view_proj * transform * vec4<f32>(input.position, 1.0);

    return out;
}

@fragment
fn fragment(output: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(output.color, 1.0);
}
