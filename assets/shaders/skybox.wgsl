// Material
@group(0) @binding(0) var skybox: texture_cube<f32>;
@group(0) @binding(1) var sky_sampler: sampler;

// Camera
@group(1) @binding(0) var<uniform> cam_uniform: CameraUniform;

struct CameraUniform {
    position: vec3<f32>,
    _padding: u32,
    view_proj: mat4x4<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) texcoord: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) dir: vec3<f32>,
}

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Keep skybox centered on the camera to remove parallax translation
    let world_pos = input.position + cam_uniform.position;

    out.clip_position = cam_uniform.view_proj * vec4<f32>(world_pos, 1.0);
    out.clip_position.z = out.clip_position.w;
    out.dir = normalize(input.position);

    return out;
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(skybox, sky_sampler, input.dir);
}
