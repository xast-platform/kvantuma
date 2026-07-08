use flecs_ecs::prelude::*;
use xastge::app::{XastGE, window::WindowDescriptor};
use xastge::plugin::LoadPluginExt;
use xastge::{RandomNumber, Render, Update};

#[derive(Component)]
struct DemoModule;

impl Module for DemoModule {
    fn module(world: &World) {
        // Simple entity
        world.entity().set(RandomNumber { value: rand::random() });

        world.system::<()>()
            .kind(Update)
            .each(|_| {
                println!("Running update 1");
            });

        world.system::<()>()
            .kind(Update)
            .each(|_| {
                println!("Running update 2");
            });

        world.system::<&mut RandomNumber>()
            .kind(Update)
            .each(|num| {
                num.value = rand::random();
            });

        world.system::<()>()
            .kind(Render)
            .each(|_| {
                println!("Running render 1");
            });

        world.system::<()>()
            .kind(Render)
            .each(|_| {
                println!("Running render 2");
            });
    }
}

fn main() -> anyhow::Result<()> {
    XastGE::new(WindowDescriptor::default())?
        .import_module::<DemoModule>()
        .load_plugin("test-plugin")?
        .run();

    Ok(())
}