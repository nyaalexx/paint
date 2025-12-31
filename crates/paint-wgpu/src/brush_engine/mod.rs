use std::sync::Arc;

use glam::{Affine2, Vec2};
use paint_core::behaviour::{BrushState, StrokeSettings};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use wgpu::util::DeviceExt;
use zerocopy::IntoBytes as _;

use crate::context::{Context, FrameContext};
use crate::render_pipelines;
use crate::render_pipelines::stamped_brush::{Immediates, Instance};
use crate::texture::Texture;

pub struct BrushEngine {
    context: Arc<Context>,
}

impl BrushEngine {
    pub fn new(context: Arc<Context>) -> Self {
        Self { context }
    }
}

impl paint_core::behaviour::BrushEngine for BrushEngine {
    type Stroke = BrushStroke;

    fn begin_stroke(&self, settings: &StrokeSettings) -> Self::Stroke {
        BrushStroke::new(self.context.clone(), settings)
    }
}

pub struct BrushStroke {
    context: Arc<Context>,
    render_pipeline: wgpu::RenderPipeline,
    preview_texture: wgpu::Texture,
    preview_texture_view: wgpu::TextureView,
    instances: Vec<Instance>,
    last_instance: Option<Instance>,
    should_clear: bool,
    rng: SmallRng,
}

impl BrushStroke {
    pub fn new(context: Arc<Context>, settings: &StrokeSettings) -> Self {
        let render_pipeline = context
            .render_pipelines
            .get(render_pipelines::Key::StampedBrush);

        let preview_texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Brush Stroke Preview Texture"),
            size: wgpu::Extent3d {
                width: settings.canvas_resolution.x,
                height: settings.canvas_resolution.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let preview_texture_view = preview_texture.create_view(&Default::default());

        Self {
            context,
            render_pipeline,
            preview_texture,
            preview_texture_view,
            instances: Vec::new(),
            last_instance: None,
            should_clear: true,
            rng: SmallRng::from_os_rng(),
        }
    }
}

impl paint_core::behaviour::BrushStroke for BrushStroke {
    type Texture = Texture;
    type FrameContext = FrameContext;

    fn update(&mut self, state: &BrushState) {
        let radius = (state.pressure.powf(1.5) * 50.0).max(2.0);

        if let Some(prev_instance) = self.last_instance {
            let spacing = radius * 0.1;
            let dir = state.position - prev_instance.pos;
            let dist = dir.length();
            if dist > spacing {
                let dir = dir / dist;
                let mut dist_along_dir = spacing;
                while dist_along_dir < dist {
                    let pos = prev_instance.pos + dir * dist_along_dir;
                    let radius = radius * dist_along_dir / dist
                        + prev_instance.radius * (1.0 - dist_along_dir / dist);
                    let jitter_x = self.rng.random_range(-1.0..1.0) * 0.5;
                    let jitter_y = self.rng.random_range(-1.0..1.0) * 0.5;
                    let pos = pos + Vec2::new(jitter_x, jitter_y);
                    self.instances.push(Instance { pos, radius });
                    dist_along_dir += spacing;
                }
            }
        }

        let pos = state.position;
        let instance = Instance { pos, radius };

        self.instances.push(instance);
        self.last_instance = Some(instance);
    }

    fn render(&mut self, ctx: &mut FrameContext) -> Texture {
        let resolution = self.preview_texture.size();
        let resolution = Vec2::new(resolution.width as f32, resolution.height as f32);

        let pixel_to_ndc = Affine2::from_translation(Vec2::new(-1.0, 1.0))
            * Affine2::from_scale(Vec2::new(2.0, -2.0) / resolution);

        let immediates = Immediates {
            transform: pixel_to_ndc.matrix2,
            translation: pixel_to_ndc.translation,
        };

        let buffer = if self.instances.is_empty() {
            None
        } else {
            let buffer =
                self.context
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: self.instances.as_slice().as_bytes(),
                        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                    });
            Some(buffer)
        };

        let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.preview_texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: if self.should_clear {
                        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                    } else {
                        wgpu::LoadOp::Load
                    },
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        if let Some(buffer) = buffer {
            pass.set_pipeline(&self.render_pipeline);
            pass.set_immediates(0, immediates.as_bytes());
            pass.set_vertex_buffer(0, buffer.slice(..));
            pass.draw(0..6, 0..self.instances.len() as u32);
        }

        drop(pass);

        self.instances.clear();
        self.should_clear = false;

        Texture(self.preview_texture_view.clone())
    }
}
