//! Element Plus Switch.
//!
//! Tokens come from Theme Chalk `switch.scss` and `$switch-*` in
//! `common/var.scss`. The knob slides with the theme ease curve instead of
//! CSS `transition: all 300 ms`, and the checked state is exposed through the
//! accessibility toggle attribute because GPUI has no native checkbox input.

use gpui::{
    AnyElement, App, ElementId, Hsla, IntoElement, RenderOnce, SharedString, Toggled, Window, div,
    prelude::*, px, rgb,
};

use crate::{
    Theme,
    loading::inline_spinner,
    motion::{self, MotionSpeed},
    theme::PRIMARY,
    tokens,
};

type ChangeHandler = Box<dyn Fn(bool, &mut Window, &mut App)>;
type Guard = Box<dyn Fn() -> bool>;

/// `$switch-height`: the outer height, in logical pixels. The core keeps its
/// own `$switch-core-height`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SwitchSize {
    Large,
    #[default]
    Default,
    Small,
}

struct Metrics {
    /// `$switch-core-width`
    core_width: f32,
    /// `$switch-core-height`
    core_height: f32,
    /// `$switch-core-border-radius`
    radius: f32,
    /// `$switch-button-size`: the sliding knob
    button: f32,
    /// `$switch-content-padding`: the inner text inset
    content_padding: f32,
    /// `$switch-font-size`: the outside label text
    font_size: f32,
    /// `$switch-height`: the component height
    height: f32,
}

impl SwitchSize {
    fn metrics(self) -> Metrics {
        match self {
            Self::Large => Metrics {
                core_width: 50.,
                core_height: 24.,
                radius: 12.,
                button: 20.,
                content_padding: 6.,
                font_size: 14.,
                height: 40.,
            },
            Self::Default => Metrics {
                core_width: 40.,
                core_height: 20.,
                radius: 10.,
                button: 16.,
                content_padding: 4.,
                font_size: 14.,
                height: 32.,
            },
            Self::Small => Metrics {
                core_width: 30.,
                core_height: 16.,
                radius: 8.,
                button: 12.,
                content_padding: 2.,
                font_size: 12.,
                height: 24.,
            },
        }
    }
}

/// Desktop variant of Theme Chalk's 300 ms `transition-duration`: the same
/// brief feedback the library's other one-shot motion uses.
const SLIDE_SECS: f64 = 0.2;

/// The core width, clamped so the knob and its insets always stay visible
/// even when a caller passes a very small [`Switch::width`].
fn core_width(width: Option<f32>, metrics: &Metrics) -> f32 {
    width
        .unwrap_or(metrics.core_width)
        .max(metrics.button + 2. + 2. * metrics.content_padding)
}

/// One knob slide, kept in window state keyed by the switch's ID.
#[derive(Clone, Copy, Default)]
struct Knob {
    initialized: bool,
    checked: bool,
    started: f64,
}

/// Creates a switch in the default checked state. Pass an explicit ID with
/// [`Switch::id`] when several switches are built in a loop.
pub fn el_switch() -> Switch {
    Switch::default().id(ElementId::CodeLocation(*std::panic::Location::caller()))
}

/// A two-state toggle, like upstream `el-switch`.
///
/// The application owns the checked state and feeds it back through
/// [`Switch::checked`], matching upstream's `v-model`; [`Switch::onchange`]
/// reports the next value on pointer or keyboard activation. Native
/// differences: there is no hidden checkbox input, so the checked state is
/// conveyed with the accessibility toggle attribute and a visible focus ring;
/// `active-value`/`inactive-value` stay the application's concern, since the
/// controlled value is already mapped in `onchange`; `inline-prompt` renders
/// the full text with ellipsis instead of upstream's first character; and the
/// `before-change` hook is synchronous, so an asynchronous guard is the
/// caller's to schedule.
#[derive(Default, IntoElement)]
#[must_use]
pub struct Switch {
    id: Option<ElementId>,
    checked: bool,
    disabled: bool,
    loading: bool,
    inline_prompt: bool,
    size: SwitchSize,
    /// Overrides `$switch-core-width` in logical pixels, like upstream's
    /// numeric `width` prop. Percentage widths are a Web-only convenience.
    width: Option<f32>,
    speed: Option<MotionSpeed>,
    active_color: Option<Hsla>,
    inactive_color: Option<Hsla>,
    active_text: Option<SharedString>,
    inactive_text: Option<SharedString>,
    active_icon: Option<AnyElement>,
    inactive_icon: Option<AnyElement>,
    active_action_icon: Option<AnyElement>,
    inactive_action_icon: Option<AnyElement>,
    accessibility_label: Option<SharedString>,
    before_change: Option<Guard>,
    onchange: Option<ChangeHandler>,
}

