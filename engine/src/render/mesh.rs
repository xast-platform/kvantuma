use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};

use super::{Drawable, RenderDevice, buffer::BufferHandle, registry::RenderRegistry, types::*};

#[derive(Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub texcoord: Vec2,
}

impl Vertex {
    const ATTRIBS: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
        2 => Float32x2,
    ];

    pub fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBS,
        }
    }
}

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

impl Mesh<Vertex> {
    pub fn load_obj(path: &str) -> Self {
        let (mut models, _) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ignore_points: true,
                ..Default::default()
            },
        ).expect("Cannot load OBJ file");
                
        let m = models.swap_remove(0);

        let mut vertices = Vec::<Vertex>::new();
        let indices = m.mesh.indices;
        
        for i in 0..m.mesh.positions.len() / 3 {                
            let texcoord: Vec2;
            
            let position = Vec3::new(
                m.mesh.positions[i*3],
                m.mesh.positions[i*3+1],
                m.mesh.positions[i*3+2],
            );
            
            let normal = Vec3::new(
                m.mesh.normals[i*3],
                m.mesh.normals[i*3+1],
                m.mesh.normals[i*3+2],
            );
            
            if i*2 < m.mesh.texcoords.len() {
                texcoord = Vec2::new(
                    m.mesh.texcoords[i*2],
                    m.mesh.texcoords[i*2+1],
                );
            } else {
                texcoord = Vec2::ZERO;
            }
            
            vertices.push(Vertex {
                position,
                normal,
                texcoord,
            });
        }
                    
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

        if let Some(ib) = self.index_buffer {
            registry
                .get_buffer(ib) 
                .expect("Cannot call update() on Mesh")
                .fill_exact(render_device, 0, &self.indices)
                .unwrap();
        }
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