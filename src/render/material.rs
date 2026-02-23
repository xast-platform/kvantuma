use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};
use image::ImageError;
use wgpu::include_wgsl;

use crate::render::RenderDevice;
use crate::render::buffer::{BufferHandle, BufferResourceDescriptor};
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
}

#[derive(Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
pub struct Vertex {
    pub position: Vec3,
    pub texcoord: Vec2,
}

impl Vertex {
    const ATTRIBS: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x2,
    ];

    pub fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBS,
        }
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
        include_wgsl!("../../assets/shaders/basic.wgsl")
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
                &TintedTextureMaterial::shader_resource_layout(render_device),
            )
    }
}