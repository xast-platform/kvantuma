use std::{any::type_name, marker::PhantomData};
use log::LevelFilter;
use taffy::TaffyTree;
use xastge::{
    Render, Setup, Update,
    app::{
        RenderSlot, XastGE, 
        input::{Keyboard, Mouse}, 
        window::{
            Action, CursorMode, Key, MouseButton, Window, WindowDescriptor, WindowEvent, WindowMode, WindowSize,
        },
    }, 
    math::Transform, 
    render::{
        RenderSurface, 
        camera::{Camera, CameraBuffer, OrthographicCamera, PerspectiveCamera, build_orthographic_uniform, build_perspective_uniform}, 
        material::{ColorMaterial, ColorUiMaterial, Material, SkyboxMaterial}, 
        mesh::{Mesh, UiVertex, Vertex}, 
        pass::DrawDescriptor, 
        registry::{RenderRegistry, RenderRegistryModule}, 
        texture::TextureDescriptor, 
        types::*, 
        updated,
    },
    time::TimeModule, 
    ui::{
        atlas::{FontHandle, GlyphVertex},
        glyph::FontRef,
        material::TextMaterial,
    }, 
    utils::{Color, Translation},
};
use glam::{DVec2, EulerRot, Quat, Vec2, Vec3};
use flecs_ecs::{core::flecs::Singleton, prelude::*, sys::{ecs_entity_t, ecs_world_t}};

use crate::{
    systems::ui::render_ui_text, ui::{ScreenKey, Ui, UiManager, UiScreen, components::{KirText, UiPosition}, key::Screen},
};

pub type KvUiManager = UiManager<Screen>;

pub mod game;
pub mod menu;
pub mod systems;
pub mod ui;
pub mod singletons;

// #[derive(Component)]
// pub struct MainFont(pub FontHandle);

// struct KvantumaGame {
//     ui_manager: KvUiManager,
//     current_event: Vec<UiEvent>,
// }

// pub enum UiEvent {
//     Enter(Entity),
//     Exit(Entity),
// }

// #[derive(Component)]
// pub struct Hovered;

// #[derive(Component)]
// pub struct Unhovered;

// impl Game for KvantumaGame {
//     fn init(&mut self, world: &mut World, render_device: &mut RenderDevice) -> anyhow::Result<()> {
//         let font = self.registry.new_font(FontRef::try_from_slice(include_bytes!("../assets/fonts/KVANTUMA1451.ttf"))?);
//         for size in [8, 12, 18, 24, 36, 48, 64, 72] {
//             self.registry.add_font_atlas(
//                 render_device, 
//                 font, 
//                 size,
//             );
//         }

//         let ui_root = MyUi.build_ui(world);
//         self.ui_manager.add_screen(ScreenKey::MainMenu, UiScreen::new(ui_root));
//         self.ui_manager.set_screen(ScreenKey::MainMenu);

//         init_singletons(world, font);
//         self.init_skybox(world, render_device)?;
//         self.ort_cam_id = self.init_ort_camera(world, render_device)?;
//         self.persp_cam_id = self.init_persp_camera(world, render_device)?;
        
//         let size = render_device.size();

//         let atlas = self.registry.get_atlas(font, 24).unwrap();
//         self.ui_manager.recompute_layout(world, size.x as f32, size.y as f32, atlas);

//         log::info!("recompute_layout size = {size:?}");

//         Ok(())
//     }

//     fn update(&mut self, world: &mut World) -> anyhow::Result<()> {
//         for event in &self.current_event {
//             match event {
//                 UiEvent::Enter(enter) => {
//                     world.entity_from_id(*enter)
//                         .add(Hovered)
//                         .remove(Unhovered);
//                 },
//                 UiEvent::Exit(exit) => {
//                     world.entity_from_id(*exit)
//                         .add(Unhovered)
//                         .remove(Hovered);
//                 },
//             }
//         }
//         self.current_event.clear();

//         world.get::<&Time>(|time| {
//             let dt = time.delta_time();

//             world.query::<(&mut Tween<Color>, &mut TextMaterial)>()
//                 .with(Hovered)
//                 .build()
//                 .each(|(tween, mat)| {
//                     tween.elapsed = (tween.elapsed + dt).min(tween.duration);
//                     let t = (tween.elapsed / tween.duration).clamp(0.0, 1.0);
//                     mat.set_color(tween.from.lerp(tween.to, t));
//                     println!("Processing hovered: {t}");
//                 });

//             world.query::<(&mut Tween<Color>, &mut TextMaterial)>()
//                 .with(Unhovered)
//                 .build()
//                 .each(|(tween, mat)| {
//                     tween.elapsed = (tween.elapsed - dt).max(0.0);
//                     let t = (tween.elapsed / tween.duration).clamp(0.0, 1.0);
//                     mat.set_color(tween.from.lerp(tween.to, t));
//                     println!("Processing unhovered: {t}");
//                 });
//         });

