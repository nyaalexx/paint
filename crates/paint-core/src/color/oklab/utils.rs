// The following code adopted from https://bottosson.github.io/misc/ok_color.h
// Copyright(c) 2021 Björn Ottosson. MIT License.

use std::f64::consts::PI;

use super::{Okhsl, Okhsv, Oklab};

#[derive(Clone, Copy)]
pub struct Rgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

pub fn linear_srgb_to_oklab(rgb: Rgb) -> Oklab {
    let Rgb { r, g, b } = rgb;

    let l = r.mul_add(0.4122214708, g.mul_add(0.5363325363, 0.0514459929 * b));
    let m = r.mul_add(0.2119034982, g.mul_add(0.6806995451, 0.1073969566 * b));
    let s = r.mul_add(0.0883024619, g.mul_add(0.2817188376, 0.6299787005 * b));

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    Oklab::new(
        l_.mul_add(0.2104542553, m_.mul_add(0.7936177850, -0.0040720468 * s_)) as f32,
        l_.mul_add(1.9779984951, m_.mul_add(-2.4285922050, 0.4505937099 * s_)) as f32,
        l_.mul_add(0.0259040371, m_.mul_add(0.7827717662, -0.8086757660 * s_)) as f32,
    )
}

pub fn oklab_to_linear_srgb(lab: Oklab) -> Rgb {
    let Oklab { l, a, b } = lab;

    let l = f64::from(l);
    let a = f64::from(a);
    let b = f64::from(b);

    let l_ = a.mul_add(0.3963377774, b.mul_add(0.2158037573, l));
    let m_ = a.mul_add(-0.1055613458, b.mul_add(-0.0638541728, l));
    let s_ = a.mul_add(-0.0894841775, b.mul_add(-1.2914855480, l));

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    Rgb {
        r: l.mul_add(4.0767416621, m.mul_add(-3.3077115913, 0.2309699292 * s)),
        g: l.mul_add(-1.2684380046, m.mul_add(2.6097574011, -0.3413193965 * s)),
        b: l.mul_add(-0.0041960863, m.mul_add(-0.7034186147, 1.7076147010 * s)),
    }
}

/// Finds intersection of the line defined by
/// L = L0 * (1 - t) + t * L1;
/// C = t * C1;
/// a and b must be normalized so a^2 + b^2 == 1
fn find_gamut_intersection(a: f64, b: f64, l1: f64, c1: f64, l0: f64, cusp: Cusp) -> f64 {
    // Find the intersection for upper and lower half seprately
    let mut t;

    if ((l1 - l0) * cusp.c - (cusp.l - l0) * c1) <= 0.0 {
        // Lower half
        t = cusp.c * l0 / (c1 * cusp.l + cusp.c * (l0 - l1));
    } else {
        // Upper half

        // First intersect with triangle
        t = cusp.c * (l0 - 1.0) / (c1 * (cusp.l - 1.0) + cusp.c * (l0 - l1));

        // Then one step Halley's method
        {
            let dl = l1 - l0;
            let dc = c1;

            let k_l = 0.396337777 * a + 0.215803757 * b;
            let k_m = -0.105561345 * a - 0.063854172 * b;
            let k_s = -0.089484177 * a - 1.291485548 * b;

            let l_dt = dl + dc * k_l;
            let m_dt = dl + dc * k_m;
            let s_dt = dl + dc * k_s;

            // If higher accuracy is required, 2 or 3 iterations of the following block can
            // be used:
            {
                let l = l0 * (1.0 - t) + t * l1;
                let c = t * c1;

                let l_ = l + c * k_l;
                let m_ = l + c * k_m;
                let s_ = l + c * k_s;

                let l = l_ * l_ * l_;
                let m = m_ * m_ * m_;
                let s = s_ * s_ * s_;

                let ldt = 3.0 * l_dt * l_ * l_;
                let mdt = 3.0 * m_dt * m_ * m_;
                let sdt = 3.0 * s_dt * s_ * s_;

                let ldt2 = 6.0 * l_dt * l_dt * l_;
                let mdt2 = 6.0 * m_dt * m_dt * m_;
                let sdt2 = 6.0 * s_dt * s_dt * s_;

                let r = 4.076741662 * l - 3.307711591 * m + 0.230969929 * s - 1.0;
                let r1 = 4.076741662 * ldt - 3.307711591 * mdt + 0.230969929 * sdt;
                let r2 = 4.076741662 * ldt2 - 3.307711591 * mdt2 + 0.230969929 * sdt2;

                let u_r = r1 / (r1 * r1 - 0. * r * r2);
                let mut t_r = -r * u_r;

                let g = -1.268438004 * l + 2.609757401 * m - 0.341319396 * s - 1.0;
                let g1 = -1.268438004 * ldt + 2.609757401 * mdt - 0.341319396 * sdt;
                let g2 = -1.268438004 * ldt2 + 2.609757401 * mdt2 - 0.341319396 * sdt2;

                let u_g = g1 / (g1 * g1 - 0. * g * g2);
                let mut t_g = -g * u_g;

                let b = -0.004196086 * l - 0.703418614 * m + 1.707614701 * s - 1.0;
                let b1 = -0.004196086 * ldt - 0.703418614 * mdt + 1.707614701 * sdt;
                let b2 = -0.004196086 * ldt2 - 0.703418614 * mdt2 + 1.707614701 * sdt2;

                let u_b = b1 / (b1 * b1 - 0. * b * b2);
                let mut t_b = -b * u_b;

                let max = 1e6;
                t_r = if u_r >= 0.0 { t_r } else { max };
                t_g = if u_g >= 0.0 { t_g } else { max };
                t_b = if u_b >= 0.0 { t_b } else { max };

                t += f64::min(t_r, f64::min(t_g, t_b));
            }
        }
    }

    t
}

