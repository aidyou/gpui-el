//! Element Plus Rate.
//!
//! Tokens from Theme Chalk `rate.scss` and `$rate-*` in `common/var.scss`.
//! The stars are the bundled Element Plus `StarFilled` and `Star` outlines,
//! and the half star is a clipped overlay of the active icon rather than
//! upstream's decimal pseudo element.

use gpui::{
    App, Bounds, ElementId, Hsla, IntoElement, KeyDownEvent, MouseMoveEvent, Pixels, RenderOnce,
    SharedString, Window, div, prelude::*, px, rgb,
};

use std::sync::Arc;

use crate::{
    Theme,
    icon::Icon,
    theme::{self, Fill},
};

/// Shared between every star's activation closure, so one handler serves the
/// whole rate.
type ChangeHandler = Arc<dyn Fn(f32, &mut Window, &mut App)>;

/// `$rate-height`: the outer height, in logical pixels.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RateSize {
    Large,
    #[default]
    Default,
    Small,
}

struct Metrics {
    /// `$rate-height`
    height: f32,
    /// `$rate-icon-size` (`$rate-size` for small)
    icon: f32,
    /// `$rate-font-size`
    font: f32,
}

impl RateSize {
    fn metrics(self) -> Metrics {
        match self {
            Self::Large => Metrics {
                height: 40.,
                icon: 18.,
                font: 14.,
            },
            Self::Default => Metrics {
                height: 32.,
                icon: 18.,
                font: 14.,
            },
            Self::Small => Metrics {
                height: 24.,
                icon: 14.,
                font: 12.,
            },
        }
    }
}

/// `$rate-fill-color`: the default color of every active level.
const FILL: u32 = 0xf7ba2a;

/// `$rate-void-color` (`--el-border-color-darker`).
fn void_color(theme: &Theme) -> u32 {
    if theme.dark {
        theme::mix(theme.background(), 0xf5f8ff, 35)
    } else {
        0xcdd0d6
    }
}

/// Upstream's default `texts`, shown with `show_text`.
const DEFAULT_TEXTS: [&str; 5] = [
    "Extremely bad",
    "Disappointed",
    "Fair",
    "Satisfied",
    "Surprise",
];

/// Creates a rate with no value. Pass an explicit ID with [`Rate::id`] when
/// several rates are built in a loop.
pub fn el_rate() -> Rate {
    Rate::default().id(ElementId::CodeLocation(*std::panic::Location::caller()))
}

/// A star rating, like upstream `el-rate`.
///
/// The application owns the value and feeds it back through [`Rate::value`],
/// matching upstream's `v-model`; [`Rate::onchange`] reports the selection on
/// pointer or keyboard activation. Native differences: the icons are [`Icon`]
/// values instead of Vue components, so the three levels take an array and
/// arbitrary content is not supported; the hovered star does not scale, since
/// GPUI has no transform animation, but the fill still previews the
/// selection; and the score template replaces `{value}` with Rust's `f32`
/// display, which prints `3` rather than `3.0`.
#[derive(IntoElement)]
#[must_use]
pub struct Rate {
    id: Option<ElementId>,
    value: f32,
    max: u32,
    size: RateSize,
    allow_half: bool,
    clearable: bool,
    disabled: bool,
    show_text: bool,
    show_score: bool,
    texts: Vec<SharedString>,
    score_template: SharedString,
    low_threshold: f32,
    high_threshold: f32,
    colors: Option<[Hsla; 3]>,
    void_color: Option<Hsla>,
    disabled_void_color: Option<Hsla>,
    text_color: Option<Hsla>,
    icons: Option<[Icon; 3]>,
    void_icon: Option<Icon>,
    disabled_void_icon: Option<Icon>,
    accessibility_label: Option<SharedString>,
    onchange: Option<ChangeHandler>,
}

