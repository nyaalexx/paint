use glam::{Affine2, UVec2};

#[derive(Debug, Clone)]
pub struct Viewport<T> {
    pub transform: Affine2,
    pub canvas: Canvas<T>,
}

#[derive(Debug, Clone)]
pub struct Canvas<T> {
    pub resolution: UVec2,
    pub layers: Vec<Layer<T>>,
}

#[derive(Debug, Clone)]
pub enum Layer<T> {
    Texture(T),
}

#[derive(Debug, Clone)]
pub enum ColorPicker {
    /// Okhsv color space slice with constant hue.
    OkhsvHueSlice {
        /// Hue in radians, between 0 and 2π.
        hue: f32,
    },
}
