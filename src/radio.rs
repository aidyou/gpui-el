//! Element Plus Radio and RadioGroup.
//!
//! Tokens from Theme Chalk `radio.scss` and `$radio-*` in `common/var.scss`.
//! The selection dot is a small filled circle instead of upstream's scaled
//! pseudo element, and the checked state is exposed through the accessibility
//! toggle attribute because GPUI has no native radio input.

use gpui::{
    AnyElement, App, ElementId, Hsla, IntoElement, RenderOnce, SharedString, Toggled, Window, div,
    prelude::*, px, rgb,
};

use crate::{
    Direction, Theme,
    theme::{Fill, PRIMARY, WHITE},
    tokens,
};

type ChangeHandler = Box<dyn Fn(bool, &mut Window, &mut App)>;

/// Radio sizes map to `$radio-height` and `$radio-font-size`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RadioSize {
    Large,
    #[default]
    Default,
    Small,
}

struct Metrics {
    /// `$radio-height`, the outer height.
    height: f32,
    /// `$radio-font-size`, also the input size without a border.
    font_size: f32,
    /// `$radio-bordered-input-*`, the input size with a border.
    bordered_input: f32,
    /// `$radio-bordered-padding-*`
    padding_left: f32,
    padding_right: f32,
    /// `$button-border-radius` for the bordered variant.
    radius: f32,
}

impl RadioSize {
    fn metrics(self) -> Metrics {
        match self {
            Self::Large => Metrics {
                height: 40.,
                font_size: 14.,
                bordered_input: 14.,
                padding_left: 12.,
                padding_right: 20.,
                radius: tokens::radius::BASE,
            },
            Self::Default => Metrics {
                height: 32.,
                font_size: 14.,
                bordered_input: 12.,
                padding_left: 10.,
                padding_right: 16.,
                radius: tokens::radius::BASE,
            },
            Self::Small => Metrics {
                height: 24.,
                font_size: 12.,
                bordered_input: 12.,
                padding_left: 8.,
                padding_right: 12.,
                radius: tokens::radius::BASE - 1.,
            },
        }
    }
}

/// Creates an unselected radio. Pass an explicit ID with [`Radio::id`] when
/// several radios are built in a loop.
pub fn el_radio() -> Radio {
    Radio::default().id(ElementId::CodeLocation(*std::panic::Location::caller()))
}

/// One option of a mutually exclusive group, like upstream `el-radio`.
///
/// The application owns the selection and marks the chosen radio with
/// [`Radio::checked`], matching upstream's `v-model`; [`Radio::onchange`]
/// reports the activation, and the application moves the selection. Native
/// differences: there is no hidden input, so the selected state is conveyed
/// with the accessibility toggle attribute and a visible focus ring; the
/// group does not collect values, so the application identifies the picked
/// radio in its own `onchange` closure; and upstream's arrow-key navigation
/// between group items is not reproduced, native Tab order applies instead.
#[derive(Default, IntoElement)]
#[must_use]
pub struct Radio {
    id: Option<ElementId>,
    label: Option<SharedString>,
    accessibility_label: Option<SharedString>,
    content: Option<AnyElement>,
    checked: bool,
    disabled: bool,
    /// Whether `disabled` was set explicitly, so a group cannot override it.
    explicit_disabled: bool,
    size: RadioSize,
    border: bool,
    /// Whether `border` was set explicitly, so a group cannot override it.
    explicit_border: bool,
    active_color: Option<Hsla>,
    inactive_color: Option<Hsla>,
    onchange: Option<ChangeHandler>,
}

impl Radio {
    /// Creates a radio with a stable GPUI element ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self::default().id(id)
    }

    /// Enables pointer and keyboard activation for this ID.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the text next to the input, replacing any custom content.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self.content = None;
        self
    }

    /// Names icon-only and custom-content radios for assistive technology.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Replaces the label with custom GPUI content.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self.label = None;
        self
    }

    /// Marks this radio as the current selection. The application owns the
    /// selection and moves it in [`Self::onchange`].
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self.explicit_disabled = true;
        self
    }

    pub fn size(mut self, size: RadioSize) -> Self {
        self.size = size;
        self
    }
    pub fn large(self) -> Self {
        self.size(RadioSize::Large)
    }
    pub fn small(self) -> Self {
        self.size(RadioSize::Small)
    }

    /// Draws the surrounding border of the `border` attribute.
    pub fn border(mut self, border: bool) -> Self {
        self.border = border;
        self.explicit_border = true;
        self
    }

    /// Sets the selected accent color; primary by default. Disabled radios
    /// keep their neutral disabled colors.
    pub fn active_color(mut self, color: impl Into<Hsla>) -> Self {
        self.active_color = Some(color.into());
        self
    }

    /// Sets the unchecked outline color; the theme border color by default.
    /// Disabled radios keep their neutral disabled colors.
    pub fn inactive_color(mut self, color: impl Into<Hsla>) -> Self {
        self.inactive_color = Some(color.into());
        self
    }

    /// Reports that this radio was activated by pointer or keyboard. Not
    /// emitted while disabled, and clicking the already selected radio does
    /// not switch it elsewhere, like upstream.
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

