use super::{Color, Component};

/// Linear sRGB color.
///
/// This color, with `T = f32`, is the canonical color representation.
/// Most operations only support [`LinearSrgb<f32>`].
///
/// When using floating point components, this type can represent all colors,
/// including HDR (by setting components above 1.0) and WCG (by setting
/// components below 0.0). Essentially, this is scRGB.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LinearSrgb<T = f32> {
    /// Red channel.
    pub r: T,
    /// Green channel.
    pub g: T,
    /// Blue channel.
    pub b: T,
}

impl<T> LinearSrgb<T> {
    /// Constructs a color with the provided RGB components.
    pub const fn new(r: T, g: T, b: T) -> Self {
        Self { r, g, b }
    }
}

impl<T: Component> Color for LinearSrgb<T> {
    fn from_linear_srgb(c: LinearSrgb) -> Self {
        Self::new(T::from_f32(c.r), T::from_f32(c.g), T::from_f32(c.b))
    }

    fn to_linear_srgb(&self) -> LinearSrgb {
        LinearSrgb::new(self.r.as_f32(), self.g.as_f32(), self.b.as_f32())
    }
}

/// Nonlinear (gamma-corrected) sRGB color.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NonlinearSrgb<T = f32> {
    /// Red channel.
    pub r: T,
    /// Green channel.
    pub g: T,
    /// Blue channel.
    pub b: T,
}

impl<T> NonlinearSrgb<T> {
    /// Constructs a color with the provided RGB components.
    pub const fn new(r: T, g: T, b: T) -> Self {
        Self { r, g, b }
    }
}

impl<T: Component> Color for NonlinearSrgb<T> {
    fn from_linear_srgb(c: LinearSrgb) -> Self {
        let f = |x: f32| {
            let y = if x <= 0.0031308 {
                x * 12.92
            } else {
                1.055 * x.powf(1.0 / 2.4) - 0.055
            };
            T::from_f32(y)
        };

        Self::new(f(c.r), f(c.g), f(c.b))
    }

    fn to_linear_srgb(&self) -> LinearSrgb {
        let f = |x: T| {
            let x = x.as_f32();
            if x <= 0.04045 {
                x / 12.92
            } else {
                ((x + 0.055) / 1.055).powf(2.4)
            }
        };

        LinearSrgb::new(f(self.r), f(self.g), f(self.b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_nonlinear_round_trip() {
        for x in 0..=255u8 {
            let c1 = NonlinearSrgb::new(x, x, x);
            let c2 = c1.to_linear_srgb_clamped();
            let c3 = NonlinearSrgb::from_linear_srgb(c2);
            assert_eq!(c1, c3);
        }
    }
}
