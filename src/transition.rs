//! Element Plus transitions as one declarative component.
//!
//! Covers `el-fade-in`, `el-fade-in-linear`, `el-zoom-in-center`,
//! `el-collapse-transition`, and `horizontal-collapse-transition` from Theme
//! Chalk's `common/transition.scss` and `collapse-transition.vue`. The content
//! stays mounted while leaving and is removed only after the animation
//! settles, like a Vue `<transition>`. GPUI has no transform animation, so
//! the zoom effects reduce to their opacity channel.

use gpui::{
    AnyElement, App, Bounds, ElementId, IntoElement, ParentElement, Pixels, RenderOnce, Window,
    canvas, div, prelude::*, px,
};

use crate::{
    Theme,
    motion::{self, MotionSpeed},
};

/// The curve, duration, and channel of a transition.
///
/// Durations are the desktop variants of Theme Chalk's web values: brief
/// feedback instead of 300 ms fades.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TransitionEffect {
    /// `el-fade-in`: opacity over 0.2 s with `cubic-bezier(0.55, 0, 0.1, 1)`.
    #[default]
    Fade,
    /// `el-fade-in-linear`: opacity over 0.15 s linearly.
    FadeLinear,
    /// `el-zoom-in-center`: opacity over 0.2 s (upstream also scales).
    ZoomInCenter,
    /// `el-collapse-transition`: content height over 0.2 s, ease in out.
    Collapse,
    /// `horizontal-collapse-transition`: content width over 0.2 s, ease in out.
    CollapseHorizontal,
}

impl TransitionEffect {
    fn duration(self) -> f64 {
        match self {
            TransitionEffect::Fade | TransitionEffect::ZoomInCenter => 0.2,
            TransitionEffect::FadeLinear => 0.15,
            TransitionEffect::Collapse | TransitionEffect::CollapseHorizontal => 0.2,
        }
    }
    fn easing(self) -> fn(f32) -> f32 {
        match self {
            TransitionEffect::Fade | TransitionEffect::ZoomInCenter => motion::element_plus,
            TransitionEffect::FadeLinear => motion::linear,
            TransitionEffect::Collapse | TransitionEffect::CollapseHorizontal => {
                motion::ease_in_out
            }
        }
    }
    /// Whether the effect clips content by height (`true`), by width (`false`),
    /// or not at all (`None`).
    fn size(self) -> Option<bool> {
        match self {
            TransitionEffect::Collapse => Some(true),
            TransitionEffect::CollapseHorizontal => Some(false),
            _ => None,
        }
    }
}

/// Animates content in and out, following Element Plus transitions. Render the
/// transition every frame and set `visible` to the target state; the component
/// keeps the content mounted while it leaves and drops it once settled.
///
/// ```
/// use gpui_el::{TransitionEffect, el_transition};
/// let open = true;
/// let panel = el_transition("panel", open, TransitionEffect::Collapse)
///     .child("Contents");
/// ```
#[derive(IntoElement)]
#[must_use]
pub struct Transition {
    id: ElementId,
    visible: bool,
    effect: TransitionEffect,
    speed: Option<MotionSpeed>,
    child: AnyElement,
}

/// Creates a transition for the given target visibility and effect. Add the
/// animated content with [`Transition::child`].
pub fn el_transition(
    id: impl Into<ElementId>,
    visible: bool,
    effect: TransitionEffect,
) -> Transition {
    Transition {
        id: id.into(),
        visible,
        effect,
        speed: None,
        child: div().into_any_element(),
    }
}

impl Transition {
    pub fn new(id: impl Into<ElementId>, visible: bool, effect: TransitionEffect) -> Self {
        el_transition(id, visible, effect)
    }

    /// Overrides the theme's [`MotionSpeed`] for this transition.
    pub fn speed(mut self, speed: MotionSpeed) -> Self {
        self.speed = Some(speed);
        self
    }

    /// Sets the content to animate.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = child.into_any_element();
        self
    }
}

/// One animation phase, kept in window state keyed by the transition's ID.
#[derive(Clone, Copy, Default)]
struct Phase {
    initialized: bool,
    visible: bool,
    entering: bool,
    started: f64,
    /// Last measured natural size of the content, for collapse effects.
    measured: f32,
}

