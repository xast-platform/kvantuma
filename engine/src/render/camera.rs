use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use crate::Transform;

#[derive(Clone, Debug, PartialEq)]
pub enum CameraType {
    LookAt,
    FirstPerson,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProjectionType {
    Perspective,
    Orthographic,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Camera {
    ty: CameraType,
    proj: ProjectionType,
    aspect: f32,
    fovy: f32,
    near: f32,
    far: f32,
}

impl Camera {
    pub fn new(aspect: f32, ty: CameraType, proj: ProjectionType) -> Camera {
        Camera {
            ty,
            proj,
            aspect,
            fovy: 45.0,
            near: 0.1,
            far: 100.0,
        }
    }

    pub fn build_view_projection(&self, transform: &Transform) -> Mat4 {
        let view = match self.ty {
            CameraType::FirstPerson => {
                Mat4::from_quat(transform.rotation.conjugate()) *
                Mat4::from_translation(-transform.translation)
            }

            CameraType::LookAt => {
                let eye = transform.translation;
                let target = Vec3::ZERO;
                Mat4::look_at_rh(eye, target, Vec3::Y)
            }
        };

        let projection = match self.proj {
            ProjectionType::Perspective => {
                Mat4::perspective_rh(
                    self.fovy.to_radians(),
                    self.aspect,
                    self.near,
                    self.far,
                )
            }
            ProjectionType::Orthographic => {
                let h = (self.far - self.near) / 2.0;
                let w = h * self.aspect;

                Mat4::orthographic_rh(
                    -w, w,
                    -h, h,
                    self.near,
                    self.far,
                )
            }
        };

        projection * view
    }

    /// Sets the aspect ratio of the camera's view.
    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}

/// Uniform data structure for the camera, used for passing camera information to the GPU.
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct CameraUniform {
    position: Vec3,
    _padding: u32,
    view_projection: Mat4,
}

impl Default for CameraUniform {
    fn default() -> Self {
        CameraUniform {
            position: Vec3::ZERO,
            view_projection: Mat4::IDENTITY,
            _padding: 0,
        }
    }
}

impl CameraUniform {
    pub fn new(camera: &Camera, transform: &Transform) -> CameraUniform {
        CameraUniform {
            position: transform.translation,
            view_projection: camera.build_view_projection(transform),
            _padding: 0,
        }
    }
}