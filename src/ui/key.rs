use flecs_ecs::macros::Component;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Component)]
pub enum Screen {
    MainMenu,
}