// Finds the maximum saturation possible for a given hue that fits in sRGB
// Saturation here is defined as S = C/L
// a and b must be normalized so a^2 + b^2 == 1
fn compute_max_saturation(a: f64, b: f64) -> f64 {
    // Max saturation will be when one of r, g or b goes below zero.

    // Select different coefficients depending on which component goes below zero
    // first
    let [k0, k1, k2, k3, k4, wl, wm, ws] = if (-1.8817033_f64).mul_add(a, -(0.8093649 * b)) > 1.0 {
        // Red component
        [
            1.1908628, 1.7657673, 0.5966264, 0.755152, 0.5677124, 4.0767417, -3.3077116, 0.23096994,
        ]
    } else if 1.8144411_f64.mul_add(a, -(1.1944528 * b)) > 1.0 {
        // Green component
        [
            0.73956515,
            -0.45954404,
            0.08285427,
            0.1254107,
            0.14503204,
            -1.268438,
            2.6097574,
            -0.34131938,
        ]
    } else {
        // Blue component
        [
            1.3573365,
            -0.00915799,
            -1.1513021,
            -0.50559606,
            0.00692167,
            -0.0041960863,
            -0.7034186,
            1.7076147,
        ]
    };

    // Approximate max saturation using a polynomial:
    let mut saturation = (k4 * a).mul_add(b, (k3 * a).mul_add(a, k2.mul_add(b, k1.mul_add(a, k0))));
    debug_assert!(saturation.is_finite());

    // Do one step Halley's method to get closer
    // this gives an error less than 10e6, except for some blue hues where the dS/dh
    // is close to infinite this should be sufficient for most applications,
    // otherwise do two/three steps

    let k_l = 0.39633778_f64.mul_add(a, 0.21580376 * b);
    let k_m = (-0.105561346_f64).mul_add(a, -(0.06385417 * b));
    let k_s = (-0.08948418_f64).mul_add(a, -(1.2914855 * b));

    {
        let l_ = saturation.mul_add(k_l, 1.);
        let m_ = saturation.mul_add(k_m, 1.);
        let s_ = saturation.mul_add(k_s, 1.);

        let l = l_ * l_ * l_;
        let m = m_ * m_ * m_;
        let s = s_ * s_ * s_;

        let l_d_s = 3. * (k_l * l_) * l_;
        let m_d_s = 3. * (k_m * m_) * m_;
        let s_d_s = 3. * (k_s * s_) * s_;

        let l_d_s2 = 6. * k_l * (k_l * l_);
        let m_d_s2 = 6. * k_m * (k_m * m_);
        let s_d_s2 = 6. * k_s * (k_s * s_);

        let f = ws.mul_add(s, wl.mul_add(l, wm * m));
        let f1 = ws.mul_add(s_d_s, wl.mul_add(l_d_s, wm * m_d_s));
        debug_assert!(f1 != 0.);
        let f2 = ws.mul_add(s_d_s2, wl.mul_add(l_d_s2, wm * m_d_s2));

        let div = f1.mul_add(f1, -(0.5 * f * f2));
        debug_assert!(div != 0.);
        saturation -= f * f1 / div;
    }

    saturation
}

