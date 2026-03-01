@group(0) @binding(0) var atlas: texture_2d<f32>;
@group(0) @binding(1) var atlas_sampler: sampler;

const text_color: vec3<f32> = vec3<f32>(1.0);

struct Out {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

var<push_constant> transform: mat4x4<f32>;

@vertex
fn vertex(@location(0) pos: vec2<f32>, @location(1) uv: vec2<f32>) -> Out {
    var out: Out;
    out.pos = transform * vec4<f32>(pos, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@fragment
fn fragment(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let a = textureSample(atlas, atlas_sampler, uv).r;
    return vec4<f32>(mix(vec3<f32>(0.0), text_color, a), 1.0);
}