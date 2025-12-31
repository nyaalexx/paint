use std::sync::Arc;

use crate::surface::Window;

pub mod ffi {
    use jni::JNIEnv;
    use jni::objects::JObject;
    use jni_fn::jni_fn;

    use super::*;

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.GpuContext$Native")]
    pub fn create(_env: JNIEnv, _this: JObject) -> usize {
        let gpu = GpuContext::new();
        Box::into_raw(Box::new(gpu)) as usize
    }

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.GpuContext$Native")]
    pub fn destroy(_env: JNIEnv, _this: JObject, ptr: usize) {
        unsafe {
            drop(Box::from_raw(ptr as *mut GpuContext));
        }
    }
}

#[derive(Clone)]
pub struct GpuContext {
    pub context: Arc<paint_wgpu::Context>,

    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
}

impl GpuContext {
    pub fn new() -> Self {
        tracing::trace!("Creating a GPU context");

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        let adapter_fut = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        });
        let adapter = pollster::block_on(adapter_fut).unwrap();

        let info = adapter.get_info();
        tracing::trace!("Adapter info: {info:#?}");

        // Request the WGPU device
        let device_fut = adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: paint_wgpu::get_required_wgpu_features(),
            required_limits: paint_wgpu::get_required_wgpu_limits(),
            ..Default::default()
        });
        let (device, queue) = pollster::block_on(device_fut).unwrap();

        let context = Arc::new(paint_wgpu::Context::new(device.clone(), queue));

        Self {
            context,
            instance,
            adapter,
            device,
        }
    }

    pub fn create_surface(&self, window: Window) -> wgpu::Surface<'static> {
        let surface_target = wgpu::SurfaceTarget::from(window);
        self.instance.create_surface(surface_target).unwrap()
    }
}
