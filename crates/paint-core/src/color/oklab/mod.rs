mod utils;

use super::{Color, LinearSrgb};

/// Color in Oklab color space.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Oklab {
    /// Lightness.
    ///
    /// Typically between 0 and 1 for sRGB colors.
    pub l: f32,
    /// Chroma A.
    ///
    /// Typically between -0.4 and 0.4 for sRGB colors.
    pub a: f32,
    /// Chroma B.
    ///
    /// Typically between -0.4 and 0.4 for sRGB colors.
    pub b: f32,
}

impl Oklab {
    /// Creates a new [`Oklab`] color with the given components.
    pub const fn new(l: f32, a: f32, b: f32) -> Self {
        Self { l, a, b }
    }
}

impl Color for Oklab {
    fn from_linear_srgb(c: LinearSrgb) -> Self {
        utils::linear_srgb_to_oklab(utils::Rgb {
            r: f64::from(c.r),
            g: f64::from(c.g),
            b: f64::from(c.b),
        })
    }

    fn to_linear_srgb(&self) -> LinearSrgb {
        let rgb = utils::oklab_to_linear_srgb(*self);
        LinearSrgb::new(rgb.r as f32, rgb.g as f32, rgb.b as f32)
    }

    fn to_linear_srgb_clamped(&self) -> LinearSrgb {
        let rgb = utils::oklab_to_linear_srgb(*self);
        let rgb = utils::gamut_clip_adaptive_l0_0_5(rgb, *self);
        LinearSrgb::new(rgb.r as f32, rgb.g as f32, rgb.b as f32)
    }
}

/// Color in Okhsl color space.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Okhsl {
    /// Hue, in radians between 0 and 2π.
    pub h: f32,
    /// Saturation, between 0 and 1.
    pub s: f32,
    /// Lightness, between 0 and 1.
    pub l: f32,
}

impl Okhsl {
    /// Creates a new [`Okhsl`] color with the given components.
    pub const fn new(h: f32, s: f32, l: f32) -> Self {
        Self { h, s, l }
    }

    /// Converts [`Oklab`] to [`Okhsl`].
    pub fn from_oklab(lab: Oklab) -> Self {
        utils::oklab_to_okhsl(lab)
    }

    /// Converts [`Okhsl`] to [`Oklab`].
    pub fn to_oklab(&self) -> Oklab {
        utils::okhsl_to_oklab(*self)
    }
}

impl Color for Okhsl {
    fn from_linear_srgb(c: LinearSrgb) -> Self {
        Self::from_oklab(Oklab::from_linear_srgb(c))
    }

    fn to_linear_srgb(&self) -> LinearSrgb {
        self.to_oklab().to_linear_srgb()
    }

    fn to_linear_srgb_clamped(&self) -> LinearSrgb {
        self.to_oklab().to_linear_srgb_clamped()
    }
}

/// Color in Okhsv color space.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Okhsv {
    /// Hue, in radians between 0 and 2π.
    pub h: f32,
    /// Saturation, between 0 and 1.
    pub s: f32,
    /// Value, between 0 and 1.
    pub v: f32,
}

impl Okhsv {
    /// Creates a new [`Okhsv`] color with the given components.
    pub const fn new(h: f32, s: f32, v: f32) -> Self {
        Self { h, s, v }
    }

    /// Converts [`Oklab`] to [`Okhsv`].
    pub fn from_oklab(lab: Oklab) -> Self {
        utils::oklab_to_okhsv(lab)
    }

    /// Converts [`Okhsv`] to [`Oklab`].
    pub fn to_oklab(&self) -> Oklab {
        utils::okhsv_to_oklab(*self)
    }
}

impl Color for Okhsv {
    fn from_linear_srgb(c: LinearSrgb) -> Self {
        Self::from_oklab(Oklab::from_linear_srgb(c))
    }

    fn to_linear_srgb(&self) -> LinearSrgb {
        self.to_oklab().to_linear_srgb()
    }

    fn to_linear_srgb_clamped(&self) -> LinearSrgb {
        self.to_oklab().to_linear_srgb_clamped()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::NonlinearSrgb;

    fn test_round_trip(f: impl Fn(LinearSrgb) -> LinearSrgb) {
        let test = |c1: NonlinearSrgb<u8>| {
            let c2 = NonlinearSrgb::<u8>::from_linear_srgb(f(c1.to_linear_srgb_clamped()));
            assert_eq!(c1, c2);
        };

        for x in 0..=255u8 {
            test(NonlinearSrgb::new(x, 0, 0)); // red
            test(NonlinearSrgb::new(0, x, 0)); // green
            test(NonlinearSrgb::new(0, 0, x)); // blue
            test(NonlinearSrgb::new(x, x, x)); // gray
        }

        // various combinations of red, green, and blue
        let step = 15;
        for r in (0..=255u8).step_by(step) {
            for g in (0..=255u8).step_by(step) {
                for b in (0..=255u8).step_by(step) {
                    test(NonlinearSrgb::new(r, g, b));
                }
            }
        }
    }

    #[test]
    fn srgb_oklab_round_trip() {
        test_round_trip(|c| Oklab::from_linear_srgb(c).to_linear_srgb_clamped())
    }

    #[test]
    fn srgb_okhsv_round_trip() {
        test_round_trip(|c| Okhsv::from_linear_srgb(c).to_linear_srgb_clamped())
    }

    #[test]
    fn srgb_okhsl_round_trip() {
        test_round_trip(|c| Okhsl::from_linear_srgb(c).to_linear_srgb_clamped())
    }
}