impl RenderOnce for Radio {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let metrics = self.size.metrics();
        let input = if self.border {
            metrics.bordered_input
        } else {
            metrics.font_size
        };
        // Upstream fills the input with the primary color and centers a 4 px
        // white dot when selected; the disabled states swap in neutral fills.
        let border = theme.border(false);
        let active_color: Hsla = self.active_color.unwrap_or_else(|| rgb(PRIMARY).into());
        let inactive_color: Hsla = self.inactive_color.unwrap_or_else(|| rgb(border).into());
        let (input_bg, input_border, dot) = if self.disabled {
            if self.checked {
                (
                    rgb(theme.fill(Fill::Light)).into(),
                    rgb(border).into(),
                    theme.disabled_text(),
                )
            } else {
                (
                    rgb(theme.fill(Fill::Light)).into(),
                    rgb(border).into(),
                    theme.fill(Fill::Light),
                )
            }
        } else if self.checked {
            (active_color, active_color, WHITE)
        } else {
            (
                rgb(theme.background()).into(),
                inactive_color,
                theme.background(),
            )
        };
        let label_color: Hsla = if self.disabled {
            rgb(theme.disabled_text()).into()
        } else if self.checked {
            active_color
        } else {
            rgb(theme.regular_text()).into()
        };
        let accessibility_label = self.accessibility_label.or_else(|| self.label.clone());
        let content = self
            .content
            .or_else(|| self.label.map(|label| label.into_any_element()));
        let root = div()
            .flex()
            .flex_none()
            .items_center()
            .h(px(metrics.height))
            .text_size(px(metrics.font_size))
            .text_color(label_color)
            .when_some(theme.resolved_font(cx), |el, family| el.font_family(family))
            .when(self.border, |el| {
                el.border_1()
                    .border_color(if self.disabled {
                        rgb(theme.border_lighter()).into()
                    } else if self.checked {
                        active_color
                    } else {
                        inactive_color
                    })
                    .rounded(px(metrics.radius))
                    .pl(px(metrics.padding_left))
                    .pr(px(metrics.padding_right))
            })
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(px(input))
                    .rounded_full()
                    .border_1()
                    .border_color(input_border)
                    .bg(input_bg)
                    .when(!self.disabled, |el| {
                        el.hover(move |style| style.border_color(active_color))
                    })
                    .when(self.checked, |el| {
                        // `$radio` `::after`: a 4 px dot at half the input.
                        el.child(
                            div()
                                .size(px((input / 2.) - 3.))
                                .rounded_full()
                                .bg(rgb(dot)),
                        )
                    }),
            )
            .when_some(content, |el, content| {
                el.child(div().ml(px(8.)).line_height(px(1.)).child(content))
            });
        let Some(id) = self.id else {
            return root.into_any_element();
        };
        let mut root = root
            .id(id)
            .role(gpui::Role::RadioButton)
            .aria_toggled(if self.checked {
                Toggled::True
            } else {
                Toggled::False
            });
        if let Some(label) = accessibility_label {
            root = root.aria_label(label);
        }
        if self.disabled {
            return root.cursor_not_allowed().into_any_element();
        }
        let checked = self.checked;
        if let Some(handler) = self.onchange {
            root = root
                .cursor_pointer()
                .focusable()
                .tab_stop(true)
                // Upstream outlines the input with the primary color while the
                // hidden input is focused; the native equivalent is a focus
                // ring on the radio itself.
                .focus_visible(move |s| {
                    s.shadow(vec![gpui::BoxShadow {
                        color: active_color,
                        offset: gpui::point(px(0.), px(0.)),
                        blur_radius: px(0.),
                        spread_radius: px(2.),
                        inset: false,
                    }])
                })
                // Selecting the current radio again is a no-op upstream, so
                // the handler only fires for a real change.
                .on_click(move |_, window, cx| {
                    if !checked {
                        handler(true, window, cx);
                    }
                });
        }
        root.into_any_element()
    }
}

/// Creates an empty radio group. Add radios with [`RadioGroup::radio`].
pub fn el_radio_group() -> RadioGroup {
    RadioGroup::default().id(ElementId::CodeLocation(*std::panic::Location::caller()))
}

