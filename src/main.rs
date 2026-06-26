use flecs_ecs::sys::{ecs_entity_t, ecs_world_t};
use flecs_ecs::prelude::*;
use flecs::system::System as SystemLabel;
use libloading::{Library, Symbol};

fn load_plugin(path: &str, api: &PluginApi) {
    let lib = unsafe {
        Library::new(path).unwrap()
    };

    unsafe {
        let func: Symbol<unsafe extern "C" fn(*const PluginApi)> =
            lib.get(b"register_systems").unwrap();

        func(api);
    }

    // keep library alive
    std::mem::forget(lib);
}

#[repr(C)]
pub struct PluginApi {
    // World
    pub world: *mut ecs_world_t,
    // Labels
    pub update: ecs_entity_t,
    pub render: ecs_entity_t,
    // Components
    pub random_number: ecs_entity_t,
}

fn main() {
    #[derive(Component)]
    struct UpdateLabel;

    #[derive(Component)]
    struct RenderLabel;

    #[derive(Component)]
    #[repr(C)]
    struct RandomNumber {
        pub value: u16
    }

    let world = World::new();

    // Register components
    let update_label_id = world.component::<UpdateLabel>().id();
    let render_label_id = world.component::<RenderLabel>().id();
    let random_number_id = world.component::<RandomNumber>().id();

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

    println!("world ptr = {:p}", world.ptr_mut());
    // Register C systems
    load_plugin(
        "./cdylib/libplugin.so", 
        &PluginApi { 
            world: world.ptr_mut(), 
            update: *update_label_id, 
            render: *render_label_id, 
            random_number: *random_number_id,
        },
    );

    // Run world
    for _ in 0..5 {
        world.run_pipeline(*update_pipeline);
        world.run_pipeline(*render_pipeline);
    }
}