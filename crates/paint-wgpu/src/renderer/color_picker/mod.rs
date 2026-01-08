mod slice;

use std::f32::consts::PI;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;

use paint_core::presentation;
use zerocopy::IntoBytes;

use crate::context::GlobalContext;
use crate::{FrameContext, bind_group_layouts, render_pipelines};

#[derive(Debug)]
pub struct ColorPickerRenderer {
    context: Arc<GlobalContext>,
    sampler: wgpu::Sampler,
    slice_cache: slice::Cache,
    t: AtomicU32, // TODO: remove
}

impl ColorPickerRenderer {
    pub fn new(context: Arc<GlobalContext>) -> Self {
        let sampler = create_sampler(&context.device);

        let slice_cache = slice::Cache::new(
            context.device.clone(),
            context.queue.clone(),
            slice::CacheSettings {
                width: 256,
                height: 256,
                num_fixed_slices: 32,
            },
        );

        Self {
            context,
            sampler,
            slice_cache,
            t: AtomicU32::new(0),
        }
    }

    pub fn render(
        &self,
        mut ctx: FrameContext,
        target: &wgpu::Texture,
        _color_picker: &presentation::ColorPicker,
    ) {
        // TODO: remove
        let t = self.t.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let t = (t % 500) as f32 / 500.0;

        let target_view = target.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            ..Default::default()
        });

        let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        let hit = self
            .slice_cache
            .get_or_schedule(slice::Kind::OkhsvHueSlice, t * 2.0 * PI);

        match hit {
            Some(slice::CacheHit::Exact { texture_view }) => {
                let pipeline = self
                    .context
                    .render_pipelines
                    .get(render_pipelines::Key::FullscreenTriangle);
                pass.set_pipeline(&pipeline);

                let bind_group = bind_group_layouts::sampled_textures::create_bind_group(
                    &self.context.device,
                    &self.context.bind_group_layouts,
                    &self.sampler,
                    &[&texture_view],
                );
                pass.set_bind_group(0, &bind_group, &[]);
                pass.draw(0..3, 0..1);
            }

            Some(slice::CacheHit::Interpolated {
                texture_view_a,
                texture_view_b,
                alpha,
            }) => {
                let pipeline = self
                    .context
                    .render_pipelines
                    .get(render_pipelines::Key::FullscreenTriangleInterpolateTwoTextures);
                pass.set_pipeline(&pipeline);

                pass.set_immediates(
                    0,
                    render_pipelines::fullscreen_triangle_interpolate_two_textures::Immediates {
                        alpha,
                    }
                    .as_bytes(),
                );

                let bind_group = bind_group_layouts::sampled_textures::create_bind_group(
                    &self.context.device,
                    &self.context.bind_group_layouts,
                    &self.sampler,
                    &[&texture_view_a, &texture_view_b],
                );
                pass.set_bind_group(0, &bind_group, &[]);
                pass.draw(0..3, 0..1);
            }

            _ => (),
        }

        drop(pass);

        let command_buffer = ctx.encoder.finish();
        self.context.queue.submit(std::iter::once(command_buffer));
    }
}

fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: None,
        min_filter: wgpu::FilterMode::Linear,
        mag_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    })
}
