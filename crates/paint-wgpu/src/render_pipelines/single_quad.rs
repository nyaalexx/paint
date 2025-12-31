use glam::{Mat2, Vec2};

use crate::{bind_group_layouts, pipeline_layouts, shaders};

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, zerocopy::IntoBytes, zerocopy::Immutable)]
pub struct Immediates {
    pub transform: Mat2,
    pub translation: Vec2,
}

pub fn compile(
    device: &wgpu::Device,
    shaders: &shaders::Storage,
    pipeline_layouts: &pipeline_layouts::Storage,
) -> wgpu::RenderPipeline {
    let shader = shaders.get(shaders::Key::SingleQuad);

    let layout = pipeline_layouts.get(pipeline_layouts::Key {
        bind_group_layouts: vec![bind_group_layouts::Key::SingleSampledTexture],
        immediate_size: std::mem::size_of::<Immediates>() as u32,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("SingleQuad Render Pipeline"),
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
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::all(),
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}
