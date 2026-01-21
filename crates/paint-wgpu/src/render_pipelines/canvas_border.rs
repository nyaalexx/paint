use std::mem;

use glam::Vec2;

use crate::{pipeline_layouts, shaders};

#[repr(C)]
#[derive(Debug, Clone, Copy, zerocopy::IntoBytes, zerocopy::Immutable)]
pub struct Vertex {
    pub pos_ndc: Vec2,
    pub corner_dist: f32,
}

pub fn compile(
    device: &wgpu::Device,
    shaders: &shaders::Storage,
    pipeline_layouts: &pipeline_layouts::Storage,
) -> wgpu::RenderPipeline {
    let shader = shaders.get(shaders::Key::CanvasBorder);

    let layout = pipeline_layouts.get(pipeline_layouts::Key {
        bind_group_layouts: vec![],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("SingleQuad Render Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vertex"),
            compilation_options: Default::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: mem::size_of::<Vertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: mem::offset_of!(Vertex, pos_ndc) as u64,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32,
                        offset: mem::offset_of!(Vertex, corner_dist) as u64,
                        shader_location: 1,
                    },
                ],
            }],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fragment"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::all(),
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}
