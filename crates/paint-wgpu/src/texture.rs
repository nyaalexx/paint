#[derive(Debug, Clone)]
pub struct Texture(pub wgpu::TextureView);

impl paint_core::behaviour::Texture for Texture {}
