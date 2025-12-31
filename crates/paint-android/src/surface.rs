use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use ndk::native_window::NativeWindow;
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};

use crate::gpu::GpuContext;
use crate::renderer::Renderer;

pub mod ffi {
    use jni::JNIEnv;
    use jni::objects::JObject;
    use jni_fn::jni_fn;

    use super::*;
    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.Surface$Native")]
    pub fn create(env: JNIEnv, _this: JObject, gpu_ptr: usize, surface: JObject) -> usize {
        let gpu = unsafe { &*(gpu_ptr as *const GpuContext) };

        let Some(native_window) =
            (unsafe { NativeWindow::from_surface(env.get_raw(), surface.as_raw()) })
        else {
            return 0;
        };

        let window = Window::from(native_window);
        let surface = Surface::new(gpu, window);
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
    pub fn attachRenderer(_env: JNIEnv, _this: JObject, ptr: usize, renderer_ptr: usize) {
        assert!(ptr != 0 && renderer_ptr != 0);
        let surface = unsafe { &*(ptr as *const Arc<Surface>) };
        let renderer = unsafe { &*(renderer_ptr as *const Arc<dyn Renderer>) };
        Arc::clone(surface).attach_renderer(Arc::clone(renderer));
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
    renderer: Option<Arc<dyn Renderer>>,
    render_thread: Option<JoinHandle<()>>,
    width: u32,
    height: u32,
    reconfigure_needed: bool,
}

impl Surface {
    /// Wraps an Android [`Window`] into a GPU-rendered surface.
    pub fn new(gpu: &GpuContext, window: Window) -> Self {
        let width = window.width();
        let height = window.height();

        let surface = gpu.create_surface(window);

        let caps = surface.get_capabilities(&gpu.adapter);
        tracing::trace!("Surface capabilities: {caps:#?}");

        let format = choose_best_format(&caps.formats);

        Self {
            device: gpu.device.clone(),
            surface,
            format,
            state: Mutex::new(State {
                renderer: None,
                render_thread: None,
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
                present_mode: wgpu::PresentMode::AutoNoVsync,
                desired_maximum_frame_latency: 1, // prioritizing latency
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![self.format.add_srgb_suffix()],
            },
        );
    }

    /// Attaches a renderer and starts the render thread.
    pub fn attach_renderer(self: Arc<Self>, renderer: Arc<dyn Renderer>) {
        self.detach_renderer();

        let weak_surface = Arc::downgrade(&self);
        let weak_renderer = Arc::downgrade(&renderer);

        let render_thread = std::thread::spawn(move || {
            loop {
                let Some(renderer) = weak_renderer.upgrade() else {
                    break;
                };

                let Some(surface) = weak_surface.upgrade() else {
                    break;
                };

                // Reconfigure the surface if needed.
                // It has to happen on the same thread as `get_current_texture` ideally
                surface.reconfigure();

                let surface_texture = match surface.surface.get_current_texture() {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::trace!("Frame acquisition error: {e}");
                        continue;
                    }
                };

                renderer.render(&surface_texture.texture);

                surface_texture.present();

                // This will block until the renderer decides to render the next frame (e.g.
                // after user input), or when `renderer.stop()` is called
                renderer.update();
            }
        });

        let mut state = self.state.lock().unwrap();

        state.renderer = Some(renderer);
        state.render_thread = Some(render_thread);
    }

    /// Stops the render thread and detaches the renderer.
    fn detach_renderer(&self) {
        let mut state = self.state.lock().unwrap();

        if let Some(renderer) = state.renderer.take() {
            renderer.interrupt();
        }

        if let Some(render_thread) = state.render_thread.take() {
            // unlock mutex before attempting to join
            drop(state);

            // join only if we're not the render thread itself
            if std::thread::current().id() != render_thread.thread().id() {
                render_thread.join().unwrap();
            }
        }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        tracing::trace!("Destroying surface");
        self.detach_renderer();
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
