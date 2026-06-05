use flecs_ecs::{core::{Entity, World}, macros::Component};
use serde::{Deserialize, Serialize};
use xastge::{Transform, render::texture::TextureHandle};

#[derive(Component)]
pub struct UiPosition {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Component)]
pub struct KirRow {
    pub children: Vec<Entity>,
}

#[derive(Component)]
pub struct KirCol {
    pub col_number: u8,
    pub children: Vec<Entity>,
}

#[derive(Component)]
pub struct KirText {
    pub value: String,
}

#[derive(Component)]
pub struct KirImage {
    pub image: TextureHandle,
}

#[derive(Component)]
pub struct KirButton {
    pub nine_patch: NinePatch,
}

pub struct NinePatch {
    pub texture: TextureHandle,
    pub config: NinePatchConfig,
}

/// ```text
///            left_top_w   center_top_w  right_top_w               
///                    │        │          │                        
///                    │        │          │                        
///                 ┌──▼─────┐ ┌▼┐ ┌───────▼─┐                      
///                 │        │ │ │ │         │                      
///                 │        │ │ │ │         ◄───────right_top_h    
///                 │        │ │ │ │         │                      
///                 │        │ │ │ │         │                      
///                 └────────┘ └─┘ └─────────┘                      
/// left_middle_w───────┐       ┌────────────────────center_middle_w
///                 ┌───▼────┐ ┌▼┐ ┌─────────┐                      
///                 │        │ │ │ │         ◄───────right_middle_h 
///                 └────────┘ └─┘ └────▲────┘                      
/// left_bottom_w───────┐               └────────────right_middle_w 
///                 ┌───▼────┐ ┌─┐ ┌─────────┐                      
///                 │        │ │ │ │         │                      
///                 │        │ │ │ │         ◄───────right_bottom_h 
///                 │        │ │ │ │         │                      
///                 │        │ │ │ │         │                      
///                 └────────┘ └▲┘ └────▲────┘                      
/// center_bottom_w─────────────┘       └────────────right_bottom_w 
/// ```
#[derive(Serialize, Deserialize)]
pub struct NinePatchConfig {
    pub left_top_w: u16,
    pub center_top_w: u16,
    pub right_top_w: u16,
    pub right_top_h: u16,

    pub left_middle_w: u16,
    pub center_middle_w: u16,
    pub right_middle_w: u16,
    pub right_middle_h: u16,

    pub left_bottom_w: u16,
    pub center_bottom_w: u16,
    pub right_bottom_w: u16,
    pub right_bottom_h: u16,
}

pub fn col(world: &World, num: u8, entities: &[Entity]) -> Entity {
    *world.entity()
        .set(KirCol {
            col_number: num,
            children: entities.to_vec(),
        })
}

pub fn text(world: &World, value: &str) -> Entity {
    *world.entity()
        .set(KirText {
            value: value.to_owned(),
        })
        .set(Transform::default())
}

pub fn row(world: &World, entities: &[Entity]) -> Entity {
    *world.entity()
        .set(KirRow {
            children: entities.to_vec(),
        })
}

pub fn image(world: &World, image: TextureHandle) -> Entity {
    *world.entity()
        .set(KirImage {
            image,
        })
        .set(Transform::default())
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

    ($world:expr, image ( $image:expr )) => {
        $crate::ui::components::image($world, $image)
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