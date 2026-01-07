use std::f32::consts::PI;
use std::sync::Arc;

use paint_core::color::{Color, NonlinearSrgb, Okhsl, Okhsv};
use paint_core::presentation;
use wgpu::util::DeviceExt as _;

use crate::context::GlobalContext;
use crate::{FrameContext, bind_group_layouts, render_pipelines};

#[derive(Debug)]
pub struct ColorPickerRenderer {
    context: Arc<GlobalContext>,
    sampler: wgpu::Sampler,
}

impl ColorPickerRenderer {
    pub fn new(context: Arc<GlobalContext>) -> Self {
        let sampler = create_sampler(&context.device);
        Self { context, sampler }
    }

    pub fn render(
        &self,
        mut ctx: FrameContext,
        target: &wgpu::Texture,
        _color_picker: &presentation::ColorPicker,
    ) {
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
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        let pipeline = self
            .context
            .render_pipelines
            .get(render_pipelines::Key::FullscreenTriangle);
        pass.set_pipeline(&pipeline);

        let texture_data = rasterize_okhsv_hue_slice(64, 64, 0.2);
        let texture = self.context.device.create_texture_with_data(
            &self.context.queue,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: 64,
                    height: 64,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &texture_data,
        );

        let bind_group = bind_group_layouts::single_sampled_texture::create_bind_group(
            &self.context.device,
            &self.context.bind_group_layouts,
            &texture.create_view(&Default::default()),
            &self.sampler,
        );
        pass.set_bind_group(0, &bind_group, &[]);

        pass.draw(0..3, 0..1);

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

fn rasterize_rectangular_plot<C: Color, F: Fn(f32, f32) -> C>(
    width: u32,
    height: u32,
    f: F,
) -> Vec<u8> {
    let width = width as usize;
    let height = height as usize;

    let mut buf = vec![0; 4 * width * height];

    for x in 0..width {
        for y in 0..height {
            let fx = (x as f32) / (width as f32 - 1.0);
            let fy = 1.0 - (y as f32) / (height as f32 - 1.0);
            let color = f(fx, fy).to_linear_srgb_clamped();
            let rgb = NonlinearSrgb::<u8>::from_linear_srgb(color);
            buf[4 * (y * width + x) + 0] = rgb.r;
            buf[4 * (y * width + x) + 1] = rgb.g;
            buf[4 * (y * width + x) + 2] = rgb.b;
            buf[4 * (y * width + x) + 3] = 255;
        }
    }

    buf
}

#[allow(unused)]
fn rasterize_circular_plot<C: Color, F: Fn(f32, f32) -> C>(
    width: u32,
    height: u32,
    f: F,
) -> Vec<u8> {
    rasterize_rectangular_plot(width, height, |x, y| {
        let x = 2.0 * x - 1.0;
        let y = 2.0 * y - 1.0;
        let angle = PI + f32::atan2(-y, -x);
        let radius = f32::hypot(x, y).clamp(0.0, 1.0);
        f(angle, radius)
    })
}

fn rasterize_okhsv_hue_slice(width: u32, height: u32, hue: f32) -> Vec<u8> {
    rasterize_rectangular_plot(width, height, |x, y| Okhsv::new(hue, x, y))
}

#[allow(unused)]
fn rasterize_okhsl_lightness_slice(width: u32, height: u32, lightness: f32) -> Vec<u8> {
    rasterize_circular_plot(width, height, |angle, radius| {
        Okhsl::new(angle, radius, lightness)
    })
}
