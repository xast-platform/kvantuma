use bytemuck::Pod;

use super::{Drawable, RenderDevice, buffer::BufferHandle, registry::RenderRegistry, types::*};

#[derive(Default, Debug)]
pub struct Mesh<V> {
    pub vertices: Vec<V>,
    pub indices: Vec<u32>,

    pub vertex_buffer: Option<BufferHandle>,
    pub index_buffer: Option<BufferHandle>,
}

impl<V> Mesh<V> {
    pub fn new(vertices: Vec<V>, indices: Vec<u32>) -> Mesh<V> {
        Mesh {
            vertices,
            indices,
            vertex_buffer: None,
            index_buffer: None,
        }
    }
}

impl<V: Pod> Drawable for Mesh<V> {
    fn update(&mut self, render_device: &mut RenderDevice, registry: &mut RenderRegistry) {
        if self.vertex_buffer.is_none() {
            self.vertex_buffer = Some(
                registry.new_buffer::<V>(render_device, self.vertices.len(), BufferUsages::VERTEX)
            );
        }

        if self.index_buffer.is_none() && !self.indices.is_empty() {
            self.index_buffer = Some(
                registry.new_buffer::<u32>(render_device, self.indices.len(), BufferUsages::INDEX)
            );
        }

        let Some(vb) = self.vertex_buffer else { unreachable!() };
        
        registry
            .get_buffer(vb) 
            .expect("Cannot call update() on Mesh")
            .fill_exact(render_device, 0, &self.vertices)
            .unwrap();

        let Some(ib) = self.index_buffer else { unreachable!() };
        
        registry
            .get_buffer(ib) 
            .expect("Cannot call update() on Mesh")
            .fill_exact(render_device, 0, &self.indices)
            .unwrap();
    }

    fn vertex_buffer(&self) -> BufferHandle {
        self.vertex_buffer
            .expect("Mesh is not set up with update()")
    }

    fn index_buffer(&self) -> Option<BufferHandle> {
        match self.indices.len() {
            0 => None,
            _ => Some(
                self.index_buffer
                    .expect("Mesh is not set up with update()")
            ),
        }
    }

    fn indices(&self) -> u32 {
        self.indices.len() as u32
    }
}