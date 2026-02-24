//! Texture module contains types and utilities for managing textures,
//! including texture creation, resource descriptors, and usage flags.

use bitflags::bitflags;
use glam::UVec2;
use image::{ImageBuffer, Pixel};
use slotmap::new_key_type;
use crate::render::registry::RenderRegistry;

use super::types::*;
use super::{RenderSurface, RenderDevice};

new_key_type! {
    pub struct TextureHandle;
}

impl TextureHandle {
    pub fn and_then_mut<F>(
        self,
        registry: &mut RenderRegistry,
        mut f: F,
    ) -> TextureHandle
    where
        F: FnMut(&mut Texture),
    {
        if let Some(tex) = registry.get_texture_mut(self) {
            f(tex);
        }

        self
    }

    pub fn and_then<F, R>(
        self,
        registry: &RenderRegistry,
        mut f: F,
    ) -> TextureHandle
    where
        F: FnMut(&Texture) -> R,
    {
        if let Some(tex) = registry.get_texture(self) {
            f(tex);
        }

        self
    }
}

/// Describes a texture, including its size, format, usage, and filtering mode.
#[derive(Debug, Clone)]
pub struct TextureDescriptor {
    /// Width of the texture in pixels.
    pub width: u32,
    /// Height of the texture in pixels.
    pub height: u32,
    /// Optional depth value (for 3D textures).
    pub depth: Option<u32>,
    /// Filtering mode used for sampling the texture.
    pub filter: FilterMode,
    /// Dimensionality of the texture (1D, 2D, or 3D).
    pub dimension: TextureDimension,
    /// Usage flags specifying how the texture will be used.
    pub usage: TextureUsages,
    /// Format of the texture
    pub format: TextureFormat,
    /// Number of mip levels for the texture.
    pub mip_level_count: u32,
    /// A human-readable label for debugging purposes. Displayed, when
    /// error affiliated with the texture occures
    pub label: String,
}

impl Default for TextureDescriptor {
    fn default() -> Self {
        TextureDescriptor {
            width: 1,
            height: 1,
            depth: None,
            filter: wgpu::FilterMode::Linear,
            dimension: wgpu::TextureDimension::D2,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            format: Texture::DEFAULT_FORMAT,
            mip_level_count: 1,
            label: "Unnamed Texture".to_string(),
        }
    }
}

bitflags! {
    /// Flags indicating how a texture resource will be used in shaders.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TextureResourceUsage: u8 {
        /// The texture will be used as a sampled texture.
        const TEXTURE = 1;
        /// The texture will be used with a sampler.
        const SAMPLER = 1 << 1;
        /// The texture will be used as a storage texture.
        const STORAGE = 1 << 2;
    }
}

/// Describes a texture resource, defining how it will be accessed in a shader.
pub struct TextureResourceDescriptor {
    /// The intended usage of the texture resource.
    pub usage: TextureResourceUsage,
    /// The expected sample type when used as a sampled texture.
    pub sample_type: Option<TextureSampleType>,
    /// The type of sampler binding when used as a sampler.
    pub sampler_binding_type: Option<SamplerBindingType>,
    pub dimension: TextureDimension,
    pub format: TextureFormat,
}

/// A structure representing a GPU texture, including its view and sampler.
#[derive(Debug)]
pub struct Texture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    descriptor: TextureDescriptor,
}

impl RenderSurface for Texture {
    fn view(&self) -> &TextureView {
        if !self.descriptor.usage.contains(wgpu::TextureUsages::RENDER_ATTACHMENT) {
            panic!("Texture, used as render surface, must have RENDER_ATTACHMENT usage");
        }

        &self.view
    }
}

impl Texture {
    pub const DEFAULT_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

    /// Creates a new texture with the specified descriptor.
    pub fn new(
        render_device: &RenderDevice, 
        descriptor: TextureDescriptor,
    ) -> Texture {
        let size = wgpu::Extent3d {
            width: descriptor.width,
            height: descriptor.height,
            depth_or_array_layers: descriptor.depth.unwrap_or(1),
        };

        let texture = render_device.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(format!("{} Texture", descriptor.label).as_str()),
            size,
            mip_level_count: descriptor.mip_level_count,
            sample_count: 1,
            dimension: descriptor.dimension,
            format: descriptor.format,
            usage: descriptor.usage,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = render_device.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(format!("{} Texture Sampler", descriptor.label).as_str()),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: descriptor.filter,
            min_filter: descriptor.filter,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Texture { 
            texture, 
            view, 
            sampler,
            descriptor,
        }
    }

    pub fn fill<P: Pixel<Subpixel = u8>>(
        &self, 
        render_device: &RenderDevice,
        image: &ImageBuffer<P, Vec<P::Subpixel>>,
    ) {
        let bytes_per_pixel = std::mem::size_of::<P>();
        let bytes_per_row = bytes_per_pixel * image.width() as usize;

        render_device.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: self.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            image.as_raw(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row as u32),
                rows_per_image: Some(image.height()),
            },
            wgpu::Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            }
        );
    }

    /// Resizes the texture to match a new surface size.
    pub fn resize(&mut self, render_device: &RenderDevice, size: UVec2) {
        let mut descr = self.descriptor.clone();
        descr.width = size.x;
        descr.height = size.y;
        *self = Texture::new(render_device, descr);
    }
    
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }
    
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
    
    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
    
    pub fn descriptor(&self) -> &TextureDescriptor {
        &self.descriptor
    }
}