use std::collections::{HashMap, hash_map};
use std::f32::consts::PI;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

use paint_core::color::{Color, NonlinearSrgb, Okhsv};
use wgpu::util::DeviceExt as _;

/// Kind of 3D color space and a 2D slice of that space.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Kind {
    /// Constant hue Okhsv slice.
    OkhsvHueSlice,
}

fn create_fixed_slice_jobs(num_slices: u32) -> Vec<(Kind, f32)> {
    assert!(num_slices >= 2);

    // generate the slices in this order: first, last, first + 1, last - 1, ...
    // we have to reverse because job run in the opposite order (Vec::pop)
    let indices = (0..num_slices).rev().map(|i| {
        if i % 2 == 0 {
            i / 2
        } else {
            num_slices - 1 - i / 2
        }
    });

    indices
        .flat_map(|i| {
            let constant = (i as f32) / (num_slices as f32 - 1.0);
            [(Kind::OkhsvHueSlice, constant * 2.0 * PI)]
        })
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

/// Uploads a texture storing a rasterized slice, returning its view.
fn create_slice_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
    data: &[u8],
) -> wgpu::TextureView {
    let texture = device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
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
        data,
    );

    texture.create_view(&Default::default())
}

/// Rasterizes a 2D slice of a 3D color space.
///
/// Returns a row-major sRGBA texture, 4 bytes per pixel.
fn rasterize_slice(width: u32, height: u32, kind: Kind, constant: f32) -> Vec<u8> {
    match kind {
        Kind::OkhsvHueSlice => {
            rasterize_rectangular_plot(width, height, |x, y| Okhsv::new(constant, x, y))
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
            buf[4 * (y * width + x)] = rgb.r;
            buf[4 * (y * width + x) + 1] = rgb.g;
            buf[4 * (y * width + x) + 2] = rgb.b;
            buf[4 * (y * width + x) + 3] = 255;
        }
    }

    buf
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
) -> Vec<u8> {
    rasterize_rectangular_plot(width, height, |x, y| {
        let x = 2.0 * x - 1.0;
        let y = 2.0 * y - 1.0;
        let angle = PI + f32::atan2(-y, -x);
        let radius = f32::hypot(x, y).clamp(0.0, 1.0);
        f(angle, radius)
    })
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

/// Stores cached 2D slices of a 3D color space.
#[derive(Debug)]
pub struct Cache {
    shared: Arc<Shared>,
    worker_thread_handle: Option<JoinHandle<()>>,
}

impl Cache {
    /// Creates a new [`Cache`] with the given settings.
    ///
    /// Spawns a background computation thread which will be stopped when this
    /// cache is destroyed.
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, settings: CacheSettings) -> Cache {
        assert!(settings.width >= 2);
        assert!(settings.height >= 2);
        assert!(settings.num_fixed_slices >= 2);

        let fixed_jobs = create_fixed_slice_jobs(settings.num_fixed_slices);

        let shared = Arc::new(Shared {
            device,
            queue,
            settings,
            slices: Mutex::new(HashMap::new()),
            dynamic_jobs: Mutex::new(HashMap::new()),
            is_running: Mutex::new(true),
            wake_flag: Mutex::new(true),
            wake_condvar: Condvar::new(),
        });

        let mut worker_thread = WorkerThread {
            shared: shared.clone(),
            dynamic_jobs: HashMap::new(),
            fixed_jobs,
        };

        let worker_thread_handle = std::thread::spawn(move || worker_thread.run());

        Self {
            shared,
            worker_thread_handle: Some(worker_thread_handle),
        }
    }

    /// Gets a slice or an interpolated pair of slices from the cache.
    ///
    /// If there's no exact hit, schedules a slice to be rendered.
    pub fn get_or_schedule(&self, kind: Kind, constant: f32) -> Option<CacheHit> {
        let res = self.get(kind, constant);

        if !matches!(&res, Some(CacheHit::Exact { .. })) {
            self.schedule(kind, constant);
        }

        res
    }

    fn get(&self, kind: Kind, constant: f32) -> Option<CacheHit> {
        let slices = self.shared.slices.lock().unwrap();
        let list = slices.get(&kind)?;

        if let Some(slice) = &list.dynamic_slice
            && slice.constant == constant
        {
            return Some(CacheHit::Exact {
                texture_view: slice.texture_view.clone(),
            });
        }

        // find slices to interpolate from using the sorted list
        let slice_b_idx = list
            .fixed_slices
            .iter()
            .position(|slice| slice.constant >= constant)?;

        let slice_b = &list.fixed_slices[slice_b_idx];

        if slice_b.constant == constant {
            return Some(CacheHit::Exact {
                texture_view: slice_b.texture_view.clone(),
            });
        }

        let slice_a_idx = slice_b_idx.checked_sub(1)?;
        let slice_a = &list.fixed_slices[slice_a_idx];

        let alpha = (constant - slice_a.constant) / (slice_b.constant - slice_a.constant);

        Some(CacheHit::Interpolated {
            texture_view_a: slice_a.texture_view.clone(),
            texture_view_b: slice_b.texture_view.clone(),
            alpha,
        })
    }

    fn schedule(&self, kind: Kind, constant: f32) {
        let mut jobs = self.shared.dynamic_jobs.lock().unwrap();

        match jobs.entry(kind) {
            hash_map::Entry::Occupied(entry) if *entry.get() == constant => {
                // already working on this job, no need to wake
                return;
            }
            hash_map::Entry::Occupied(mut entry) => {
                // there's a scheduled job for another value, but we only keep one dynamic slice
                // so we overwrite
                *entry.get_mut() = constant;
            }
            hash_map::Entry::Vacant(entry) => {
                // create a new job
                entry.insert(constant);
            }
        }

        drop(jobs); // unlock mutex before waking
        self.wake_worker();
    }

    fn wake_worker(&self) {
        let mut flag = self.shared.wake_flag.lock().unwrap();
        *flag = true;
        self.shared.wake_condvar.notify_one();
    }

    fn stop_worker(&mut self) {
        *self.shared.is_running.lock().unwrap() = false;
        self.wake_worker();
        self.worker_thread_handle.take().unwrap().join().unwrap();
    }
}

