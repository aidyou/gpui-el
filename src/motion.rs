//! Shared animation primitives: a process-wide clock and CSS easing curves.
//!
//! [`Loading`](crate::Loading) and [`Transition`](crate::Transition) share
//! these so their motion stays consistent and respects the reduced-motion
//! preference.

use std::sync::OnceLock;
use std::time::Instant;

use gpui::{App, Window};

use crate::Theme;

/// Seconds since the first animation in this process, or 0 while reduced
/// motion is enabled. Schedules the next animation frame, so callers repaint
/// for as long as they keep calling this.
pub(crate) fn animation_time(window: &mut Window, cx: &App) -> f64 {
    if Theme::motion_reduced(cx) {
        return 0.;
    }
    window.request_animation_frame();
    elapsed()
}

/// Seconds since the first animation in this process, without scheduling a
/// frame. Use [`animation_time`] from paint code that needs repaints.
pub(crate) fn elapsed() -> f64 {
    static EPOCH: OnceLock<Instant> = OnceLock::new();
    EPOCH.get_or_init(Instant::now).elapsed().as_secs_f64()
}

/// Samples the y value of a CSS `cubic-bezier(x1, y1, x2, y2)` curve at the
/// given x position. The curve's x grows monotonically, so the matching
/// parameter is found by bisection.
pub(crate) fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    if x <= 0. || x >= 1. {
        return x;
    }
    let (mut lo, mut hi) = (0., 1.);
    for _ in 0..24 {
        let t = (lo + hi) / 2.;
        let inv = 1. - t;
        let curve_x = 3. * inv * inv * t * x1 + 3. * inv * t * t * x2 + t * t * t;
        if curve_x < x {
            lo = t;
        } else {
            hi = t;
        }
    }
    let t = (lo + hi) / 2.;
    let inv = 1. - t;
    3. * inv * inv * t * y1 + 3. * inv * t * t * y2 + t * t * t
}

/// CSS `ease-in-out`, as used by `el-collapse-transition`.
pub(crate) fn ease_in_out(x: f32) -> f32 {
    cubic_bezier(0.42, 0., 0.58, 1., x)
}

/// CSS `ease-in`, as used by the ChatSpeed loading motion.
pub(crate) fn ease_in(x: f32) -> f32 {
    cubic_bezier(0.42, 0., 1., 1., x)
}

/// Theme Chalk's `cubic-bezier(0.55, 0, 0.1, 1)` from `el-fade-in` and
/// `el-zoom-in-center`: quick to rise, slow to settle.
pub(crate) fn element_plus(x: f32) -> f32 {
    cubic_bezier(0.55, 0., 0.1, 1., x)
}

/// Linear progress, for uniform rotation and `el-fade-in-linear`.
pub(crate) fn linear(x: f32) -> f32 {
    x
}

/// Position within each revolution of a continuous animation, 0 up to 1.
pub(crate) fn turn_phase(elapsed: f64, period: f64) -> f32 {
    (elapsed % period / period) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_and_symmetric_curves_pass_through_their_ends() {
        for x in [0., 0.25, 0.5, 0.75, 1.] {
            assert!((cubic_bezier(0., 0., 1., 1., x) - x).abs() < 1e-5);
        }
        assert_eq!(ease_in_out(0.), 0.);
        assert_eq!(ease_in_out(1.), 1.);
        assert!((ease_in_out(0.5) - 0.5).abs() < 1e-5);
        assert_eq!(ease_in(0.), 0.);
        assert_eq!(ease_in(1.), 1.);
        assert_eq!(element_plus(0.), 0.);
        assert_eq!(element_plus(1.), 1.);
    }

    #[test]
    fn curves_are_monotone_and_shape_as_expected() {
        let mut previous = 0.;
        for i in 1..=20 {
            let x = i as f32 / 20.;
            let value = ease_in(x);
            assert!(value >= previous, "ease-in must not overshoot");
            assert!(value <= x, "ease-in must stay below the diagonal");
            previous = value;
        }
        // Theme Chalk's curve lingers low, sweeps through the middle, and
        // settles early.
        assert!(element_plus(0.2) < 0.2);
        assert!(element_plus(0.5) > 0.5);
        assert!(element_plus(0.8) > 0.8);
        assert!(element_plus(0.5) > ease_in_out(0.5));
    }

    #[test]
    fn loading_curves_keep_their_sampled_values() {
        assert!((ease_in(0.5) - 0.31536).abs() < 0.0001);
        assert!((ease_in_out(0.5) - 0.5).abs() < 1e-5);
    }
}
