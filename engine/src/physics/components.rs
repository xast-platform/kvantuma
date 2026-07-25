use flecs_ecs::macros::Component;
use rapier3d::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct RigidBody {
    pub(crate) handle: RigidBodyHandle,
}

#[derive(Component, Clone, Copy)]
pub struct Collider {
    pub(crate) handle: ColliderHandle,
}

#[derive(Component, Clone)]
pub struct ColliderDescriptor {
    pub kind: ColliderKind,
}

#[derive(Clone)]
pub enum ColliderKind {
    Ball {
        radius: f32,
    },
    Cuboid {
        width: f32,
        height: f32,
        depth: f32,
    },
    Pill {
        half_height: f32, 
        radius: f32,
    },
    Mesh {
        mesh_type: ColliderMeshType,
    },
    Combined {
        colliders: Box<[ColliderDescriptor]>,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColliderMeshType {
    Convex,
    Concave
}

#[derive(Component, Clone)]
pub struct RigidBodyDescriptor {
    pub kind: RigidBodyKind,
}

#[repr(C)]
#[derive(Clone)]
pub enum RigidBodyKind {
    Static,
    Dynamic,
}