impl RenderOnce for Transition {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| Phase::default());
        let now = motion::elapsed();
        let reduced = Theme::motion_reduced(cx);
        let speed = self
            .speed
            .unwrap_or_else(|| Theme::current(cx).motion_speed);
        let duration = speed.scale(self.effect.duration());
        let easing = self.effect.easing();
        let mut phase = *state.read(cx);
        if !phase.initialized || phase.visible != self.visible {
            // A fresh phase starts now; a reversal continues from the amount
            // the previous phase was still showing.
            let in_flight = phase.initialized && !reduced && (now - phase.started) / duration < 1.;
            let displayed = if in_flight {
                let t = ((now - phase.started) / duration).clamp(0., 1.) as f32;
                if phase.entering {
                    easing(t)
                } else {
                    1. - easing(t)
                }
            } else {
                1.
            };
            phase.initialized = true;
            phase.visible = self.visible;
            phase.entering = self.visible;
            phase.started = if in_flight {
                now - displayed as f64 * duration
            } else {
                now
            };
            state.update(cx, |stored, _| *stored = phase);
        }
        let t = ((now - phase.started) / duration).clamp(0., 1.) as f32;
        let settled = t >= 1. || reduced;
        if !settled {
            window.request_animation_frame();
        }
        if !phase.entering && settled {
            // The leave animation finished: render nothing.
            return div().into_any_element();
        }
        let progress = if settled {
            1.
        } else if phase.entering {
            easing(t)
        } else {
            1. - easing(t)
        };
        match self.effect.size() {
            None => div().opacity(progress).child(self.child).into_any_element(),
            Some(vertical) => {
                let shown = phase.measured * progress;
                let state = state.clone();
                div()
                    .overflow_hidden()
                    .when(!settled && vertical, |el| el.h(px(shown)))
                    .when(!settled && !vertical, |el| el.w(px(shown)).flex())
                    .child(
                        div()
                            .relative()
                            .when(!vertical, |el| el.flex_none())
                            .child(self.child)
                            .child(
                                canvas(
                                    move |bounds: Bounds<Pixels>, _, cx| {
                                        let size = if vertical {
                                            f32::from(bounds.size.height)
                                        } else {
                                            f32::from(bounds.size.width)
                                        };
                                        state.update(cx, |s, _| {
                                            if (s.measured - size).abs() > 0.5 {
                                                s.measured = size;
                                            }
                                        });
                                    },
                                    |_, _, _, _| {},
                                )
                                .absolute()
                                .inset_0(),
                            ),
                    )
                    .into_any_element()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, rc::Rc};

    #[test]
    fn effects_map_to_upstream_durations_and_channels() {
        assert_eq!(TransitionEffect::Fade.duration(), 0.2);
        assert_eq!(TransitionEffect::FadeLinear.duration(), 0.15);
        assert_eq!(TransitionEffect::Collapse.duration(), 0.2);
        assert_eq!(TransitionEffect::Fade.size(), None);
        assert_eq!(TransitionEffect::ZoomInCenter.size(), None);
        assert_eq!(TransitionEffect::Collapse.size(), Some(true));
        assert_eq!(TransitionEffect::CollapseHorizontal.size(), Some(false));
    }

    #[gpui::test]
    fn collapse_measures_content_and_settles_under_reduced_motion(cx: &mut gpui::TestAppContext) {
        struct Panel(Rc<Cell<f32>>, Rc<Cell<bool>>);
        impl gpui::Render for Panel {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let height = self.0.clone();
                let mounted = self.1.clone();
                div().w(px(200.)).child(
                    el_transition("panel", true, TransitionEffect::Collapse).child(
                        div()
                            .relative()
                            .h(px(80.))
                            .child(
                                canvas(
                                    move |bounds, _, _| {
                                        height.set(f32::from(bounds.size.height));
                                    },
                                    |_, _, _, _| {},
                                )
                                .absolute()
                                .inset_0(),
                            )
                            .child(canvas(move |_, _, _| mounted.set(true), |_, _, _, _| {})),
                    ),
                )
            }
        }
        let height = Rc::new(Cell::new(0.));
        let mounted = Rc::new(Cell::new(false));
        let (_, view) = cx.add_window_view({
            let height = height.clone();
            let mounted = mounted.clone();
            move |_, cx| {
                cx.set_reduce_motion(true);
                Panel(height, mounted)
            }
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        assert!(mounted.get(), "content renders when visible");
        // The transition settled immediately, so the content keeps its natural
        // height instead of the measured animation height.
        assert_eq!(height.get(), 80.);
    }
}
