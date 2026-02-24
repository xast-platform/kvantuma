//! Pipeline module contains types and utilities for working with shaders,
//! including macros for WGSL and SPIR-V inclusion, as well as shader resource management.

use wgpu::PipelineCompilationOptions;
use crate::render::shader_resource::ShaderResourceLayout;

use super::RenderDevice;
use super::types::*;

#[cfg(doc)]
use super::pass::RenderPass;

/// Represents a graphics or compute pipeline. Used to describe rendering
/// process in a [`RenderPass`]
pub enum Pipeline {
    /// A render pipeline.
    Render(wgpu::RenderPipeline),
    /// A compute pipeline.
    Compute(wgpu::ComputePipeline),
}

/// Descriptor for creating a render pipeline.
pub struct RenderPipelineDescriptor<'a> {
    /// The shader used in the pipeline.
    pub shader: wgpu::ShaderModuleDescriptor<'static>,
    /// The shader resources (buffers, textures) used in the pipeline.
    pub bindings: &'a [&'a ShaderResourceLayout],
    /// The label for the pipeline. Displayed, when any error connected with
    /// the pipeline occures
    pub label: &'a str,
    /// Indicates 
    pub vertex_layout: Option<VertexBufferLayout<'static>>,
    /// The surface formats used in the pipeline. Count and formats must
    /// match ones in render pass
    pub surface_formats: &'a [wgpu::TextureFormat],
}

/// Descriptor for creating a compute pipeline.
pub struct ComputePipelineDescriptor<'a> {
    /// The shader used in the pipeline.
    pub shader: wgpu::ShaderModuleDescriptor<'static>,
    /// The shader resources (buffers, textures) used in the pipeline.
    pub bindings: &'a [&'a ShaderResourceLayout],
    /// The label for the pipeline. Displayed, when any error connected with
    /// the pipeline occures
    pub label: &'a str,
}

impl Pipeline {
    /// Creates a new rendering pipeline using the provided descriptor.
    pub fn new_render(
        render_device: &RenderDevice,
        descriptor: &RenderPipelineDescriptor<'_>,
    ) -> Pipeline {
        let shader = render_device.device.create_shader_module(descriptor.shader.clone());

        let layout = render_device.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("{} Render Pipeline Layout", descriptor.label).as_str()),
            bind_group_layouts: &descriptor.bindings
                .to_vec()
                .iter()
                .map(|b| &b.bind_group_layout)
                .collect::<Vec<_>>(),
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                range: 0..128,
            }],
        });

        let buffers = descriptor.vertex_layout
            .as_ref()
            .map_or(Vec::new(), |l| vec![l.clone()]);
        
        let pipeline = render_device.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(format!("{} Render Pipeline", descriptor.label).as_str()),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex"), 
                buffers: &buffers,
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment"),
                targets: &descriptor.surface_formats
                    .iter()
                    .map(|format| Some(wgpu::ColorTargetState {
                        format: *format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }))
                    .collect::<Vec<_>>(),
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, 
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, 
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1, 
                mask: !0, 
                alpha_to_coverage_enabled: false, 
            },
            multiview: None, 
            cache: None,
        });

        Pipeline::Render(pipeline)
    }

    /// Creates a new compute pipeline using the provided descriptor.
    pub fn new_compute(
        render_device: &RenderDevice,
        descriptor: &ComputePipelineDescriptor<'_>,
    ) -> Pipeline {
        let shader = render_device.device.create_shader_module(descriptor.shader.clone());

        let layout = render_device.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("{} Compute Pipeline Layout", descriptor.label).as_str()),
            bind_group_layouts: &descriptor.bindings
                .to_vec()
                .iter()
                .map(|b| &b.bind_group_layout)
                .collect::<Vec<_>>(),
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::COMPUTE,
                range: 0..128,
            }],
        });

        let pipeline = render_device.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(format!("{} Compute Pipeline", descriptor.label).as_str()),
            layout: Some(&layout),
            module: &shader,
            entry_point: Some("compute"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        Pipeline::Compute(pipeline)
    }
}