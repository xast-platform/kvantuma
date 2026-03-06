use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use crate::{
    Transform, 
    render::{
        RenderDevice, 
        buffer::{BufferHandle, BufferResourceDescriptor}, 
        registry::RenderRegistry, 
        shader_resource::{ShaderResource, ShaderResourceLayout},
        types::*,
    }
};

pub struct CameraBuffer {
    handle: BufferHandle,
    resource: ShaderResource,
}

impl CameraBuffer {
    pub fn new(
        render_device: &RenderDevice, 
        registry: &mut RenderRegistry,
        layout: &ShaderResourceLayout,
    ) -> CameraBuffer {
        let handle = registry.new_buffer::<CameraUniform>(
            render_device, 
            1, 
            BufferUsages::UNIFORM,
        );
        let buffer = registry.get_buffer(handle)
            .unwrap();
        let resource = ShaderResource::builder()
            .with_buffer(&buffer)
            .build(render_device, layout);

        CameraBuffer {
            handle,
            resource,
        }
    }

    pub fn handle(&self) -> BufferHandle {
        self.handle
    }
    
    pub fn layout(render_device: &RenderDevice) -> ShaderResourceLayout {
        ShaderResourceLayout::builder()
            .with_label("Camera Buffer")
            .with_buffer(&BufferResourceDescriptor {
                visibility: ShaderStages::VERTEX_FRAGMENT,
                buffer_type: BufferBindingType::Uniform,
            })
            .build(render_device)
    }
    
    pub fn resource(&self) -> &ShaderResource {
        &self.resource
    }
}

#[derive(Clone, Debug)]
pub struct Camera {
    pub near: f32,
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            near: 0.1,
            far: 1000.0,
        }
    }
}

impl Camera {
    pub fn view_matrix(transform: &Transform) -> Mat4 {
        Mat4::from_quat(transform.rotation.conjugate())
            * Mat4::from_translation(-transform.translation)
    }
}

#[derive(Clone, Debug)]
pub struct PerspectiveCamera {
    pub fovy: f32,
    pub aspect: f32,
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        Self {
            fovy: 45.0f32.to_radians(),
            aspect: 16.0 / 9.0,
        }
    }
}

impl PerspectiveCamera {
    pub fn new(fovy_deg: f32, aspect: f32) -> Self {
        Self {
            fovy: fovy_deg.to_radians(),
            aspect,
        }
    }

    pub fn from_aspect(aspect: f32) -> Self {
        Self {
            aspect,
            ..Default::default()
        }
    }

    pub fn projection_matrix(&self, near: f32, far: f32) -> Mat4 {
        Mat4::perspective_rh(self.fovy, self.aspect, near, far)
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}

#[derive(Clone, Debug)]
pub struct OrthographicCamera {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
}

impl OrthographicCamera {
    pub fn new(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
    ) -> Self {
        Self {
            left,
            right,
            bottom,
            top,
        }
    }

    /// Create camera in pixel-perfect screen space (0..width, height..0)
    pub fn from_viewport(width: f32, height: f32) -> Self {
        Self {
            left: 0.0,
            right: width,
            bottom: 0.0,
            top: height,
        }
    }

    /// Create camera centered around origin with given size
    pub fn from_size(width: f32, height: f32) -> Self {
        let half_w = width * 0.5;
        let half_h = height * 0.5;

        Self {
            left: -half_w,
            right: half_w,
            bottom: -half_h,
            top: half_h,
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        *self = OrthographicCamera::from_size(width, height);
    }

    pub fn resize_viewport(&mut self, width: f32, height: f32) {
        *self = OrthographicCamera::from_viewport(width, height);
    }

    pub fn projection_matrix(&self, near: f32, far: f32) -> Mat4 {
        Mat4::orthographic_rh(
            self.left,
            self.right,
            self.bottom,
            self.top,
            near,
            far,
        )
    }
}

pub fn build_perspective_uniform(
    camera: &Camera,
    projection: &PerspectiveCamera,
    transform: &Transform,
) -> CameraUniform {
    let view = Camera::view_matrix(transform);
    let proj = projection.projection_matrix(camera.near, camera.far);
    let view_proj = proj * view;

    CameraUniform {
        position: transform.translation,
        view_proj,
        _padding: 0,
    }
}

pub fn build_orthographic_uniform(
    camera: &Camera,
    projection: &OrthographicCamera,
    transform: &Transform,
) -> CameraUniform {
    let view = Camera::view_matrix(transform);
    let proj = projection.projection_matrix(camera.near, camera.far);
    let view_proj = proj * view;

    CameraUniform {
        position: transform.translation,
        view_proj,
        _padding: 0,
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct CameraUniform {
    pub position: Vec3,
    pub _padding: u32,
    pub view_proj: Mat4,
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            view_proj: Mat4::IDENTITY,
            _padding: 0,
        }
    }
}