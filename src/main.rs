use log::LevelFilter;
use xastge::{
    Transform, 
    app::{
        App, Game,
        window::{
            Action, CursorMode, Key, MouseButton, WindowController, WindowDescriptor, WindowEvent, WindowMode,
        },
    }, 
    render::{
        RenderDevice, RenderSurface, 
        camera::{Camera, CameraBuffer, OrthographicCamera, PerspectiveCamera}, 
        error::RenderError, 
        material::{ColorMaterial, SkyboxMaterial},
        mesh::{Mesh, Vertex}, 
        pass::DrawDescriptor,
        registry::RenderRegistry,
        texture::TextureDescriptor, 
        types::*,
        updated,
    }, 
    ui::{
        atlas::{FontHandle, GlyphVertex},
        glyph::FontRef,
        material::TextMaterial,
    },
};

pub type KvUiManager = UiManager<ScreenKey>;

pub mod game;
pub mod menu;
pub mod systems;
pub mod ui;
pub mod singletons;

use glam::{EulerRot, Quat, Vec2, Vec3};
use flecs_ecs::prelude::*;

use crate::{
    singletons::init_singletons, 
    systems::{
        camera::update_camera_buffer,
        ui::render_ui_text,
    },
    ui::{Ui, UiManager, UiScreen, key::ScreenKey},
};

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

#[derive(Component)]
pub struct SkyboxTag;

struct KvantumaGame {
    registry: RenderRegistry,
    ort_cam_id: Entity,
    persp_cam_id: Entity,
    ui_manager: KvUiManager,
}

impl Game for KvantumaGame {
    fn init(&mut self, world: &mut World, render_device: &mut RenderDevice) -> anyhow::Result<()> {
        self.register_materials(render_device);

        let font = self.registry.new_font(FontRef::try_from_slice(include_bytes!("../assets/fonts/KVANTUMA1451.ttf"))?);
        for size in [8, 12, 18, 24, 36, 48, 64, 72] {
            self.registry.add_font_atlas(
                render_device, 
                font, 
                size,
            );
        }

        init_singletons(world, font);
        self.init_skybox(world, render_device)?;
        self.ort_cam_id = self.init_ort_camera(world, render_device)?;
        self.persp_cam_id = self.init_persp_camera(world, render_device)?;

        let test_ui = MyUi;
        let ui_root = test_ui.build_ui(world);
        self.ui_manager.add_screen(ScreenKey::MainMenu, UiScreen::new(ui_root));
        self.ui_manager.set_screen(ScreenKey::MainMenu);

        Ok(())
    }

    fn update(&mut self, world: &mut World) -> anyhow::Result<()> {
        self.movement_system(world);
        
        Ok(())
    }

    fn input(
        &mut self,
        event: &WindowEvent,
        world: &mut World,
        window: &mut WindowController<'_>,
    ) -> anyhow::Result<()> {
        match event {
            WindowEvent::FramebufferSize(width, height) => self.resize(world, width, height),
            WindowEvent::CursorPos(x, y) => self.rotate_cam(world, x, y),
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => self.process_escape(world, window),
            WindowEvent::Key(key, _, action, _) => self.process_keys(world, key, action),
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => self.process_mouse(world, window),
            WindowEvent::Close => { /* save later */ },
            _ => {},
        }

        Ok(())
    }

    fn render(&mut self, world: &mut World, render_device: &mut RenderDevice) -> Result<(), RenderError> {
        if self.ui_manager.is_dirty() {
            let screen_width = self.ui_manager.screen_width();
            let screen_height = self.ui_manager.screen_height();
            
            if let Some(screen) = self.ui_manager.get_current_screen_mut() {
                screen.recompute_layout(world, screen_width, screen_height);
                screen.apply_layout_to_entities(world);
            }
            
            world.get::<&MainFont>(|font| {
                render_ui_text(world, &mut self.registry, font.0, 24, render_device);
            });
            
            self.ui_manager.mark_clean();
        }

        update_camera_buffer(world, render_device, &self.registry);
        
        let canvas = render_device.canvas()?;
        let canvases: &[&dyn RenderSurface] = &[&canvas];
        let mut ctx = render_device.draw_ctx();

        self.render_skybox(world, render_device, canvases, &mut ctx);
        self.render_color(world, render_device, canvases, &mut ctx);
        self.render_ui_text(world, render_device, canvases, &mut ctx);

        ctx.apply(canvas, render_device);

        Ok(())
    }
}

