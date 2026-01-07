mod bind_group_layouts;
mod pipeline_layouts;
mod render_pipelines;
mod shaders;
mod utils;

mod brush_engine;
mod compositor;
mod context;
mod renderer;
mod texture;

pub use self::brush_engine::{BrushEngine, BrushStroke};
pub use self::compositor::Compositor;
pub use self::context::{GlobalContext, FrameContext};
pub use self::renderer::color_picker::ColorPickerRenderer;
pub use self::renderer::viewport::ViewportRenderer;
pub use self::texture::Texture;

pub fn get_required_wgpu_features() -> wgpu::Features {
    wgpu::Features::IMMEDIATES
}

pub fn get_required_wgpu_limits() -> wgpu::Limits {
    wgpu::Limits {
        max_immediate_size: 128,
        ..wgpu::Limits::defaults()
    }
}
