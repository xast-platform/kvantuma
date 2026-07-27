use flecs_ecs::{addons::Module, core::World, core::flecs::Singleton, macros::Component, prelude::*};
use glam::Vec3;
use rapier3d::pipeline::{
    DebugRenderBackend, DebugRenderMode, DebugRenderObject, DebugRenderPipeline, DebugRenderStyle,
};

use crate::{
    Render, Setup, Update,
    app::RenderSlot,
    math::Transform,
    physics::handler::PhysicsHandler,
    render::{
        Drawable, RenderDevice, RenderSurface,
        buffer::BufferHandle,
        camera::{CameraBuffer, PerspectiveCamera},
        include_wgsl,
        material::Material,
        mesh::DebugLineVertex,
        pass::DrawDescriptor,
        registry::RenderRegistry,
        shader_resource::{ShaderResource, ShaderResourceLayout},
        types::*,
    },
};

struct LineCollector<'a> {
    vertices: &'a mut Vec<DebugLineVertex>,
}

impl<'a> DebugRenderBackend for LineCollector<'a> {
    fn draw_line(&mut self, _object: DebugRenderObject, a: Vec3, b: Vec3, color: [f32; 4]) {
        let color = hsla_to_rgb(color);
        self.vertices.push(DebugLineVertex { position: a, color });
        self.vertices.push(DebugLineVertex { position: b, color });
    }
}

fn hsla_to_rgb([h, s, l, _a]: [f32; 4]) -> Vec3 {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = (h / 60.0) % 6.0;
    let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match h_prime as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    Vec3::new(r1 + m, g1 + m, b1 + m)
}

#[derive(Component)]
pub struct PhysicsDebugLines {
    debug_pipeline: DebugRenderPipeline,
    vertices: Vec<DebugLineVertex>,
    buffer: Option<BufferHandle>,
}

impl PhysicsDebugLines {
    fn new() -> Self {
        Self {
            debug_pipeline: DebugRenderPipeline::new(
                DebugRenderStyle::default(),
                DebugRenderMode::COLLIDER_SHAPES,
            ),
            vertices: Vec::new(),
            buffer: None,
        }
    }

    fn rebuild(&mut self, handler: &PhysicsHandler) {
        self.vertices.clear();

        let PhysicsDebugLines { debug_pipeline, vertices, .. } = self;
        let mut collector = LineCollector { vertices };
        handler.debug_render_colliders(debug_pipeline, &mut collector);
    }

    fn upload(&mut self, render_device: &RenderDevice, registry: &mut RenderRegistry) {
        let capacity = self.vertices.len().max(1);

        let handle = *self.buffer.get_or_insert_with(|| {
            registry.new_buffer::<DebugLineVertex>(render_device, capacity, BufferUsages::VERTEX)
        });

        let buffer = registry.get_buffer_mut(handle)
            .expect("Physics debug line buffer is not registered");

        if buffer.capacity() != capacity {
            buffer.resize::<DebugLineVertex>(render_device, capacity);
        }

        buffer.fill_exact(render_device, 0, &self.vertices)
            .expect("Failed to upload physics debug lines");
    }
}

impl Drawable for PhysicsDebugLines {
    fn update(&mut self, render_device: &mut RenderDevice, registry: &mut RenderRegistry) {
        self.upload(render_device, registry);
    }

    fn vertex_buffer(&self) -> BufferHandle {
        self.buffer.expect("PhysicsDebugLines is not set up with update()")
    }

    fn index_buffer(&self) -> Option<BufferHandle> {
        None
    }

    fn indices(&self) -> u32 {
        self.vertices.len() as u32
    }
}

#[derive(Debug, Component)]
pub struct PhysicsDebugMaterial;

impl Material for PhysicsDebugMaterial {
    fn shader() -> ShaderModuleDescriptor<'static> {
        include_wgsl!("../../../assets/shaders/debug_line.wgsl")
    }

    fn vertex_layout() -> Option<VertexBufferLayout<'static>> {
        Some(DebugLineVertex::vertex_buffer_layout())
    }

    fn shader_resource_layout(render_device: &RenderDevice) -> ShaderResourceLayout {
        ShaderResourceLayout::builder()
            .with_label("Physics Debug Material")
            .build(render_device)
    }

    fn shader_resource(
        &self,
        render_device: &RenderDevice,
        _registry: &RenderRegistry,
    ) -> ShaderResource {
        ShaderResource::builder()
            .build(render_device, &Self::shader_resource_layout(render_device))
    }

    fn topology() -> PrimitiveTopology {
        PrimitiveTopology::LineList
    }

    fn cull_mode() -> Option<Face> {
        None
    }

    fn depth_write_enabled() -> bool {
        false
    }

    fn load_op() -> LoadOp<GpuColor> {
        LoadOp::Load
    }
}

#[derive(Component)]
pub struct PhysicsRenderDebugModule;

impl Module for PhysicsRenderDebugModule {
    fn module(world: &World) {
        world.component::<PhysicsDebugLines>().add_trait::<Singleton>();
        world.set(PhysicsDebugLines::new());

        world.system::<(Option<&mut RenderRegistry>, &RenderSlot)>()
            .kind(Setup)
            .each(|(maybe_registry, render_slot)| {
                if let Some(registry) = maybe_registry {
                    let camera_buffer = CameraBuffer::layout(render_slot.device());
                    registry.register_material::<PhysicsDebugMaterial>(render_slot.device(), &[&camera_buffer]);
                } else {
                    panic!("No render registry found while registering `PhysicsDebugMaterial`!");
                }
            });

        // Rebuild the collider wireframe lines from the current physics state
        world.system::<(&PhysicsHandler, &mut PhysicsDebugLines)>()
            .kind(Update)
            .each(|(handler, debug_lines)| {
                debug_lines.rebuild(handler);
            });

        let camera_query = world.query::<&CameraBuffer>()
            .with(PerspectiveCamera::id())
            .build();

        world.system::<(&mut RenderSlot, &mut RenderRegistry, &mut PhysicsDebugLines)>()
            .kind(Render)
            .each(move |(slot, registry, debug_lines)| {
                debug_lines.upload(slot.device(), registry);

                let (device, canvas, ctx) = slot.parts_mut();
                let canvases: &[&dyn RenderSurface] = &[canvas];
                let mut render_pass = ctx.render_pass(
                    canvases,
                    device.depth_texture(),
                    Operations {
                        load: PhysicsDebugMaterial::load_op(),
                        store: PhysicsDebugMaterial::store_op(),
                    },
                );

                camera_query.each(|cam_buffer| {
                    render_pass.draw(device, registry, DrawDescriptor::<_, _> {
                        drawable: Some(&*debug_lines),
                        instance_data: Some(&Transform::default()),
                        global_shader_resources: &[cam_buffer.resource()],
                        material: &PhysicsDebugMaterial,
                    });
                });
            });
    }
}