impl Switch {
    /// Creates a switch with a stable GPUI element ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self::default().id(id)
    }

    /// Enables pointer and keyboard activation for this ID.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the checked state. The application owns this value and updates it
    /// in [`Self::onchange`], matching upstream's controlled usage.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Shows a spinner in the knob and blocks activation, like upstream.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Moves the text or icon inside the core instead of beside it.
    pub fn inline_prompt(mut self, inline: bool) -> Self {
        self.inline_prompt = inline;
        self
    }

    pub fn size(mut self, size: SwitchSize) -> Self {
        self.size = size;
        self
    }
    pub fn large(self) -> Self {
        self.size(SwitchSize::Large)
    }
    pub fn small(self) -> Self {
        self.size(SwitchSize::Small)
    }

    /// Sets the core width in logical pixels, overriding the size default.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Overrides the theme speed of the knob slide.
    pub fn speed(mut self, speed: MotionSpeed) -> Self {
        self.speed = Some(speed);
        self
    }
    pub fn fast(self) -> Self {
        self.speed(MotionSpeed::Fast)
    }
    pub fn slow(self) -> Self {
        self.speed(MotionSpeed::Slow)
    }
    pub fn normal(self) -> Self {
        self.speed(MotionSpeed::Normal)
    }

    /// Sets `$switch-on-color`, which defaults to the primary color.
    pub fn active_color(mut self, color: impl Into<Hsla>) -> Self {
        self.active_color = Some(color.into());
        self
    }

    /// Sets `$switch-off-color`, which defaults to the border color.
    pub fn inactive_color(mut self, color: impl Into<Hsla>) -> Self {
        self.inactive_color = Some(color.into());
        self
    }

    /// Text shown beside the core while checked. Ignored by
    /// [`Self::active_icon`], which takes precedence.
    pub fn active_text(mut self, text: impl Into<SharedString>) -> Self {
        self.active_text = Some(text.into());
        self
    }

    /// Text shown beside the core while unchecked. Ignored by
    /// [`Self::inactive_icon`], which takes precedence.
    pub fn inactive_text(mut self, text: impl Into<SharedString>) -> Self {
        self.inactive_text = Some(text.into());
        self
    }

    /// Content shown beside the core while checked, replacing
    /// [`Self::active_text`].
    pub fn active_icon(mut self, icon: impl IntoElement) -> Self {
        self.active_icon = Some(icon.into_any_element());
        self
    }

    /// Content shown beside the core while unchecked, replacing
    /// [`Self::inactive_text`].
    pub fn inactive_icon(mut self, icon: impl IntoElement) -> Self {
        self.inactive_icon = Some(icon.into_any_element());
        self
    }

    /// Content drawn inside the knob while checked, like upstream's
    /// `active-action-icon`.
    pub fn active_action_icon(mut self, icon: impl IntoElement) -> Self {
        self.active_action_icon = Some(icon.into_any_element());
        self
    }

    /// Content drawn inside the knob while unchecked, like upstream's
    /// `inactive-action-icon`.
    pub fn inactive_action_icon(mut self, icon: impl IntoElement) -> Self {
        self.inactive_action_icon = Some(icon.into_any_element());
        self
    }

    /// Names icon-only switches for assistive technology.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Gates every activation. Returning `false` leaves the state untouched,
    /// like upstream's `before-change` returning `false` or rejecting.
    pub fn before_change(mut self, guard: impl Fn() -> bool + 'static) -> Self {
        self.before_change = Some(Box::new(guard));
        self
    }

    /// Reports the next checked state on native pointer or keyboard
    /// activation. Not emitted while disabled or loading.
    #[track_caller]
    pub fn onchange(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.onchange = Some(Box::new(handler));
        self.ensure_id();
        self
    }

    #[track_caller]
    fn ensure_id(&mut self) {
        if self.id.is_none() {
            self.id = Some(ElementId::CodeLocation(*std::panic::Location::caller()));
        }
    }
}

