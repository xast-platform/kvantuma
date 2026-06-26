use std::time::Instant;

use log::LevelFilter;
use xastge::{
    Transform, app::{
        App, Game,
        window::{
            Action, CursorMode, Key, MouseButton, WindowController, WindowDescriptor, WindowEvent, WindowMode,
        },
    }, render::{
        RenderDevice, RenderSurface, 
        camera::{Camera, CameraBuffer, OrthographicCamera, PerspectiveCamera}, 
        error::RenderError, 
        material::{ColorMaterial, ColorUiMaterial, SkyboxMaterial},
        mesh::{Mesh, UiVertex, Vertex}, 
        pass::DrawDescriptor,
        registry::RenderRegistry,
        texture::TextureDescriptor, 
        types::*,
        updated,
    }, ui::{
        atlas::{FontHandle, GlyphVertex},
        glyph::FontRef,
        material::TextMaterial,
    }, utils::{Color, Translation},
};

use flecs::system::System as SystemLabel;

pub type KvUiManager = UiManager<ScreenKey>;

pub mod game;
pub mod menu;
pub mod systems;
pub mod ui;
pub mod singletons;

use glam::{EulerRot, Quat, Vec2, Vec3};
use flecs_ecs::{prelude::*, sys::{ecs_entity_t, ecs_world_t}};

use crate::{
    singletons::init_singletons, 
    systems::{
        camera::update_camera_buffer,
        ui::render_ui_text,
    },
    ui::{Ui, UiManager, UiScreen, components::{KirText, UiPosition}, key::ScreenKey},
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
    current_event: Vec<UiEvent>,
}

pub enum UiEvent {
    Enter(Entity),
    Exit(Entity),
}

#[derive(Component)]
pub struct Hovered;

#[derive(Component)]
pub struct Unhovered;

#[derive(Component)]
pub struct Time {
    delta: f32,
    last_frame: Instant,
}

impl Default for Time {
    fn default() -> Self {
        Time { delta: 0.0, last_frame: Instant::now() }
    }
}

impl Time {
    pub fn new() -> Self {
        Time::default()
    }

    pub fn delta_time(&self) -> f32 {
        self.delta
    }

    pub fn update(&mut self, new: Instant) {
        let dt = (new - self.last_frame).as_secs_f32();
        self.last_frame = new;
        self.delta = dt;
    }
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

        let ui_root = MyUi.build_ui(world);
        self.ui_manager.add_screen(ScreenKey::MainMenu, UiScreen::new(ui_root));
        self.ui_manager.set_screen(ScreenKey::MainMenu);

        init_singletons(world, font);
        self.init_skybox(world, render_device)?;
        self.ort_cam_id = self.init_ort_camera(world, render_device)?;
        self.persp_cam_id = self.init_persp_camera(world, render_device)?;
        
        let size = render_device.size();

        let atlas = self.registry.get_atlas(font, 24).unwrap();
        self.ui_manager.recompute_layout(world, size.x as f32, size.y as f32, atlas);

        log::info!("recompute_layout size = {size:?}");

