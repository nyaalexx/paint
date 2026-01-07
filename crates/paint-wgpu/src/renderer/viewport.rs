use std::sync::Arc;

use glam::{Affine2, Vec2};
use paint_core::presentation;
use zerocopy::IntoBytes;

use crate::context::GlobalContext;
use crate::texture::Texture;
use crate::{FrameContext, bind_group_layouts, render_pipelines};

#[derive(Debug)]
pub struct ViewportRenderer {
    context: Arc<GlobalContext>,
    default_bind_group: wgpu::BindGroup,
}

impl ViewportRenderer {
    pub fn new(context: Arc<GlobalContext>) -> Self {
        let default_bind_group = bind_group_layouts::single_sampled_texture::create_bind_group(
            &context.device,
            &context.bind_group_layouts,
            &context.default_texture_view,
            &context.default_sampler,
        );

        Self {
            context,
            default_bind_group,
        }
    }

    pub fn render(
        &self,
        mut ctx: FrameContext,
        target: &wgpu::Texture,
        viewport: &presentation::Viewport<Texture>,
    ) {
        let resolution = Vec2::new(target.width() as f32, target.height() as f32);

        let pixel_to_ndc = Affine2::from_translation(Vec2::new(-1.0, 1.0))
            * Affine2::from_scale(Vec2::new(2.0, -2.0) / resolution);

        let transform = pixel_to_ndc
            * viewport.transform
            * Affine2::from_scale(viewport.canvas.resolution.as_vec2());

        let immediates = render_pipelines::single_quad::Immediates {
            transform: transform.matrix2,
            translation: transform.translation,
        };

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
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        let pipeline = self
            .context
            .render_pipelines
            .get(render_pipelines::Key::SingleQuad);
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &self.default_bind_group, &[]);
        pass.set_immediates(0, immediates.as_bytes());

        pass.draw(0..6, 0..1);

        for layer in &viewport.canvas.layers {
            match layer {
                presentation::Layer::Texture(texture) => {
                    let bind_group = bind_group_layouts::single_sampled_texture::create_bind_group(
                        &self.context.device,
                        &self.context.bind_group_layouts,
                        &texture.0,
                        &self.context.default_sampler,
                    );

                    pass.set_bind_group(0, &bind_group, &[]);
                }
            }

            pass.draw(0..6, 0..1);
        }

        drop(pass);

        let command_buffer = ctx.encoder.finish();
        self.context.queue.submit(std::iter::once(command_buffer));
    }
}
