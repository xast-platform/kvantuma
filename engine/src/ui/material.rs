use crate::{
    render::{
        RenderDevice, buffer::{BufferHandle, BufferResourceDescriptor}, include_wgsl, material::Material, registry::RenderRegistry, shader_resource::{ShaderResource, ShaderResourceLayout}, texture::{TextureHandle, TextureResourceDescriptor, TextureResourceUsage}, types::*
    }, 
    ui::atlas::GlyphVertex,
};

use flecs_ecs::prelude::*;
use glam::Vec3;

#[derive(Clone, Component)]
pub struct TextMaterial {
    color: Vec3,
    color_buffer: BufferHandle,
    atlas: TextureHandle,
}

impl TextMaterial {
    pub fn new(
        color: Vec3,
        atlas: TextureHandle,
        render_device: &RenderDevice,
        registry: &mut RenderRegistry,
    ) -> TextMaterial {
        let color_buffer = registry
            .new_buffer::<Vec3>(render_device, 1, BufferUsages::UNIFORM)
            .and_then_mut(registry, |b| b.fill(render_device, 0, &[color]));

        TextMaterial { 
            color, 
            color_buffer, 
            atlas,
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

impl Material for TextMaterial {
    fn shader() -> ShaderModuleDescriptor<'static> {
        include_wgsl!("../../../assets/shaders/text.wgsl")
    }

    fn vertex_layout() -> Option<VertexBufferLayout<'static>> {
        Some(GlyphVertex::vertex_buffer_layout())
    }

    fn shader_resource_layout(render_device: &RenderDevice) -> ShaderResourceLayout {
        ShaderResourceLayout::builder()
            .with_label("Text Material")
            .with_texture(&TextureResourceDescriptor {
                usage: TextureResourceUsage::TEXTURE | TextureResourceUsage::SAMPLER,
                sample_type: Some(TextureSampleType::Float { filterable: true }),
                sampler_binding_type: Some(SamplerBindingType::Filtering),
                dimension: TextureDimension::D2,
                view_dimension: TextureViewDimension::D2,
                format: TextureFormat::R8Unorm,
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
                registry.get_texture(self.atlas).unwrap(),
                TextureResourceUsage::TEXTURE | TextureResourceUsage::SAMPLER,
            )
            .with_buffer(registry.get_buffer(self.color_buffer).unwrap())
            .build(
                render_device,
                &TextMaterial::shader_resource_layout(render_device),
            )
    }
}