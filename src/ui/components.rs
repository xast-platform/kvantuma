use flecs_ecs::{core::{Entity, World}, macros::Component};
use xastge::Transform;

#[derive(Component)]
pub struct UiPosition {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Component)]
pub struct EcsRow {
    pub children: Vec<Entity>,
}

#[derive(Component)]
pub struct EcsCol {
    pub col_number: u8,
    pub children: Vec<Entity>,
}

#[derive(Component)]
pub struct EcsText {
    pub value: String,
}

pub fn col(world: &World, num: u8, entities: &[Entity]) -> Entity {
    *world.entity()
        .set(EcsCol {
            col_number: num,
            children: entities.to_vec(),
        })
}

pub fn text(world: &World, value: &str) -> Entity {
    *world.entity()
        .set(EcsText {
            value: value.to_owned(),
        })
        .set(Transform::default())
}

pub fn row(world: &World, entities: &[Entity]) -> Entity {
    *world.entity()
        .set(EcsRow {
            children: entities.to_vec(),
        })
}


#[macro_export]
macro_rules! ui {
    ($world:expr, row { $($inner:tt)* }) => {
        $crate::ui::components::row($world, &ui!(@list $world, $($inner)*))
    };

    ($world:expr, col ( $num:expr ) { $($inner:tt)* }) => {
        $crate::ui::components::col($world, $num, &ui!(@list $world, $($inner)*))
    };

    ($world:expr, text ( $value:expr )) => {
        $crate::ui::components::text($world, $value)
    };

    // LIST BUILDER
    (@list $world:expr,) => {
        Vec::<Entity>::new()
    };

    (@list $world:expr, text ( $value:expr ) $($rest:tt)*) => {{
        let mut v: Vec<Entity> = vec![$crate::ui::components::text($world, $value)];
        v.extend(ui!(@list $world, $($rest)*));
        v
    }};

    (@list $world:expr, col ( $num:expr ) { $($inner:tt)* } $($rest:tt)*) => {{
        let mut v: Vec<Entity> = vec![$crate::ui::components::col($world, $num, &ui!(@list $world, $($inner)*))];
        v.extend(ui!(@list $world, $($rest)*));
        v
    }};

    (@list $world:expr, row { $($inner:tt)* } $($rest:tt)*) => {{
        let mut v: Vec<Entity> = vec![$crate::ui::row($world, &ui!(@list $world, $($inner)*))];
        v.extend(ui!(@list $world, $($rest)*));
        v
    }};
}