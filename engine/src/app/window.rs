use glfw::GlfwReceiver;

pub struct WindowDescriptor {
    pub title: &'static str,
    pub width: u32,
    pub height: u32,
    pub mode: WindowMode,
}

impl Default for WindowDescriptor {
    fn default() -> Self {
        Self {
            title: "Blank Game",
            width: 800,
            height: 600,
            mode: WindowMode::Windowed,
        }
    }
}

pub enum WindowMode {
    Windowed,
    Fullscreen,
}

pub type Events = GlfwReceiver<(f64, WindowEvent)>;

pub use glfw::{WindowEvent, Modifiers, Action, Scancode, Key, MouseButton};