impl Default for Rate {
    fn default() -> Self {
        Self {
            id: None,
            value: 0.,
            max: 5,
            size: RateSize::Default,
            allow_half: false,
            clearable: false,
            disabled: false,
            show_text: false,
            show_score: false,
            texts: Vec::new(),
            score_template: "{value}".into(),
            low_threshold: 2.,
            high_threshold: 4.,
            colors: None,
            void_color: None,
            disabled_void_color: None,
            text_color: None,
            icons: None,
            void_icon: None,
            disabled_void_icon: None,
            accessibility_label: None,
            onchange: None,
        }
    }
}

impl Rate {
    /// Creates a rate with a stable GPUI element ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self::default().id(id)
    }

    /// Enables pointer and keyboard activation for this ID.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the rating. Halves are kept when [`Self::allow_half`] is on.
    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    /// Sets the highest rating, five stars by default.
    pub fn max(mut self, max: u32) -> Self {
        self.max = max.max(1);
        self
    }

    pub fn size(mut self, size: RateSize) -> Self {
        self.size = size;
        self
    }
    pub fn large(self) -> Self {
        self.size(RateSize::Large)
    }
    pub fn small(self) -> Self {
        self.size(RateSize::Small)
    }

    /// Lets the pointer and arrow keys pick half stars.
    pub fn allow_half(mut self, allow_half: bool) -> Self {
        self.allow_half = allow_half;
        self
    }

    /// Resets the value to zero when the current value is picked again.
    pub fn clearable(mut self, clearable: bool) -> Self {
        self.clearable = clearable;
        self
    }

    /// Makes the rating read-only, like upstream's `disabled`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Shows the text that matches the current score, from [`Self::texts`].
    pub fn show_text(mut self, show_text: bool) -> Self {
        self.show_text = show_text;
        self
    }

    /// Shows the score through [`Self::score_template`]. Wins over
    /// [`Self::show_text`], like upstream.
    pub fn show_score(mut self, show_score: bool) -> Self {
        self.show_score = show_score;
        self
    }

    /// Sets the texts for [`Self::show_text`], one per whole star. Empty
    /// falls back to upstream's English defaults.
    pub fn texts(mut self, texts: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.texts = texts.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the template for [`Self::show_score`], `{value}` by default.
    pub fn score_template(mut self, template: impl Into<SharedString>) -> Self {
        self.score_template = template.into();
        self
    }

    /// Sets the top of the low score level, two by default. The value itself
    /// belongs to the low level.
    pub fn low_threshold(mut self, threshold: f32) -> Self {
        self.low_threshold = threshold;
        self
    }

    /// Sets the top of the medium score level, four by default. The value
    /// itself belongs to the high level.
    pub fn high_threshold(mut self, threshold: f32) -> Self {
        self.high_threshold = threshold;
        self
    }

    /// Sets the active colors of the three score levels. Defaults to
    /// `$rate-fill-color` at every level.
    pub fn colors(mut self, colors: [impl Into<Hsla>; 3]) -> Self {
        self.colors = Some(colors.map(Into::into));
        self
    }

    /// Sets the color of the unselected stars, `$rate-void-color` by default.
    pub fn void_color(mut self, color: impl Into<Hsla>) -> Self {
        self.void_color = Some(color.into());
        self
    }

    /// Sets the color of the unselected read-only stars, `$fill-color` by
    /// default.
    pub fn disabled_void_color(mut self, color: impl Into<Hsla>) -> Self {
        self.disabled_void_color = Some(color.into());
        self
    }

    /// Sets the color of the score text, `$text-color-primary` by default.
    pub fn text_color(mut self, color: impl Into<Hsla>) -> Self {
        self.text_color = Some(color.into());
        self
    }

    /// Sets the icon of every active level, `StarFilled` by default.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        let icon = icon.into();
        self.icons = Some([icon.clone(), icon.clone(), icon]);
        self
    }

    /// Sets the icons of the three score levels, low to high.
    pub fn icons(mut self, icons: [impl Into<Icon>; 3]) -> Self {
        self.icons = Some(icons.map(Into::into));
        self
    }

    /// Sets the icon of the unselected stars, `Star` by default.
    pub fn void_icon(mut self, icon: impl Into<Icon>) -> Self {
        self.void_icon = Some(icon.into());
        self
    }

    /// Sets the icon of the unselected read-only stars, `StarFilled` by
    /// default.
    pub fn disabled_void_icon(mut self, icon: impl Into<Icon>) -> Self {
        self.disabled_void_icon = Some(icon.into());
        self
    }

    /// Names the rate for assistive technology; defaults to "rating" like
    /// upstream's `aria-label`.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Reports the selected value on pointer or keyboard activation. Not
    /// emitted while read-only.
    #[track_caller]
    pub fn onchange(mut self, handler: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.onchange = Some(Arc::new(handler));
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

/// The score level of `value`, using upstream's thresholds: `value <= low` is
/// low, `value < high` is medium, everything else is high.
fn level(value: f32, low: f32, high: f32) -> usize {
    if value <= low {
        0
    } else if value < high {
        1
    } else {
        2
    }
}

impl RenderOnce for Rate {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let metrics = self.size.metrics();
        let max = self.max;
        let value = self.value.clamp(0., max as f32);
        let fill: Hsla = rgb(FILL).into();
        let colors = self.colors.unwrap_or([fill; 3]);
        let void = self
            .void_color
            .unwrap_or_else(|| rgb(void_color(&theme)).into());
        let disabled_void = self
            .disabled_void_color
            .unwrap_or_else(|| rgb(theme.fill(Fill::Light)).into());
        let text_color = self
            .text_color
            .unwrap_or_else(|| rgb(theme.primary_text()).into());
        let step = if self.allow_half { 0.5 } else { 1. };

        let Some(id) = self.id else {
            return div().into_any_element();
        };
        // The hover preview is transient state the application never owns, so
        // it lives in window state keyed by this rate's ID.
        let state = window.use_keyed_state(id.clone(), cx, |_, _| None::<f32>);
        let preview = *state.read(cx);
        let current = if self.disabled {
            value
        } else {
            preview.unwrap_or(value).clamp(0., max as f32)
        };

        // One active icon and color for the whole rate, chosen from the level
        // of the current value, like upstream's `activeComponent`.
        let level = level(current, self.low_threshold, self.high_threshold);
        let [low_color, mid_color, high_color] = colors;
        let active_color = match level {
            0 => low_color,
            1 => mid_color,
            _ => high_color,
        };
        let active_icon = match &self.icons {
            Some([low, mid, high]) => match level {
                0 => low.clone(),
                1 => mid.clone(),
                _ => high.clone(),
            },
            None => Icon::from_pack(crate::IconPack::ElementPlus, "StarFilled"),
        }
        .fallback(metrics.icon, active_color);
        let void_icon = match &self.void_icon {
            Some(icon) => icon.clone(),
            None => Icon::from_pack(crate::IconPack::ElementPlus, "Star"),
        }
        .fallback(metrics.icon, void);
        let disabled_void_icon = match &self.disabled_void_icon {
            Some(icon) => icon.clone(),
            None => Icon::from_pack(crate::IconPack::ElementPlus, "StarFilled"),
        }
        .fallback(metrics.icon, disabled_void);

        let mut stars = Vec::with_capacity(max as usize);
        for index in 1..=max {
            let star = index as f32;
            let filled = star <= current;
            // The overlap of the value with this star: 0.5 for a half star,
            // 0.7 for the read-only decimal upstream clips to 70 %.
            let fraction = (current - (index - 1) as f32).clamp(0., 1.);
            let decimal = fraction > 0. && fraction < 1. && (self.allow_half || self.disabled);
            let icon = if filled {
                active_icon.clone()
            } else if self.disabled {
                disabled_void_icon.clone()
            } else {
                void_icon.clone()
            };
            let item_id: ElementId = (id.clone(), format!("star{index}")).into();
            let mut item = div()
                .id(item_id.clone())
                .relative()
                .flex_none()
                .size(px(metrics.icon))
                .when(index < max, |el| el.mr(px(6.)))
                .child(icon)
                .when(decimal, |el| {
                    // The active icon clipped to the fractional width, over
                    // the void star underneath.
                    el.child(
                        div()
                            .absolute()
                            .left_0()
                            .top_0()
                            .bottom_0()
                            .w(px(metrics.icon * fraction))
                            .overflow_hidden()
                            .child(active_icon.clone()),
                    )
                });
            if !self.disabled {
                let handler = self.onchange.clone();
                let clearable = self.clearable;
                let pick = move |picked: f32, window: &mut Window, cx: &mut App| {
                    // Clearing re-picks the current value, like upstream.
                    let picked = if clearable && picked == value {
                        0.
                    } else {
                        picked
                    };
                    if let Some(handler) = &handler {
                        handler(picked, window, cx);
                    }
                };
                if self.allow_half {
                    let left_state = state.clone();
                    let left_pick = pick.clone();
                    let left = div()
                        .id((item_id.clone(), "left"))
                        .absolute()
                        .left_0()
                        .top_0()
                        .bottom_0()
                        .w(px(metrics.icon / 2.))
                        .on_mouse_move(move |_: &MouseMoveEvent, _, cx| {
                            left_state.update(cx, |stored, _| *stored = Some(star - 0.5));
                        })
                        .on_click(move |_, window, cx| left_pick(star - 0.5, window, cx));
                    let right_state = state.clone();
                    let right_pick = pick.clone();
                    let right = div()
                        .id((item_id.clone(), "right"))
                        .absolute()
                        .right_0()
                        .top_0()
                        .bottom_0()
                        .w(px(metrics.icon / 2.))
                        .on_mouse_move(move |_: &MouseMoveEvent, _, cx| {
                            right_state.update(cx, |stored, _| *stored = Some(star));
                        })
                        .on_click(move |_, window, cx| right_pick(star, window, cx));
                    item = item.child(left).child(right);
                } else {
                    let full_state = state.clone();
                    let full = div()
                        .id((item_id.clone(), "full"))
                        .absolute()
                        .inset_0()
                        .on_mouse_move(move |_: &MouseMoveEvent, _, cx| {
                            full_state.update(cx, |stored, _| *stored = Some(star));
                        })
                        .on_click(move |_, window, cx| pick(star, window, cx));
                    item = item.child(full);
                }
            }
            stars.push(item);
        }

        // Upstream shows the score template with `show_score`, the matching
        // text with `show_text`, and nothing at all for a zero value.
        let text: Option<SharedString> = if self.show_score {
            Some(SharedString::from(
                self.score_template
                    .replace("{value}", &format!("{current}")),
            ))
        } else if self.show_text && current > 0. {
            let texts = if self.texts.is_empty() {
                DEFAULT_TEXTS
                    .into_iter()
                    .map(SharedString::from)
                    .collect::<Vec<_>>()
            } else {
                self.texts.clone()
            };
            texts
                .get((current.ceil() as usize).saturating_sub(1))
                .cloned()
        } else {
            None
        };

        let mut root = div()
            .flex()
            .flex_none()
            .items_center()
            .h(px(metrics.height))
            .text_size(px(metrics.font))
            .id(id)
            .role(gpui::Role::Slider)
            .aria_numeric_value(current as f64)
            .aria_min_numeric_value(0.)
            .aria_max_numeric_value(max as f64)
            .aria_numeric_value_step(step as f64)
            .when_some(text.clone(), |el, text| el.aria_value(text))
            .aria_label(self.accessibility_label.unwrap_or_else(|| "rating".into()))
            .children(stars)
            .when_some(text, |el, text| {
                el.child(div().ml(px(6.)).text_color(text_color).child(text))
            });
        if self.disabled {
            return root.into_any_element();
        }
        let exit_state = state.clone();
        root = root
            .cursor_pointer()
            .focusable()
            .tab_stop(true)
            .on_mouse_exit(move |_, _, cx| {
                exit_state.update(cx, |stored, _| *stored = None);
            });
        // gpui-fast only synthesizes MouseExitEvent when the pointer leaves
        // the window, not when it moves between elements, so the preview
        // would outlive the hover. Like the platform's own hover styling,
        // arm a window-level move listener each frame and drop the preview
        // once the position falls outside the bounds measured here.
        let preview = state.clone();
        root = root.relative().child(
            gpui::canvas(
                move |bounds: Bounds<Pixels>, _window: &mut Window, _cx: &mut App| bounds,
                move |bounds: Bounds<Pixels>, _t, window: &mut Window, _cx: &mut App| {
                    let state = preview.clone();
                    window.on_mouse_event(
                        move |event: &MouseMoveEvent, phase, _window, cx| {
                            if phase == gpui::DispatchPhase::Bubble
                                && !bounds.contains(&event.position)
                                && state.read(cx).is_some()
                            {
                                state.update(cx, |stored, _| *stored = None);
                            }
                        },
                    );
                },
            )
            .absolute()
            .inset_0(),
        );
        if let Some(handler) = self.onchange {
            root = root.on_key_down(move |event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_ref();
                let next = match key {
                    "up" | "right" => Some(current + step),
                    "down" | "left" => Some(current - step),
                    _ => None,
                };
                if let Some(next) = next.map(|next| next.clamp(0., max as f32))
                    && (next - current).abs() > f32::EPSILON
                {
                    cx.stop_propagation();
                    handler(next, window, cx);
                }
            });
        }
        root.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::el_button;

    #[test]
    fn sizes_map_the_theme_chalk_tokens() {
        assert_eq!(RateSize::Large.metrics().height, 40.);
        assert_eq!(RateSize::Default.metrics().height, 32.);
        assert_eq!(RateSize::Small.metrics().height, 24.);
        assert_eq!(RateSize::Large.metrics().icon, 18.);
        assert_eq!(RateSize::Small.metrics().icon, 14.);
        assert_eq!(RateSize::Small.metrics().font, 12.);
    }

    #[test]
    fn levels_follow_the_upstream_thresholds() {
        assert_eq!(level(0., 2., 4.), 0);
        assert_eq!(level(2., 2., 4.), 0);
        assert_eq!(level(2.5, 2., 4.), 1);
        assert_eq!(level(3.99, 2., 4.), 1);
        assert_eq!(level(4., 2., 4.), 2);
        assert_eq!(level(5., 2., 4.), 2);
    }

    #[test]
    fn fluent_settings_do_not_reset_other_dimensions() {
        let widget = el_rate()
            .value(3.5)
            .max(10)
            .allow_half(true)
            .clearable(true)
            .show_text(true)
            .small();
        assert_eq!(widget.value, 3.5);
        assert_eq!(widget.max, 10);
        assert!(widget.allow_half);
        assert!(widget.clearable);
        assert!(widget.show_text);
        assert_eq!(widget.size, RateSize::Small);
        assert_eq!(el_rate().max(0).max, 1);
        assert_eq!(el_rate().value(4.).max, 5);
    }

    #[test]
    fn icon_setters_fill_all_levels() {
        let rate = el_rate().icon(Icon::from_pack(crate::IconPack::ElementPlus, "StarFilled"));
        let icons = rate.icons.unwrap();
        assert_eq!(icons.len(), 3);
        let rate = el_rate().icons([
            Icon::from_pack(crate::IconPack::ElementPlus, "StarFilled"),
            Icon::from_pack(crate::IconPack::ElementPlus, "StarFilled"),
            Icon::from_pack(crate::IconPack::ElementPlus, "StarFilled"),
        ]);
        assert!(rate.icons.is_some());
        assert!(el_rate().void_icon(Icon::named("star")).void_icon.is_some());
    }

    struct Harness {
        picked: std::rc::Rc<std::cell::Cell<f32>>,
        value: std::rc::Rc<std::cell::Cell<f32>>,
    }

    impl gpui::Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
            let picked = self.picked.clone();
            let value = self.value.clone();
            let view = cx.entity();
            div()
                .id("rate-tests")
                .tab_group()
                .flex()
                .flex_col()
                .size_full()
                .child({
                    let picked = picked.clone();
                    let value = value.clone();
                    el_rate()
                        .id("basic")
                        .value(value.get())
                        .onchange(move |picked_value, _, cx| {
                            picked.set(picked_value);
                            value.set(picked_value);
                            view.update(cx, |_, cx| cx.notify());
                        })
                })
                .child({
                    let picked = picked.clone();
                    el_rate()
                        .id("half")
                        .allow_half(true)
                        .value(2.5)
                        .onchange(move |picked_value, _, _| picked.set(picked_value))
                })
                .child(el_rate().id("blocked").value(3.).disabled(true))
        }
    }

    #[gpui::test]
    fn pointer_and_keyboard_pick_whole_and_half_values(cx: &mut gpui::TestAppContext) {
        let picked = std::rc::Rc::new(std::cell::Cell::new(0.));
        let value = std::rc::Rc::new(std::cell::Cell::new(0.));
        let (_, view) = cx.add_window_view({
            let picked = picked.clone();
            let value = value.clone();
            move |_, cx| {
                cx.set_reduce_motion(true);
                Harness {
                    picked: picked.clone(),
                    value: value.clone(),
                }
            }
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        // Stars are 18 px on a 24 px pitch (18 + 6 margin): row 1 sits at
        // y 0..32, so the third star spans x 48..66.
        view.simulate_click(gpui::point(px(57.), px(16.)), gpui::Modifiers::default());
        assert_eq!(picked.get(), 3.);
        // Arrow keys step by one.
        press_key(view, "right");
        assert_eq!(picked.get(), 4.);
        press_key(view, "up");
        assert_eq!(picked.get(), 5.);
        press_key(view, "down");
        assert_eq!(picked.get(), 4.);
        // Clamped at the top and bottom.
        press_key(view, "up");
        assert_eq!(picked.get(), 5.);
        press_key(view, "left");
        press_key(view, "left");
        press_key(view, "left");
        press_key(view, "left");
        press_key(view, "left");
        press_key(view, "left");
        assert_eq!(picked.get(), 0.);
        // The half rate sits on row 2 (y 32..64); its third star's left half
        // (x 48..57) picks 2.5 and the right half (x 57..66) picks 3.
        view.simulate_click(gpui::point(px(52.), px(48.)), gpui::Modifiers::default());
        assert_eq!(picked.get(), 2.5);
        view.simulate_click(gpui::point(px(61.), px(48.)), gpui::Modifiers::default());
        assert_eq!(picked.get(), 3.);
        // Read-only rates never report.
        view.simulate_click(gpui::point(px(57.), px(80.)), gpui::Modifiers::default());
        assert_eq!(picked.get(), 3.);
    }

    struct ResetHarness {
        value: std::rc::Rc<std::cell::Cell<f32>>,
    }

    impl gpui::Render for ResetHarness {
        fn render(&mut self, _: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
            let view = cx.entity();
            let value = self.value.clone();
            div()
                .id("rate-reset-tests")
                .tab_group()
                .flex()
                .flex_col()
                .size_full()
                .on_key_down(move |event: &gpui::KeyDownEvent, _, _| {
                    eprintln!("ROOT KEY {:?}", event.keystroke.key);
                })
                .child(
                    el_rate()
                        .id("controlled")
                        .value(value.get())
                        .onchange(move |picked, _, cx| {
                            view.update(cx, |this, cx| {
                                this.value.set(picked);
                                cx.notify();
                            });
                        }),
                )
                .child(
                    el_button()
                        .id("reset")
                        .small()
                        .label("reset")
                        .onclick(cx.listener(|this, _, _, cx| {
                            this.value.set(0.);
                            cx.notify();
                        })),
                )
        }
    }

    #[gpui::test]
    fn a_controlled_rate_follows_a_reset_to_zero(cx: &mut gpui::TestAppContext) {
        let value = std::rc::Rc::new(std::cell::Cell::new(0.));
        let (_, view) = cx.add_window_view({
            let value = value.clone();
            move |_, cx| {
                cx.set_reduce_motion(true);
                ResetHarness { value }
            }
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        // Hover the fifth star and pick it, then move the pointer off the
        // rate onto the reset button below and click it, as a pointer user
        // would.
        view.simulate_mouse_move(
            gpui::point(px(105.), px(16.)),
            None,
            gpui::Modifiers::default(),
        );
        view.simulate_click(gpui::point(px(105.), px(16.)), gpui::Modifiers::default());
        assert_eq!(value.get(), 5.);
        view.simulate_mouse_move(
            gpui::point(px(30.), px(44.)),
            None,
            gpui::Modifiers::default(),
        );
        // The reset button spans y 32..56.
        view.simulate_click(gpui::point(px(30.), px(44.)), gpui::Modifiers::default());
        assert_eq!(value.get(), 0.);
        view.update(|window, cx| window.draw(cx).clear(cx));
        // The display follows the reset: clicking the gap between the first
        // two stars focuses the rate without picking, and a step right
        // reports 1, not 6 left over from the pre-reset hover preview.
        view.simulate_click(gpui::point(px(20.), px(16.)), gpui::Modifiers::default());
        view.update(|window, cx| window.draw(cx).clear(cx));
        press_key(view, "right");
        assert_eq!(value.get(), 1.);
    }

    #[gpui::test]
    fn the_hover_preview_clears_when_the_pointer_leaves(cx: &mut gpui::TestAppContext) {
        let value = std::rc::Rc::new(std::cell::Cell::new(0.));
        let (_, view) = cx.add_window_view({
            let value = value.clone();
            move |_, cx| {
                cx.set_reduce_motion(true);
                ResetHarness { value }
            }
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        // Pick the third star; the click also focuses the rate, so arrow
        // keys step from the displayed value.
        view.simulate_click(gpui::point(px(57.), px(16.)), gpui::Modifiers::default());
        assert_eq!(value.get(), 3.);
        view.update(|window, cx| window.draw(cx).clear(cx));
        // Hovering the fifth star previews 5, so a step left reports 4, not
        // 2: the preview drives the display while the pointer is over the
        // rate.
        view.simulate_mouse_move(
            gpui::point(px(105.), px(16.)),
            None,
            gpui::Modifiers::default(),
        );
        view.update(|window, cx| window.draw(cx).clear(cx));
        press_key(view, "left");
        assert_eq!(value.get(), 4.);
        // Once the pointer leaves the rate the preview clears and the display
        // falls back to the controlled value, so a step right reports 5.
        view.simulate_mouse_move(
            gpui::point(px(30.), px(44.)),
            None,
            gpui::Modifiers::default(),
        );
        view.update(|window, cx| window.draw(cx).clear(cx));
        press_key(view, "right");
        assert_eq!(value.get(), 5.);
    }

    fn press_key(view: &mut gpui::VisualTestContext, key: &str) {
        let keystroke = view
            .update(|_, cx| gpui::Keystroke::parse_with_os(key, cx.platform_info().os).unwrap());
        view.simulate_event(gpui::KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        view.simulate_event(gpui::KeyUpEvent { keystroke });
        view.update(|window, cx| window.draw(cx).clear(cx));
    }
}
