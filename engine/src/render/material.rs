use flecs_ecs::macros::Component;
use glam::Vec3;
use image::ImageError;
use wgpu::include_wgsl;

use crate::render::RenderDevice;
use crate::render::buffer::{BufferHandle, BufferResourceDescriptor};
use crate::render::mesh::Vertex;
use crate::render::texture::{Texture, TextureDescriptor, TextureResourceDescriptor, TextureResourceUsage};

use super::{shader_resource::{ShaderResource, ShaderResourceLayout}, registry::RenderRegistry, texture::TextureHandle};
use super::types::*;

pub trait Material {
    fn shader() -> ShaderModuleDescriptor<'static>;

    fn vertex_layout() -> Option<VertexBufferLayout<'static>>;

    fn shader_resource_layout(render_device: &RenderDevice) -> ShaderResourceLayout;

    fn shader_resource(
        &self, 
        render_device: &RenderDevice,
        registry: &RenderRegistry,
    ) -> ShaderResource;

    fn blend_state() -> BlendState {
        BlendState::ALPHA_BLENDING
    }

    fn depth_write_enabled() -> bool {
        true
    }

    fn depth_compare() -> CompareFunction {
        CompareFunction::Less
    }

    fn front_face() -> FrontFace {
        FrontFace::Ccw
    }

    fn cull_mode() -> Option<Face> {
        Some(Face::Back)
    }
}

#[derive(Debug)]
pub struct TintedTextureMaterial {
    pub albedo: TextureHandle,
    pub tint: Vec3,
    pub tint_buffer: BufferHandle,
}

impl TintedTextureMaterial {
    pub fn new(
        path: &str, 
        tint: Vec3,
        render_device: &RenderDevice,
        registry: &mut RenderRegistry,
    ) -> Result<TintedTextureMaterial, ImageError> {
        let albedo = registry
            .load_texture(render_device, path, TextureDescriptor::default())?;

        let tint_buffer = registry
            .new_buffer::<Vec3>(render_device, 1, BufferUsages::UNIFORM)
            .and_then_mut(registry, |b| b.fill(render_device, 0, &[tint]));

        Ok(TintedTextureMaterial {
            albedo,
            tint,
            tint_buffer,
        })
    }

    pub fn update_tint(
        &mut self,
        new_tint: Vec3,
        render_device: &RenderDevice,
        registry: &mut RenderRegistry,
    ) {
        self.tint = new_tint;
        registry
            .get_buffer(self.tint_buffer)
            .expect("Cannot update tint buffer")
            .fill_exact(render_device, 0, &[new_tint])
            .expect("Failed to update tint buffer");
    }
}

impl Material for TintedTextureMaterial {
    fn shader() -> ShaderModuleDescriptor<'static> {
        include_wgsl!("../../../assets/shaders/basic.wgsl")
    }

    fn vertex_layout() -> Option<VertexBufferLayout<'static>> {
        Some(Vertex::vertex_buffer_layout())
    }

    fn shader_resource_layout(render_device: &RenderDevice) -> ShaderResourceLayout {
        ShaderResourceLayout::builder()
            .with_label("Tinted Texture Material")
            .with_texture(&TextureResourceDescriptor {
                usage: TextureResourceUsage::TEXTURE | TextureResourceUsage::SAMPLER,
                sample_type: Some(TextureSampleType::Float { filterable: true }),
                sampler_binding_type: Some(SamplerBindingType::Filtering),
                dimension: TextureDimension::D2,
                view_dimension: TextureViewDimension::D2,
                format: Texture::DEFAULT_FORMAT,
            })
            .with_buffer(&BufferResourceDescriptor {
                visibility: ShaderStages::FRAGMENT,
                buffer_type: BufferBindingType::Uniform,
            })
            .build(render_device)
    }

    fn shader_resource(
        &self, 
        render_device: &RenderDevice,
        registry: &RenderRegistry,
    ) -> ShaderResource {
        ShaderResource::builder()
            .with_texture(
                registry.get_texture(self.albedo).unwrap(),
                TextureResourceUsage::TEXTURE | TextureResourceUsage::SAMPLER,
            )
            .with_buffer(registry.get_buffer(self.tint_buffer).unwrap())
            .build(
                render_device, 
                &Self::shader_resource_layout(render_device),
            )
    }
}

