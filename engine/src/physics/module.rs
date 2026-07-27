use flecs_ecs::{core::flecs::Singleton, prelude::*};
use rapier3d::{dynamics::RigidBodyBuilder, geometry::ColliderBuilder};

use crate::{
    Update, math::Transform, physics::{
        components::{ColliderDescriptor, ColliderKind, ColliderMeshType, RigidBody, RigidBodyDescriptor, RigidBodyKind}, handler::PhysicsHandler, render_debug::PhysicsRenderDebugModule,
    }, render::mesh::{Mesh, Vertex},
};

#[derive(Component)]
pub struct PhysicsModule<const DEBUG: bool>;

impl<const DEBUG: bool> Module for PhysicsModule<DEBUG> {
    fn module(world: &World) {
        world.component::<PhysicsHandler>().add_trait::<Singleton>();
        world.set(PhysicsHandler::new());

        // Setup physics entities
        world.system::<(
            // Resource
            &mut PhysicsHandler,
            // Entity
            &Transform, &Mesh<Vertex>, &ColliderDescriptor, &RigidBodyDescriptor,
        )>()
            .kind(Update)
            .each_entity(|e, (handler, t, mesh, col_desc, rb_desc)| {
                let mut rb_builder = rigid_body_builder_helper(rb_desc.clone()).build();
                rb_builder.set_translation(t.translation, false);
                rb_builder.set_rotation(t.rotation, false);
                
                let rb = handler.new_rigid_body(rb_builder);
                let Some(col_builder) = collider_builder_helper(col_desc.clone(), mesh) else {
                    return;
                };
                let col = handler.new_collider(col_builder, rb);
                
                e.remove(ColliderDescriptor::id());
                e.remove(RigidBodyDescriptor::id());

                e.set(rb);
                e.set(col);
            });

        // Run physics pipeline
        world.system::<&mut PhysicsHandler>()
            .kind(Update)
            .each(PhysicsHandler::step);

        // Synchronize physics
        world.system::<(
            // Resource
            &mut PhysicsHandler,
            // Entity
            &mut Transform, &RigidBody,
        )>()
            .kind(Update)
            .each(|(handler, t, rb)| {
                let Some(rb) = handler.get_rigid_body(*rb) else {
                    return
                };

                t.translation = rb.translation();
                t.rotation = *rb.rotation();
            });

        if DEBUG {
            world.import::<PhysicsRenderDebugModule>();
        }
    }
}

fn rigid_body_builder_helper(
    rb_desc: RigidBodyDescriptor,
) -> RigidBodyBuilder {
    use RigidBodyKind::*;

    match rb_desc.kind {
        Static => RigidBodyBuilder::fixed(),
        Dynamic => RigidBodyBuilder::dynamic(),
    }
}

fn collider_builder_helper(
    col_desc: ColliderDescriptor, 
    mesh: &Mesh<Vertex>,
) -> Option<ColliderBuilder> {
    use ColliderKind::*;
    use ColliderMeshType::*;

    match col_desc.kind {
        Ball { radius } => 
            Some(ColliderBuilder::ball(radius)),
        Cuboid { width, height, depth } => 
            Some(ColliderBuilder::cuboid(width, height, depth)),
        Pill { half_height, radius } =>
            Some(ColliderBuilder::capsule_y(half_height, radius)),
        Mesh { mesh_type: Convex } => 
            ColliderBuilder::convex_mesh(
                mesh.vertices
                    .iter()
                    .map(|v| v.position)
                    .collect(), 
                mesh.indices
                    .as_slice()
                    .as_chunks::<3>().0,
            ),
        Mesh { mesh_type: Concave } =>
            ColliderBuilder::trimesh(
                mesh.vertices
                    .iter()
                    .map(|v| v.position)
                    .collect(), 
                mesh.indices
                    .as_slice()
                    .as_chunks::<3>().0
                    .to_vec(),
            ).ok(),
        Combined { colliders } =>
            Some(ColliderBuilder::compound(
                colliders
                    .into_iter()
                    .filter_map(|col| collider_builder_helper(col, mesh))
                    .map(|c| (c.position, c.shape))
                    .collect()
            )),
    }
}