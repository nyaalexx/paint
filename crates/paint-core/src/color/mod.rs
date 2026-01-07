mod oklab;
mod srgb;

use half::f16;

pub use self::oklab::{Okhsl, Okhsv, Oklab};
pub use self::srgb::{LinearSrgb, NonlinearSrgb};

/// A color component.
pub trait Component: Copy + PartialEq + std::fmt::Debug {
    /// The logical minimum value of the component, usually 0.
    const MIN: Self;

    /// The logical maximum value of the component, 1.0 for floating point
    /// formats and `uXX::MAX` for integer formats.
    const MAX: Self;

    /// Converts `f32` to the component.
    ///
    /// This is usually a lossy conversion, unless it's already `f32`.
    ///
    /// NaN and Inf values should be represented as 0 in integer formats, and
    /// as-is in floating point formats.
    fn from_f32(v: f32) -> Self;

    /// Converts `f32` to the component.
    ///
    /// This is usually a lossless conversion.
    fn as_f32(self) -> f32;
}

impl Component for u8 {
    const MIN: Self = 0;
    const MAX: Self = u8::MAX;

    fn from_f32(v: f32) -> Self {
        if !v.is_finite() {
            return 0;
        }

        let max = Self::MAX as f32;
        (v * max).round().clamp(0.0, max) as u8
    }

    fn as_f32(self) -> f32 {
        (self as f32) / (Self::MAX as f32)
    }
}

impl Component for u16 {
    const MIN: Self = 0;
    const MAX: Self = u16::MAX;

    fn from_f32(v: f32) -> Self {
        if !v.is_finite() {
            return 0;
        }

        let max = Self::MAX as f32;
        (v * max).round().clamp(0.0, max) as u16
    }

    fn as_f32(self) -> f32 {
        (self as f32) / (Self::MAX as f32)
    }
}

impl Component for f16 {
    const MIN: Self = f16::ZERO;
    const MAX: Self = f16::ONE;

    fn from_f32(v: f32) -> Self {
        f16::from_f32(v)
    }

    fn as_f32(self) -> f32 {
        f16::to_f32(self)
    }
}

impl Component for f32 {
    const MIN: Self = 0.0;
    const MAX: Self = 1.0;

    fn from_f32(v: f32) -> Self {
        v
    }

    fn as_f32(self) -> f32 {
        self
    }
}

/// A color representation.
///
/// The canonical color representation is [`LinearSrgb`] (with [`f32`]
/// components by default), so only conversions to/from the canonical are
/// specified. Most color operations also happen on [`LinearSrgb`].
pub trait Color {
    /// Constructs a color from [`LinearSrgb`].
    ///
    /// If the format supports it, it should preserve the color as is, including
    /// out of gamut colors.
    fn from_linear_srgb(c: LinearSrgb) -> Self;

    /// Converts the color to [`LinearSrgb`], preserving out of gamut colors.
    ///
    /// The resulting color may have components below 0 or above 1.
    fn to_linear_srgb(&self) -> LinearSrgb;

    /// Converts the color to [`LinearSrgb`], clamping out of gamut colors.
    ///
    /// The resulting color should have all RGB components between 0 and 1.
    ///
    /// The default implementation uses [`to_linear_srgb()`] and then performs
    /// clamping in RGB space.
    fn to_linear_srgb_clamped(&self) -> LinearSrgb {
        let mut rgb = self.to_linear_srgb();
        rgb.r = rgb.r.clamp(0.0, 1.0);
        rgb.g = rgb.g.clamp(0.0, 1.0);
        rgb.b = rgb.b.clamp(0.0, 1.0);
        rgb
    }
}

/// Color with an alpha channel. Uses straight alpha (non-premultiplied).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WithAlpha<C, A = f32> {
    /// The color.
    pub color: C,
    /// Alpha channel.
    pub alpha: A,
}

impl<C: Color, A: Component> WithAlpha<C, A> {
    /// Combines the color and the alpha.
    pub fn new(color: C, alpha: A) -> Self {
        Self { color, alpha }
    }

    /// Creates a fully opaque color.
    pub fn opaque(color: C) -> Self {
        Self::new(color, A::MAX)
    }

    /// Creates a fully transparent color.
    pub fn transparent(color: C) -> Self {
        Self::new(color, A::MIN)
    }
}

impl<C: Color, A: Component> Color for WithAlpha<C, A> {
    /// Constructs a color with full opacity.
    fn from_linear_srgb(c: LinearSrgb) -> Self {
        Self::opaque(C::from_linear_srgb(c))
    }

    fn to_linear_srgb(&self) -> LinearSrgb {
        self.color.to_linear_srgb()
    }

    fn to_linear_srgb_clamped(&self) -> LinearSrgb {
        self.color.to_linear_srgb_clamped()
    }
}