impl RenderOnce for Switch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let metrics = self.size.metrics();
        let active_text = self.active_text.clone();
        let inactive_text = self.inactive_text.clone();
        let core_width = core_width(self.width, &metrics);
        let on = self.active_color.unwrap_or_else(|| rgb(PRIMARY).into());
        let off = self
            .inactive_color
            .unwrap_or_else(|| rgb(theme.border(false)).into());

        let label = |icon: Option<AnyElement>, text: Option<SharedString>| {
            icon.or_else(|| {
                text.map(|text| {
                    div()
                        .text_size(px(metrics.font_size))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .line_height(px(metrics.core_height))
                        .child(text)
                        .into_any_element()
                })
            })
        };
        // The same texts also feed the inline prompt, so the strings are
        // copied for the labels and again for the core.
        let inactive_label = label(self.inactive_icon, inactive_text.clone());
        let active_label = label(self.active_icon, active_text.clone());

        // Outside labels: upstream fades both in and out; the native control
        // shows the label that belongs to the current state. An inline prompt
        // moves the text into the core instead.
        let side = |content: Option<AnyElement>, margin_left: bool| {
            content.map(|content| {
                div()
                    .flex()
                    .items_center()
                    .when(margin_left, |el| el.ml(px(10.)))
                    .when(!margin_left, |el| el.mr(px(10.)))
                    .child(content)
                    .into_any_element()
            })
        };
        let (left, right) = if self.inline_prompt {
            (None, None)
        } else if self.checked {
            (None, active_label)
        } else {
            (inactive_label, None)
        };

        let root = div()
            .flex()
            .flex_none()
            .items_center()
            .h(px(metrics.height))
            .when_some(theme.resolved_font(cx), |el, family| el.font_family(family))
            .children(side(left, false))
            .children(side(right, true));

        let Some(id) = self.id else {
            return root.into_any_element();
        };

        let state = window.use_keyed_state(id.clone(), cx, |_, _| Knob::default());
        let now = motion::elapsed();
        let reduced = Theme::motion_reduced(cx);
        let duration = self.speed.unwrap_or(theme.motion_speed).scale(SLIDE_SECS);
        let mut knob = *state.read(cx);
        if !knob.initialized || knob.checked != self.checked {
            // A fresh slide starts now; a reversal continues from the position
            // the previous slide was still showing.
            let in_flight = knob.initialized && !reduced && (now - knob.started) / duration < 1.;
            let position = if in_flight {
                let t = ((now - knob.started) / duration).clamp(0., 1.) as f32;
                if knob.checked {
                    motion::element_plus(t)
                } else {
                    1. - motion::element_plus(t)
                }
            } else {
                0.
            };
            knob.initialized = true;
            knob.checked = self.checked;
            knob.started = if in_flight {
                now - position as f64 * duration
            } else {
                now
            };
            state.update(cx, |stored, _| *stored = knob);
        }
        let t = ((now - knob.started) / duration).clamp(0., 1.) as f32;
        let settled = t >= 1. || reduced;
        if !settled {
            window.request_animation_frame();
        }
        let position = if settled {
            if self.checked { 1. } else { 0. }
        } else if knob.checked {
            motion::element_plus(t)
        } else {
            1. - motion::element_plus(t)
        };

        // 1 px inset on each side, matching upstream's `left: 1px` and
        // `left: calc(100% - button - 1px)`.
        let travel = core_width - metrics.button - 2.;
        let knob_left = 1. + position * travel;
        let knob_top = (metrics.core_height - metrics.button) / 2.;
        let action = if self.loading {
            inline_spinner(metrics.button - 4.)
        } else if self.checked {
            self.active_action_icon
                .unwrap_or_else(|| div().into_any_element())
        } else {
            self.inactive_action_icon
                .unwrap_or_else(|| div().into_any_element())
        };
        let inner = if self.inline_prompt {
            // Upstream pads the inner text away from the knob; the native
            // control pins it to the opposite side of the core.
            let text = if self.checked {
                active_text
            } else {
                inactive_text
            };
            text.map(|text| {
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .flex()
                    .items_center()
                    .text_size(px(tokens::font_size::SMALL))
                    .text_color(rgb(0xffffff))
                    .when(self.checked, |el| el.left(px(metrics.content_padding)))
                    .when(!self.checked, |el| el.right(px(metrics.content_padding)))
                    .overflow_hidden()
                    .child(text)
                    .into_any_element()
            })
        } else {
            None
        };

        let core = div()
            .relative()
            .flex()
            .items_center()
            .w(px(core_width))
            .h(px(metrics.core_height))
            .rounded(px(metrics.radius))
            .border_1()
            .border_color(if self.checked { on } else { off })
            .bg(if self.checked { on } else { off })
            .when_some(inner, |el, inner| el.child(inner))
            .child(
                div()
                    .absolute()
                    .top(px(knob_top))
                    .left(px(knob_left))
                    .size(px(metrics.button))
                    .rounded_full()
                    .bg(rgb(0xffffff))
                    .flex()
                    .items_center()
                    .justify_center()
                    .overflow_hidden()
                    .child(action),
            );

        let mut root = root
            .id(id)
            .role(gpui::Role::Switch)
            .aria_toggled(if self.checked {
                Toggled::True
            } else {
                Toggled::False
            })
            .when_some(self.accessibility_label, |el, label| el.aria_label(label))
            .child(core);
        if self.disabled || self.loading {
            // Upstream dims the whole control while disabled; loading only
            // replaces the knob content and blocks activation.
            root = root.when(self.disabled, |el| el.opacity(0.6));
            return root.cursor_not_allowed().into_any_element();
        }
        let checked = self.checked;
        let guard = self.before_change;
        if let Some(handler) = self.onchange {
            root = root
                .cursor_pointer()
                .focusable()
                .tab_stop(true)
                // Upstream outlines the core with the on color while the
                // hidden input is focused; the native equivalent is a focus
                // ring on the switch itself.
                .focus_visible(move |s| {
                    s.shadow(vec![gpui::BoxShadow {
                        color: on,
                        offset: gpui::point(px(0.), px(0.)),
                        blur_radius: px(0.),
                        spread_radius: px(2.),
                        inset: false,
                    }])
                })
                .on_click(move |_, window, cx| {
                    if guard.as_ref().is_some_and(|guard| !guard()) {
                        return;
                    }
                    handler(!checked, window, cx);
                });
        }
        root.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_map_the_theme_chalk_tokens() {
        let large = SwitchSize::Large.metrics();
        assert_eq!(
            (
                large.core_width,
                large.core_height,
                large.radius,
                large.button
            ),
            (50., 24., 12., 20.)
        );
        let default = SwitchSize::Default.metrics();
        assert_eq!(
            (
                default.core_width,
                default.core_height,
                default.radius,
                default.button
            ),
            (40., 20., 10., 16.)
        );
        let small = SwitchSize::Small.metrics();
        assert_eq!(
            (
                small.core_width,
                small.core_height,
                small.radius,
                small.button
            ),
            (30., 16., 8., 12.)
        );
        assert_eq!(default.content_padding, 4.);
        assert_eq!(default.font_size, 14.);
        assert_eq!(default.height, 32.);
    }

    #[test]
    fn width_stays_wide_enough_for_the_knob() {
        let metrics = SwitchSize::Default.metrics();
        let narrow = core_width(Some(10.), &metrics);
        assert_eq!(narrow, metrics.button + 2. + 2. * metrics.content_padding);
        assert_eq!(core_width(None, &metrics), metrics.core_width);
        assert_eq!(core_width(Some(60.), &metrics), 60.);
        // The small size keeps its knob and insets visible as well.
        let small = SwitchSize::Small.metrics();
        assert_eq!(
            core_width(Some(0.), &small),
            small.button + 2. + 2. * small.content_padding
        );
    }

    #[test]
    fn fluent_settings_do_not_reset_other_dimensions() {
        let widget = el_switch()
            .checked(true)
            .disabled(false)
            .loading(true)
            .small()
            .inline_prompt(true)
            .active_text("On")
            .inactive_text("Off");
        assert!(widget.checked);
        assert!(!widget.disabled);
        assert!(widget.loading);
        assert_eq!(widget.size, SwitchSize::Small);
        assert!(widget.inline_prompt);
        assert!(el_switch().checked(true).small().checked);
        assert!(!el_switch().inline_prompt(false).inline_prompt);
    }

    struct Harness {
        toggles: std::rc::Rc<std::cell::Cell<usize>>,
        checked: std::rc::Rc<std::cell::Cell<bool>>,
        guarded: std::rc::Rc<std::cell::Cell<usize>>,
    }

    impl gpui::Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
            let toggles = self.toggles.clone();
            let checked = self.checked.clone();
            let guarded = self.guarded.clone();
            let view = cx.entity();
            div()
                .id("switch-tests")
                .tab_group()
                .flex()
                .flex_col()
                .size_full()
                .child({
                    let checked = checked.clone();
                    el_switch().id("toggle").checked(checked.get()).onchange(
                        move |now_checked, _, cx| {
                            toggles.set(toggles.get() + 1);
                            checked.set(now_checked);
                            view.update(cx, |_, cx| cx.notify());
                        },
                    )
                })
                .child({
                    let checked = checked.clone();
                    el_switch()
                        .id("blocked")
                        .checked(false)
                        .disabled(true)
                        .onchange(move |now_checked, _, _| checked.set(now_checked))
                })
                .child({
                    let checked = checked.clone();
                    el_switch()
                        .id("guard")
                        .checked(false)
                        .before_change(move || {
                            guarded.set(guarded.get() + 1);
                            false
                        })
                        .onchange(move |now_checked, _, _| checked.set(now_checked))
                })
        }
    }

    #[gpui::test]
    fn pointer_and_keyboard_toggle_once_per_activation(cx: &mut gpui::TestAppContext) {
        let toggles = std::rc::Rc::new(std::cell::Cell::new(0));
        let checked = std::rc::Rc::new(std::cell::Cell::new(false));
        let guarded = std::rc::Rc::new(std::cell::Cell::new(0));
        let (_, view) = cx.add_window_view({
            let toggles = toggles.clone();
            let checked = checked.clone();
            let guarded = guarded.clone();
            move |_, cx| {
                cx.set_reduce_motion(true);
                Harness {
                    toggles,
                    checked,
                    guarded,
                }
            }
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        // Rows: 32, 32, 32 px tall switches.
        view.simulate_click(gpui::point(px(20.), px(16.)), gpui::Modifiers::default());
        assert_eq!(toggles.get(), 1);
        assert!(checked.get());
        // Keyboard activation reaches the focused switch as well.
        press_enter(view);
        assert_eq!(toggles.get(), 2);
        assert!(!checked.get());
        // Disabled switches swallow the activation.
        view.simulate_click(gpui::point(px(20.), px(48.)), gpui::Modifiers::default());
        assert_eq!(toggles.get(), 2);
        // A rejecting guard leaves the state untouched but is consulted.
        view.simulate_click(gpui::point(px(20.), px(80.)), gpui::Modifiers::default());
        assert_eq!(guarded.get(), 1);
        assert!(!checked.get());
    }

    fn press_enter(view: &mut gpui::VisualTestContext) {
        let keystroke = view.update(|_, cx| {
            gpui::Keystroke::parse_with_os("enter", cx.platform_info().os).unwrap()
        });
        view.simulate_event(gpui::KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        view.simulate_event(gpui::KeyUpEvent { keystroke });
        view.update(|window, cx| window.draw(cx).clear(cx));
    }
}
