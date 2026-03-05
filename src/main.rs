use log::LevelFilter;
use xastge::{
    Transform, 
    app::{
        App, Game,
        window::{WindowDescriptor, WindowEvent, WindowMode},
    }, 
    render::{
        Drawable, RenderDevice, RenderSurface, 
        camera::{Camera, CameraBuffer, OrthographicCamera, PerspectiveCamera}, 
        error::RenderError, 
        include_wgsl, 
        material::Material,
        mesh::Mesh, 
        pass::DrawDescriptor,
        registry::RenderRegistry, 
        shader_resource::{ShaderResource, ShaderResourceLayout}, 
        texture::{TextureHandle, TextureResourceDescriptor, TextureResourceUsage}, 
        types::*,
    }, 
    ui::{
        atlas::GlyphVertex, 
        glyph::FontRef,
    }
};

pub mod game;
pub mod systems;

use glam::{Quat, Vec2, Vec3};
use hecs::{With, World};

use crate::systems::camera::update_camera_buffer;

struct KvantumaGame {
    registry: RenderRegistry,
}

#[derive(Clone)]
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
        self.registry.register_material::<TextMaterial>(
            render_device, 
            &[&camera_layout]
        );

        let font = self.registry.new_font(
            FontRef::try_from_slice(include_bytes!("../assets/fonts/KVANTUMA1451.ttf"))?
        );
        self.registry.add_font_atlas(render_device, font, 128);
        let atlas = self.registry
            .get_atlas(font, 128)
            .unwrap();

        let text_material = TextMaterial {
            atlas: atlas.texture(),
        };

        let text1 = "KV^NTUMA";
        let mut mesh1 = atlas.generate_mesh(text1, Vec2::ZERO, 5.0);
        let transform1 = Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
        };
        mesh1.update(render_device, &mut self.registry);

        world.spawn((mesh1, text_material.clone(), transform1));

        let size = render_device.size();
        world.spawn((
            OrthographicCamera::from_viewport(size.x as f32, size.y as f32),
            Camera::default(),
            Transform {
                translation: Vec3::new(0.0, 0.0, 1.0),
                ..Default::default()
            },
            CameraBuffer::new(render_device, &mut self.registry, &camera_layout),
        ));
        world.spawn((
            PerspectiveCamera::from_aspect(size.x as f32 / size.y as f32),
            Camera::default(),
            Transform {
                translation: Vec3::new(0.0, 0.0, 5.0),
                ..Default::default()
            },
            CameraBuffer::new(render_device, &mut self.registry, &camera_layout),
        ));

        Ok(())
    }

    fn update(&mut self, _world: &mut World) -> anyhow::Result<()> {
        Ok(())
    }

    fn input(&mut self, event: &WindowEvent, world: &mut World) -> anyhow::Result<()> {
        match event {
            WindowEvent::FramebufferSize(width, height) => {
                for (_, ort_cam) in &mut world.query::<With<&mut OrthographicCamera, &Camera>>() {
                    ort_cam.resize_viewport(*width as f32, *height as f32);
                }

                for (_, persp_cam) in &mut world.query::<With<&mut PerspectiveCamera, &Camera>>() {
                    persp_cam.set_aspect(*width as f32 / *height as f32);
                }
            },
            WindowEvent::Close => {
                // save later
            }
            _ => {},
        }

        Ok(())
    }

    fn render(&mut self, world: &mut World, render_device: &mut RenderDevice) -> Result<(), RenderError> {
        update_camera_buffer(world, render_device, &self.registry);
        
        let canvas = render_device.canvas()?;
        let canvases: &[&dyn RenderSurface] = &[&canvas];
        let mut ctx = render_device.draw_ctx();

        // ----------- UI render pass -----------
        {
            let mut ui_cam_query = world.query::<With<&CameraBuffer, &OrthographicCamera>>();
            let (_, ui_cam_buffer) = ui_cam_query.into_iter().next().unwrap();

            for (_, (mesh, mat, t)) in &mut world.query::<(&Mesh<GlyphVertex>, &TextMaterial, &Transform)>() {
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
            }
        }

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
        }, 
        KvantumaGame {
            registry: RenderRegistry::new(),
        },
    )?.run();

    Ok(())
}
