use flecs_ecs::macros::Component;
use slotmap::new_key_type;

use crate::menu::main_menu::MainMenuData;

#[derive(Component)]
pub enum GameState {
    MainMenu(MainMenuData),
    LoadingLevel(Level),
    LoadedLevel(Level)
}

pub struct Level(pub SceneHandle);

new_key_type! {
    pub struct SceneHandle;
}