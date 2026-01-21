use std::collections::VecDeque;
use std::sync::Arc;

use glam::{Affine2, Vec2};
use zerocopy::IntoBytes as _;

use crate::{FrameContext, GlobalContext, Texture, bind_group_layouts, render_pipelines};

pub struct Compositor {
    context: Arc<GlobalContext>,
    pipeline: wgpu::RenderPipeline,
    canvas_texture_view: wgpu::TextureView,
    actions: VecDeque<Action>,
}

enum Action {
    Clear,
    PutTexture(Texture),
}

impl Compositor {
    pub fn new(context: Arc<GlobalContext>) -> Self {
        let pipeline = context
            .render_pipelines
            .get(render_pipelines::Key::SingleQuad);

        let canvas_texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 2304,
                height: 1440,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let canvas_texture_view = canvas_texture.create_view(&Default::default());
        let mut actions = VecDeque::new();
        actions.push_back(Action::Clear);

        Self {
            context,
            pipeline,
            canvas_texture_view,
            actions,
        }
    }

    fn action_clear(&mut self, ctx: &mut FrameContext) {
        let _pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.canvas_texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
    }

    fn action_put_texture(&mut self, ctx: &mut FrameContext, texture: Texture) {
        let transform = Affine2::from_translation(Vec2::new(-1.0, 1.0))
            * Affine2::from_scale(Vec2::new(2.0, -2.0));

        let immediates = render_pipelines::single_quad::Immediates {
            transform: transform.matrix2,
            translation: transform.translation,
        };

        let bind_group = bind_group_layouts::sampled_textures::create_bind_group(
            &self.context.device,
            &self.context.bind_group_layouts,
            &self.context.default_sampler,
            &[&texture.0],
        );

        let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.canvas_texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.set_immediates(0, immediates.as_bytes());
        pass.draw(0..6, 0..1);
    }
}

impl paint_core::behaviour::Compositor for Compositor {
    type Texture = Texture;
    type Context = FrameContext;

    fn put_texture(&mut self, texture: Self::Texture) {
        self.actions.push_back(Action::PutTexture(texture));
    }

    fn render(&mut self, ctx: &mut FrameContext) -> Self::Texture {
        while let Some(action) = self.actions.pop_front() {
            match action {
                Action::Clear => self.action_clear(ctx),
                Action::PutTexture(texture) => self.action_put_texture(ctx, texture),
            }
        }

        Texture(self.canvas_texture_view.clone())
    }
}
