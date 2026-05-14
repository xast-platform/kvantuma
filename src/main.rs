use log::LevelFilter;
use xastge::{
    Transform, 
    app::{
        App, Game,
        window::{Action, CursorMode, Key, MouseButton, WindowController, WindowDescriptor, WindowEvent, WindowMode},
    }, 
    render::{
        RenderDevice, RenderSurface, 
        camera::{Camera, CameraBuffer, OrthographicCamera, PerspectiveCamera}, 
        error::RenderError, 
        include_wgsl, 
        material::{ColorMaterial, SkyboxMaterial},
        mesh::{Mesh, Vertex}, 
        pass::DrawDescriptor,
        registry::RenderRegistry,
        texture::TextureDescriptor, 
        types::*, updated,
    }, 
    ui::{
        atlas::{FontHandle, GlyphVertex}, 
        glyph::FontRef, material::TextMaterial,
    },
};

pub mod game;
pub mod menu;
pub mod systems;
pub mod ui;
pub mod singletons;

use glam::{EulerRot, Quat, Vec2, Vec3};
use flecs_ecs::prelude::*;

use crate::{game::GameState, singletons::init_singletons, systems::camera::update_camera_buffer, ui::row};

#[derive(Component)]
pub struct FpsCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub sensitivity: f32,
    pub move_speed: f32,
}

impl FpsCamera {
    pub fn forward(&self) -> Vec3 {
        (Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0) * Vec3::NEG_Z)
            .normalize()
    }
}

#[derive(Component, Default)]
pub struct MouseState {
    pub last_pos: Option<Vec2>,
    pub captured: bool,
}

impl MouseState {
    pub fn new(captured: bool) -> Self {
        Self {
            last_pos: None,
            captured,
        }
    }
}

#[derive(Component, Default, Clone, Copy)]
pub struct MovementInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
}

#[derive(Component)]
pub struct MainFont(pub FontHandle);

struct KvantumaGame {
    registry: RenderRegistry,
    ort_cam_id: Entity,
    persp_cam_id: Entity,
    ui_manager: UiManager,
}

#[derive(Component)]
struct SkyboxTag;

impl Game for KvantumaGame {
    fn init(&mut self, world: &mut World, render_device: &mut RenderDevice) -> anyhow::Result<()> {
        let camera_layout = CameraBuffer::layout(render_device);
        let size = render_device.size();

        self.registry.register_material::<TextMaterial>(render_device, &[&camera_layout]);
        self.registry.register_material::<ColorMaterial>(render_device, &[&camera_layout]);
        self.registry.register_material::<SkyboxMaterial>(render_device, &[&camera_layout]);

        let font = self.registry.new_font(FontRef::try_from_slice(include_bytes!("../assets/fonts/KVANTUMA1451.ttf"))?);
        
        for size in 10..=72 {
            self.registry.add_font_atlas(
                render_device, 
                font, 
                size,
            );
        }
        
        // Initialize UI
        use crate::ui::{text, button, UiScreen, col};
        self.ui_manager.main_menu_screen = Some(UiScreen::new(
            row(vec![
                col(3, vec![]),
                col(6, vec![
                    text("KVΛNTUMA".to_owned()),
                    button("New Game".to_owned(), None),
                    button("Continue".to_owned(), None),
                    button("Settings".to_owned(), None),
                    button("Quit".to_owned(), None),
                ]),
            ]),
            self.ui_manager.screen_width,
            self.ui_manager.screen_height,
        ));

        world.entity()
            .set(updated(
                Mesh::load_obj("./assets/meshes/cube.obj"), 
                render_device, 
                &mut self.registry,
            ))
            .set(ColorMaterial::new(
                Vec3::new(0.0, 0.0, 1.0), 
                render_device, 
                &mut self.registry,
            ))
            .set(Transform::default());

        // SKYBOX
        world.entity()
            .set(updated(
                Mesh::load_obj("./assets/meshes/cube.obj"),
                render_device,
                &mut self.registry,
            ))
            .set(SkyboxMaterial::new(self.registry.load_cubemap(
                render_device,
                [
                    "./assets/textures/skyboxes/sky1_cubemap_faces/right_cubemap.png",
                    "./assets/textures/skyboxes/sky1_cubemap_faces/left_cubemap.png",
                    "./assets/textures/skyboxes/sky1_cubemap_faces/top_cubemap.png",
                    "./assets/textures/skyboxes/sky1_cubemap_faces/bottom_cubemap.png",
                    "./assets/textures/skyboxes/sky1_cubemap_faces/front_cubemap.png",
                    "./assets/textures/skyboxes/sky1_cubemap_faces/back_cubemap.png",
                ],
                TextureDescriptor::default(),
            )?))
            .add(SkyboxTag)
            .set(Transform {
                translation: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                scale: Vec3::splat(200.0),
            });

        // ORT CAMERA
        self.ort_cam_id = world.entity()
            .set(OrthographicCamera::from_viewport(size.x as f32, size.y as f32))
            .set(Camera::default())
            .set(Transform {
                translation: Vec3::new(0.0, 0.0, 1.0),
                ..Default::default()
            })
            .set(CameraBuffer::new(render_device, &mut self.registry, &camera_layout))
            .id();

        // PERSP CAMERA
        self.persp_cam_id = world.entity()
            .set(PerspectiveCamera::from_aspect(size.x as f32 / size.y as f32))
            .set(Camera::default())
            .set(Transform {
                translation: Vec3::new(5.0, 5.0, 5.0),
                ..Default::default()
            })
            .set(CameraBuffer::new(render_device, &mut self.registry, &camera_layout))
            .set(FpsCamera {
                yaw: 0.0,
                pitch: 0.0,
                sensitivity: 0.002,
                move_speed: 0.08,
            })
            .id();

        // SINGLETONS
        init_singletons(world, font);

        Ok(())
    }

