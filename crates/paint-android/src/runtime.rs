use std::sync::Arc;

mod ffi {
    use jni::JNIEnv;
    use jni::objects::JObject;
    use jni_fn::jni_fn;

    use super::*;

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.Runtime$Native")]
    pub fn create(_env: JNIEnv, _this: JObject) -> usize {
        let runtime = Runtime::new();
        Box::into_raw(Box::new(runtime)) as usize
    }

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.Runtime$Native")]
    pub fn destroy(_env: JNIEnv, _this: JObject, ptr: usize) {
        unsafe {
            drop(Box::from_raw(ptr as *mut Runtime));
        }
    }
}

pub struct Runtime {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,

    pub context: Arc<paint_wgpu::GlobalContext>,
    pub viewport_renderer: Arc<paint_wgpu::ViewportRenderer>,
    pub color_picker_renderer: Arc<paint_wgpu::ColorPickerRenderer>,
}

impl Runtime {
    pub fn new() -> Self {
        let start_time = std::time::Instant::now();
        tracing::info!("Initializing runtime");

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        let adapter_fut = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        });
        let adapter = pollster::block_on(adapter_fut).unwrap();

        let info = adapter.get_info();
        tracing::info!("Adapter info: {info:#?}");

        let device_fut = adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: paint_wgpu::get_required_wgpu_features(),
            required_limits: paint_wgpu::get_required_wgpu_limits(),
            ..Default::default()
        });
        let (device, queue) = pollster::block_on(device_fut).unwrap();

        let context = Arc::new(paint_wgpu::GlobalContext::new(device.clone(), queue));

        let viewport_renderer = Arc::new(paint_wgpu::ViewportRenderer::new(context.clone()));
        let color_picker_renderer = Arc::new(paint_wgpu::ColorPickerRenderer::new(context.clone()));

        tracing::info!("Finished initialization in {:?}", start_time.elapsed());

        Self {
            instance,
            adapter,
            device,
            context,
            viewport_renderer,
            color_picker_renderer,
        }
    }
}