fn scale_l(l_vt: f64, c_vt: f64, a_: f64, b_: f64) -> f64 {
    let rgb_scale = oklab_to_linear_srgb(Oklab::new(
        l_vt as f32,
        (a_ * c_vt) as f32,
        (b_ * c_vt) as f32,
    ));
    let rgb_max = rgb_scale.r.max(rgb_scale.g).max(rgb_scale.b.max(0.0));
    f64::cbrt(1. / rgb_max)
}

#[derive(Clone, Copy)]
struct Cusp {
    l: f64,
    c: f64,
}

/// Finds L_cusp and C_cusp for a given hue.
/// a and b must be normalized so a^2 + b^2 == 1
fn find_cusp(a: f64, b: f64) -> Cusp {
    debug_assert!(a.is_finite());
    debug_assert!(b.is_finite());

    // First, find the maximum saturation (saturation S = C/L)
    let s_cusp = compute_max_saturation(a, b);

    // Convert to linear sRGB to find the first point where at least
    // one of r,g or b >= 1:
    let l_cusp = scale_l(1.0, s_cusp, a, b);
    let c_cusp = l_cusp * s_cusp;

    Cusp {
        l: l_cusp,
        c: c_cusp,
    }
}

pub fn gamut_clip_adaptive_l0_0_5(mut rgb: Rgb, lab: Oklab) -> Rgb {
    let eps = 1e-3;
    let max = 1.0 + eps;
    let min = -eps;

    if rgb.r > max || rgb.g > max || rgb.b > max || rgb.r < min || rgb.g < min || rgb.b < min {
        let alpha = 0.05;

        let l = f64::from(lab.l);
        let eps = 0.00001;
        let (a, b) = (f64::from(lab.a), f64::from(lab.b));
        let c = f64::max(eps, f64::hypot(a, b));
        let (a_, b_) = (a / c, b / c);

        let ld = l - 0.5;
        let e1 = 0.5 + f64::abs(ld) + alpha * c;
        let l0 = 0.5 * (1.0 + f64::signum(ld) * (e1 - f64::sqrt(e1 * e1 - 2.0 * f64::abs(ld))));

        let t = find_gamut_intersection(a_, b_, l, c, l0, find_cusp(a_, b_));
        let l_clipped = l0 * (1.0 - t) + t * l;
        let c_clipped = t * c;

        rgb = oklab_to_linear_srgb(Oklab::new(
            l_clipped as f32,
            (c_clipped * a_) as f32,
            (c_clipped * b_) as f32,
        ))
    }

    rgb.r = rgb.r.clamp(0.0, 1.0);
    rgb.g = rgb.g.clamp(0.0, 1.0);
    rgb.b = rgb.b.clamp(0.0, 1.0);

    rgb
}

fn toe(x: f64) -> f64 {
    let k_1: f64 = 0.206;
    let k_2: f64 = 0.03;
    let k_3: f64 = (1. + k_1) / (1. + k_2);

    0.5 * (k_3.mul_add(x, -k_1)
        + k_3
            .mul_add(x, -k_1)
            .mul_add(k_3.mul_add(x, -k_1), 4. * k_2 * (k_3 * x))
            .sqrt())
}

fn toe_inv(x: f64) -> f64 {
    let k_1 = 0.206;
    let k_2 = 0.03;
    let k_3 = (1. + k_1) / (1. + k_2);
    x.mul_add(x, k_1 * x) / (k_3 * (x + k_2))
}

#[derive(Clone, Copy)]
struct St {
    s: f64,
    t: f64,
}

