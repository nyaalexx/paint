use std::sync::{Arc, Mutex};

use ndk::native_window::NativeWindow;
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};

use crate::runtime::Runtime;

pub mod ffi {
    use jni::JNIEnv;
    use jni::objects::JObject;
    use jni_fn::jni_fn;

    use super::*;

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.Surface$Native")]
    pub fn create(env: JNIEnv, _this: JObject, runtime_ptr: usize, surface: JObject) -> usize {
        let runtime = unsafe { &*(runtime_ptr as *const Runtime) };

        let Some(native_window) =
            (unsafe { NativeWindow::from_surface(env.get_raw(), surface.as_raw()) })
        else {
            return 0;
        };

        let window = Window::from(native_window);
        let surface = Surface::new(runtime, window);
        Box::into_raw(Box::new(Arc::new(surface))) as usize
    }

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.Surface$Native")]
    pub fn resize(_env: JNIEnv, _this: JObject, ptr: usize, width: u32, height: u32) {
        assert!(ptr != 0);
        let surface = unsafe { &*(ptr as *const Arc<Surface>) };
        surface.resize(width, height);
    }

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.Surface$Native")]
    pub fn destroy(_env: JNIEnv, _this: JObject, ptr: usize) {
        assert!(ptr != 0);
        let _surface = unsafe { Box::from_raw(ptr as *mut Arc<Surface>) };
    }
}

/// An Android surface
pub struct Surface {
    device: wgpu::Device,
    surface: wgpu::Surface<'static>,
    format: wgpu::TextureFormat,
    state: Mutex<State>,
}

/// Internal mutable state
struct State {
    width: u32,
    height: u32,
    reconfigure_needed: bool,
}

impl Surface {
    /// Wraps an Android [`Window`] into a GPU-rendered surface.
    pub fn new(runtime: &Runtime, window: Window) -> Self {
        let width = window.width();
        let height = window.height();

        let surface_target = wgpu::SurfaceTarget::from(window);
        let surface = runtime.instance.create_surface(surface_target).unwrap();

        let caps = surface.get_capabilities(&runtime.adapter);
        tracing::trace!("Surface capabilities: {caps:#?}");

        let format = choose_best_format(&caps.formats);

        Self {
            device: runtime.device.clone(),
            surface,
            format,
            state: Mutex::new(State {
                width,
                height,
                reconfigure_needed: true,
            }),
        }
    }

    /// Update the size of the surface.
    ///
    /// The actual surface reconfiguration is deferred onto the render thread.
    pub fn resize(&self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            tracing::warn!("Tried to resize to 0x0 surface, ignoring");
            return;
        }

        let mut state = self.state.lock().unwrap();

        if state.width == width && state.height == height {
            return;
        }

        state.width = width;
        state.height = height;
        state.reconfigure_needed = true;
    }

    /// Reconfigures the underlying surface if necessary.
    fn reconfigure(&self) {
        let (width, height) = {
            let mut state = self.state.lock().unwrap();
            if !state.reconfigure_needed {
                return;
            }

            state.reconfigure_needed = false;

            (state.width, state.height)
        };

        tracing::trace!(
            "Configuring surface to {width}x{height}, {:?} format",
            self.format
        );

        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.format,
                width,
                height,
                present_mode: wgpu::PresentMode::AutoVsync,
                desired_maximum_frame_latency: 1, // prioritizing latency
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![self.format.add_srgb_suffix()],
            },
        );
    }

    pub fn render(&self, callback: impl FnOnce(&wgpu::Texture)) {
        self.reconfigure();

        let surface_texture = match self.surface.get_current_texture() {
            Ok(v) => v,
            Err(e) => {
                tracing::trace!("Frame acquisition error: {e}");
                return; // skip this frame
            }
        };

        callback(&surface_texture.texture);
        surface_texture.present();
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        tracing::trace!("Destroying surface");
    }
}

/// Chooses the best format for the surface
fn choose_best_format(formats: &[wgpu::TextureFormat]) -> wgpu::TextureFormat {
    // from high to low priority
    let preference = [
        wgpu::TextureFormat::Rgba8Unorm, // TODO: other formats maybe
    ];

    formats
        .iter()
        .copied()
        .min_by_key(|format| {
            preference
                .iter()
                .position(|v| v == format)
                .unwrap_or(usize::MAX)
        })
        .expect("there should be at least one supported format")
}

/// Wraps an Android [`NativeWindow`], adding support for raw-window-handle
/// descriptors.
pub struct Window {
    native_window: NativeWindow,
}

impl Window {
    pub fn width(&self) -> u32 {
        self.native_window.width() as u32
    }

    pub fn height(&self) -> u32 {
        self.native_window.height() as u32
    }
}

impl From<NativeWindow> for Window {
    fn from(native_window: NativeWindow) -> Self {
        Self { native_window }
    }
}

impl HasWindowHandle for Window {
    fn window_handle(&self) -> Result<wgpu::rwh::WindowHandle<'_>, wgpu::rwh::HandleError> {
        let android_handle =
            wgpu::rwh::AndroidNdkWindowHandle::new(self.native_window.ptr().cast());
        let raw_handle = wgpu::rwh::RawWindowHandle::AndroidNdk(android_handle);
        Ok(unsafe { wgpu::rwh::WindowHandle::borrow_raw(raw_handle) })
    }
}

impl HasDisplayHandle for Window {
    fn display_handle(&self) -> Result<wgpu::rwh::DisplayHandle<'_>, wgpu::rwh::HandleError> {
        Ok(wgpu::rwh::DisplayHandle::android())
    }
}
