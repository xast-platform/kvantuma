use flecs_ecs::core::World;
use flecs_ecs::prelude::Builder;
use glam::UVec2;
use glfw::{Glfw, PWindow, WindowEvent};
use flecs_ecs::addons::Module;
use flecs_ecs::core::{Entity, WorldGet, query_builder::QueryBuilderImpl, utility::traits::IdOperations};
use flecs_ecs::macros::Component;
use flecs_ecs::core::flecs::system::System;

use crate::{
    Render, Setup, Update, 
    app::{
        helper::{GameLoopCallbacks, game_loop},
        input::{Keyboard, Mouse},
        window::{Events, WindowDescriptor, WindowMode, WindowSize}
    }, 
    error::GameError, 
    plugin::{self, LoadPluginExt}, 
    render::{
        RenderDevice,
        error::RenderError,
    },
};

pub mod base;
pub mod helper;
pub mod input;
pub mod time;
pub mod window;

pub struct EcsPipelines {
    setup_pipeline: Entity,
    update_pipeline: Entity,
    render_pipeline: Entity,
}

impl EcsPipelines {
    pub fn new(world: &World) -> EcsPipelines {
        EcsPipelines {
            setup_pipeline: world.pipeline()
                .with(System)
                .with(Setup)
                .build()
                .id(),
            update_pipeline: world.pipeline()
                .with(System)
                .with(Update)
                .build()
                .id(),
            render_pipeline: world.pipeline()
                .with(System)
                .with(Render)
                .build()
                .id(),
        }
    }
}

/// Holds the `RenderDevice` _ONLY_ while the Render pipeline is running.
#[derive(Component)]
pub struct RenderDeviceSlot(pub Option<RenderDevice>);

impl RenderDeviceSlot {
    pub fn get(&self) -> &RenderDevice {
        self.0.as_ref().expect("RenderDevice is only available while the Render pipeline runs")
    }

    pub fn get_mut(&mut self) -> &mut RenderDevice {
        self.0.as_mut().expect("RenderDevice is only available while the Render pipeline runs")
    }
}

/// Render-tagged systems report a failed frame here.
/// 
/// Drained by `XastGE::run` after the Render pipeline runs.
#[derive(Component, Default)]
pub struct RenderErrorSlot(pub Option<RenderError>);

pub struct GameState {
    pub world: World,
    pub render_device: Option<RenderDevice>,
    pub pipelines: EcsPipelines,
}

pub struct XastGE {
    world: World,
    render_device: RenderDevice,
    events: Events,
    glfw: Glfw,
    window: PWindow,
    pipelines: EcsPipelines,
}

impl XastGE {
    pub fn new(desc: WindowDescriptor) -> Result<XastGE, GameError> {
        let mut glfw = glfw::init(glfw::fail_on_errors)?;
        glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));

        let world = World::new();
        let pipelines = EcsPipelines::new(&world);

        world.set(WindowSize {
            width: desc.width as f32,
            height: desc.height as f32,
        });
        world.set(Keyboard::default());
        world.set(Mouse::default());
        world.set(RenderDeviceSlot(None));
        world.set(RenderErrorSlot::default());

        let (mut window, events) = glfw.with_primary_monitor(|glfw, m| {
            glfw.create_window(
                desc.width,
                desc.height,
                desc.title,
                match desc.mode {
                    WindowMode::Windowed => glfw::WindowMode::Windowed,
                    WindowMode::Fullscreen => m.map_or(
                        glfw::WindowMode::Windowed,
                        |m| glfw::WindowMode::FullScreen(m),
                    ),
                },
            ).expect("Cannot create GLFW window")
        });

        let render_device = pollster::block_on(RenderDevice::new(&window))?;

        window.set_all_polling(true);
        window.set_cursor_mode(desc.cursor_mode);

        Ok(XastGE {
            world,
            render_device,
            events,
            glfw,
            window,
            pipelines,
        })
    }

    pub fn import_module<M: Module>(self) -> Self {
        self.world.import::<M>();
        self
    }

    pub fn load_plugin(self, name: &str) -> Result<Self, plugin::LoadPluginError> {
        self.world.load_plugin(name)?;
        Ok(self)
    }

    pub fn run(self) {
        let XastGE { 
            world, 
            render_device, 
            events, 
            glfw, 
            window, 
            pipelines,
        } = self;

        world.run_pipeline_time(pipelines.setup_pipeline, 0.0);

        let game_state = GameState {
            world,
            render_device: Some(render_device),
            pipelines,
        };

        game_loop(
            glfw,
            events,
            window,
            game_state,
            240, 0.1,
            GameLoopCallbacks {
                update: |g| {
                    let dt = g.fixed_time_step() as f32;
                    g.game.world.run_pipeline_time(g.game.pipelines.update_pipeline, dt);
                },
                render: |g| {
                    g.game.world.get::<(&mut Keyboard, &mut Mouse)>(|(kb, mouse)| {
                        kb.clear_frame();
                        mouse.clear_frame();
                    });

                    let device = g.game.render_device.take()
                        .expect("RenderDevice is missing outside the Render pipeline");
                    g.game.world.set(RenderDeviceSlot(Some(device)));

                    g.game.world.run_pipeline_time(g.game.pipelines.render_pipeline, 0.0);

                    let device = g.game.world
                        .get::<&mut RenderDeviceSlot>(|slot| slot.0.take())
                        .expect("Render pipeline must not remove the RenderDevice singleton");
                    g.game.render_device = Some(device);

                    if let Some(err) = g.game.world.get::<&mut RenderErrorSlot>(|slot| slot.0.take()) {
                        match err {
                            RenderError::Lost => {
                                log::error!("The underlying surface has changed, and therefore the swap chain must be updated");
                                if let Some(device) = g.game.render_device.as_mut() {
                                    device.resize();
                                }
                            }
                            RenderError::OutOfMemory => {
                                log::error!("LOST surface, drop frame");
                            }
                            e => {
                                panic!("Dropped frame with error: {e}");
                            }
                        }
                    }
                },
                handler: |g, e| {
                    #[allow(clippy::single_match)]
                    match e {
                        WindowEvent::FramebufferSize(w, h) => {
                            if let Some(device) = g.game.render_device.as_mut() {
                                device.resize_with(UVec2::new(*w as u32, *h as u32));
                            }
                            g.game.world.get::<&mut WindowSize>(|wsize| {
                                wsize.width = *w as f32;
                                wsize.height = *h as f32;
                            });
                        }
                        // Other events
                        _ => {}
                    }

                    g.game.world.get::<(&mut Keyboard, &mut Mouse)>(|(kb, mouse)| {
                        kb.handle_event(e);
                        mouse.handle_event(e);
                    });
                },
            },
        );
    }
}
