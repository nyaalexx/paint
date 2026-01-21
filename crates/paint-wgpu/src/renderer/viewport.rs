use std::sync::Arc;

use glam::{Affine2, Vec2};
use paint_core::presentation;
use wgpu::util::DeviceExt;
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
        let default_bind_group = bind_group_layouts::sampled_textures::create_bind_group(
            &context.device,
            &context.bind_group_layouts,
            &context.default_sampler,
            &[&context.default_texture_view],
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
        let start_time = std::time::Instant::now();

        let resolution = Vec2::new(target.width() as f32, target.height() as f32);

        let pixel_to_ndc = Affine2::from_translation(Vec2::new(-1.0, 1.0))
            * Affine2::from_scale(Vec2::new(2.0, -2.0) / resolution);

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

        self.render_canvas_layers(&mut pass, pixel_to_ndc, viewport);
        self.render_canvas_border(&mut pass, pixel_to_ndc, viewport);
        drop(pass);

        // TODO: should finish and submit somewhere outside
        let command_buffer = ctx.encoder.finish();
        self.context.queue.submit(std::iter::once(command_buffer));

        tracing::trace!("viewport rendering CPU time is {:?}", start_time.elapsed());
    }

    fn render_canvas_border(
        &self,
        pass: &mut wgpu::RenderPass,
        pixel_to_ndc: Affine2,
        viewport: &presentation::Viewport<Texture>,
    ) {
        // assuming scale is uniform
        let scale = viewport.transform.to_scale_angle_translation().0.x;

        // border size in positive direction, in canvas space
        let bp = 3.0 / scale;
        // border size in negative direction, in canvas space
        let bn = -3.0 / scale;

        let size = viewport.canvas.resolution.as_vec2();

        // vertex positions relative to canvas origin (triangle strip topology)
        let positions = [
            // positive
            Vec2::new(-bp, -bp),
            Vec2::new(0.0, 0.0),
            Vec2::new(size.x + bp, -bp),
            Vec2::new(size.x, 0.0),
            Vec2::new(size.x + bp, size.y + bp),
            Vec2::new(size.x, size.y),
            Vec2::new(-bp, size.y + bp),
            Vec2::new(0.0, size.y),
            Vec2::new(-bp, -bp),
            Vec2::new(0.0, 0.0),
            // negative
            Vec2::new(-bn, -bn),
            Vec2::new(0.0, 0.0),
            Vec2::new(size.x + bn, -bn),
            Vec2::new(size.x, 0.0),
            Vec2::new(size.x + bn, size.y + bn),
            Vec2::new(size.x, size.y),
            Vec2::new(-bn, size.y + bn),
            Vec2::new(0.0, size.y),
            Vec2::new(-bn, -bn),
            Vec2::new(0.0, 0.0),
        ];

        // corner positions in canvas space
        let corners = [
            Vec2::new(0.0, 0.0),
            Vec2::new(size.x, 0.0),
            Vec2::new(0.0, size.y),
            Vec2::new(size.x, size.y),
        ];

        let vertices = positions.map(|pos| {
            // find nearest corner
            let (corner_pos, _) = corners
                .iter()
                .copied()
                .map(|c| (c, (c - pos).length_squared()))
                .min_by(|(_, a), (_, b)| f32::total_cmp(&a, &b))
                .unwrap();

            let pos_px = viewport.transform.transform_point2(pos);
            let pos_ndc = pixel_to_ndc.transform_point2(pos_px);
            let corner_pos_px = viewport.transform.transform_point2(corner_pos);
            let corner_dist = (corner_pos_px - pos_px).length();

            render_pipelines::canvas_border::Vertex {
                pos_ndc,
                corner_dist,
            }
        });

        let vertex_buffer =
            self.context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: vertices.as_bytes(),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let pipeline = self
            .context
            .render_pipelines
            .get(render_pipelines::Key::CanvasBorder);

        pass.set_pipeline(&pipeline);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..(vertices.len() as u32), 0..1);
    }

    fn render_canvas_layers(
        &self,
        pass: &mut wgpu::RenderPass,
        pixel_to_ndc: Affine2,
        viewport: &presentation::Viewport<Texture>,
    ) {
        let transform = pixel_to_ndc
            * viewport.transform
            * Affine2::from_scale(viewport.canvas.resolution.as_vec2());

        let pipeline = self
            .context
            .render_pipelines
            .get(render_pipelines::Key::SingleQuad);

        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &self.default_bind_group, &[]);

        let immediates = render_pipelines::single_quad::Immediates {
            transform: transform.matrix2,
            translation: transform.translation,
        };
        pass.set_immediates(0, immediates.as_bytes());

        pass.draw(0..6, 0..1);

        for layer in &viewport.canvas.layers {
            match layer {
                presentation::Layer::Texture(texture) => {
                    let bind_group = bind_group_layouts::sampled_textures::create_bind_group(
                        &self.context.device,
                        &self.context.bind_group_layouts,
                        &self.context.default_sampler,
                        &[&texture.0],
                    );

                    pass.set_bind_group(0, &bind_group, &[]);
                }
            }

            pass.draw(0..6, 0..1);
        }
    }
}
