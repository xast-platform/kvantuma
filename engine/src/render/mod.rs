use bytemuck::Pod;
use glam::{IVec2, Quat, UVec2, Vec3};
use glfw::Window;
use serde::{Deserialize, Serialize};

use crate::{error::GameError, render::{buffer::{BufferHandle, BufferStorage}, draw_context::DrawContext, error::RenderError, pipeline::{Pipeline}, shader_resource::{ShaderResource}, registry::RenderRegistry, texture::{Texture, TextureDescriptor}}};

pub mod error;
pub mod buffer;
pub mod texture;
pub mod pipeline;
pub mod registry;
pub mod material;
pub mod pass;
pub mod draw_context;
pub mod shader_resource;
pub mod mesh;

pub mod types {
    pub use wgpu::{
        BufferUsages,
        ShaderStages,
        BufferBindingType,
        FilterMode,
        TextureDimension,
        TextureUsages,
        TextureFormat,
        TextureSampleType,
        Extent3d,
        ShaderSource,
        TextureView,
        SamplerBindingType,
        ShaderModuleDescriptor,
        VertexBufferLayout,
        Operations,
        LoadOp,
        StoreOp,
        Color,
    };
}

pub use wgpu::include_wgsl;

use types::*;

pub struct RenderDevice {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: UVec2,
    depth_texture: Option<Texture>,
}

impl RenderDevice {
    pub async fn new(window: &Window) -> Result<RenderDevice, GameError> {
        let size = IVec2::from(window.get_framebuffer_size()).as_uvec2();

        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN, 
            ..Default::default()
        };
        let instance = wgpu::Instance::new(&instance_descriptor);
        
        let surface = unsafe {
            let target = wgpu::SurfaceTargetUnsafe::from_window(&window)
                .map_err(|e| RenderError::HandleError(e.to_string()))?;
            instance.create_surface_unsafe(target)
                .map_err(RenderError::from)?
        };

        let adapter_descriptor = wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };
        
        let adapter = instance.request_adapter(&adapter_descriptor).await
            .map_err(RenderError::from)?;

        let device_descriptor = wgpu::DeviceDescriptor {
            required_features: wgpu::Features::PUSH_CONSTANTS,
            required_limits: wgpu::Limits {
                max_push_constant_size: 128,
                ..Default::default()
            },
            label: Some("Logical device"),
            ..Default::default()
        };

        let (device, queue) = adapter.request_device(&device_descriptor).await
            .map_err(RenderError::from)?;

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats
            .iter()
            .copied()
            .find(|f | f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.x,
            height: size.y,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };
        surface.configure(&device, &config);

        let mut render_device = RenderDevice { 
            surface, 
            device, 
            queue, 
            config, 
            size,
            depth_texture: None,
        };

        render_device.depth_texture = Some(Texture::new(
            &render_device, 
            TextureDescriptor {
                width: render_device.config.width,
                height: render_device.config.height,
                filter: wgpu::FilterMode::Linear,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                depth: None,
                mip_level_count: 1,
                label: "Depth Data".to_string(),
            },
        ));

        Ok(render_device)
    }

     /// Retrieves the current canvas for drawing.
    pub fn canvas(&self) -> Result<Canvas, RenderError> {
        let texture = self.surface.get_current_texture()?;
        let view = texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Canvas { texture, view })
    }

    /// Creates a new drawing context for issuing draw commands.
    pub fn draw_ctx(&self) -> DrawContext {
        DrawContext {
            encoder: self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default()),
        }
    }

    /// Resizes the render_device to the current window size.
    pub fn resize(&mut self) {
        self.resize_with(self.size);
    }

    /// Resizes the render_device to a specified size.
    pub fn resize_with(&mut self, new_size: UVec2) {
        if new_size.x == 0 || new_size.y == 0 { return }

        self.size = new_size;
        self.config.width = new_size.x;
        self.config.height = new_size.y;
        self.surface.configure(&self.device, &self.config);

        if let Some(depth_texture) = &self.depth_texture {
            let mut depth_descr = depth_texture.descriptor().clone();
            depth_descr.width = self.config.width;
            depth_descr.height = self.config.height;
            self.depth_texture = Some(Texture::new(self, depth_descr));
        }
    }

    /// Retrieves the current size of the render_device.
    pub fn size(&self) -> UVec2 {
        self.size
    }

    /// Retrieves the depth texture used for depth testing.
    pub fn depth_texture(&self) -> &Texture {
        self.depth_texture.as_ref().unwrap()
    }

    /// Retrieve current surface format
    pub fn surface_format(&self) -> TextureFormat {
        self.config.format
    }
}

/// Trait for surface, which are meant to be rendered to. E.g. Canvas 
/// or texture with RENDER_ATTACHMENT usage
pub trait RenderSurface {
    /// Get rendering view of the surface
    fn view(&self) -> &types::TextureView;
}

/// Represents the canvas used for rendering.
pub struct Canvas {
    texture: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
}

impl RenderSurface for Canvas {
    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

/// Trait for drawable objects.
pub trait Drawable {
    /// Updates the drawable object's render_device data
    fn update(&mut self, render_device: &mut RenderDevice, registry: &mut RenderRegistry);

    /// Retrieves the ID of the vertex buffer used by the drawable.
    fn vertex_buffer(&self) -> BufferHandle;

    fn index_buffer(&self) -> Option<BufferHandle>;

    fn indices(&self) -> u32;
}

/// Trait used to convert Rust data structures to GPU-friendly ones.
pub trait InstanceData {
    /// GPU-friendly equivalent type.
    type UniformData: Pod;

    /// Get uniform GPU data from current instance.
    fn uniform_data(&self) -> Self::UniformData;
}

impl<I: Pod> InstanceData for I {
    type UniformData = I;

    fn uniform_data(&self) -> Self::UniformData {
        *self
    }
}

/// Enumeration of different types of transformations.
#[derive(Clone, Copy, Default, Debug, Hash, PartialEq, Serialize, Deserialize)]
pub enum TransformationType {
    /// First person transformation. Calculation performed is ROTATION_MATRIX * TRANSLATION_MATRIX
    #[default]
    FirstPerson,
    /// Look-at transformation. Calculation performed is TRANSLATION_MATRIX * ROTATION_MATRIX
    LookAt,
}


/// Common transformation trait for regular and rt-transforms. Used in voxel model
/// loading and chunks creation
pub trait Transformation {
    /// Create new transform from default transformation parameters.
    fn from_transform(
        transformation_type: TransformationType,
        translation: Vec3, 
        rotation: Quat, 
        pivot: Vec3,
    ) -> Self;
}