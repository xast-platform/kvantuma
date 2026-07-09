use std::ops::{Deref, DerefMut};

use flecs_ecs::macros::Component;
use glfw::{GlfwReceiver, PWindow};

pub struct WindowDescriptor {
    pub title: &'static str,
    pub width: u32,
    pub height: u32,
    pub mode: WindowMode,
    pub cursor_mode: glfw::CursorMode,
}

impl Default for WindowDescriptor {
    fn default() -> Self {
        Self {
            title: "Blank Game",
            width: 800,
            height: 600,
            mode: WindowMode::Windowed,
            cursor_mode: glfw::CursorMode::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct WindowSize {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) is_changed: bool,
}

impl WindowSize {
    pub fn is_changed(&self) -> bool {
        self.is_changed
    }
    
    pub fn width(&self) -> f32 {
        self.width
    }
    
    pub fn height(&self) -> f32 {
        self.height
    }
}

/// Singleton giving ECS systems access to the game window.
#[derive(Component, Default)]
pub struct Window(Option<PWindow>);

impl Window {
    pub(crate) fn install(&mut self, window: PWindow) {
        self.0 = Some(window);
    }

    pub(crate) fn take(&mut self) -> PWindow {
        self.0.take().expect("Window is only available while the Update pipeline runs")
    }
}

impl Deref for Window {
    type Target = PWindow;

    fn deref(&self) -> &PWindow {
        self.0.as_ref().expect("Window is only available while the Update pipeline runs")
    }
}

impl DerefMut for Window {
    fn deref_mut(&mut self) -> &mut PWindow {
        self.0.as_mut().expect("Window is only available while the Update pipeline runs")
    }
}

pub enum WindowMode {
    Windowed,
    Fullscreen,
}

pub type Events = GlfwReceiver<(f64, WindowEvent)>;

pub use glfw::{Action, CursorMode, Key, Modifiers, MouseButton, Scancode, WindowEvent};