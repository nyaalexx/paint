use std::borrow::Cow;
use std::sync::Arc;

use glam::{Affine2, UVec2, Vec2};
use paint_core::behaviour::DownloadedTexture;
use paint_core::persistence;
use zerocopy::IntoBytes as _;

use crate::{FrameContext, GlobalContext, Texture, bind_group_layouts, render_pipelines};

pub struct Compositor {
    context: Arc<GlobalContext>,
    pipeline: wgpu::RenderPipeline,
    canvas_texture_view: wgpu::TextureView,
    should_clear: bool,
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let canvas_texture_view = canvas_texture.create_view(&Default::default());

        Self {
            context,
            pipeline,
            canvas_texture_view,
            should_clear: true,
        }
    }
}

impl paint_core::behaviour::Compositor for Compositor {
    type Texture = Texture;
    type Context = FrameContext;

    fn put_texture(&mut self, ctx: &mut Self::Context, texture: Self::Texture) {
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
                    load: if self.should_clear {
                        self.should_clear = false;
                        wgpu::LoadOp::Clear(wgpu::Color::WHITE)
                    } else {
                        wgpu::LoadOp::Load
                    },
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

    fn render(&mut self, _ctx: &mut Self::Context) -> Self::Texture {
        Texture(self.canvas_texture_view.clone())
    }

    fn download(
        &mut self,
        ctx: &mut Self::Context,
    ) -> impl Future<Output = impl DownloadedTexture> + Send + 'static {
        let texture_size = self.canvas_texture_view.texture().size();
        let bytes_per_block = 4; // for now the format is hardcoded
        let bytes_per_row = (bytes_per_block * texture_size.width).next_multiple_of(256);
        let rows_per_image = texture_size.height;
        let buffer_size = u64::from(bytes_per_row) * u64::from(rows_per_image);

        let buffer = self.context.device.create_buffer(&wgpu::BufferDescriptor {
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
            label: None,
        });

        let src = wgpu::TexelCopyTextureInfo {
            texture: self.canvas_texture_view.texture(),
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        };

        let dst = wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(rows_per_image),
            },
        };

        ctx.encoder.copy_texture_to_buffer(src, dst, texture_size);

        let (sender, receiver) = oneshot::channel();
        ctx.encoder
            .map_buffer_on_submit(&buffer.clone(), wgpu::MapMode::Read, .., move |res| {
                res.unwrap();
                tracing::debug!("Downloaded texture of {buffer_size} bytes");

                let tex = DownloadedTextureImpl {
                    resolution: UVec2::new(texture_size.width, texture_size.height),
                    format: persistence::TextureFormat::Rgba8NonlinearSrgb,
                    buffer_view: buffer.get_mapped_range(..),
                    row_stride: bytes_per_row as usize,
                };

                let _ = sender.send(tex);
            });

        async move { receiver.await.unwrap() }
    }
}

#[derive(Debug)]
struct DownloadedTextureImpl {
    resolution: UVec2,
    format: persistence::TextureFormat,
    buffer_view: wgpu::BufferView,
    row_stride: usize,
}

impl DownloadedTexture for DownloadedTextureImpl {
    fn as_persistence(&self) -> persistence::Texture<'_> {
        persistence::Texture {
            resolution: self.resolution,
            format: self.format,
            data: Cow::Borrowed(&self.buffer_view),
            row_stride: self.row_stride,
        }
    }
}
