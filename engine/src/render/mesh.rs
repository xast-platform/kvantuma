use bytemuck::{Pod, Zeroable};
use flecs_ecs::macros::Component;
use glam::{Vec2, Vec3, Vec4};

use crate::utils::Rect;

use super::{Drawable, RenderDevice, buffer::BufferHandle, registry::RenderRegistry, types::*};

#[derive(Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub texcoord: Vec2,
}

#[derive(Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
pub struct UiVertex {
    pub pos: Vec2,
}

impl UiVertex {
    const ATTRIBS: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x2,
    ];

    pub fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UiVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBS,
        }
    }
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

#[derive(Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
pub struct SkinnedVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub texcoord: Vec2,
    pub joint_indices: Vec4,
    pub joint_weights: Vec4,
}

impl SkinnedVertex {
    const ATTRIBS: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
        2 => Float32x2,
        3 => Float32x4,
        4 => Float32x4,
    ];

    pub fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SkinnedVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBS,
        }
    }

    pub fn from_vertex(vertex: Vertex) -> Self {
        Self {
            position: vertex.position,
            normal: vertex.normal,
            texcoord: vertex.texcoord,
            joint_indices: Vec4::ZERO,
            joint_weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        }
    }
}

#[derive(Debug, Component)]
pub struct Mesh<V: Send + Sync + 'static> {
    pub vertices: Vec<V>,
    pub indices: Vec<u32>,

    pub vertex_buffer: Option<BufferHandle>,
    pub index_buffer: Option<BufferHandle>,
}

impl<V: Send + Sync + 'static> Default for Mesh<V> {
    fn default() -> Self {
        Mesh { 
            vertices: vec![], 
            indices: vec![], 
            vertex_buffer: None, 
            index_buffer: None,
         }
    }
}

impl<V: Send + Sync + 'static> Mesh<V> {
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

impl Mesh<UiVertex> {
    pub fn outline_rect_mesh(rect: Rect, thickness: f32) -> Mesh<UiVertex> {
        let x = rect.x;
        let y = rect.y;
        let w = rect.w;
        let h = rect.h;
        let t = thickness;

        let vertices = vec![
            // Top
            UiVertex { pos: Vec2::new(x, y) },
            UiVertex { pos: Vec2::new(x + w, y) },
            UiVertex { pos: Vec2::new(x + w, y + t) },
            UiVertex { pos: Vec2::new(x, y + t) },

            // Bottom
            UiVertex { pos: Vec2::new(x, y + h - t) },
            UiVertex { pos: Vec2::new(x + w, y + h - t) },
            UiVertex { pos: Vec2::new(x + w, y + h) },
            UiVertex { pos: Vec2::new(x, y + h) },

            // Left
            UiVertex { pos: Vec2::new(x, y + t) },
            UiVertex { pos: Vec2::new(x + t, y + t) },
            UiVertex { pos: Vec2::new(x + t, y + h - t) },
            UiVertex { pos: Vec2::new(x, y + h - t) },

            // Right
            UiVertex { pos: Vec2::new(x + w - t, y + t) },
            UiVertex { pos: Vec2::new(x + w, y + t) },
            UiVertex { pos: Vec2::new(x + w, y + h - t) },
            UiVertex { pos: Vec2::new(x + w - t, y + h - t) },
        ];

        let indices = vec![
            // Top
            0, 1, 2,
            0, 2, 3,

            // Bottom
            4, 5, 6,
            4, 6, 7,

            // Left
            8, 9, 10,
            8, 10, 11,

            // Right
            12, 13, 14,
            12, 14, 15,
        ];

        Mesh {
            vertices,
            indices,
            vertex_buffer: None,
            index_buffer: None,
        }
    }
}

impl<V: Pod + Send + Sync + 'static> Drawable for Mesh<V> {
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