fn get_st_max(cusp: Cusp) -> St {
    let l = cusp.l;
    let c = cusp.c;
    St {
        s: c / l,
        t: c / (1.0 - l),
    }
}

// Returns a smooth approximation of the location of the cusp
// This polynomial was created by an optimization process
// It has been designed so that S_mid < S_max and T_mid < T_max
fn get_st_mid(a_: f64, b_: f64) -> St {
    debug_assert!(a_.is_finite());
    debug_assert!(b_.is_finite());

    let s = 0.11516993
        + 1. / a_.mul_add(
            a_.mul_add(
                a_.mul_add(
                    4.69891_f64.mul_add(a_, 5.387708_f64.mul_add(b_, -4.2489457)),
                    10.02301_f64.mul_add(-b_, -2.1370494),
                ),
                1.751984_f64.mul_add(b_, -2.1955736),
            ),
            4.1590123_f64.mul_add(b_, 7.4477897),
        );

    let t = 0.11239642
        + 1. / a_.mul_add(
            a_.mul_add(
                a_.mul_add(
                    0.14661872_f64.mul_add(-a_, 0.45399568_f64.mul_add(-b_, 0.00299215)),
                    0.6122399_f64.mul_add(b_, -0.27087943),
                ),
                0.9014812_f64.mul_add(b_, 0.40370612),
            ),
            0.6812438_f64.mul_add(-b_, 1.6132032),
        );

    debug_assert!(s.is_finite());
    debug_assert!(t.is_finite());

    St { s, t }
}

#[derive(Clone, Copy)]
struct Cs {
    c_0: f64,
    c_mid: f64,
    c_max: f64,
}

fn get_cs(l: f64, a_: f64, b_: f64) -> Cs {
    debug_assert!(l > 0. && l < 1. && l.is_finite());
    debug_assert!(a_.is_finite());
    debug_assert!(b_.is_finite());

    let cusp = find_cusp(a_, b_);

    let c_max = find_gamut_intersection(a_, b_, l, 1.0, l, cusp);
    let St { s: s_max, t: t_max } = get_st_max(cusp);

    let k = c_max / (l * s_max).min((1. - l) * t_max);
    debug_assert!(k.is_finite());

    let c_mid = {
        let St { s: s_mid, t: t_mid } = get_st_mid(a_, b_);

        let c_a = l * s_mid;
        let c_b = (1. - l) * t_mid;
        let ca4 = (c_a * c_a) * (c_a * c_a);
        let cb4 = (c_b * c_b) * (c_b * c_b);

        0.9 * k * ((1. / (1. / ca4 + 1. / cb4)).sqrt()).sqrt()
    };

    let c_0 = {
        let c_a = l * 0.4;
        let c_b = (1. - l) * 0.8;

        (1. / (1. / (c_a * c_a) + 1. / (c_b * c_b))).sqrt()
    };

    debug_assert!(c_0.is_finite());
    debug_assert!(c_mid.is_finite());
    debug_assert!(c_max.is_finite());

    Cs { c_0, c_mid, c_max }
}

pub fn okhsl_to_oklab(hsl: Okhsl) -> Oklab {
    let h = f64::from(hsl.h);
    let s = f64::from(hsl.s);
    let l = f64::from(hsl.l);

    if l >= 1.0 {
        return Oklab::new(1.0, 0.0, 0.0);
    }

    if l <= 0.0 {
        return Oklab::new(0.0, 0.0, 0.0);
    }

    let (b_, a_) = h.sin_cos();
    let l = toe_inv(l);

    let Cs { c_0, c_mid, c_max } = get_cs(l, a_, b_);

    let mid = 0.8;
    let mid_inv = 1.25;

    let (c, t, k_0, k_1, k_2);

    if s < mid {
        t = mid_inv * s;

        k_1 = mid * c_0;
        k_2 = 1.0 - k_1 / c_mid;

        c = t * k_1 / (1.0 - k_2 * t);
    } else {
        t = (s - mid) / (1.0 - mid);

        k_0 = c_mid;
        k_1 = (1.0 - mid) * c_mid * c_mid * mid_inv * mid_inv / c_0;
        k_2 = 1.0 - (k_1) / (c_max - c_mid);

        c = k_0 + t * k_1 / (1.0 - k_2 * t);
    }

    Oklab::new(l as f32, (c * a_) as f32, (c * b_) as f32)
}