impl KvantumaGame {
    fn process_escape(&self, world: &mut World, window: &mut WindowController<'_>) {
        world.get::<&mut MouseState>(|mouse| {
            if mouse.captured {
                window.set_cursor_mode(CursorMode::Normal);
                mouse.captured = false;
                mouse.last_pos = None;
            }
        });
    }

    fn process_mouse(&self, world: &mut World, window: &mut WindowController<'_>) {
        world.get::<&mut MouseState>(|mouse| {
            if !mouse.captured {
                window.set_cursor_mode(CursorMode::Disabled);
                mouse.captured = true;
                mouse.last_pos = None;
            }
        });
    }

    fn process_keys(&self, world: &mut World, key: &Key, action: &Action) {
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
    }

    fn rotate_cam(&self, world: &mut World, x: &f64, y: &f64) {
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
    }

    fn register_materials(&mut self, render_device: &mut RenderDevice) {
        let camera_layout = CameraBuffer::layout(render_device);

        self.registry.register_material::<TextMaterial>(render_device, &[&camera_layout]);
        self.registry.register_material::<ColorMaterial>(render_device, &[&camera_layout]);
        self.registry.register_material::<SkyboxMaterial>(render_device, &[&camera_layout]);
    }

    fn render_skybox(&mut self, world: &mut World, render_device: &mut RenderDevice, canvases: &[&(dyn RenderSurface + 'static)], ctx: &mut xastge::render::draw_context::DrawContext) {
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
    }

    fn render_color(&mut self, world: &mut World, render_device: &mut RenderDevice, canvases: &[&(dyn RenderSurface + 'static)], ctx: &mut xastge::render::draw_context::DrawContext) {
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
    }

    fn render_ui_text(&mut self, world: &mut World, render_device: &mut RenderDevice, canvases: &[&(dyn RenderSurface + 'static)], ctx: &mut xastge::render::draw_context::DrawContext) {
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
    }

    fn resize(&mut self, world: &mut World, width: &i32, height: &i32) {
        let w = *width as f32;
        let h = *height as f32;
        world.each::<(&mut OrthographicCamera, &Camera)>(|(ort_cam, _cam)| {
            ort_cam.resize_viewport(w, h);
        });
        world.each::<(&mut PerspectiveCamera, &Camera)>(|(persp_cam, _cam)| {
            persp_cam.set_aspect(w / h);
        });
        self.ui_manager.recompute_layout(world, w, h);
    }

    fn init_skybox(
        &mut self,
        world: &World,
        render_device: &mut RenderDevice,
    ) -> anyhow::Result<()> {
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

        Ok(())
    }

    fn init_ort_camera(
        &mut self,
        world: &World,
        render_device: &mut RenderDevice,
    ) -> anyhow::Result<Entity> {
        let size = render_device.size();

        Ok(
            world.entity()
                .set(OrthographicCamera::from_viewport(size.x as f32, size.y as f32))
                .set(Camera::default())
                .set(Transform {
                    translation: Vec3::new(0.0, 0.0, 1.0),
                    ..Default::default()
                })
                .set(CameraBuffer::new(render_device, &mut self.registry, &CameraBuffer::layout(render_device)))
                .id()
        )
    }

    fn init_persp_camera(
        &mut self,
        world: &World,
        render_device: &mut RenderDevice,
    ) -> anyhow::Result<Entity> {
        let size = render_device.size();

        Ok(
            world.entity()
                .set(PerspectiveCamera::from_aspect(size.x as f32 / size.y as f32))
                .set(Camera::default())
                .set(Transform {
                    translation: Vec3::new(5.0, 5.0, 5.0),
                    ..Default::default()
                })
                .set(CameraBuffer::new(render_device, &mut self.registry, &CameraBuffer::layout(render_device)))
                .set(FpsCamera {
                    yaw: 0.0,
                    pitch: 0.0,
                    sensitivity: 0.002,
                    move_speed: 0.08,
                })
                .id()
        )
    }

    fn movement_system(
        &self,
        world: &World,
    ) {
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

        world.query::<&mut Transform>()
            .with(SkyboxTag)
            .build()
            .each(|t| {
                t.translation = camera_translation;
            });
    }
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
            ui_manager: KvUiManager::new(1024.0, 1024.0),
        },
    )?.run();

    Ok(())
}

pub struct MyUi;


impl Ui for MyUi {
    fn build_ui(&self, world: &mut World) -> Entity {
        ui!(world,
            row {
                col (6) {
                    text("A")
                    text("B")
                }
                col (6) {
                    text("C")
                    text("D")
                    text("E")
                }
            }
        )
    }
}