    fn update(&mut self, world: &mut World) -> anyhow::Result<()> {
        let mut movement = MovementInput::default();
        world.get::<&MovementInput>(|input| {
            movement = *input;
        });

        world.each::<(&mut Transform, &FpsCamera, &Camera)>(|(t, fps, _)| {
            let rotation = Quat::from_euler(EulerRot::YXZ, fps.yaw, fps.pitch, 0.0);
            t.rotation = rotation;

            let forward = (rotation * Vec3::NEG_Z).normalize();
            let right = (rotation * Vec3::X).normalize();

            let mut direction = Vec3::ZERO;
            if movement.forward {
                direction += forward;
            }
            if movement.backward {
                direction -= forward;
            }
            if movement.left {
                direction -= right;
            }
            if movement.right {
                direction += right;
            }

            if direction.length_squared() > 0.0 {
                t.translation += direction.normalize() * fps.move_speed;
            }
        });

        let mut camera_translation = Vec3::ZERO;
        world
            .entity_from_id(self.persp_cam_id)
            .get::<&Transform>(|cam_t| {
                camera_translation = cam_t.translation;
            });

        world
            .query::<&mut Transform>()
            .with(SkyboxTag)
            .build()
            .each(|t| {
                t.translation = camera_translation;
            });
        
        Ok(())
    }

