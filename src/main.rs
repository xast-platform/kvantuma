use flecs_ecs::prelude::*;
use flecs::system::System as SystemLabel;
use xastge::plugin::LoadPluginExt;
use xastge::{RandomNumber, RenderLabel, UpdateLabel};

fn main() {
    let world = World::new();

    // Simple entity
    world.entity().set(RandomNumber { value: rand::random() });

    // Rust systems
    let update_pipeline = world
        .pipeline()
        .with(SystemLabel)
        .with(UpdateLabel)
        .build();

    let render_pipeline = world
        .pipeline()
        .with(SystemLabel)
        .with(RenderLabel)
        .build();

    world.system::<()>()
        .kind(UpdateLabel)
        .each(|_| {
            println!("Running update 1");
        });

    world.system::<()>()
        .kind(UpdateLabel)
        .each(|_| {
            println!("Running update 2");
        });

    world.system::<&mut RandomNumber>()
        .kind(UpdateLabel)
        .each(|num| {
            num.value = rand::random();
        });

    world.system::<()>()
        .kind(RenderLabel)
        .each(|_| {
            println!("Running render 1");
        });

    world.system::<()>()
        .kind(RenderLabel)
        .each(|_| {
            println!("Running render 2");
        });

    world.load_plugin("testplugin").unwrap_or_else(|e| {
        panic!("{e}");
    });

    // Run world
    for _ in 0..5 {
        world.run_pipeline(*update_pipeline);
        world.run_pipeline(*render_pipeline);
    }
}