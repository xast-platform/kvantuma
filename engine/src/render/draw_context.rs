use super::*;
use super::pass::*;

/// Represents a drawing context used for issuing draw commands.
pub struct DrawContext {
    pub(super) encoder: wgpu::CommandEncoder,
}

impl DrawContext {
    /// Begins a new render pass with the specified canvas and depth texture
    /// for initializing draw process.
    pub fn render_pass<'a>(
        &'a mut self,
        canvases: &'a [&'a dyn RenderSurface],
        depth_texture: &'a Texture,
    ) -> RenderPass<'a> {
        let pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &canvases
                .iter()
                .map(|canvas| {
                    Some(wgpu::RenderPassColorAttachment {
                        view: canvas.view(),
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })
                })
                .collect::<Vec<_>>(),
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_texture.view(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        RenderPass { pass }
    }

    /// Begins a new compute pass with the specified canvas and depth texture
    /// for initializing compute process.
    pub fn compute_pass(&mut self) -> ComputePass<'_> {
        let pass = self.encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute pass"),
            timestamp_writes: None,
        });

        ComputePass { pass }
    }

    /// Clear given buffer to zeros
    pub fn clear_buffer<T: Pod>(&mut self, buffer: &BufferStorage) {
        self.encoder.clear_buffer(buffer.inner(), 0, None);
    }

    /// Copy data from one buffer to another. Offsets and copy size are given in
    /// number of elements of `T`, not in bytes
    pub fn copy_buffer<T: Pod>(
        &mut self,
        from: &BufferStorage,
        from_offset: u64,
        to: &BufferStorage,
        to_offset: u64,
        copy_size: u64,
    ) {
        self.encoder.copy_buffer_to_buffer(
            from.inner(), 
            from_offset * size_of::<T>() as u64, 
            to.inner(), 
            to_offset * size_of::<T>() as u64, 
            copy_size * size_of::<T>() as u64,
        );
    }

    /// Copy data from one texture to another
    pub fn copy_texture(&mut self, from: &Texture, to: &Texture) {
        self.encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: from.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: to.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: to.descriptor().width,
                height: to.descriptor().height,
                depth_or_array_layers: 1,
            }
        );
    }

    /// Applies the drawing commands and presents the canvas.
    pub fn apply(self, canvas: Canvas, render_device: &RenderDevice) {        
        render_device.queue.submit(std::iter::once(self.encoder.finish()));
        canvas.texture.present();
    }
}