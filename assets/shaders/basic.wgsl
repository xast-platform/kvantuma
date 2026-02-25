@group(0) @binding(0) var m_texture: texture_2d<f32>;
@group(0) @binding(1) var m_sampler: sampler;
@group(0) @binding(2) var<uniform> tint: vec3<f32>;

var<push_constant> transform: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) texcoord: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texcoord: vec2<f32>,
    @location(1) frag_pos: vec3<f32>,
};

@vertex
fn vertex(
    input: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.texcoord = input.texcoord;
    out.frag_pos = input.position;
    out.clip_position = transform * vec4<f32>(input.position, 1.0);

    return out;
}

@fragment
fn fragment(output: VertexOutput) -> @location(0) vec4<f32> {
    // let color = textureSample(m_texture, m_sampler, output.texcoord);
    // return vec4<f32>(color.rgb * tint, color.a);
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}