use pretty_type_name::pretty_type_name;

use crate::render::material::Material;

use super::*;

pub struct ComputePass<'a> {
    pub(super) pass: wgpu::ComputePass<'a>,
}

pub struct ComputeDescriptor<'a, 'b, T> {
    pub instance_data: Option<&'b dyn InstanceData<UniformData = T>>,
    pub pipeline: &'a Pipeline,
    pub shader_resources: &'b [&'a ShaderResource],
    pub size: UVec2,
}

impl<'a> ComputePass<'a> {
    pub fn compute<T: Pod>(&mut self, descriptor: ComputeDescriptor<'a, '_, T>) {
        if let Pipeline::Compute(p) = descriptor.pipeline {
            self.pass.set_pipeline(p);
        } else {
            panic!("Cannot use render pipeline in compute() command");
        }

        for (i, binding) in descriptor.shader_resources.iter().enumerate() {
            self.pass.set_bind_group(i as u32, &binding.bind_group, &[]);
        }

        if let Some(instance_data) = descriptor.instance_data {
            self.pass.set_push_constants(
                0,
                bytemuck::cast_slice(&[instance_data.uniform_data()]),
            );
        }

        self.pass.dispatch_workgroups(
            descriptor.size.x / 16, 
            descriptor.size.y / 16, 
            1,
        );
    }
}

/// Represents a render pass used for drawing.
pub struct RenderPass<'a> {
    pub(super) pass: wgpu::RenderPass<'a>
}

pub struct DrawDescriptor<'a, 'b, T, M: Material> {
    pub drawable: Option<&'b dyn Drawable>,
    pub instance_data: Option<&'b dyn InstanceData<UniformData = T>>,
    pub material: &'a M,
}

impl<'a> RenderPass<'a> {
    pub fn draw<T: Pod, M: Material + 'static>(
        &mut self,
        render_device: &RenderDevice,
        registry: &RenderRegistry,
        descriptor: DrawDescriptor<'a, '_, T, M>,
    ) {
        let shader_resource = descriptor.material.shader_resource(render_device, registry);
        let Some(pipeline) = registry.get_pipeline::<M>() else {
            log::error!("Material `{}` is not registered", pretty_type_name::<M>());
            return;
        };

        if let Pipeline::Render(p) = pipeline {
            self.pass.set_pipeline(p);
        } else {
            panic!("Cannot use compute pipeline in draw() command");
        }

        self.pass.set_bind_group(0, &shader_resource.bind_group, &[]);

        if let Some(instance_data) = descriptor.instance_data {
            self.pass.set_push_constants(
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                0,
                bytemuck::cast_slice(&[instance_data.uniform_data()]),
            );
        }
        
        if let Some(drawable) = descriptor.drawable {
            let Some(buffer) = registry.get_buffer(drawable.vertex_buffer()) else {
                log::error!("This drawable vertex buffer is not initialized");
                return;
            };

            self.pass.set_vertex_buffer(0, buffer.inner().slice(..)); 

            if let Some(handle) = drawable.index_buffer() {
                let Some(index_buffer) = registry.get_buffer(handle) else {
                    log::error!("This drawable has invalid index buffer");
                    return;
                };

                self.pass.set_index_buffer(index_buffer.inner().slice(..), wgpu::IndexFormat::Uint32);
                self.pass.draw_indexed(0..drawable.indices(), 0, 0..1);
            } else {
                self.pass.draw(0..buffer.capacity() as u32, 0..1);
            }
        } else {
            self.pass.draw(0..6, 0..1);
        }
    }
}