    fn input(
        &mut self,
        event: &WindowEvent,
        world: &mut World,
        window: &mut WindowController<'_>,
    ) -> anyhow::Result<()> {
        match event {
            WindowEvent::FramebufferSize(width, height) => {
                let w = *width as f32;
                let h = *height as f32;
                
                world.each::<(&mut OrthographicCamera, &Camera)>(|(ort_cam, _cam)| {
                    ort_cam.resize_viewport(w, h);
                });

                world.each::<(&mut PerspectiveCamera, &Camera)>(|(persp_cam, _cam)| {
                    persp_cam.set_aspect(w / h);
                });
                
                // Update UI layout for new screen size
                self.ui_manager.screen_width = w;
                self.ui_manager.screen_height = h;
                if let Some(screen) = &mut self.ui_manager.main_menu_screen {
                    screen.recompute_layout(w, h);
                }
            },
            WindowEvent::CursorPos(x, y) => {
                let current = Vec2::new(*x as f32, *y as f32);

                world.get::<&mut MouseState>(|mouse| {
                    if !mouse.captured {
                        return;
                    }

                    if let Some(last) = mouse.last_pos {
                        let delta = current - last;

                        world.each::<(&mut FpsCamera,)>(|(fps,)| {
                            fps.yaw   -= delta.x * fps.sensitivity;
                            fps.pitch -= delta.y * fps.sensitivity;

                            fps.pitch = fps.pitch.clamp(-1.54, 1.54);
                        });
                    }

                    mouse.last_pos = Some(current);
                });
            },
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                world.get::<&mut MouseState>(|mouse| {
                    if mouse.captured {
                        window.set_cursor_mode(CursorMode::Normal);
                        mouse.captured = false;
                        mouse.last_pos = None;
                    }
                });
            },
            WindowEvent::Key(key, _, action, _) => {
                world.get::<&mut MovementInput>(|input| {
                    let pressed = matches!(*action, Action::Press | Action::Repeat);

                    match *key {
                        Key::W => input.forward = pressed,
                        Key::S => input.backward = pressed,
                        Key::A => input.left = pressed,
                        Key::D => input.right = pressed,
                        _ => {}
                    }
                });
            },
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                world.get::<&mut MouseState>(|mouse| {
                    if !mouse.captured {
                        window.set_cursor_mode(CursorMode::Disabled);
                        mouse.captured = true;
                        mouse.last_pos = None;
                    }
                });
            },
            WindowEvent::Close => {
                // save later
            },
            _ => {},
        }

        Ok(())
    }

    fn render(&mut self, world: &mut World, render_device: &mut RenderDevice) -> Result<(), RenderError> {
        world.get::<&GameState>(|state| {
            ui_system(state, &mut self.ui_manager, world);
        });
        
        world.get::<&MainFont>(|font| {
            text_rendering_system(world, &mut self.registry, font, render_device);
        });

        update_camera_buffer(world, render_device, &self.registry);
        
        let canvas = render_device.canvas()?;
        let canvases: &[&dyn RenderSurface] = &[&canvas];
        let mut ctx = render_device.draw_ctx();

        world.each::<(&Mesh<Vertex>, &SkyboxMaterial, &Transform)>(|(mesh, mat, t)| {
            world
                .entity_from_id(self.persp_cam_id)
                .get::<(&CameraBuffer, &PerspectiveCamera)>(|(cam_buffer, _)| {
                    let mut render_pass = ctx.render_pass(
                        canvases,
                        render_device.depth_texture(),
                        Operations {
                            load: LoadOp::Clear(Color::BLACK),
                            store: StoreOp::Store,
                        },
                    );
                    render_pass.draw(render_device, &self.registry, DrawDescriptor::<_, _> {
                        drawable: Some(mesh),
                        instance_data: Some(t),
                        global_shader_resources: &[cam_buffer.resource()],
                        material: mat,
                    });
                });
        });

        world.each::<(&Mesh<Vertex>, &ColorMaterial, &Transform)>(|(mesh, mat, t)| {
            world
                .entity_from_id(self.persp_cam_id)
                .get::<(&CameraBuffer, &PerspectiveCamera)>(|(cam_buffer, _)| {
                    let mut render_pass = ctx.render_pass(
                        canvases, 
                        render_device.depth_texture(),
                        Operations {
                            load: LoadOp::Load,
                            store: StoreOp::Store,
                        },
                    );
                    render_pass.draw(render_device, &self.registry, DrawDescriptor::<_, _> {
                        drawable: Some(mesh),
                        instance_data: Some(t),
                        global_shader_resources: &[cam_buffer.resource()],
                        material: mat,
                    });
                });
        });

        world.each::<(&Mesh<GlyphVertex>, &TextMaterial, &Transform)>(|(mesh, mat, t)| {
            world
                .entity_from_id(self.ort_cam_id)
                .get::<(&CameraBuffer, &OrthographicCamera)>(|(ui_cam_buffer, _)| {
                    let mut render_pass = ctx.render_pass(
                        canvases, 
                        render_device.depth_texture(),
                        Operations {
                            load: LoadOp::Load,
                            store: StoreOp::Store,
                        },
                    );
                    render_pass.draw(render_device, &self.registry, DrawDescriptor::<_, _> {
                        drawable: Some(mesh),
                        instance_data: Some(t),
                        global_shader_resources: &[ui_cam_buffer.resource()],
                        material: mat,
                    });
                });
        });

        ctx.apply(canvas, render_device);

        Ok(())
    }
}