//         self.movement_system(world);

//         Ok(())
//     }

//     fn render(&mut self, world: &mut World, render_device: &mut RenderDevice) -> Result<(), RenderError> {        
//         if self.ui_manager.is_dirty() {
//             let size = render_device.size();
//             world.get::<&MainFont>(|font| {
//                 let atlas = self.registry.get_atlas(font.0, 24).unwrap();
//                 self.ui_manager.recompute_layout(world, size.x as f32, size.y as f32, atlas);
                
//                 if let Some(screen) = self.ui_manager.get_current_screen_mut() {
//                     screen.apply_layout_to_entities(world);
//                 }
//                 render_ui_text(world, &mut self.registry, font.0, 24, render_device);
//             });
            
//             self.ui_manager.mark_clean();
//         }
//         Ok(())
//     }
// }

#[derive(Component)]
pub struct UiModule<K: ScreenKey>(PhantomData<K>);

impl<K: ScreenKey> Module for UiModule<K> {
    fn module(world: &World) {
        // Init UiManager
        let mut screen_size = Vec2::default();
        world.get::<&WindowSize>(|size| {
            screen_size.x = size.width();
            screen_size.y = size.height();
        });

        world.component::<UiManager<K>>().add_trait::<Singleton>();
        world.set(UiManager::<K>::new(screen_size.x, screen_size.y));

        // Resize UiManager
        world.system::<(&WindowSize, &mut UiManager<K>)>()
            .kind(Update)
            .each(|(size, ui_manager)| {               
                if size.is_changed() {
                    ui_manager.mark_dirty();
                }
            });
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

// Must be done in render of Text
// mat.update(&self.registry, render_device);

pub struct MyUi;

impl Ui for MyUi {
    fn build_ui(&self, world: &mut World) -> Entity {
        ui! { world,
            row {
                col (6) {
                    text("AGGAGAGAGJJJJ")
                    text("B")
                }
                col (6) {
                    text("C")
                    text("D")
                    text("E")
                }
            }
        }
    }
}

#[derive(Component)]
pub struct SkyboxTag;

#[derive(Component)]
pub struct FlyCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub sensitivity: f32,
    pub move_speed: f32,
}

impl FlyCamera {
    pub fn forward(&self) -> Vec3 {
        (Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0) * Vec3::NEG_Z)
            .normalize()
    }
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct MovementInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
}

#[derive(Component)]
pub struct GenericCameraModule;

impl Module for GenericCameraModule {
    fn module(world: &World) {
        // Resize cameras
        let ort_query = world.query::<&mut OrthographicCamera>()
            .with(Camera::id())
            .build();

        let persp_query = world.query::<&mut PerspectiveCamera>()
            .with(Camera::id())
            .build();

        world.system::<&WindowSize>()
            .kind(Update)
            .each(move |size| {               
                if size.is_changed() {
                    ort_query.each(|ort_cam| {
                        ort_cam.resize_viewport(size.width(), size.height());
                    });
                    persp_query.each(|persp_cam| {
                        persp_cam.set_aspect(size.width() / size.height());
                    });
                }                
            });

        // Update camera buffers
        let ort_query = world.query::<(&Camera, &OrthographicCamera, &Transform, &CameraBuffer)>().build();
        let persp_query = world.query::<(&Camera, &PerspectiveCamera, &Transform, &CameraBuffer)>().build();

        world.system::<(&RenderSlot, &RenderRegistry)>()
            .kind(Render)
            .each(move |(render_slot, registry)| {
                ort_query.each(|(cam, ort_cam, t, buf)| {
                    let uniform = build_orthographic_uniform(cam, ort_cam, t);
                    if let Some(buf) = registry.get_buffer(buf.handle()) {
                        buf.fill_exact(render_slot.device(), 0, &[uniform]).unwrap_or_else(|e| {
                            log::error!("{e}");
                        });
                    } else {
                        log::error!("Camera buffer not found in registry");
                    }
                });

                persp_query.each(|(cam, persp_cam, t, buf)| {
                    let uniform = build_perspective_uniform(cam, persp_cam, t);
                    if let Some(buf) = registry.get_buffer(buf.handle()) {
                        buf.fill_exact(render_slot.device(), 0, &[uniform]).unwrap_or_else(|e| {
                            log::error!("{e}");
                        });
                    } else {
                        log::error!("Camera buffer not found in registry");
                    }
                });
            });
    }
}

#[derive(Component)]
pub struct FlyCameraModule;

