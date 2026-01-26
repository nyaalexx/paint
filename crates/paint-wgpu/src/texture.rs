use std::borrow::Cow;

use glam::UVec2;
use paint_core::persistence;

use crate::FrameContext;

#[derive(Debug, Clone)]
pub struct Texture(pub wgpu::TextureView);

impl paint_core::behaviour::Texture for Texture {
    type Context = FrameContext;
    type Downloaded = DownloadedTexture;

    fn download(
        &self,
        ctx: &mut Self::Context,
    ) -> impl Future<Output = Self::Downloaded> + Send + 'static {
        let texture_size = self.0.texture().size();

        let (bytes_per_block, persistence_format) = match self.0.texture().format() {
            wgpu::TextureFormat::Rgba8UnormSrgb => {
                (4, persistence::TextureFormat::Rgba8NonlinearSrgb)
            }
            _ => unimplemented!(),
        };

        let bytes_per_row = (bytes_per_block * texture_size.width).next_multiple_of(256);
        let rows_per_image = texture_size.height;
        let buffer_size = u64::from(bytes_per_row) * u64::from(rows_per_image);

        let buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
            label: None,
        });

        let src = wgpu::TexelCopyTextureInfo {
            texture: self.0.texture(),
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

                let tex = DownloadedTexture {
                    resolution: UVec2::new(texture_size.width, texture_size.height),
                    format: persistence_format,
                    buffer_view: buffer.get_mapped_range(..),
                    row_stride: bytes_per_row as usize,
                };

                let _ = sender.send(tex);
            });

        async move { receiver.await.unwrap() }
    }
}

#[derive(Debug)]
pub struct DownloadedTexture {
    resolution: UVec2,
    format: persistence::TextureFormat,
    buffer_view: wgpu::BufferView,
    row_stride: usize,
}

impl paint_core::behaviour::DownloadedTexture for DownloadedTexture {
    fn as_persistence(&self) -> persistence::Texture<'_> {
        persistence::Texture {
            resolution: self.resolution,
            format: self.format,
            data: Cow::Borrowed(&self.buffer_view),
            row_stride: self.row_stride,
        }
    }
}
