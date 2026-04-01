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

pub struct WindowController<'a> {
    window: &'a mut PWindow,
}

impl<'a> WindowController<'a> {
    pub fn new(window: &'a mut PWindow) -> Self {
        Self { window }
    }

    pub fn set_cursor_mode(&mut self, mode: glfw::CursorMode) {
        self.window.set_cursor_mode(mode);
    }
}

pub enum WindowMode {
    Windowed,
    Fullscreen,
}

pub type Events = GlfwReceiver<(f64, WindowEvent)>;

pub use glfw::{Action, CursorMode, Key, Modifiers, MouseButton, Scancode, WindowEvent};