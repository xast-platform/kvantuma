struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) texcoord: vec2<f32>,
    @location(3) joint_indices: vec4<f32>,
    @location(4) joint_weights: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) texcoord: vec2<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    position: vec3<f32>,
}

struct ModelUniform {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

struct JointMatrices {
    matrices: array<mat4x4<f32>, 64>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> model: ModelUniform;

@group(1) @binding(1)
var<uniform> joint_matrices: JointMatrices;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    let joint0 = i32(in.joint_indices.x);
    let joint1 = i32(in.joint_indices.y);
    let joint2 = i32(in.joint_indices.z);
    let joint3 = i32(in.joint_indices.w);
    
    let skin_matrix = 
        joint_matrices.matrices[joint0] * in.joint_weights.x +
        joint_matrices.matrices[joint1] * in.joint_weights.y +
        joint_matrices.matrices[joint2] * in.joint_weights.z +
        joint_matrices.matrices[joint3] * in.joint_weights.w;
    
    let skinned_position = skin_matrix * vec4<f32>(in.position, 1.0);
    let skinned_normal = mat3x3<f32>(
        skin_matrix[0].xyz,
        skin_matrix[1].xyz,
        skin_matrix[2].xyz
    ) * in.normal;
    
    let world_position = model.model * skinned_position;
    out.world_position = world_position.xyz;
    out.world_normal = normalize((model.normal_matrix * vec4<f32>(skinned_normal, 0.0)).xyz);
    out.texcoord = in.texcoord;
    out.clip_position = camera.view_proj * world_position;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let diffuse = max(dot(in.world_normal, light_dir), 0.0);
    let ambient = 0.3;
    let lighting = ambient + diffuse * 0.7;
    
    return vec4<f32>(vec3<f32>(lighting), 1.0);
}
