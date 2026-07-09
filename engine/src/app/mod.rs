use flecs_ecs::core::World;
use flecs_ecs::core::flecs::Singleton;
use flecs_ecs::prelude::Builder;
use glam::UVec2;
use glfw::{Glfw, PWindow, WindowEvent};
use flecs_ecs::addons::Module;
use flecs_ecs::core::{Entity, WorldGet, query_builder::QueryBuilderImpl, utility::traits::IdOperations};
use flecs_ecs::macros::Component;
use flecs_ecs::core::flecs::system::System;

use crate::render::Canvas;
use crate::render::draw_context::DrawContext;
use crate::{
    Render, Setup, Update, 
    app::{
        helper::{GameLoopCallbacks, game_loop},
        input::{Keyboard, Mouse},
        window::{Events, Window, WindowDescriptor, WindowMode, WindowSize}
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
#[derive(Component, Default)]
pub struct RenderSlot {
    render_device: Option<RenderDevice>,
    canvas: Option<Canvas>,
    draw_context: Option<DrawContext>,
}

impl RenderSlot {
    #[inline]
    pub fn device(&self) -> &RenderDevice {
        self.render_device.as_ref().expect("RenderDevice is only available while the Setup or Render pipeline runs")
    }

    #[inline]
    pub fn device_mut(&mut self) -> &mut RenderDevice {
        self.render_device.as_mut().expect("RenderDevice is only available while the Setup or Render pipeline runs")
    }

    #[inline]
    pub fn canvas(&self) -> &Canvas {
        self.canvas.as_ref().expect("Canvas is only available while the Render pipeline runs")
    }

    #[inline]
    pub fn canvas_mut(&mut self) -> &mut Canvas {
        self.canvas.as_mut().expect("Canvas is only available while the Render pipeline runs")
    }

    #[inline]
    pub fn draw_ctx(&self) -> &DrawContext {
        self.draw_context.as_ref().expect("DrawContext is only available while the Render pipeline runs")
    }

    #[inline]
    pub fn draw_ctx_mut(&mut self) -> &mut DrawContext {
        self.draw_context.as_mut().expect("DrawContext is only available while the Render pipeline runs")
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

        world.component::<DrawContext>().add_trait::<Singleton>();
        world.component::<Canvas>().add_trait::<Singleton>();

        world.component::<WindowSize>().add_trait::<Singleton>();
        world.set(WindowSize {
            width: desc.width as f32,
            height: desc.height as f32,
        });

        world.component::<Window>().add_trait::<Singleton>();
        world.set(Window::default());


        world.component::<Keyboard>().add_trait::<Singleton>();
        world.set(Keyboard::default());

        world.component::<Mouse>().add_trait::<Singleton>();
        world.set(Mouse::default());

        world.component::<RenderSlot>().add_trait::<Singleton>();
        world.set(RenderSlot::default());

        world.component::<RenderErrorSlot>().add_trait::<Singleton>();
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

        let mut game_state = GameState {
            world,
            render_device: Some(render_device),
            pipelines,
        };

        let device = game_state.render_device.take()
            .expect("RenderDevice is missing outside the Render pipeline");
        game_state.world.set(RenderSlot {
            render_device: Some(device),
            ..Default::default()
        });

        game_state.world.run_pipeline_time(game_state.pipelines.setup_pipeline, 0.0);

        let device = game_state.world
            .get::<&mut RenderSlot>(|slot| slot.render_device.take())
            .expect("Setup pipeline must not clear the RenderSlot singleton");
        game_state.render_device = Some(device);

        game_loop(
            glfw,
            events,
            window,
            game_state,
            240, 0.1,
            GameLoopCallbacks {
                update: |g| {
                    let dt = g.fixed_time_step() as f32;

                    let window = g.window.take().expect("window is missing outside the Update pipeline");
                    g.game.world.get::<&mut Window>(|w| w.install(window));

                    g.game.world.run_pipeline_time(g.game.pipelines.update_pipeline, dt);

                    let window = g.game.world.get::<&mut Window>(|w| w.take());
                    g.window = Some(window);
                },
                render: |g| {
                    g.game.world.get::<(&mut Keyboard, &mut Mouse)>(|(kb, mouse)| {
                        kb.clear_frame();
                        mouse.clear_frame();
                    });

                    let device = g.game.render_device.take()
                        .expect("RenderDevice is missing outside the Render pipeline");

                    let draw_ctx = device.draw_ctx();
                    let canvas = match device.canvas() {
                        Ok(val) => Some(val),
                        Err(e) => {
                            log::error!("Cannot acquire canvas: {e}");
                            None
                        }
                    };

                    g.game.world.set(RenderSlot {
                        render_device: Some(device),
                        draw_context: Some(draw_ctx),
                        canvas,
                    });

                    g.game.world.run_pipeline_time(g.game.pipelines.render_pipeline, 0.0);

                    let (device, draw_ctx, canvas) = g.game.world
                        .get::<&mut RenderSlot>(|slot| zip3(
                            slot.render_device.take(),
                            slot.draw_context.take(),
                            slot.canvas.take(),
                        ))
                        .expect("Render pipeline must not clear the RenderSlot singleton");

                    draw_ctx.apply(canvas, &device);

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

fn zip3<A, B, C>(a: Option<A>, b: Option<B>, c: Option<C>) -> Option<(A, B, C)> {
    Some((a?, b?, c?))
}