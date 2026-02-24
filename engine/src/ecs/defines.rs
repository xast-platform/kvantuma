use bytemuck::Pod;
use taffy::TaffyTree;

use crate::render::material::TintedTextureMaterial;
use crate::component;
use crate::render::mesh::Mesh;
use super::component::*;

component! { EXTERN: TintedTextureMaterial }
component! { EXTERN: TaffyTree }

impl<V: Pod + 'static> Component for Mesh<V> {
    fn component_id() -> ComponentId {
        component_id::<Self>()
    }
    fn id(&self) -> ComponentId {
        component_id::<Self>()
    }
    fn layout(&self) -> std::alloc::Layout {
        std::alloc::Layout::new::<Self>()
    }
    fn kind(&self) -> ComponentKind {
        ComponentKind::Pod
    }
    fn drop_fn(&self) -> Option<unsafe fn(*mut u8)> {
        None
    }
}