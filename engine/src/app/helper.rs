use glfw::{Glfw, GlfwReceiver};
use glfw::{PWindow, WindowEvent};

use super::base::*;
use super::time::*;

pub struct GameLoopCallbacks<G> {
    pub update: fn(&mut GameLoop<G, Time, Option<PWindow>>),
    pub render: fn(&mut GameLoop<G, Time, Option<PWindow>>),
    pub handler: fn(&mut GameLoop<G, Time, Option<PWindow>>, &WindowEvent),
}

pub fn game_loop<G>(
    mut glfw: Glfw,
    events: GlfwReceiver<(f64, WindowEvent)>,
    window: PWindow,
    game: G,
    updates_per_second: u32,
    max_frame_time: f64,
    callbacks: GameLoopCallbacks<G>,
)
    where
        G: 'static,
{
    let GameLoopCallbacks { mut update, mut render, handler } = callbacks;

    let mut game_loop = GameLoop::<G, Time, Option<PWindow>>::new(
        game,
        updates_per_second,
        max_frame_time,
        Some(window),
    );

    while !game_loop.window.as_ref().expect("window is checked out only within update/render").should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handler(&mut game_loop, &event);
        }

        if !game_loop.next_frame(&mut update, &mut render) {
            game_loop.window.as_mut().expect("window is checked out only within update/render").set_should_close(true);
        }
    }
}