impl Drop for Cache {
    fn drop(&mut self) {
        self.stop_worker();
    }
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
struct Shared {
    device: wgpu::Device,
    queue: wgpu::Queue,
    settings: CacheSettings,
    slices: Mutex<HashMap<Kind, SliceList>>,
    dynamic_jobs: Mutex<HashMap<Kind, f32>>,
    is_running: Mutex<bool>,
    wake_flag: Mutex<bool>,
    wake_condvar: Condvar,
}

#[derive(Debug, Default)]
struct SliceList {
    /// Fixed slices, evenly spread and built on initialization.
    ///
    /// Sorted by constant.
    fixed_slices: Vec<Slice>,
    /// A single dynamic slice, taking any arbitrary value.
    dynamic_slice: Option<Slice>,
}

struct WorkerThread {
    shared: Arc<Shared>,
    // here it's a map of kind -> constant, because we want 1 dynamic slice per kind
    dynamic_jobs: HashMap<Kind, f32>,
    // here it's a list, because we want many fixed slices per kind
    fixed_jobs: Vec<(Kind, f32)>,
}

impl WorkerThread {
    pub fn run(&mut self) {
        while self.is_running() {
            self.collect_dynamic_jobs();
            if self.has_any_jobs() {
                self.run_single_job();
            } else {
                self.wait_until_woken();
            }
        }
    }

    fn is_running(&self) -> bool {
        *self.shared.is_running.lock().unwrap()
    }

    fn collect_dynamic_jobs(&mut self) {
        let mut dynamic_jobs = self.shared.dynamic_jobs.lock().unwrap();
        for (kind, constant) in dynamic_jobs.drain() {
            self.dynamic_jobs.insert(kind, constant);
        }
    }

    fn has_any_jobs(&self) -> bool {
        !self.dynamic_jobs.is_empty() || !self.fixed_jobs.is_empty()
    }

    fn wait_until_woken(&self) {
        let mut flag = self.shared.wake_flag.lock().unwrap();
        flag = self
            .shared
            .wake_condvar
            .wait_while(flag, |wake| !*wake)
            .unwrap();
        *flag = false;
    }

    fn run_single_job(&mut self) {
        if let Some(&kind) = self.dynamic_jobs.keys().next() {
            let constant = self.dynamic_jobs.remove(&kind).unwrap();
            let slice = self.build_slice(kind, constant);
            self.insert_dynamic_slice(kind, slice);
        } else if let Some((kind, constant)) = self.fixed_jobs.pop() {
            let slice = self.build_slice(kind, constant);
            self.insert_fixed_slice(kind, slice);
        }
    }

    fn build_slice(&self, kind: Kind, constant: f32) -> Slice {
        let data = rasterize_slice(
            self.shared.settings.width,
            self.shared.settings.height,
            kind,
            constant,
        );

        let texture_view = create_slice_texture(
            &self.shared.device,
            &self.shared.queue,
            self.shared.settings.width,
            self.shared.settings.height,
            &data,
        );

        Slice {
            constant,
            texture_view,
        }
    }

    fn insert_dynamic_slice(&self, kind: Kind, slice: Slice) {
        tracing::trace!("Adding dynamic slice {kind:?} at {:?}", slice.constant);
        let mut slices = self.shared.slices.lock().unwrap();
        let list = slices.entry(kind).or_default();
        list.dynamic_slice = Some(slice);
    }

    fn insert_fixed_slice(&self, kind: Kind, slice: Slice) {
        tracing::trace!("Adding fixed slice {kind:?} at {:?}", slice.constant);
        let mut slices = self.shared.slices.lock().unwrap();
        let list = slices.entry(kind).or_default();
        list.fixed_slices.push(slice);
        list.fixed_slices
            .sort_unstable_by(|s1, s2| f32::total_cmp(&s1.constant, &s2.constant));
        list.fixed_slices.dedup_by_key(|s| s.constant);
    }
}
