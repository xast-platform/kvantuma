use log::LevelFilter;
use xastge::{
    Transform, 
    app::{
        App, Game,
        window::{WindowDescriptor, WindowEvent, WindowMode},
    }, 
    component, 
    ecs::world::{ComponentWrite, World}, 
    glam::{Quat, Vec2, Vec3}, 
    render::{
        Drawable, RenderDevice, RenderSurface, buffer::BufferHandle, error::RenderError, include_wgsl, material::{Material, TintedTextureMaterial}, mesh::{Mesh, Vertex}, pass::DrawDescriptor, registry::RenderRegistry, shader_resource::{ShaderResource, ShaderResourceLayout}, texture::{TextureHandle, TextureResourceDescriptor, TextureResourceUsage}, types::*
    }, 
    ui::{
        atlas::GlyphVertex, 
        glyph::FontRef,
    }
};

#[derive(Debug)]
pub struct Triangle {
    pub vertex_data: [Vertex; 3],
    pub vertex_buffer: Option<BufferHandle>,
}

component! { EXTERN: Triangle }

impl Drawable for Triangle {
    fn update(
        &mut self, 
        render_device: &mut RenderDevice,
        registry: &mut RenderRegistry,
    ) {
        if self.vertex_buffer.is_none() {
            self.vertex_buffer = Some(
                registry.new_buffer::<Vertex>(render_device, 3, BufferUsages::VERTEX)
            );
        }

        let Some(handle) = self.vertex_buffer else { unreachable!() };
        
        registry
            .get_buffer(handle) 
            .expect("Cannot call update() on Triangle")
            .fill_exact(render_device, 0, &self.vertex_data)
            .unwrap();
    }
    

    fn vertex_buffer(&self) -> BufferHandle {
        self.vertex_buffer
            .expect("Triangle is not set up with update()")
    }

    fn index_buffer(&self) -> Option<BufferHandle> {
        None
    }

    fn indices(&self) -> u32 {
        0
    }
}

impl Default for Triangle {
    fn default() -> Self {
        Self {
            vertex_data: [
                Vertex {
                    position: Vec3::new(0.0, 0.5, 0.0),
                    normal: Vec3::new(0.0, 0.0, 1.0),
                    texcoord: Vec2::new(0.5, 0.0),
                },
                Vertex {
                    position: Vec3::new(-0.5, -0.5, 0.0),
                    normal: Vec3::new(0.0, 0.0, 1.0),
                    texcoord: Vec2::new(0.0, 1.0),
                },
                Vertex {
                    position: Vec3::new(0.5, -0.5, 0.0),
                    normal: Vec3::new(0.0, 0.0, 1.0),
                    texcoord: Vec2::new(1.0, 1.0),
                },
            ],
            vertex_buffer: None,
        }
    }
}

struct KvantumaGame {
    registry: RenderRegistry,
    write_batch: Vec<ComponentWrite>,
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

component! { EXTERN: TextMaterial }

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

        let text1 = "the quick brown fox jumps over the lazy dog!";
        let mut mesh1 = atlas.generate_mesh(text1, Vec2::ZERO);
        let transform1 = Transform {
            translation: Vec3::new(-0.5, 0.0, 0.0),
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
        };

        let text2 = "THE QUICK BROWN FOX JUMPS OVER THE LAZY DOG!";
        let mut mesh2 = atlas.generate_mesh(text2, Vec2::ZERO);
        let transform2 = Transform {
            translation: Vec3::new(-0.7, 0.5, 0.0),
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
        };

        mesh1.update(render_device, &mut self.registry);
        mesh2.update(render_device, &mut self.registry);

        world.spawn((mesh1, text_material.clone(), transform1));
        world.spawn((mesh2, text_material, transform2));
        world.spawn((mdl, material, transform_monkey));

        world.spawn((true, 0));

        Ok(())
    }

    fn update(&mut self, world: &mut World) -> anyhow::Result<()> {
        world.for_each::<(&i32, &bool), _>(|e, (i, b)| {
            log::info!("Bool: {}, Int: {}", b, i);
            self.write_batch.push(ComponentWrite::new(e, i + 1));
            self.write_batch.push(ComponentWrite::new(e, !b));
        });

        for w in &self.write_batch {
            world.apply(w);
        }

        self.write_batch.clear();

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
            let mut render_pass = ctx.render_pass(
                canvases, 
                render_device.depth_texture(),
                Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                }
            );
            world.for_each::<(&Mesh<Vertex>, &TintedTextureMaterial, &Transform), _>(|_,(mesh, material, tr)| {
                render_pass.draw(render_device, &self.registry, DrawDescriptor::<_, _> {
                    drawable: Some(mesh),
                    instance_data: Some(tr),
                    material,
                });
            });
        }

        // ----------- UI render pass -----------
        // {
        //     let mut render_pass = ctx.render_pass(
        //         canvases, 
        //         render_device.depth_texture(),
        //         Operations {
        //             load: LoadOp::Load,
        //             store: StoreOp::Store,
        //         },
        //     );
        //     world.for_each::<(&Mesh<GlyphVertex>, &TextMaterial, &Transform), _>(|(mesh, mat, t)| {
        //         render_pass.draw(render_device, &self.registry, DrawDescriptor::<_, _> {
        //             drawable: Some(mesh),
        //             instance_data: Some(t),
        //             material: mat,
        //         });
        //     });
        // }

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
            width: 1280,
            height: 720,
            title: "KVΛNTUMA",
            mode: WindowMode::Windowed,
        }, 
        KvantumaGame {
            registry: RenderRegistry::new(),
            write_batch: Vec::new(),
        },
    )?.run();

    Ok(())
}