pub struct UiManager {
    pub main_menu_screen: Option<crate::ui::UiScreen<()>>,
    pub ui_entities: Vec<Entity>,
    pub screen_width: f32,
    pub screen_height: f32,
}

#[allow(clippy::single_match)]
fn ui_system(
    state: &GameState,
    ui: &mut UiManager,
    world: &World,
) {
    for entity_id in &ui.ui_entities {
        world.entity_from_id(*entity_id).destruct();
    }
    ui.ui_entities.clear();

    match state {
        GameState::MainMenu(_mm_data) => {
            if let Some(screen) = &ui.main_menu_screen {
                let nodes = screen.nodes();
                
                for (node, pos) in nodes {
                    use crate::ui::{UiNode, UiText, UiButton, UiPosition};
                    
                    let screen_pos = Vec2::new(pos.x, ui.screen_height - pos.y);
                    
                    let entity = match node {
                        UiNode::Text { value, .. } => {
                            world.entity()
                                .set(UiText { value: value.to_string(), font_size: 32 })
                                .set(UiPosition { screen_pos })
                        }
                        UiNode::Button { text, .. } => {
                            world.entity()
                                .set(UiButton { text: text.to_string(), font_size: 32 })
                                .set(UiPosition { screen_pos })
                        }
                        _ => continue,
                    };
                    
                    ui.ui_entities.push(entity.id());
                }
            }
        }
        _ => {}
    }
}

fn text_rendering_system(
    world: &World,
    registry: &mut RenderRegistry,
    font: &MainFont,
    render_device: &mut RenderDevice,
) {
    world.query::<(&crate::ui::UiText, &crate::ui::UiPosition)>()
        .build()
        .each_entity(|entity, (ui_text, ui_pos)| {
            let atlas = registry.get_atlas(font.0, ui_text.font_size).unwrap();
            let mesh = atlas.generate_mesh(&ui_text.value, Vec2::ZERO, 1.0);
            
            entity
                .set(TextMaterial { atlas: atlas.texture() })
                .set(updated(mesh, render_device, registry))
                .set(Transform {
                    translation: Vec3::new(ui_pos.screen_pos.x, ui_pos.screen_pos.y, 0.0),
                    scale: Vec3::ONE,
                    rotation: Quat::IDENTITY,
                });
        });
    
    world.query::<(&crate::ui::UiButton, &crate::ui::UiPosition)>()
        .build()
        .each_entity(|entity, (ui_button, ui_pos)| {
            let atlas = registry.get_atlas(font.0, ui_button.font_size).unwrap();
            let mesh = atlas.generate_mesh(&ui_button.text, Vec2::ZERO, 1.0);
            
            entity
                .set(TextMaterial { atlas: atlas.texture() })
                .set(updated(mesh, render_device, registry))
                .set(Transform {
                    translation: Vec3::new(ui_pos.screen_pos.x, ui_pos.screen_pos.y, 0.0),
                    scale: Vec3::ONE,
                    rotation: Quat::IDENTITY,
                });
        });
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Info)
        .filter_module("wgpu_hal", LevelFilter::Off)
        .init();

    App::new(
        WindowDescriptor {
            width: 1024,
            height: 1024,
            title: "KVΛNTUMA",
            mode: WindowMode::Windowed,
            cursor_mode: CursorMode::Disabled,
        }, 
        KvantumaGame {
            registry: RenderRegistry::new(),
            ort_cam_id: Entity::null(),
            persp_cam_id: Entity::null(),
            ui_manager: UiManager { 
                main_menu_screen: None,
                ui_entities: Vec::new(),
                screen_width: 1024.0,
                screen_height: 1024.0,
            },
        },
    )?.run();

    Ok(())
}