pub fn oklab_to_okhsl(lab: Oklab) -> Okhsl {
    let l = f64::from(lab.l);
    let a = f64::from(lab.a);
    let b = f64::from(lab.b);

    if l <= 0.0 {
        return Okhsl::new(0.0, 0.0, 0.0);
    }

    if l >= 1.0 {
        return Okhsl::new(0.0, 0.0, 1.0);
    }

    let c = f64::hypot(a, b);
    let a_ = a / c;
    let b_ = b / c;

    let h = PI + f64::atan2(-b, -a);

    let Cs { c_0, c_mid, c_max } = get_cs(l, a_, b_);

    let mid = 0.8;
    let mid_inv = 1.25;

    let s = if c < c_mid {
        let k_1 = mid * c_0;
        let k_2 = 1.0 - k_1 / c_mid;

        let t = c / (k_1 + k_2 * c);
        t * mid
    } else {
        let k_0 = c_mid;
        let k_1 = (1.0 - mid) * c_mid * c_mid * mid_inv * mid_inv / c_0;
        let k_2 = 1.0 - (k_1) / (c_max - c_mid);

        let t = (c - k_0) / (k_1 + k_2 * (c - k_0));
        mid + (1.0 - mid) * t
    };

    let l = toe(l);
    Okhsl::new(h as f32, s as f32, l as f32)
}

pub fn okhsv_to_oklab(hsv: Okhsv) -> Oklab {
    let h = f64::from(hsv.h);
    let s = f64::from(hsv.s);
    let v = f64::from(hsv.v);

    let (b_, a_) = h.sin_cos();

    let cusp = find_cusp(a_, b_);
    let St { s: s_max, t: t_max } = get_st_max(cusp);
    let s_0 = 0.5;
    let k = 1.0 - s_0 / s_max;

    let l_v = 1.0 - s * s_0 / (s_0 + t_max - t_max * k * s);
    let c_v = s * t_max * s_0 / (s_0 + t_max - t_max * k * s);

    let mut l = v * l_v;
    if l <= 0. || l >= 1.0 {
        return Oklab::new(l.clamp(0.0, 1.0) as f32, 0.0, 0.0);
    }

    let mut c = v * c_v;

    let l_vt = toe_inv(l_v);
    let c_vt = c_v * l_vt / l_v;

    let l_new = toe_inv(l);
    c = c * l_new / l;
    l = l_new;

    let scale_l = scale_l(l_vt, c_vt, a_, b_);
    l *= scale_l;
    c *= scale_l;

    Oklab::new(l as f32, (c * a_) as f32, (c * b_) as f32)
}

pub fn oklab_to_okhsv(lab: Oklab) -> Okhsv {
    let l = f64::from(lab.l);
    let a = f64::from(lab.a);
    let b = f64::from(lab.b);

    if l <= 0.0 {
        return Okhsv::new(0.0, 0.0, 0.0);
    }

    if l >= 1.0 {
        return Okhsv::new(0.0, 0.0, 1.0);
    }

    if !(l > 0.0 && l < 1.0 && (a != 0.0 || b != 0.0)) {
        return Okhsv::new(0.0, 0.0, toe(l) as f32);
    }

    let c = f64::hypot(a, b);
    let a_ = a / c;
    let b_ = b / c;

    let mut l = l;
    let h = PI + f64::atan2(-b, -a);

    let cusp = find_cusp(a_, b_);
    let St { s: s_max, t: t_max } = get_st_max(cusp);
    let s_0 = 0.5;
    let k = 1.00 - s_0 / s_max;

    let t = t_max / (c + l * t_max);
    let l_v = t * l;
    let c_v = t * c;

    let l_vt = toe_inv(l_v);
    let c_vt = c_v * l_vt / l_v;

    let scale_l = scale_l(l_vt, c_vt, a_, b_);
    l /= scale_l;
    l = toe(l);

    let v = l / l_v;
    let s = (s_0 + t_max) * c_v / ((t_max * s_0) + t_max * k * c_v);

    Okhsv::new(h as f32, s as f32, v as f32)
}