impl Module for FlyCameraModule {
    fn module(world: &World) {
        world.component::<MovementInput>().add_trait::<Singleton>();
        world.set(MovementInput::default());

        world.component::<MouseState>().add_trait::<Singleton>();
        world.set(MouseState::new(true));

        // Setup cameras
        let w = world.clone();
        world.system::<(&mut RenderRegistry, &RenderSlot, &WindowSize)>()
            .kind(Setup)
            .each(move |(registry, render_slot, size)| {
                // Orthographic camera
                w.entity()
                    .set(OrthographicCamera::from_viewport(size.width(), size.height()))
                    .set(Camera::default())
                    .set(Transform {
                        translation: Vec3::new(0.0, 0.0, 1.0),
                        ..Default::default()
                    })
                    .set(CameraBuffer::new(render_slot.device(), registry));

                // Perspective camera
                w.entity()
                    .set(PerspectiveCamera::from_aspect(size.width() / size.height()))
                    .set(Camera::default())
                    .set(Transform {
                        translation: Vec3::new(5.0, 5.0, 5.0),
                        ..Default::default()
                    })
                    .set(CameraBuffer::new(render_slot.device(), registry))
                    .set(FlyCamera {
                        yaw: 0.0,
                        pitch: 0.0,
                        sensitivity: 0.002,
                        move_speed: 0.08,
                    });
            });

        // Movement input processing
        let query = world.query::<&mut FlyCamera>().build();
        world.system::<(&mut MovementInput, &Keyboard, &Mouse, &mut MouseState, &mut Window)>()
            .kind(Update)
            .each(move |(input, keyboard, mouse, mouse_state, window)| {
                if keyboard.just_pressed(Key::Escape) {
                    if mouse_state.captured {
                        window.set_cursor_mode(CursorMode::Normal);
                        mouse_state.captured = false;
                    } else {
                        window.set_cursor_mode(CursorMode::Disabled);
                        mouse_state.captured = true;
                    }

                    mouse_state.last_pos = None;
                }

                input.forward = keyboard.is_pressed(Key::W);
                input.backward = keyboard.is_pressed(Key::S);
                input.left = keyboard.is_pressed(Key::A);
                input.right = keyboard.is_pressed(Key::D);

                let current_pos = mouse.position();
                if mouse_state.captured {
                    // Rotate camera
                    if let Some(last) = mouse_state.last_pos {
                        let delta = current_pos - last;

                        query.each(|fly_cam| {
                            fly_cam.yaw   -= (delta.x as f32) * fly_cam.sensitivity;
                            fly_cam.pitch -= (delta.y as f32) * fly_cam.sensitivity;
                            fly_cam.pitch = fly_cam.pitch.clamp(-1.54, 1.54);
                        });
                    }

                    mouse_state.last_pos = Some(current_pos);
                } else {
                    // TODO: Move cursor
                    // self.current_event.extend(self.ui_manager.hit_test(
                    //     current_pos,
                    //     UiEvent::Enter,
                    //     UiEvent::Exit,
                    // ));
                }
            });
            
        // Update camera transformation
        let query = world.query::<&mut Transform>()
            .with(SkyboxTag)
            .build();

        world.system::<(&mut Transform, &FlyCamera, &MovementInput)>()
            .kind(Update)
            .with(Camera::id())
            .each(move |(t, fly_cam, input)| {
                let rotation = Quat::from_euler(EulerRot::YXZ, fly_cam.yaw, fly_cam.pitch, 0.0);
                t.rotation = rotation;

                let forward = (rotation * Vec3::NEG_Z).normalize();
                let right = (rotation * Vec3::X).normalize();

                let mut direction = Vec3::ZERO;
                if input.forward {
                    direction += forward;
                }
                
                if input.backward {
                    direction -= forward;
                }

                if input.left {
                    direction -= right;
                }

                if input.right {
                    direction += right;
                }

                if direction.length_squared() > 0.0 {
                    t.translation += direction.normalize() * fly_cam.move_speed;
                }

                query.each(|skybox_t| {
                    skybox_t.translation = t.translation;
                });
            });
    }
}

