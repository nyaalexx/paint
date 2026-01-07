use std::mem;

use glam::{Mat2, Vec2};

use crate::{pipeline_layouts, shaders};

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, zerocopy::IntoBytes, zerocopy::Immutable)]
pub struct Immediates {
    pub transform: Mat2,
    pub translation: Vec2,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, zerocopy::IntoBytes, zerocopy::Immutable)]
pub struct Instance {
    pub pos: Vec2,
    pub radius: f32,
}

pub fn compile(
    device: &wgpu::Device,
    shaders: &shaders::Storage,
    pipeline_layouts: &pipeline_layouts::Storage,
) -> wgpu::RenderPipeline {
    let shader = shaders.get(shaders::Key::StampedBrush);

    let layout = pipeline_layouts.get(pipeline_layouts::Key {
        bind_group_layouts: vec![],
        immediate_size: mem::size_of::<Immediates>() as u32,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("StampedBrush Render Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vertex"),
            compilation_options: Default::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: mem::size_of::<Instance>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: mem::offset_of!(Instance, pos) as u64,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32,
                        offset: mem::offset_of!(Instance, radius) as u64,
                        shader_location: 1,
                    },
                ],
            }],
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
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Max,
                    },
                }),
                write_mask: wgpu::ColorWrites::all(),
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}
