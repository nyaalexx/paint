mod viewport;

use std::sync::Arc;

use jni::JNIEnv;
use jni::objects::JObject;
use jni_fn::jni_fn;

pub use self::viewport::ViewportRenderer;

pub trait Renderer: Send + Sync {
    /// Interrupts any blocking waits in [`Renderer::update`]
    fn interrupt(&self);

    /// Waits for input and updates state.
    ///
    /// This method should:
    ///  1. Wait until input (possibly through some channel)
    ///  2. Possibly block for frame pacing
    ///  3. Return when
    ///     - a new frame should be rendered ([`Renderer::render`])
    ///     - [`Renderer::stop`] was called
    fn update(&self);

    /// Renders the next frame into the provided texture.
    fn render(&self, texture: &wgpu::Texture);
}

#[unsafe(no_mangle)]
#[jni_fn("site.nyaalex.paint.rust.Renderer$Native")]
pub fn destroy(_env: JNIEnv, _this: JObject, ptr: usize) {
    let _renderer = unsafe { Box::from_raw(ptr as *mut Arc<dyn Renderer>) };
}
