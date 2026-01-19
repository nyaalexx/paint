use std::sync::Arc;

use paint_core::presentation;

use crate::gpu::GpuContext;
use crate::surface::Surface;

pub mod ffi {
    use jni::JNIEnv;
    use jni::objects::JObject;
    use jni_fn::jni_fn;

    use super::*;

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.ColorPickerRenderer$Native")]
    pub fn create(_env: JNIEnv, _this: JObject, gpu_ptr: usize) -> usize {
        let gpu = unsafe { &*(gpu_ptr as *const GpuContext) };
        let behaviour = ColorPickerRenderer::new(gpu);
        Box::into_raw(Box::new(behaviour)) as usize
    }

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.ColorPickerRenderer$Native")]
    pub fn renderOkhsvHueSlice(
        _env: JNIEnv,
        _this: JObject,
        this_ptr: usize,
        surface_ptr: usize,
        hue: f32,
    ) {
        let this = unsafe { &*(this_ptr as *const ColorPickerRenderer) };
        let surface = unsafe { &*(surface_ptr as *const Arc<Surface>) };
        this.render(&surface, &presentation::ColorPicker::OkhsvHueSlice { hue });
    }

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.ColorPickerRenderer$Native")]
    pub fn destroy(_env: JNIEnv, _this: JObject, ptr: usize) {
        unsafe {
            drop(Box::from_raw(ptr as *mut ColorPickerRenderer));
        }
    }
}

pub struct ColorPickerRenderer {
    global_context: Arc<paint_wgpu::GlobalContext>,
    inner: Arc<paint_wgpu::ColorPickerRenderer>,
}

impl ColorPickerRenderer {
    pub fn new(gpu: &GpuContext) -> Self {
        Self {
            global_context: gpu.context.clone(),
            inner: Arc::new(paint_wgpu::ColorPickerRenderer::new(gpu.context.clone())),
        }
    }

    pub fn render(&self, surface: &Surface, color_picker: &presentation::ColorPicker) {
        let ctx = paint_wgpu::FrameContext::new(&self.global_context);
        surface.render(|target| {
            tracing::info!("render color picekr");
            self.inner.render(ctx, target, &color_picker);
        });
    }
}
