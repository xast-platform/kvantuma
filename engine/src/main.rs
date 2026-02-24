use ab_glyph::{Font, FontRef, Glyph, PxScale, point};
use glam::{Vec2, Vec3};
use kvantuma::{
    app::{
        App, Game,
        window::{WindowDescriptor, WindowMode},
    }, 
    component, 
    ecs::world::World,
    render::{
        Drawable, RenderDevice, RenderSurface, buffer::BufferHandle, error::RenderError, material::{Material, TintedTextureMaterial, Vertex}, mesh::Mesh, pass::DrawDescriptor, registry::RenderRegistry, shader_resource::{ShaderResource, ShaderResourceLayout}, texture::{TextureHandle, TextureResourceDescriptor, TextureResourceUsage}, types::*
    }, ui::atlas::GlyphVertex
};
use wgpu::include_wgsl;

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
                    texcoord: Vec2::new(0.5, 0.0),
                },
                Vertex {
                    position: Vec3::new(-0.5, -0.5, 0.0),
                    texcoord: Vec2::new(0.0, 1.0),
                },
                Vertex {
                    position: Vec3::new(0.5, -0.5, 0.0),
                    texcoord: Vec2::new(1.0, 1.0),
                },
            ],
            vertex_buffer: None,
        }
    }
}

struct KvantumaGame {
    registry: RenderRegistry,
}

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
        
        // Restore original triangle
        let mut triangle = Triangle {
            vertex_data: [
                Vertex {
                    position: Vec3::new(0.0, 0.5, 0.0),
                    texcoord: Vec2::new(0.5, 0.0),
                },
                Vertex {
                    position: Vec3::new(-0.5, -0.5, 0.0),
                    texcoord: Vec2::new(0.0, 1.0),
                },
                Vertex {
                    position: Vec3::new(0.5, -0.5, 0.0),
                    texcoord: Vec2::new(1.0, 1.0),
                },
            ],
            vertex_buffer: None,
        };
        triangle.update(render_device, &mut self.registry);

        let material = TintedTextureMaterial::new(
            "assets/textures/test.png", 
            Vec3::new(0.0, 1.0, 0.5), 
            render_device, 
            &mut self.registry,
        )?;

        // Offset second triangle slightly to the left
        let mut triangle2 = Triangle {
            vertex_data: [
                Vertex {
                    position: Vec3::new(-0.2, 0.5, 0.0),
                    texcoord: Vec2::new(0.5, 0.0),
                },
                Vertex {
                    position: Vec3::new(-0.7, -0.5, 0.0),
                    texcoord: Vec2::new(0.0, 1.0),
                },
                Vertex {
                    position: Vec3::new(0.3, -0.5, 0.0),
                    texcoord: Vec2::new(1.0, 1.0),
                },
            ],
            vertex_buffer: None,
        };
        triangle2.update(render_device, &mut self.registry);

        let material2 = TintedTextureMaterial::new(
            "assets/textures/test.png", 
            Vec3::new(0.0, 1.0, 0.5), 
            render_device, 
            &mut self.registry,
        )?;

        let font = self.registry.new_font(
            FontRef::try_from_slice(include_bytes!("../assets/fonts/KVANTUMA1451.ttf"))?
        );
        self.registry.add_font_atlas(render_device, font, 64);
        let atlas = self.registry
            .get_atlas(font, 64)
            .unwrap();

        let text = "Hello text!";
        let mut mesh = atlas.generate_mesh(text, Vec2::ZERO);
        println!("Mesh vertices: {} indices: {}", mesh.vertices.len(), mesh.indices.len());
        mesh.update(render_device, &mut self.registry);

        let atlas = self.registry
            .get_atlas(font, 64)
            .unwrap();

        let text_material = TextMaterial {
            atlas: atlas.texture(),
        };
        
        world.spawn((mesh, text_material));
        world.spawn((triangle2, material2));
        world.spawn((triangle, material));

        Ok(())
    }

    fn update(&mut self, _world: &mut World) -> anyhow::Result<()> {
        Ok(())
    }

    fn input(&mut self, _event: &glfw::WindowEvent, _world: &mut World) -> anyhow::Result<bool> {
        Ok(false)
    }

    fn render(&mut self, world: &mut World, render_device: &mut RenderDevice) -> Result<(), RenderError> {
        let canvas = render_device.canvas()?;
        let canvases: &[&dyn RenderSurface] = &[&canvas];
        let mut ctx = render_device.draw_ctx();

        {
            world.for_each::<(&Triangle, &mut TintedTextureMaterial), _>(|(triangle, material)| {
                let mut render_pass = ctx.render_pass(canvases, render_device.depth_texture());
                material.update_tint(rand::random(), render_device, &mut self.registry);
                render_pass.draw(render_device, &self.registry, DrawDescriptor::<(), _> {
                    drawable: Some(triangle),
                    instance_data: None,
                    material,
                });
            });
        }

        {
            world.for_each::<(&Mesh<GlyphVertex>, &TextMaterial), _>(|(mesh, material)| {
                let mut render_pass = ctx.render_pass(canvases, render_device.depth_texture());
                render_pass.draw(render_device, &self.registry, DrawDescriptor::<(), _> {
                    drawable: Some(mesh),
                    instance_data: None,
                    material,
                });
            });
        }

        ctx.apply(canvas, render_device);

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    App::new(
        WindowDescriptor {
            width: 1280,
            height: 720,
            title: "KVΛNTUMA",
            mode: WindowMode::Windowed,
        }, 
        KvantumaGame {
            registry: RenderRegistry::new(),
        },
    )?.run();

    Ok(())
}
