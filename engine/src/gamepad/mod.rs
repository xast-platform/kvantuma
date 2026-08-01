use std::{
    collections::HashMap,
    fs, io,
    hash::Hash,
    path::Path,
};

use flecs_ecs::{core::flecs::Singleton, prelude::*};
use gilrs::{Gilrs, GilrsBuilder};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use gilrs::{Axis as GamepadAxis, Button as GamepadButton, GamepadId};

/// Buttons of a MaxFire G-12U joystick; `gilrs` can't name these, see [`GamepadBindings`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaxFireButton {
    /// South
    Key1,
    /// East
    Key2,
    /// West
    Key3,
    /// North
    Key4,

    /// Left top bumper
    Key5,
    /// Right top bumper
    Key6,
    /// Left bottom bumper
    Key7,
    /// Right bottom bumper
    Key8,

    /// Center left
    Key9,
    /// Center right
    Key10,
    Mode,

    LeftJoystick,
    RightJoystick,

    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

/// The two analog sticks of a MaxFire G-12U joystick; these do resolve through `gilrs` normally.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaxFireAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
}

impl From<MaxFireAxis> for GamepadAxis {
    fn from(axis: MaxFireAxis) -> Self {
        use MaxFireAxis::*;

        match axis {
            LeftStickX => GamepadAxis::LeftStickX,
            LeftStickY => GamepadAxis::LeftStickY,
            RightStickX => GamepadAxis::RightStickX,
            RightStickY => GamepadAxis::RightStickY,
        }
    }
}

/// Persisted `B -> Code` bindings for a device whose buttons `gilrs` can't name, captured once via [`Gamepads::begin_calibration`]/[`Gamepads::take_calibrated_code`] and loadable per device profile (e.g. `maxfire.ron`, `othershit.ron`).
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct GamepadBindings<B: Eq + Hash + Send + Sync + 'static> {
    codes: HashMap<B, u32>,
}

impl<B: Eq + Hash + Send + Sync + 'static> Default for GamepadBindings<B> {
    fn default() -> Self {
        Self { codes: HashMap::new() }
    }
}

impl<B: Copy + Eq + Hash + Send + Sync + 'static + Serialize + for<'de> Deserialize<'de>>
    GamepadBindings<B>
{
    pub fn load(path: impl AsRef<Path>) -> Result<Self, GamepadBindingsError> {
        let text = fs::read_to_string(path)?;
        Ok(ron::from_str(&text)?)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), GamepadBindingsError> {
        let text = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())?;
        fs::write(path, text)?;
        Ok(())
    }

    pub fn bind(&mut self, button: B, code: u32) {
        self.codes.insert(button, code);
    }

    pub fn is_bound(&self, button: B) -> bool {
        self.codes.contains_key(&button)
    }
}

#[derive(Debug, Error)]
pub enum GamepadBindingsError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("Failed to parse bindings: {0}")]
    Deserialize(#[from] ron::error::SpannedError),
    #[error("Failed to serialize bindings: {0}")]
    Serialize(#[from] ron::Error),
}

/// Singleton resource wrapping `gilrs`; pumped once a frame by [`GamepadModule`].
#[derive(Component)]
pub struct Gamepads {
    gilrs: Mutex<Gilrs>,
    calibrating: Mutex<Option<GamepadId>>,
    captured: Mutex<Option<u32>>,
}

impl Gamepads {
    pub fn new() -> Result<Self, gilrs::Error> {
        Ok(Self {
            gilrs: Mutex::new(GilrsBuilder::new().build()?),
            calibrating: Mutex::new(None),
            captured: Mutex::new(None),
        })
    }

    /// Arms calibration: the next button pressed on `id` is captured, see [`Gamepads::take_calibrated_code`].
    pub fn begin_calibration(&self, id: GamepadId) {
        *self.calibrating.lock() = Some(id);
        *self.captured.lock() = None;
    }

    /// The code captured since [`Gamepads::begin_calibration`], if a button has been pressed yet.
    pub fn take_calibrated_code(&self) -> Option<u32> {
        self.captured.lock().take()
    }

    /// Drains pending OS events so `gilrs`'s internal per-gamepad state is current. Called once a frame by [`GamepadModule`].
    pub(crate) fn poll_events(&self) {
        let mut gilrs = self.gilrs.lock();
        let calibrating = *self.calibrating.lock();

        while let Some(event) = gilrs.next_event() {
            if let (Some(target_id), gilrs::EventType::ButtonPressed(_, code)) =
                (calibrating, event.event)
                && event.id == target_id
            {
                *self.captured.lock() = Some(code.into_u32());
                *self.calibrating.lock() = None;
            }
        }
    }

    /// Whether `button` is held, resolved through its calibrated [`GamepadBindings`] entry. Returns `false` if disconnected or unbound rather than panicking.
    pub fn is_bound_pressed<B: Copy + Eq + Hash + Send + Sync + 'static>(
        &self,
        id: GamepadId,
        bindings: &GamepadBindings<B>,
        button: B,
    ) -> bool {
        let Some(&target) = bindings.codes.get(&button) else {
            return false;
        };

        let gilrs = self.gilrs.lock();
        let Some(gamepad) = gilrs.connected_gamepad(id) else {
            return false;
        };

        gamepad
            .state()
            .buttons()
            .find(|(code, _)| code.into_u32() == target)
            .is_some_and(|(_, data)| data.is_pressed())
    }

    pub fn is_pressed(&self, id: GamepadId, button: impl Into<GamepadButton>) -> bool {
        self.gilrs.lock().gamepad(id).is_pressed(button.into())
    }

    pub fn value(&self, id: GamepadId, axis: impl Into<GamepadAxis>) -> f32 {
        self.gilrs.lock().gamepad(id).value(axis.into())
    }

    /// IDs of every gamepad currently connected.
    pub fn connected_ids(&self) -> Vec<GamepadId> {
        self.gilrs.lock().gamepads().map(|(id, _)| id).collect()
    }

    /// First connected gamepad, if any.
    pub fn first_connected(&self) -> Option<GamepadId> {
        self.gilrs.lock().gamepads().next().map(|(id, _)| id)
    }
}

#[derive(Component)]
pub struct GamepadModule;

impl Module for GamepadModule {
    fn module(world: &World) {
        world.component::<Gamepads>().add_trait::<Singleton>();
        world.set(Gamepads::new().expect("Failed to initialize gilrs"));

        world.system::<&Gamepads>()
            .kind(crate::Update)
            .each(Gamepads::poll_events);
    }
}
