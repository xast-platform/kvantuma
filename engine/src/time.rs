use std::time::Instant;

use flecs_ecs::prelude::*;

use crate::Update;

#[derive(Component)]
pub struct Time {
    delta: f32,
    last_frame: Instant,
}

impl Default for Time {
    fn default() -> Self {
        Time { delta: 0.0, last_frame: Instant::now() }
    }
}

impl Time {
    pub fn new() -> Self {
        Time::default()
    }

    pub fn delta_time(&self) -> f32 {
        self.delta
    }

    pub fn update(&mut self, new: Instant) {
        let dt = (new - self.last_frame).as_secs_f32();
        self.last_frame = new;
        self.delta = dt;
    }
}

#[derive(Component)]
pub struct TimeModule;

impl Module for TimeModule {
    fn module(world: &World) {
        world.system::<&mut Time>()
            .kind(Update)
            .each(|time| {
                time.update(Instant::now());
            });
    }
}