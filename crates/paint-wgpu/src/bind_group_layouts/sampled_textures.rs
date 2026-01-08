pub fn create(device: &wgpu::Device, count: usize) -> wgpu::BindGroupLayout {
    let mut entries = Vec::with_capacity(count + 1);

    entries.push(wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    });

    for i in 0..count {
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: (i + 1) as u32,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        })
    }

    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Sampled Textures Bind Group Layout"),
        entries: &entries,
    })
}

pub fn create_bind_group(
    device: &wgpu::Device,
    bind_group_layouts: &super::Storage,
    sampler: &wgpu::Sampler,
    texture_views: &[&wgpu::TextureView],
) -> wgpu::BindGroup {
    let mut entries = Vec::with_capacity(texture_views.len() + 1);

    entries.push(wgpu::BindGroupEntry {
        binding: 0,
        resource: wgpu::BindingResource::Sampler(sampler),
    });

    for (i, texture_view) in texture_views.iter().enumerate() {
        entries.push(wgpu::BindGroupEntry {
            binding: (i + 1) as u32,
            resource: wgpu::BindingResource::TextureView(texture_view),
        });
    }

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Sampled Textures Bind Group"),
        layout: &bind_group_layouts.get(super::Key::SampledTextures {
            num_texture_bindings: texture_views.len(),
        }),
        entries: &entries,
    })
}
