pub mod app;
pub mod render;
pub mod physics;
pub mod ui;
pub mod error;
pub mod utils;
pub mod plugin;
pub mod math;

use flecs_ecs::macros::Component;

#[derive(Component)]
pub struct Setup;

#[derive(Component)]
pub struct Update;

#[derive(Component)]
pub struct Render;

#[derive(Component)]
#[repr(C)]
pub struct RandomNumber {
    pub value: u16
}