#[macro_export]
macro_rules! setup_cam_material_module {
    ($vertex:ident, $mat:ident, $cam:ident) => { 
        paste::paste! {
            #[derive(Component, Default, Debug)]
            pub struct [<$mat Module>];

            impl Module for [<$mat Module>] {
                fn module(world: &World) {
                    world.system::<(Option<&mut RenderRegistry>, &RenderSlot)>()
                        .kind(Setup)
                        .each(|(maybe_registry, render_slot)| {
                            if let Some(registry) = maybe_registry {
                                let camera_buffer = CameraBuffer::layout(render_slot.device());
                                registry.register_material::<$mat>(render_slot.device(), &[&camera_buffer]);
                            } else {
                                panic!("No render registry found while registering `{}`!", type_name::<$mat>());
                            }
                        });

                    let camera_query = world.query::<&CameraBuffer>()
                        .with($cam::id())
                        .build();

                    let drawable_query = world.query::<(&Mesh<$vertex>, &$mat, &Transform)>().build();

                    world.system::<(&mut RenderSlot, &RenderRegistry)>()
                        .kind(Render)
                        .each(move |(slot, registry)| {
                            let (device, canvas, ctx) = slot.parts_mut();
                            let canvases: &[&dyn RenderSurface] = &[canvas];
                            let mut render_pass = ctx.render_pass(
                                canvases, 
                                device.depth_texture(),
                                Operations {
                                    load: $mat::load_op(),
                                    store: $mat::store_op(),
                                },
                            );
                            
                            drawable_query.each(|(mesh, mat, t)| {
                                camera_query.each(|cam_buffer| {
                                    render_pass.draw(device, registry, DrawDescriptor::<_, _> {
                                        drawable: Some(mesh),
                                        instance_data: Some(t),
                                        global_shader_resources: &[cam_buffer.resource()],
                                        material: mat,
                                    });
                                });
                            });
                        });
                }
            }
        }
    };
}

setup_cam_material_module!(Vertex, ColorMaterial, PerspectiveCamera);
setup_cam_material_module!(GlyphVertex, TextMaterial, OrthographicCamera);
setup_cam_material_module!(UiVertex, ColorUiMaterial, OrthographicCamera);
setup_cam_material_module!(Vertex, SkyboxMaterial, PerspectiveCamera);

#[derive(Component)]
pub struct TestCubeModule;

impl Module for TestCubeModule {
    fn module(world: &World) {
        let w = world.clone();
        world.system::<(&mut RenderRegistry, &mut RenderSlot)>()
            .kind(Setup)
            .each(move |(registry, render_slot)| {
                w.entity()
                    .set(updated(Mesh::load_obj("assets/meshes/cube.obj"), render_slot.device_mut(), registry))
                    .set(ColorMaterial::new(Color::CYAN, render_slot.device(), registry))
                    .set(Transform::default());
            });
    }
}

#[derive(Component, Default)]
pub struct MouseState {
    pub last_pos: Option<DVec2>,
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

#[derive(Component)]
pub struct InitSkyboxModule;

impl Module for InitSkyboxModule {
    fn module(world: &World) {
        let w = world.clone();
        world.system::<(&mut RenderSlot, &mut RenderRegistry)>()
            .kind(Setup)
            .each(move |(slot, registry)| {
                w.entity()
                    .set(updated(Mesh::load_obj("./assets/meshes/cube.obj"), slot.device_mut(), registry))
                    .set(SkyboxMaterial::new(
                        registry.load_cubemap(
                            slot.device(),
                            [
                                "./assets/textures/skyboxes/sky1_cubemap_faces/right_cubemap.png",
                                "./assets/textures/skyboxes/sky1_cubemap_faces/left_cubemap.png",
                                "./assets/textures/skyboxes/sky1_cubemap_faces/top_cubemap.png",
                                "./assets/textures/skyboxes/sky1_cubemap_faces/bottom_cubemap.png",
                                "./assets/textures/skyboxes/sky1_cubemap_faces/front_cubemap.png",
                                "./assets/textures/skyboxes/sky1_cubemap_faces/back_cubemap.png",
                            ],
                            TextureDescriptor::default(),
                        ).expect("Cannot load cubemap")
                    ))
                    .add(SkyboxTag)
                    .set(Transform {
                        translation: Vec3::ZERO,
                        rotation: Quat::IDENTITY,
                        scale: Vec3::splat(200.0),
                    });
            });
    }
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Info)
        .filter_module("wgpu_hal", LevelFilter::Off)
        .init();

    XastGE::new(WindowDescriptor {
        width: 1024,
        height: 1024,
        title: "KVΛNTUMA v0.1",
        mode: WindowMode::Windowed,
        cursor_mode: CursorMode::Disabled,
    })?
        .import_module::<TimeModule>()
        .import_module::<RenderRegistryModule>()
        .import_module::<TestCubeModule>()
        .import_module::<FlyCameraModule>()
        .import_module::<GenericCameraModule>()

        .import_module::<SkyboxMaterialModule>()
        .import_module::<ColorUiMaterialModule>()
        .import_module::<ColorMaterialModule>()
        .import_module::<TextMaterialModule>()

        .import_module::<InitSkyboxModule>()
        // .load_plugin("test-plugin")?
        .run();

    Ok(())
}