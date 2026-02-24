use super::RenderDevice;
use super::{buffer::{BufferStorage, BufferResourceDescriptor}, texture::{Texture, TextureResourceDescriptor, TextureResourceUsage}};

pub struct ShaderResourceLayoutBuilder {
    label: Option<String>,
    bind_group_layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl ShaderResourceLayoutBuilder {
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_buffer(
        mut self,
        descriptor: &BufferResourceDescriptor,
    ) -> Self {
        self.bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.bind_group_layout_entries.len() as u32,
            visibility: descriptor.visibility,
            ty: wgpu::BindingType::Buffer {
                ty: descriptor.buffer_type,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });

        self
    }

    pub fn with_texture(
        mut self,
        descriptor: &TextureResourceDescriptor,
    ) -> Self {
        let view_dimension = match descriptor.dimension {
            wgpu::TextureDimension::D1 => wgpu::TextureViewDimension::D1,
            wgpu::TextureDimension::D2 => wgpu::TextureViewDimension::D2,
            wgpu::TextureDimension::D3 => wgpu::TextureViewDimension::D3,
        };

        let bind_group_layout_entries = descriptor.usage
            .iter()
            .enumerate()
            .filter_map(|(i, usage)| {
                match usage {
                    TextureResourceUsage::TEXTURE => {
                        Some(wgpu::BindGroupLayoutEntry {
                            binding: (self.bind_group_layout_entries.len() + i) as u32,
                            visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Texture {
                                sample_type: descriptor.sample_type.unwrap_or_else(|| {
                                    panic!("Must specify sample type for texture with TextureResourceUsage::TEXTURE");
                                }),
                                view_dimension,
                                multisampled: false,
                            },
                            count: None,
                        })
                    },
                    TextureResourceUsage::SAMPLER => {
                        Some(wgpu::BindGroupLayoutEntry {
                            binding: (self.bind_group_layout_entries.len() + i) as u32,
                            visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Sampler(
                                descriptor.sampler_binding_type
                                    .expect("Must specify sampler binding type for TextureResourceUsage::SAMPLER")
                            ),
                            count: None,
                        })
                    },
                    TextureResourceUsage::STORAGE => {
                        Some(wgpu::BindGroupLayoutEntry {
                            binding: (self.bind_group_layout_entries.len() + i) as u32,
                            visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format: descriptor.format,
                                view_dimension,
                            },
                            count: None,
                        })
                    },
                    _ => None,
                }
            })
            .collect::<Vec<_>>();

        self.bind_group_layout_entries.extend(bind_group_layout_entries);

        self
    }

    pub fn build(self, render_device: &RenderDevice) -> ShaderResourceLayout {
        let bind_group_layout = render_device.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: self.label
                .as_ref()
                .map(|label| format!("{label} Bind Group Layout"))
                .as_deref(),
            entries: &self.bind_group_layout_entries,
        });

        ShaderResourceLayout {
            label: self.label,
            bind_group_layout,
        }
    }
}

pub struct ShaderResourceBuilder<'a> {
    bind_group_entries: Vec<wgpu::BindGroupEntry<'a>>,
}

impl<'a> ShaderResourceBuilder<'a> {
    pub fn with_buffer(
        mut self,
        buffer: &'a BufferStorage,
    ) -> Self {
        self.bind_group_entries.push(wgpu::BindGroupEntry {
            binding: self.bind_group_entries.len() as u32,
            resource: buffer.inner().as_entire_binding(),
        });

        self
    }

    pub fn with_texture(
        mut self,
        texture: &'a Texture,
        usage: TextureResourceUsage,
    ) -> Self {
        let bind_group_entries = usage
            .iter()
            .enumerate()
            .filter_map(|(i, usage)| {
                match usage {
                    TextureResourceUsage::STORAGE | TextureResourceUsage::TEXTURE => {
                        Some(wgpu::BindGroupEntry {
                            binding: (self.bind_group_entries.len() + i) as u32,
                            resource: wgpu::BindingResource::TextureView(texture.view())
                        },)
                    },
                    TextureResourceUsage::SAMPLER => {
                        Some(wgpu::BindGroupEntry {
                            binding: (self.bind_group_entries.len() + i) as u32,
                            resource: wgpu::BindingResource::Sampler(texture.sampler()),
                        })
                    },
                    _ => None,
                }
            })
            .collect::<Vec<_>>();

        self.bind_group_entries.extend(bind_group_entries);

        self
    }

    pub fn build(
        &self, 
        render_device: &RenderDevice,
        layout: &ShaderResourceLayout,
    ) -> ShaderResource {
        let bind_group = render_device.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: layout.label
                .as_ref()
                .map(|label| format!("{label} Bind Group"))
                .as_deref(),
            layout: &layout.bind_group_layout,
            entries: &self.bind_group_entries,
        });

        ShaderResource { bind_group }
    }
}

#[derive(Debug)]
pub struct ShaderResourceLayout {
    label: Option<String>,
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
}

impl ShaderResourceLayout {
    pub fn builder() -> ShaderResourceLayoutBuilder {
        ShaderResourceLayoutBuilder {
            bind_group_layout_entries: vec![],
            label: None,
        }
    }
}

#[derive(Debug)]
pub struct ShaderResource {
    pub(crate) bind_group: wgpu::BindGroup,
}

impl ShaderResource {
    pub fn builder<'a>() -> ShaderResourceBuilder<'a> {
        ShaderResourceBuilder {
            bind_group_entries: vec![],
        }
    }
}