#[derive(Debug, Clone, Component)]
pub struct SkyboxMaterial {
    pub cubemap: TextureHandle,
}

impl SkyboxMaterial {
    pub fn new(cubemap: TextureHandle) -> Self {
        Self { cubemap }
    }
}

impl Material for SkyboxMaterial {
    fn shader() -> ShaderModuleDescriptor<'static> {
        include_wgsl!("../../../assets/shaders/skybox.wgsl")
    }

    fn vertex_layout() -> Option<VertexBufferLayout<'static>> {
        Some(Vertex::vertex_buffer_layout())
    }

    fn shader_resource_layout(render_device: &RenderDevice) -> ShaderResourceLayout {
        ShaderResourceLayout::builder()
            .with_label("Skybox Material")
            .with_texture(&TextureResourceDescriptor {
                usage: TextureResourceUsage::TEXTURE | TextureResourceUsage::SAMPLER,
                sample_type: Some(TextureSampleType::Float { filterable: true }),
                sampler_binding_type: Some(SamplerBindingType::Filtering),
                dimension: TextureDimension::D2,
                view_dimension: TextureViewDimension::Cube,
                format: Texture::DEFAULT_FORMAT,
            })
            .build(render_device)
    }

    fn shader_resource(
        &self,
        render_device: &RenderDevice,
        registry: &RenderRegistry,
    ) -> ShaderResource {
        ShaderResource::builder()
            .with_texture(
                registry.get_texture(self.cubemap).unwrap(),
                TextureResourceUsage::TEXTURE | TextureResourceUsage::SAMPLER,
            )
            .build(
                render_device,
                &Self::shader_resource_layout(render_device),
            )
    }

    fn depth_write_enabled() -> bool {
        false
    }

    fn depth_compare() -> CompareFunction {
        CompareFunction::LessEqual
    }

    fn cull_mode() -> Option<Face> {
        Some(Face::Front)
    }
}

#[derive(Debug, Component)]
pub struct ColorMaterial {
    pub color: Vec3,
    pub color_buffer: BufferHandle,
}

impl ColorMaterial {
    pub fn new(
        color: Vec3,
        render_device: &RenderDevice,
        registry: &mut RenderRegistry,
    ) -> ColorMaterial {
        let color_buffer = registry
            .new_buffer::<Vec3>(render_device, 1, BufferUsages::UNIFORM)
            .and_then_mut(registry, |b| b.fill(render_device, 0, &[color]));

        ColorMaterial {
            color,
            color_buffer,
        }
    }

    pub fn update_color(
        &mut self,
        new_color: Vec3,
        render_device: &RenderDevice,
        registry: &mut RenderRegistry,
    ) {
        self.color = new_color;
        registry
            .get_buffer(self.color_buffer)
            .expect("Cannot update color buffer")
            .fill_exact(render_device, 0, &[new_color])
            .expect("Failed to update color buffer");
    }
}

impl Material for ColorMaterial {
    fn shader() -> ShaderModuleDescriptor<'static> {
        include_wgsl!("../../../assets/shaders/color.wgsl")
    }

    fn vertex_layout() -> Option<VertexBufferLayout<'static>> {
        Some(Vertex::vertex_buffer_layout())
    }

    fn shader_resource_layout(render_device: &RenderDevice) -> ShaderResourceLayout {
        ShaderResourceLayout::builder()
            .with_label("Color Material")
            .with_buffer(&BufferResourceDescriptor {
                visibility: ShaderStages::FRAGMENT,
                buffer_type: BufferBindingType::Uniform,
            })
            .build(render_device)
    }

    fn shader_resource(
        &self, 
        render_device: &RenderDevice,
        registry: &RenderRegistry,
    ) -> ShaderResource {
        ShaderResource::builder()
            .with_buffer(registry.get_buffer(self.color_buffer).unwrap())
            .build(
                render_device, 
                &Self::shader_resource_layout(render_device),
            )
    }
}