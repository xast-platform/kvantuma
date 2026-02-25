pub mod app;
pub mod ecs;
pub mod render;
pub mod physics;
pub mod ui;
pub mod error;
pub mod utils;

pub use glam;

use crate::render::InstanceData;

pub struct Transform {
    pub translation: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            translation: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }
}

component! { POD: Transform }

impl Transform {
    pub fn to_matrix(&self) -> glam::Mat4 {
        let translation = glam::Mat4::from_translation(self.translation);
        let rotation = glam::Mat4::from_quat(self.rotation);
        let scale = glam::Mat4::from_scale(self.scale);
        glam::Mat4::IDENTITY * translation * rotation * scale
    }
}

impl InstanceData for Transform {
    type UniformData = glam::Mat4;

    fn uniform_data(&self) -> Self::UniformData {
        self.to_matrix()
    }
}