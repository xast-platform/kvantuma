use log::LevelFilter;
use xastge::{
    Transform, 
    app::{
        App, Game,
        window::{WindowDescriptor, WindowEvent, WindowMode},
    }, 
    render::{
        Drawable, RenderDevice, RenderSurface, error::RenderError, include_wgsl, material::{Material, TintedTextureMaterial}, mesh::{Mesh, Vertex}, pass::DrawDescriptor, registry::RenderRegistry, shader_resource::{ShaderResource, ShaderResourceLayout}, texture::{TextureHandle, TextureResourceDescriptor, TextureResourceUsage}, types::*
    }, 
    ui::{
        atlas::GlyphVertex, 
        glyph::FontRef,
    }
};

use glam::{Quat, Vec2, Vec3};
use hecs::{World};

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
        self.registry.register_material::<TintedTextureMaterial>(render_device);
        self.registry.register_material::<TextMaterial>(render_device);

        let material = TintedTextureMaterial::new(
            "assets/textures/texture.jpg", 
            Vec3::new(0.0, 1.0, 0.5), 
            render_device, 
            &mut self.registry,
        )?;

        let mut mdl = Mesh::load_obj("assets/meshes/monkey.obj");
        mdl.update(render_device, &mut self.registry);

        let transform_monkey = Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            ..Default::default()
        };

        let font = self.registry.new_font(
            FontRef::try_from_slice(include_bytes!("../assets/fonts/KVANTUMA1451.ttf"))?
        );
        self.registry.add_font_atlas(render_device, font, 64);
        let atlas = self.registry
            .get_atlas(font, 64)
            .unwrap();

        let text_material = TextMaterial {
            atlas: atlas.texture(),
        };

        atlas.image().save("atlas.png")?;

        // panic!();

        let text1 = "the quick brown fox jumps over the lazy dog!";
        let mut mesh1 = atlas.generate_mesh(text1, Vec2::ZERO);
        let transform1 = Transform {
            translation: Vec3::new(-0.5, 0.0, 0.0),
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
        };

        // let text2 = "THE QUICK BROWN FOX JUMPS OVER THE LAZY DOG!";
        // let mut mesh2 = atlas.generate_mesh(text2, Vec2::ZERO);
        // let transform2 = Transform {
        //     translation: Vec3::new(-0.7, 0.5, 0.0),
        //     scale: Vec3::ONE,
        //     rotation: Quat::IDENTITY,
        // };

        mesh1.update(render_device, &mut self.registry);
        // mesh2.update(render_device, &mut self.registry);

        world.spawn((mesh1, text_material.clone(), transform1));
        // world.spawn((mesh2, text_material, transform2));
        world.spawn((mdl, material, transform_monkey));

        world.spawn((true, 0));

        Ok(())
    }

    fn update(&mut self, _world: &mut World) -> anyhow::Result<()> {
        Ok(())
    }

    fn input(&mut self, _event: &WindowEvent, _world: &mut World) -> anyhow::Result<bool> {
        Ok(false)
    }

    fn render(&mut self, world: &mut World, render_device: &mut RenderDevice) -> Result<(), RenderError> {
        let canvas = render_device.canvas()?;
        let canvases: &[&dyn RenderSurface] = &[&canvas];
        let mut ctx = render_device.draw_ctx();

        {
            for (_, (mesh, material, tr)) in &mut world.query::<(&Mesh<Vertex>, &TintedTextureMaterial, &Transform)>() {
                let mut render_pass = ctx.render_pass(
                    canvases, 
                    render_device.depth_texture(),
                    Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    }
                );
                
                // render_pass.draw(render_device, &self.registry, DrawDescriptor::<_, _> {
                //     drawable: Some(mesh),
                //     instance_data: Some(tr),
                //     material,
                // });
            }
        }

        // ----------- UI render pass -----------
        {
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
