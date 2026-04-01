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
        material::{ColorMaterial, Material, SkyboxMaterial},
        mesh::{Mesh, Vertex}, 
        pass::DrawDescriptor,
        registry::RenderRegistry, 
        shader_resource::{ShaderResource, ShaderResourceLayout}, 
        texture::{TextureDescriptor, TextureHandle, TextureResourceDescriptor, TextureResourceUsage}, 
        types::*, updated,
    }, 
    ui::{
        atlas::GlyphVertex, 
        glyph::FontRef,
    },
};

pub mod game;
pub mod systems;

use glam::{EulerRot, Quat, Vec2, Vec3};
use flecs_ecs::prelude::*;

use crate::systems::camera::update_camera_buffer;

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

struct KvantumaGame {
    registry: RenderRegistry,
    ort_cam_id: Entity,
    persp_cam_id: Entity,
}

#[derive(Component)]
struct SkyboxTag;

#[derive(Clone, Component)]
pub struct TextMaterial {
    atlas: TextureHandle,
}

impl Material for TextMaterial {
    fn shader() -> ShaderModuleDescriptor<'static> {
        include_wgsl!("../assets/shaders/text.wgsl")
    }

    fn vertex_layout() -> Option<VertexBufferLayout<'static>> {
        Some(GlyphVertex::vertex_buffer_layout())
    }

    fn shader_resource_layout(render_device: &RenderDevice) -> ShaderResourceLayout {
        ShaderResourceLayout::builder()
            .with_label("Text Material")
            .with_texture(&TextureResourceDescriptor {
                usage: TextureResourceUsage::TEXTURE | TextureResourceUsage::SAMPLER,
                sample_type: Some(TextureSampleType::Float { filterable: true }),
                sampler_binding_type: Some(SamplerBindingType::Filtering),
                dimension: TextureDimension::D2,
                view_dimension: TextureViewDimension::D2,
                format: TextureFormat::R8Unorm,
            })
            .build(render_device)
    }

    fn shader_resource(
        &self,
        render_device: &RenderDevice,
        registry: &RenderRegistry,
    ) -> ShaderResource {
        ShaderResource::builder()
            .with_texture(
                registry.get_texture(self.atlas).unwrap(),
                TextureResourceUsage::TEXTURE | TextureResourceUsage::SAMPLER,
            )
            .build(
                render_device,
                &TextMaterial::shader_resource_layout(render_device),
            )
    }
}

impl Game for KvantumaGame {
    fn init(&mut self, world: &mut World, render_device: &mut RenderDevice) -> anyhow::Result<()> {
        let camera_layout = CameraBuffer::layout(render_device);
        let size = render_device.size();

        self.registry.register_material::<TextMaterial>(render_device, &[&camera_layout]);
        self.registry.register_material::<ColorMaterial>(render_device, &[&camera_layout]);
        self.registry.register_material::<SkyboxMaterial>(render_device, &[&camera_layout]);

        let font = self.registry.new_font(FontRef::try_from_slice(include_bytes!("../assets/fonts/KVANTUMA1451.ttf"))?);
        self.registry.add_font_atlas(render_device, font, 72);
        let atlas = self.registry.get_atlas(font, 72).unwrap();

        world.entity()
            .set(TextMaterial {
                atlas: atlas.texture(),
            })
            .set(updated(
                atlas.generate_mesh("KV^NTUMA", Vec2::new(0.0, 0.0), 3.0), 
                render_device, 
                &mut self.registry,
            ))
            .set(Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::ONE,
                rotation: Quat::IDENTITY,
            });

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

        self.ort_cam_id = world.entity()
            .set(OrthographicCamera::from_viewport(size.x as f32, size.y as f32))
            .set(Camera::default())
            .set(Transform {
                translation: Vec3::new(0.0, 0.0, 1.0),
                ..Default::default()
            })
            .set(CameraBuffer::new(render_device, &mut self.registry, &camera_layout))
            .id();

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

        world.set(MouseState::new(true));
        world.set(MovementInput::default());

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
                world.each::<(&mut OrthographicCamera, &Camera)>(|(ort_cam, _cam)| {
                    ort_cam.resize_viewport(*width as f32, *height as f32);
                });

                world.each::<(&mut PerspectiveCamera, &Camera)>(|(persp_cam, _cam)| {
                    persp_cam.set_aspect(*width as f32 / *height as f32);
                });
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
        },
    )?.run();

    Ok(())
}
