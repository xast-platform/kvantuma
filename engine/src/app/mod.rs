use glam::UVec2;
use glfw::{Glfw, PWindow, WindowEvent};

use crate::{app::{helper::{GameLoopCallbacks, game_loop}, window::{Events, WindowDescriptor, WindowMode}}, ecs::world::World, error::GameError, render::{RenderDevice, error::RenderError}};

pub mod base;
pub mod helper;
pub mod time;
pub mod window;

pub struct GameState<G> {
    pub game: G,
    pub world: World,
    pub render_device: RenderDevice,
}

pub trait Game {
    fn init(
        &mut self, 
        world: &mut World, 
        render_device: &mut RenderDevice,
    ) -> anyhow::Result<()>;

    fn update(
        &mut self, 
        world: &mut World,
    ) -> anyhow::Result<()>;

    fn input(
        &mut self, 
        event: &WindowEvent, 
        world: &mut World,
    ) -> anyhow::Result<bool>;

    fn render(
        &mut self, 
        world: &mut World, 
        render_device: &mut RenderDevice,
    ) -> Result<(), RenderError>;
}

pub struct App<G> {
    world: World,
    render_device: RenderDevice,
    events: Events,
    glfw: Glfw,
    window: PWindow,
    state: G,
}

impl<G> App<G> {
    pub fn new(desc: WindowDescriptor, state: G) -> Result<App<G>, GameError> {
        let mut glfw = glfw::init(glfw::fail_on_errors)?;
        glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));

        let world = World::new();

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

        window.set_framebuffer_size_polling(true);
        window.set_key_polling(true);
        window.set_mouse_button_polling(true);
        window.set_pos_polling(true);

        Ok(App { 
            world, 
            render_device, 
            events,
            glfw,
            state,
            window,
        })
    }
}

impl<G: Game + 'static> App<G> {
    pub fn run(mut self) {
        self.state.init(&mut self.world, &mut self.render_device)
            .unwrap_or_else(|e| panic!("Failed to initialize game: {e}"));

        let game_state = GameState {
            game: self.state,
            world: self.world,
            render_device: self.render_device,
        };

        game_loop(
            self.glfw, 
            self.events, 
            self.window, 
            game_state, 
            240, 0.1, 
            GameLoopCallbacks {
                update: |g| g.game.game.update(&mut g.game.world)
                    .unwrap_or_else(|e| panic!("Failed game update: {e}")),
                render: |g| match g.game.game.render(&mut g.game.world, &mut g.game.render_device) {
                    Ok(_) => {}
                    Err(RenderError::Lost) => {
                        log::error!("The underlying surface has changed, and therefore the swap chain must be updated");
                    }
                    Err(RenderError::OutOfMemory) => {
                        log::error!("LOST surface, drop frame");
                    }
                    Err(e) => {
                        panic!("Dropped frame with error: {e}");
                    }
                },
                handler: |g, e| {
                    #[allow(clippy::single_match)]
                    match e {
                        WindowEvent::FramebufferSize(w, h) => {
                            g.game.render_device.resize_with(UVec2::new(*w as u32, *h as u32));
                        }
                        // Other events
                        _ => {}
                    }

                    g.game.game.input(e, &mut g.game.world)
                        .unwrap_or_else(|err| panic!("Failed game input: {err}"));
                },
            },
        );
    }
}