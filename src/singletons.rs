use flecs_ecs::core::World;
use xastge::ui::atlas::FontHandle;

use crate::{MainFont, MouseState, MovementInput, game::GameState, menu::main_menu::{MainMenuButton, MainMenuData}};

pub fn init_singletons(world: &mut World, font: FontHandle) {
    world.set(MainFont(font));
    world.set(MouseState::new(true));
    world.set(MovementInput::default());
    world.set(GameState::MainMenu(MainMenuData::Home { 
        heading: "KVANTUMA".to_owned(), 
        buttons: vec![
            MainMenuButton {
                text: "New game".to_owned(),
            }
        ], 
        hovered_button: -1,
    }));
}