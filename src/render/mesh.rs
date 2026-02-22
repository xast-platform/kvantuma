use crate::render::Drawable;

pub struct Mesh<V> {
    pub vertices: Vec<V>,
    pub indices: Vec<u32>
}

impl<V> Drawable for Mesh<V> {
    fn update(&mut self, _render_device: &mut super::RenderDevice, _world: &mut super::registry::RenderRegistry) {
        todo!()
    }

    fn vertex_buffer(&self) -> super::buffer::BufferHandle {
        todo!()
    }
}