use crate::{
    render::{
        RenderDevice, include_wgsl, 
        material::Material,
        registry::RenderRegistry, 
        shader_resource::{ShaderResource, ShaderResourceLayout}, 
        texture::{TextureHandle, TextureResourceDescriptor, TextureResourceUsage}, 
        types::*,
    }, 
    ui::
        atlas::GlyphVertex
    ,
};

use flecs_ecs::prelude::*;

#[derive(Clone, Component)]
pub struct TextMaterial {
    pub atlas: TextureHandle,
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
            .build(
                render_device,
                &TextMaterial::shader_resource_layout(render_device),
            )
    }
}