/// Lays out mutually exclusive radios and propagates shared settings to them.
///
/// Native differences: the group does not report the picked value; each
/// [`Radio`] reports its own activation and the application moves the
/// selection. Upstream's arrow-key navigation between group items is not
/// reproduced; native Tab order applies instead.
#[derive(Default, IntoElement)]
#[must_use]
pub struct RadioGroup {
    id: Option<ElementId>,
    direction: Option<Direction>,
    size: Option<RadioSize>,
    disabled: Option<bool>,
    border: Option<bool>,
    active_color: Option<Hsla>,
    inactive_color: Option<Hsla>,
    /// Upstream spaces siblings with `margin-right: 30px`.
    gap: Option<f32>,
    accessibility_label: Option<SharedString>,
    radios: Vec<Radio>,
}

impl RadioGroup {
    /// Sets the element ID that carries the accessibility group role.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = Some(direction);
        self
    }
    pub fn horizontal(self) -> Self {
        self.direction(Direction::Horizontal)
    }
    pub fn vertical(self) -> Self {
        self.direction(Direction::Vertical)
    }

    /// Sets the size of every radio that does not choose its own.
    pub fn size(mut self, size: RadioSize) -> Self {
        self.size = Some(size);
        self
    }
    pub fn large(self) -> Self {
        self.size(RadioSize::Large)
    }
    pub fn small(self) -> Self {
        self.size(RadioSize::Small)
    }

    /// Disables every radio that does not choose its own.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }

    /// Draws a border around every radio that does not choose its own.
    pub fn border(mut self, border: bool) -> Self {
        self.border = Some(border);
        self
    }

    /// Sets the selected accent for radios that do not choose their own.
    pub fn active_color(mut self, color: impl Into<Hsla>) -> Self {
        self.active_color = Some(color.into());
        self
    }

    /// Sets the unchecked outline for radios that do not choose their own.
    pub fn inactive_color(mut self, color: impl Into<Hsla>) -> Self {
        self.inactive_color = Some(color.into());
        self
    }

    /// Overrides the 30 px gap between siblings.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = Some(gap);
        self
    }

    /// Names the group for assistive technology.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    pub fn radio(mut self, radio: Radio) -> Self {
        self.radios.push(radio);
        self
    }

    /// Applies the group's shared settings to every child that has not chosen
    /// its own. Called while rendering, so settings added after a child keep
    /// winning, like upstream's reactive propagation.
    fn apply_defaults(&mut self) {
        for radio in &mut self.radios {
            if radio.size == RadioSize::Default
                && let Some(size) = self.size
            {
                radio.size = size;
            }
            if !radio.explicit_disabled
                && let Some(disabled) = self.disabled
            {
                radio.disabled = disabled;
            }
            if !radio.explicit_border
                && let Some(border) = self.border
            {
                radio.border = border;
            }
            if radio.active_color.is_none()
                && let Some(active_color) = self.active_color
            {
                radio.active_color = Some(active_color);
            }
            if radio.inactive_color.is_none()
                && let Some(inactive_color) = self.inactive_color
            {
                radio.inactive_color = Some(inactive_color);
            }
        }
    }

    pub fn radios(mut self, radios: impl IntoIterator<Item = Radio>) -> Self {
        self.radios.extend(radios);
        self
    }
}

