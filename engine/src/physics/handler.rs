use flecs_ecs::macros::Component;
use glam::Vec3;
use rapier3d::prelude::{CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet, RigidBodySet, ColliderSet, DefaultBroadPhase, NarrowPhase, PhysicsPipeline};

use crate::physics::components::{Collider, RigidBody};

pub use rapier3d::prelude::RigidBody as RigidBodyInstance;
pub use rapier3d::prelude::Collider as ColliderInstance;

#[derive(Component)]
pub struct PhysicsHandler {
    rigid_bodies: RigidBodySet,
    colliders: ColliderSet,

    pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,

    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,

    ccd_solver: CCDSolver,

    gravity: Vec3,
    integration_parameters: IntegrationParameters,
}

impl PhysicsHandler {
    pub fn new() -> PhysicsHandler {
        PhysicsHandler {
            rigid_bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            gravity: Vec3::new(0.0, -9.81, 0.0),
            integration_parameters: IntegrationParameters::default(),
            pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
        }
    }

    pub fn new_rigid_body(&mut self, builder: impl Into<RigidBodyInstance>) -> RigidBody {
        RigidBody {
            handle: self.rigid_bodies.insert(builder),
        }
    }

    pub fn new_collider(&mut self, builder: impl Into<ColliderInstance>, rigid_body: RigidBody) -> Collider {
        Collider {
            handle: self.colliders.insert_with_parent(
                builder, 
                rigid_body.handle, 
                &mut self.rigid_bodies,
            ),
        }
    }

    pub fn get_rigid_body(&self, rigid_body: RigidBody) -> Option<&RigidBodyInstance> {
        self.rigid_bodies.get(rigid_body.handle)
    }

    pub fn get_rigid_body_mut(
        &mut self,
        rigid_body: RigidBody,
    ) -> Option<&mut RigidBodyInstance> {
        self.rigid_bodies.get_mut(rigid_body.handle)
    }

    pub fn get_collider(&self, collider: Collider) -> Option<&ColliderInstance> {
        self.colliders.get(collider.handle)
    }

    pub fn get_collider_mut(
        &mut self,
        collider: Collider,
    ) -> Option<&mut ColliderInstance> {
        self.colliders.get_mut(collider.handle)
    }

    pub fn remove_collider(
        &mut self,
        collider: Collider,
    ) -> Option<ColliderInstance> {
        self.colliders.remove(
            collider.handle, 
            &mut self.island_manager, 
            &mut self.rigid_bodies, 
            false,
        )
    }

    pub fn remove_rigid_body(
        &mut self,
        rigid_body: RigidBody,
    ) -> Option<RigidBodyInstance> {
        self.rigid_bodies.remove(
            rigid_body.handle, 
            &mut self.island_manager, 
            &mut self.colliders, 
            &mut self.impulse_joints, 
            &mut self.multibody_joints, 
            false,
        )
    }

    pub fn step(&mut self) {
        self.pipeline.step(
            self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            &(),
            &(),
        );
    }
}