use flecs_ecs::prelude::*;
use xastge::{
    Transform,
    render::{RenderDevice, registry::RenderRegistry, updated},
    ui::{atlas::FontHandle, material::TextMaterial},
};
use glam::{Vec2, Vec3, Quat};

use crate::ui::components::{EcsText, UiPosition};

pub fn render_ui_text(
    world: &World,
    registry: &mut RenderRegistry,
    font: FontHandle,
    font_size: u32,
    render_device: &mut RenderDevice,
) {
    world.query::<(&EcsText, &UiPosition)>()
        .build()
        .each_entity(|entity, (text, pos)| {
            let atlas = registry.get_atlas(font, font_size).unwrap();
            let mesh = atlas.generate_mesh(&text.value, Vec2::ZERO, 1.0);
            
            entity
                .set(TextMaterial { atlas: atlas.texture() })
                .set(updated(mesh, render_device, registry))
                .set(Transform {
                    translation: Vec3::new(pos.x, pos.y, 0.0),
                    scale: Vec3::ONE,
                    rotation: Quat::IDENTITY,
                });
        });
}