        Ok(())
    }

    fn update(&mut self, world: &mut World) -> anyhow::Result<()> {
        world.get::<&mut Time>(|time| {
            time.update(Instant::now());
        });

        for event in &self.current_event {
            match event {
                UiEvent::Enter(enter) => {
                    world.entity_from_id(*enter)
                        .add(Hovered)
                        .remove(Unhovered);
                },
                UiEvent::Exit(exit) => {
                    world.entity_from_id(*exit)
                        .add(Unhovered)
                        .remove(Hovered);
                },
            }
        }
        self.current_event.clear();

        world.get::<&Time>(|time| {
            let dt = time.delta_time();

            world.query::<(&mut Tween<Color>, &mut TextMaterial)>()
                .with(Hovered)
                .build()
                .each(|(tween, mat)| {
                    tween.elapsed = (tween.elapsed + dt).min(tween.duration);
                    let t = (tween.elapsed / tween.duration).clamp(0.0, 1.0);
                    mat.set_color(tween.from.lerp(tween.to, t));
                    println!("Processing hovered: {t}");
                });

            world.query::<(&mut Tween<Color>, &mut TextMaterial)>()
                .with(Unhovered)
                .build()
                .each(|(tween, mat)| {
                    tween.elapsed = (tween.elapsed - dt).max(0.0);
                    let t = (tween.elapsed / tween.duration).clamp(0.0, 1.0);
                    mat.set_color(tween.from.lerp(tween.to, t));
                    println!("Processing unhovered: {t}");
                });
        });

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
            WindowEvent::CursorPos(x, y) => self.process_cursor_pos(world, x, y),
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
            let size = render_device.size();
            world.get::<&MainFont>(|font| {
                let atlas = self.registry.get_atlas(font.0, 24).unwrap();
                self.ui_manager.recompute_layout(world, size.x as f32, size.y as f32, atlas);
                
                if let Some(screen) = self.ui_manager.get_current_screen_mut() {
                    screen.apply_layout_to_entities(world);
                }
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

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Tween<T: 'static + Send + Sync> {
    pub from: T,
    pub to: T,
    pub duration: f32,
    pub elapsed: f32,
}

impl<T: 'static + Send + Sync> Tween<T> {
    pub fn new(from: T, to: T, duration: f32) -> Tween<T> {
        Tween {
            from, to, duration, elapsed: 0.0
        }
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

    fn process_cursor_pos(&mut self, world: &mut World, x: &f64, y: &f64) {
        let current_pos = Vec2::new(*x as f32, *y as f32);
        world.get::<&mut MouseState>(|mouse| {
            if mouse.captured {
                // Rotate camera
                if let Some(last) = mouse.last_pos {
                    let delta = current_pos - last;

                    world.each::<(&mut FpsCamera,)>(|(fps,)| {
                        fps.yaw   -= delta.x * fps.sensitivity;
                        fps.pitch -= delta.y * fps.sensitivity;
                        fps.pitch = fps.pitch.clamp(-1.54, 1.54);
                    });
                }

                mouse.last_pos = Some(current_pos);
            } else {
                // Move cursor
                self.current_event.extend(self.ui_manager.hit_test(
                    current_pos,
                    UiEvent::Enter,
                    UiEvent::Exit,
                ));
            }
        });
    }

    fn register_materials(&mut self, render_device: &mut RenderDevice) {
        let camera_layout = CameraBuffer::layout(render_device);

        self.registry.register_material::<TextMaterial>(render_device, &[&camera_layout]);
        self.registry.register_material::<ColorUiMaterial>(render_device, &[&camera_layout]);
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
                            load: LoadOp::Clear(GpuColor::BLACK),
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

        world.each::<(&Mesh<UiVertex>, &ColorUiMaterial, &Transform)>(|(mesh, mat, t)| {
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

    fn render_ui_text(&mut self, world: &mut World, render_device: &mut RenderDevice, canvases: &[&(dyn RenderSurface + 'static)], ctx: &mut xastge::render::draw_context::DrawContext) {
        world.each::<(&Mesh<GlyphVertex>, &TextMaterial, &Transform)>(|(mesh, mat, t)| {
            mat.update(&self.registry, render_device);

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
        
        self.ui_manager.mark_dirty();
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
                .set(CameraBuffer::new(render_device, &mut self.registry))
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
                .set(CameraBuffer::new(render_device, &mut self.registry))
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

// fn main() -> anyhow::Result<()> {
//     pretty_env_logger::formatted_builder()
//         .filter_level(LevelFilter::Info)
//         .filter_module("wgpu_hal", LevelFilter::Off)
//         .init();

//     App::new(
//         WindowDescriptor {
//             width: 1024,
//             height: 1024,
//             title: "KVΛNTUMA",
//             mode: WindowMode::Windowed,
//             cursor_mode: CursorMode::Disabled,
//         }, 
//         KvantumaGame {
//             registry: RenderRegistry::new(),
//             ort_cam_id: Entity::null(),
//             persp_cam_id: Entity::null(),
//             ui_manager: KvUiManager::new(1024.0, 1024.0),
//             current_event: vec![],
//         },
//     )?.run();

//     Ok(())
// }

// pub struct MyUi;

// impl Ui for MyUi {
//     fn build_ui(&self, world: &mut World) -> Entity {
//         ui! { world,
//             row {
//                 col (6) {
//                     text("AGGAGAGAGJJJJ")
//                     text("B")
//                 }
//                 col (6) {
//                     text("C")
//                     text("D")
//                     text("E")
//                 }
//             }
//         }
//     }
// }

#[repr(C)]
pub struct PluginApi {
    // World
    pub world: *mut ecs_world_t,
    // Labels
    pub update: ecs_entity_t,
    pub render: ecs_entity_t,
    // Components
    pub random_number: ecs_entity_t,
}

fn main() {
    #[derive(Component)]
    struct UpdateLabel;

    #[derive(Component)]
    struct RenderLabel;

    #[derive(Component)]
    #[repr(C)]
    struct RandomNumber {
        pub value: u16
    }

    let world = World::new();

    // Register components
    let update_label_id = world.component::<UpdateLabel>().id();
    let render_label_id = world.component::<RenderLabel>().id();
    let random_number_id = world.component::<RandomNumber>().id();

    world.entity().set(RandomNumber { value: rand::random() });

    let update_pipeline = world
        .pipeline()
        .with(SystemLabel)
        .with(UpdateLabel)
        .build();

    let render_pipeline = world
        .pipeline()
        .with(SystemLabel)
        .with(RenderLabel)
        .build();

    world.system::<()>()
        .kind(UpdateLabel)
        .each(|_| {
            println!("Running update 1");
        });

    world.system::<()>()
        .kind(UpdateLabel)
        .each(|_| {
            println!("Running update 2");
        });

    world.system::<&mut RandomNumber>()
        .kind(UpdateLabel)
        .each(|num| {
            num.value = rand::random();
        });

    world.system::<()>()
        .kind(RenderLabel)
        .each(|_| {
            println!("Running render 1");
        });

    world.system::<()>()
        .kind(RenderLabel)
        .each(|_| {
            println!("Running render 2");
        });

    world.run_pipeline(*update_pipeline);
    world.run_pipeline(*render_pipeline);
}

// unsafe extern "C" {
//     fn register_systems(world: *mut ecs_world_t);
// }