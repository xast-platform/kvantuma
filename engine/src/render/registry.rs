use std::{any::TypeId, collections::HashMap};

use ab_glyph::FontRef;
use bytemuck::Pod;
use image::ImageError;
use slotmap::SlotMap;

use crate::render::pipeline::RenderPipelineDescriptor;
use crate::render::texture::TextureDescriptor;
use crate::ui::atlas::{Atlas, FontData, FontHandle, atlas_size};

use super::types::*;
use super::{
    RenderDevice,
    buffer::{BufferHandle, BufferStorage},
    material::Material,
    pipeline::Pipeline,
    shader_resource::ShaderResourceLayout,
    texture::{Texture, TextureHandle},
};

#[derive(Default)]
pub struct RenderRegistry {
    pipelines: HashMap<TypeId, Pipeline>,
    buffers: SlotMap<BufferHandle, BufferStorage>,
    textures: SlotMap<TextureHandle, Texture>,
    fonts: SlotMap<FontHandle, FontData>,
}

impl RenderRegistry {
    pub fn new() -> RenderRegistry {
        RenderRegistry::default()
    }

    pub fn register_material<M: Material + 'static>(
        &mut self,
        render_device: &RenderDevice,
        global_layouts: &[&ShaderResourceLayout],
    ) {
        let material_layout = M::shader_resource_layout(render_device);
        let mut bindings = Vec::with_capacity(global_layouts.len() + 1);
        bindings.push(&material_layout);
        bindings.extend(global_layouts.iter().copied());

        self.pipelines
            .entry(TypeId::of::<M>())
            .or_insert(
                Pipeline::new_render(render_device, &RenderPipelineDescriptor {
                    shader: M::shader(),
                    bindings: &bindings,
                    label: &pretty_type_name::pretty_type_name::<M>(),
                    vertex_layout: M::vertex_layout(),
                    surface_formats: &[render_device.surface_format()],
                    blend_state: M::blend_state(),
                })
            );
    }

    pub fn get_pipeline<M: Material + 'static>(&self) -> Option<&Pipeline> {
        self.pipelines.get(&TypeId::of::<M>())
    }

    pub fn new_buffer<T: Pod>(
        &mut self,
        render_device: &RenderDevice,
        capacity: usize,
        usage: BufferUsages,
    ) -> BufferHandle {
        self.buffers
            .insert(BufferStorage::new::<T>(render_device, capacity, usage))
    }

    pub fn get_buffer(&self, handle: BufferHandle) -> Option<&BufferStorage> {
        self.buffers.get(handle)
    }

    pub fn get_buffer_mut(&mut self, handle: BufferHandle) -> Option<&mut BufferStorage> {
        self.buffers.get_mut(handle)
    }

    pub fn new_texture(
        &mut self,
        render_device: &RenderDevice,
        descriptor: TextureDescriptor,
    ) -> TextureHandle {
        self.textures
            .insert(Texture::new(render_device, descriptor))
    }

    pub fn load_texture(
        &mut self,
        render_device: &RenderDevice,
        path: &str,
        mut descriptor: TextureDescriptor,
    ) -> Result<TextureHandle, ImageError> {
        let image = image::open(path)?
            .to_rgba8();
        descriptor.width = image.width();
        descriptor.height = image.height();

        let texture = Texture::new(render_device, descriptor);
        texture.fill(render_device, &image);

        Ok(self.textures.insert(texture))
    }

    pub fn get_texture(&self, handle: TextureHandle) -> Option<&Texture> {
        self.textures.get(handle)
    }

    pub fn get_texture_mut(&mut self, handle: TextureHandle) -> Option<&mut Texture> {
        self.textures.get_mut(handle)
    }

    pub fn new_font(&mut self, font: FontRef<'static>) -> FontHandle {
        self.fonts.insert(FontData::new(font))
    }

    pub fn get_font(&self, handle: FontHandle) -> Option<&FontRef<'static>> {
        self.fonts.get(handle)
            .map(|d| &d.font)
    }

    pub fn add_font_atlas(&mut self, render_device: &RenderDevice, handle: FontHandle, font_size: u32) {
        if self.fonts.get(handle).is_some() {
            let Some(atlas_size) = atlas_size(font_size) else {
                log::error!("Invalid size {} for font atlas", font_size);
                return;
            };
            
            let texture = self.new_texture(render_device, TextureDescriptor {
                width: atlas_size as u32,
                height: atlas_size as u32,
                label: "Font atlas".to_owned(),
                filter: FilterMode::Nearest,
                format: TextureFormat::R8Unorm,
                ..Default::default()
            });

            let font_data = self.fonts.get_mut(handle).unwrap();
            font_data.atlases.register_atlas(font_size, &font_data.font, texture);

            let atlas = self.get_atlas(handle, font_size).unwrap();
            self.get_texture(texture)
                .unwrap()
                .fill(render_device, atlas.image());
        } else {
            log::error!("Font handle {:?} does not exist", handle);
        }
    }

    pub fn get_atlas(&self, font_handle: FontHandle, font_size: u32) -> Option<&Atlas> {
        self.fonts.get(font_handle)
            .and_then(|data| data.atlases.get_atlas(font_size))
    }
}
