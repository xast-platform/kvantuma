use flecs_ecs::core::{EntityView, World};
use serde::{Deserialize, Serialize};

#[typetag::serde(tag = "component")]
pub trait SerializableComponent {
    fn add_into(self: Box<Self>, entity: EntityView) -> EntityView;
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename = "Entity")]
pub struct SerializableEntity {
    pub components: Vec<Box<dyn SerializableComponent>>
}

#[derive(Default, Serialize, Deserialize)]
pub struct Scene {
    pub entities: Vec<SerializableEntity>,
}

pub trait SpawnSceneExt {
    fn spawn_scene(&self, scene: Scene);
}

impl SpawnSceneExt for World {
    fn spawn_scene(&self, scene: Scene) {
        for entity in scene.entities {
            entity.components
                .into_iter()
                .fold(self.entity(), |entity_view, component| {
                    component.add_into(entity_view)
                });
        }
    }
}

fn x() {
    let scene = Scene {
        entities: vec![
            SerializableEntity {
                components: vec![
                    Box::new(12),
                    Box::new(true),
                ],
            },
            SerializableEntity {
                components: vec![
                    Box::new(12),
                    Box::new(true),
                ],
            }
        ],
    };

    let world = World::new();
    world.spawn_scene(scene);
}

#[macro_export]
macro_rules! impl_ser_component {
    ($($comp:ty => $name:literal),+ $(,)?) => {
        $(
            #[typetag::serde(name = $name)]
            impl $crate::scene::SerializableComponent for $comp {
                fn add_into(self: Box<Self>, entity: EntityView) -> EntityView {
                    entity.set(*self)
                }
            }
        )+
    }
}

impl_ser_component! {
    i8  => "byte",
    u8  => "unsigned_byte",
    i16 => "short",
    u16 => "unsigned_short",
    i32 => "int",
    u32 => "unsigned_int",
    i64 => "long",
    u64 => "unsigned_long",
    f32 => "float",
    f64 => "double",
    bool => "boolean",
}