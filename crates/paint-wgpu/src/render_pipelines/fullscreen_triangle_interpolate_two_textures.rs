use std::mem;

use crate::{bind_group_layouts, pipeline_layouts, shaders};

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, zerocopy::IntoBytes, zerocopy::Immutable)]
pub struct Immediates {
    pub alpha: f32,
}

pub fn compile(
    device: &wgpu::Device,
    shaders: &shaders::Storage,
    pipeline_layouts: &pipeline_layouts::Storage,
) -> wgpu::RenderPipeline {
    let shader = shaders.get(shaders::Key::FullscreenTriangleInterpolateTwoTextures);

    let layout = pipeline_layouts.get(pipeline_layouts::Key {
        bind_group_layouts: vec![bind_group_layouts::Key::SampledTextures {
            num_texture_bindings: 2,
        }],
        immediate_size: mem::size_of::<Immediates>() as u32,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("FullscreenTriangleInterpolateTwoTextures Render Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vertex"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fragment"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: None,
                write_mask: wgpu::ColorWrites::all(),
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}
