//! Buffer module contains types related to buffers and buffer
//! resources, which transfer data from CPU to shaders on GPU

use std::mem::{size_of, size_of_val};
use bytemuck::Pod;
use pretty_type_name::pretty_type_name;
use slotmap::new_key_type;

use crate::render::registry::RenderRegistry;

use super::error::RenderError;
use super::RenderDevice;
use super::types::*;

new_key_type! {
    pub struct BufferHandle;
}

impl BufferHandle {
    pub fn and_then_mut<F>(
        self, 
        registry: &mut RenderRegistry,
        mut f: F,
    ) -> BufferHandle
        where
            F: FnMut(&mut BufferStorage)
    {
        if let Some(buf) = registry.get_buffer_mut(self) {
            f(buf);
        }

        self
    }

    pub fn and_then<F, R>(
        self, 
        registry: &RenderRegistry, 
        mut f: F,
    ) -> BufferHandle
        where
            F: FnMut(&BufferStorage) -> R
    {
        if let Some(buf) = registry.get_buffer(self) {
            f(buf);
        }

        self
    }
}

#[derive(Debug)]
pub struct BufferStorage {
    inner: wgpu::Buffer,
    capacity: usize,
}

impl BufferStorage {
    /// Creates a new buffer with the given capacity and usage. 
    /// 
    /// The capacity of the buffer is the number of elements of type `T`.
    pub fn new<T: Pod>(render_device: &RenderDevice, capacity: usize, usage: BufferUsages) -> BufferStorage {
        BufferStorage {
            inner: BufferStorage::new_inner::<T>(&render_device.device, capacity * size_of::<T>(), usage),
            capacity,
        }
    }

    /// Fills the buffer with the given data, ensuring the data 
    /// fits within the buffer capacity. 
    /// 
    /// Offset is the number of elements of type `T`, not bytes. 
    /// 
    /// Returns `BufferOverflow` error, if the data length exceeds 
    /// the buffer capacity.
    pub fn fill_exact<T: Pod>(
        &self, 
        render_device: &RenderDevice, 
        offset: u64,
        data: &[T],
    ) -> Result<(), RenderError> {
        if data.len() > self.capacity {
            return Err(RenderError::BufferOverflow(data.len()));
        }

        if !data.is_empty() {
            render_device.queue.write_buffer(&self.inner, offset * size_of::<T>() as u64, bytemuck::cast_slice(data));
        }

        Ok(())
    }

    /// Fills the buffer with the given data, resizing the buffer if necessary.
    ///
    /// Offset is the number of elements of type `T`, not bytes. 
    pub fn fill<T: Pod>(
        &mut self, 
        render_device: &RenderDevice, 
        offset: u64,
        data: &[T],
    ) {
        let bytes_to_write = size_of_val(data);
        if bytes_to_write > self.capacity * size_of::<T>() {
            self.resize::<T>(render_device, data.len());
        }

        self.fill_exact(render_device, offset, data).unwrap();
    }

    /// Resize the buffer to the given capacity, which
    /// is the number of elements of type `T`
    pub fn resize<T: Pod>(&mut self, render_device: &RenderDevice, capacity: usize) {
        self.inner = BufferStorage::new_inner::<T>(&render_device.device, capacity * size_of::<T>(), self.inner.usage());
        self.capacity = capacity;
    }
    
    pub fn inner(&self) -> &wgpu::Buffer {
        &self.inner
    }
    
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    fn new_inner<T: Pod>(device: &wgpu::Device, capacity: usize, usage: wgpu::BufferUsages) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(format!("Buffer ({:?}, {})", usage, pretty_type_name::<T>()).as_str()),
            size: capacity as u64,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}

#[cfg(doc)]
use super::pipeline::ShaderResource;
/// Used to bind generic buffer in [`ShaderResource`]
pub struct BufferResourceDescriptor {
    /// Indicates in which shader stages (Vertex, Fragment and/or Compute)
    /// current buffer is visible
    pub visibility: ShaderStages,
    /// Indicates buffer binding type:
    /// * uniform - read only, faster, small amount of data
    /// * storage - read/write, slower, bigger amount of data
    pub buffer_type: BufferBindingType,
}