impl RenderOnce for RadioGroup {
    fn render(mut self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let direction = self.direction.unwrap_or(Direction::Horizontal);
        let gap = self.gap.unwrap_or(30.);
        self.apply_defaults();
        let group = div()
            .flex()
            .flex_none()
            .when(direction == Direction::Vertical, |el| el.flex_col())
            .gap(px(gap))
            .children(self.radios);
        // The accessibility role and label need an element ID, which the
        // `el_radio_group` constructor always supplies.
        match self.id {
            Some(id) => group
                .id(id)
                .role(gpui::Role::RadioGroup)
                .when_some(self.accessibility_label, |el, label| el.aria_label(label))
                .into_any_element(),
            None => group.into_any_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_map_the_theme_chalk_tokens() {
        let large = RadioSize::Large.metrics();
        assert_eq!(
            (large.height, large.font_size, large.bordered_input),
            (40., 14., 14.)
        );
        let default = RadioSize::Default.metrics();
        assert_eq!(
            (default.height, default.font_size, default.bordered_input),
            (32., 14., 12.)
        );
        let small = RadioSize::Small.metrics();
        assert_eq!(
            (small.height, small.font_size, small.bordered_input),
            (24., 12., 12.)
        );
        // The library's desktop radius scale (3/5) instead of Theme Chalk's
        // 2/4; the bordered small size is one step down, like upstream.
        assert_eq!(
            (default.padding_left, default.padding_right, small.radius),
            (10., 16., 4.)
        );
    }

    #[test]
    fn fluent_settings_do_not_reset_other_dimensions() {
        let widget = el_radio()
            .label("Option")
            .checked(true)
            .small()
            .border(true)
            .active_color(rgb(0x13ce66))
            .inactive_color(rgb(0xff4949));
        assert_eq!(widget.label.as_deref(), Some("Option"));
        assert!(widget.checked);
        assert_eq!(widget.size, RadioSize::Small);
        assert!(widget.border);
        assert_eq!(widget.active_color, Some(rgb(0x13ce66).into()));
        assert_eq!(widget.inactive_color, Some(rgb(0xff4949).into()));
        assert!(el_radio().checked(true).small().checked);
    }

    #[test]
    fn group_defaults_do_not_override_explicit_settings() {
        let mut group = el_radio_group()
            .large()
            .disabled(true)
            .border(true)
            .active_color(rgb(0x13ce66))
            .inactive_color(rgb(0xff4949))
            .radio(
                el_radio()
                    .id("a")
                    .small()
                    .disabled(false)
                    .active_color(rgb(0x626aef))
                    .inactive_color(rgb(0x909399)),
            )
            .radio(el_radio().id("b"));
        group.apply_defaults();
        let first = &group.radios[0];
        assert_eq!(first.size, RadioSize::Small);
        assert!(!first.disabled);
        assert_eq!(first.active_color, Some(rgb(0x626aef).into()));
        assert_eq!(first.inactive_color, Some(rgb(0x909399).into()));
        let second = &group.radios[1];
        assert_eq!(second.size, RadioSize::Large);
        assert!(second.disabled);
        assert!(second.border);
        assert_eq!(second.active_color, Some(rgb(0x13ce66).into()));
        assert_eq!(second.inactive_color, Some(rgb(0xff4949).into()));
        // The 30 px sibling gap is the render default, not a stored setting.
    }

    struct Harness {
        picked: std::rc::Rc<std::cell::Cell<usize>>,
        selection: std::rc::Rc<std::cell::Cell<usize>>,
    }

    impl gpui::Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
            let selection = self.selection.get();
            el_radio_group()
                .vertical()
                .radio(radio(
                    0,
                    "First",
                    selection == 0,
                    self.picked.clone(),
                    self.selection.clone(),
                    cx.entity(),
                ))
                .radio(radio(
                    1,
                    "Second",
                    selection == 1,
                    self.picked.clone(),
                    self.selection.clone(),
                    cx.entity(),
                ))
                .radio(
                    radio(
                        2,
                        "Blocked",
                        selection == 2,
                        self.picked.clone(),
                        self.selection.clone(),
                        cx.entity(),
                    )
                    .disabled(true),
                )
        }
    }

    fn radio(
        index: usize,
        label: &'static str,
        checked: bool,
        picked: std::rc::Rc<std::cell::Cell<usize>>,
        selection: std::rc::Rc<std::cell::Cell<usize>>,
        view: gpui::Entity<Harness>,
    ) -> Radio {
        el_radio()
            .id(("radio", index))
            .label(label)
            .checked(checked)
            .onchange(move |_, _, cx| {
                picked.set(index);
                selection.set(index);
                view.update(cx, |_, cx| cx.notify());
            })
    }

    #[gpui::test]
    fn pointer_and_keyboard_select_once_per_activation(cx: &mut gpui::TestAppContext) {
        let picked = std::rc::Rc::new(std::cell::Cell::new(0));
        let selection = std::rc::Rc::new(std::cell::Cell::new(0));
        let (_, view) = cx.add_window_view({
            let picked = picked.clone();
            let selection = selection.clone();
            move |_, cx| {
                cx.set_reduce_motion(true);
                Harness {
                    picked: picked.clone(),
                    selection: selection.clone(),
                }
            }
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        // The vertical group gives every radio its own 32 px row with 30 px
        // gaps: row 1 sits at y 62..94, the disabled third at y 124..156.
        view.simulate_click(gpui::point(px(8.), px(16.)), gpui::Modifiers::default());
        assert_eq!(picked.get(), 0);
        view.simulate_click(gpui::point(px(8.), px(78.)), gpui::Modifiers::default());
        assert_eq!(picked.get(), 1);
        // Re-selecting the current radio is a no-op, like upstream.
        view.simulate_click(gpui::point(px(8.), px(78.)), gpui::Modifiers::default());
        assert_eq!(picked.get(), 1);
        // Disabled radios swallow the activation.
        view.simulate_click(gpui::point(px(8.), px(140.)), gpui::Modifiers::default());
        assert_eq!(picked.get(), 1);
        // Keyboard activation moves the selection to the focused radio.
        press_enter(view);
        assert_eq!(picked.get(), 1);
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
