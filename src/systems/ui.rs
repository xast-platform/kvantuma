use flecs_ecs::prelude::*;
use xastge::{
    Transform, render::{RenderDevice, mesh::Mesh, registry::RenderRegistry, updated}, ui::{atlas::{FontHandle, GlyphVertex}, material::TextMaterial}, utils::Color,
};
use glam::{Vec2, Vec3, Quat};

use crate::{Tween, ui::components::{KirText, UiPosition}};

pub fn render_ui_text(
    world: &World,
    registry: &mut RenderRegistry,
    font: FontHandle,
    font_size: u32,
    render_device: &mut RenderDevice,
) {
    world.query::<(&KirText, &UiPosition)>()
        .build()
        .each_entity(|entity, (text, pos)| {
            entity.get::<(Option<&Mesh<GlyphVertex>>, Option<&TextMaterial>)>(|(mesh_opt, material_opt)| {
                if mesh_opt.is_none() || material_opt.is_none() {
                    let atlas = registry.get_atlas(font, font_size).unwrap();
                    let mesh = atlas.generate_mesh(&text.value, Vec2::ZERO, 1.0);

                    entity
                        .set(Tween::<Color>::new(Color::WHITE, Color::RED, 0.1))
                        .set(TextMaterial::new(Color::WHITE, atlas.texture(), render_device, registry))
                        .set(updated(mesh, render_device, registry));
                }
            });
            
            entity.set(Transform {
                translation: Vec3::new(pos.x, pos.y, 0.0),
                scale: Vec3::ONE,
                rotation: Quat::IDENTITY,
            });
        });
}

pub fn update_ui_positions(world: &World) {
    world.query::<(&UiPosition, &mut Transform)>()
        .build()
        .each(|(pos, transform)| {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        });
}