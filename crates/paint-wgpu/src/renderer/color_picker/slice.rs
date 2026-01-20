use std::collections::HashMap;
use std::f32::consts::PI;

use paint_core::color::{Color, NonlinearSrgb, Okhsl, Okhsv};
use rayon::iter::{IntoParallelIterator, ParallelIterator as _};
use wgpu::util::DeviceExt as _;

/// Kind of 3D color space and a 2D (or 1D) slice of that space.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Kind {
    /// Constant hue Okhsv slice.
    OkhsvHueSlice,
    /// Okhsl hue vertical gradient.
    OkhslHueVerticalGradient,
}

fn create_fixed_slice_jobs(num_slices: u32) -> Vec<(Kind, f32)> {
    (0..num_slices)
        .flat_map(|i| {
            let constant = (i as f32) / (num_slices as f32 - 1.0);
            [(Kind::OkhsvHueSlice, constant * 2.0 * PI)]
        })
        .chain([(Kind::OkhslHueVerticalGradient, 0.0)])
        .collect()
}

/// Represents a 2D slice of a 3D color space.
#[derive(Debug)]
pub struct Slice {
    /// The constant coordinate (the exact kind depends on [`SliceKind`]).
    pub constant: f32,
    /// Underlying texture.
    pub texture_view: wgpu::TextureView,
}

struct TextureData {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

/// Uploads a texture storing a rasterized slice, returning its view.
fn create_slice_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    data: TextureData,
) -> wgpu::TextureView {
    let texture = device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: data.width,
                height: data.height,
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
        &data.data,
    );

    texture.create_view(&Default::default())
}

/// Rasterizes a 2D slice of a 3D color space.
///
/// Returns a row-major sRGBA texture, 4 bytes per pixel.
fn rasterize_slice(width: u32, height: u32, kind: Kind, constant: f32) -> TextureData {
    match kind {
        Kind::OkhsvHueSlice => {
            rasterize_rectangular_plot(width, height, |x, y| Okhsv::new(constant, x, y))
        }
        Kind::OkhslHueVerticalGradient => {
            rasterize_vertical_gradient(height, |y| Okhsl::new((1.0 - y) * 2.0 * PI, 1.0, 0.62))
        }
    }
}

/// Rasterizes a rectangular plot of `f(x, y)`.
///
/// Coordinates are normalized between 0 and 1, Y up.
///
/// Returns a row-major sRGBA texture, 4 bytes per pixel.
fn rasterize_rectangular_plot<C: Color, F: Fn(f32, f32) -> C>(
    width: u32,
    height: u32,
    f: F,
) -> TextureData {
    let width = width as usize;
    let height = height as usize;
    let mut data = vec![0; 4 * width * height];

    for x in 0..width {
        for y in 0..height {
            let fx = (x as f32) / (width as f32 - 1.0);
            let fy = 1.0 - (y as f32) / (height as f32 - 1.0);
            let color = f(fx, fy).to_linear_srgb_clamped();
            let rgb = NonlinearSrgb::<u8>::from_linear_srgb(color);
            data[4 * (y * width + x)] = rgb.r;
            data[4 * (y * width + x) + 1] = rgb.g;
            data[4 * (y * width + x) + 2] = rgb.b;
            data[4 * (y * width + x) + 3] = 255;
        }
    }

    TextureData {
        width: width as u32,
        height: height as u32,
        data,
    }
}

/// Rasterizes a radial plot of `f(angle, radius)`.
///
/// Angle is between 0 and 2π. Radius is between 0 and 1.
///
/// Returns a row-major sRGBA texture, 4 bytes per pixel.
#[allow(unused)] // we'll need this soon for more slice kinds
fn rasterize_circular_plot<C: Color, F: Fn(f32, f32) -> C>(
    width: u32,
    height: u32,
    f: F,
) -> TextureData {
    rasterize_rectangular_plot(width, height, |x, y| {
        let x = 2.0 * x - 1.0;
        let y = 2.0 * y - 1.0;
        let angle = PI + f32::atan2(-y, -x);
        let radius = f32::hypot(x, y).clamp(0.0, 1.0);
        f(angle, radius)
    })
}

/// Rasterizes a plot of `f(y)`.
///
/// Returns a row-major sRGBA texture, 4 bytes per pixel. Width is set to 1.
fn rasterize_vertical_gradient<C: Color, F: Fn(f32) -> C>(height: u32, f: F) -> TextureData {
    rasterize_rectangular_plot(1, height, |_x, y| f(y))
}

/// [`Cache`] settings.
#[derive(Debug, Clone, Copy)]
pub struct CacheSettings {
    /// Width of the slices.
    pub width: u32,
    /// Height of the slices.
    pub height: u32,
    /// Number of fixed slices per slice kind.
    pub num_fixed_slices: u32,
}

/// Result of [`Cache`] slice retrieval.
#[derive(Debug)]
pub enum CacheHit {
    /// Interpolation between two nearest slices.
    Interpolated {
        texture_view_a: wgpu::TextureView,
        texture_view_b: wgpu::TextureView,
        alpha: f32,
    },

    /// Exact slice for that constant exists.
    Exact { texture_view: wgpu::TextureView },
}

#[derive(Debug)]
pub struct Cache {
    kinds: HashMap<Kind, Vec<Slice>>,
}

impl Cache {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, settings: CacheSettings) -> Self {
        let fixed_slices = create_fixed_slice_jobs(settings.num_fixed_slices)
            .into_par_iter()
            .map(|(kind, constant)| {
                let data = rasterize_slice(settings.width, settings.height, kind, constant);
                let texture_view = create_slice_texture(&device, &queue, data);
                (
                    kind,
                    Slice {
                        constant,
                        texture_view,
                    },
                )
            })
            .collect::<Vec<_>>();

        let mut kinds = HashMap::<Kind, Vec<Slice>>::new();
        for (kind, slice) in fixed_slices {
            let list = kinds.entry(kind).or_default();
            list.push(slice);
        }

        Self { kinds }
    }

    pub fn get(&self, kind: Kind, constant: f32) -> Option<CacheHit> {
        let list = self.kinds.get(&kind)?;

        // find slices to interpolate from using the sorted list
        let slice_b_idx = list.iter().position(|slice| slice.constant >= constant)?;

        let slice_b = &list[slice_b_idx];

        if slice_b.constant == constant {
            return Some(CacheHit::Exact {
                texture_view: slice_b.texture_view.clone(),
            });
        }

        let slice_a_idx = slice_b_idx.checked_sub(1)?;
        let slice_a = &list[slice_a_idx];

        let alpha = (constant - slice_a.constant) / (slice_b.constant - slice_a.constant);

        Some(CacheHit::Interpolated {
            texture_view_a: slice_a.texture_view.clone(),
            texture_view_b: slice_b.texture_view.clone(),
            alpha,
        })
    }
}
