//! Element Plus Checkbox and CheckboxGroup.
//!
//! Tokens from Theme Chalk `checkbox.scss`, `check-tag.scss` is unrelated,
//! and `$checkbox-*` in `common/var.scss`. The check mark uses the bundled
//! Element Plus `Check` outline instead of upstream's rotated border trick,
//! and the indeterminate dash is a rounded bar rather than a scaled pseudo
//! element.

use gpui::{
    AnyElement, App, ElementId, IntoElement, RenderOnce, SharedString, Toggled, Window, div,
    prelude::*, px, rgb,
};

use crate::{
    Direction, Theme,
    icon::Icon,
    theme::{Fill, PRIMARY, WHITE},
    tokens,
};

type ChangeHandler = Box<dyn Fn(bool, &mut Window, &mut App)>;

/// Checkbox sizes map to `$checkbox-height` and `$checkbox-font-size`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CheckboxSize {
    Large,
    #[default]
    Default,
    Small,
}

struct Metrics {
    /// `$checkbox-height`, the outer height.
    height: f32,
    /// `$checkbox-font-size`, also the box size without a border.
    font_size: f32,
    /// `$checkbox-bordered-input-*`, the box size with a border.
    bordered_box: f32,
    /// `$checkbox-bordered-padding-left`
    padding_left: f32,
    /// `$checkbox-bordered-padding-right`
    padding_right: f32,
    /// `$button-border-radius` for the bordered variant.
    radius: f32,
}

impl CheckboxSize {
    fn metrics(self) -> Metrics {
        match self {
            Self::Large => Metrics {
                height: 40.,
                font_size: 14.,
                bordered_box: 14.,
                padding_left: 12.,
                padding_right: 20.,
                radius: tokens::radius::BASE,
            },
            Self::Default => Metrics {
                height: 32.,
                font_size: 14.,
                bordered_box: 12.,
                padding_left: 10.,
                padding_right: 16.,
                radius: tokens::radius::BASE,
            },
            Self::Small => Metrics {
                height: 24.,
                font_size: 12.,
                bordered_box: 12.,
                padding_left: 8.,
                padding_right: 12.,
                radius: tokens::radius::BASE - 1.,
            },
        }
    }
}

/// Creates an unchecked checkbox. Pass an explicit ID with
/// [`Checkbox::id`] when several checkboxes are built in a loop.
pub fn el_checkbox() -> Checkbox {
    Checkbox::default().id(ElementId::CodeLocation(*std::panic::Location::caller()))
}

/// A box that toggles a boolean, like upstream `el-checkbox`.
///
/// The application owns the checked state and feeds it back through
/// [`Checkbox::checked`], matching upstream's `v-model`; [`Checkbox::onchange`]
/// reports the next value on pointer or keyboard activation. Native
/// differences: there is no hidden input, so the checked state is conveyed
/// with the accessibility checked attribute and a visible focus ring;
/// `true-value`/`false-value` are the application's concern, since the
/// controlled value is already mapped in `onchange`; and a
/// [`CheckboxGroup`] lays out its children but does not collect their values
/// into an array — each checkbox reports its own next state, and the
/// application keeps the set.
#[derive(Default, IntoElement)]
#[must_use]
pub struct Checkbox {
    id: Option<ElementId>,
    label: Option<SharedString>,
    accessibility_label: Option<SharedString>,
    content: Option<AnyElement>,
    checked: bool,
    indeterminate: bool,
    disabled: bool,
    /// Whether `disabled` was set explicitly, so a group cannot override it.
    explicit_disabled: bool,
    size: CheckboxSize,
    border: bool,
    /// Whether `border` was set explicitly, so a group cannot override it.
    explicit_border: bool,
    onchange: Option<ChangeHandler>,
}

impl Checkbox {
    /// Creates a checkbox with a stable GPUI element ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self::default().id(id)
    }

    /// Enables pointer and keyboard activation for this ID.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the text next to the box, replacing any custom content.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self.content = None;
        self
    }

    /// Names icon-only and custom-content checkboxes for assistive technology.
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

    /// Sets the checked state. The application owns this value and updates it
    /// in [`Self::onchange`], matching upstream's controlled usage.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Shows the partial state of a parent that owns some, not all, children.
    /// Activating an indeterminate checkbox checks it, like upstream.
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self.explicit_disabled = true;
        self
    }

    pub fn size(mut self, size: CheckboxSize) -> Self {
        self.size = size;
        self
    }
    pub fn large(self) -> Self {
        self.size(CheckboxSize::Large)
    }
    pub fn small(self) -> Self {
        self.size(CheckboxSize::Small)
    }

    /// Draws the surrounding border of the `border` attribute.
    pub fn border(mut self, border: bool) -> Self {
        self.border = border;
        self.explicit_border = true;
        self
    }

    /// Reports the next checked state on native pointer or keyboard
    /// activation. Not emitted while disabled.
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

impl RenderOnce for Checkbox {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let metrics = self.size.metrics();
        let box_size = if self.border {
            metrics.bordered_box
        } else {
            metrics.font_size
        };
        // Upstream keeps the box white, the border neutral, and switches both
        // to the primary color for the checked and indeterminate states.
        let border = theme.border(false);
        let (box_bg, box_border, icon) = if self.disabled {
            if self.checked || self.indeterminate {
                (theme.fill(Fill::ExtraLight), border, theme.disabled_text())
            } else {
                (theme.fill(Fill::Light), border, theme.disabled_text())
            }
        } else if self.checked || self.indeterminate {
            (PRIMARY, PRIMARY, WHITE)
        } else {
            (theme.background(), border, WHITE)
        };
        let label_color = if self.disabled {
            theme.disabled_text()
        } else if self.checked {
            PRIMARY
        } else {
            theme.regular_text()
        };
        // The mark: a check for the checked state, a dash for indeterminate.
        let mark = if self.indeterminate {
            div()
                .w(px(box_size * 0.5))
                .h(px(2.))
                .rounded(px(tokens::radius::SMALL / 2.))
                .bg(rgb(icon))
                .into_any_element()
        } else if self.checked {
            Icon::from_pack(crate::IconPack::ElementPlus, "Check")
                .size(box_size - 4.)
                .color(rgb(icon))
                .into_any_element()
        } else {
            div().into_any_element()
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
            .text_color(rgb(label_color))
            .when_some(theme.resolved_font(cx), |el, family| el.font_family(family))
            .when(self.border, |el| {
                el.border_1()
                    .border_color(if self.disabled {
                        rgb(theme.border_lighter())
                    } else if self.checked {
                        rgb(PRIMARY)
                    } else {
                        rgb(border)
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
                    .size(px(box_size))
                    .rounded(px(tokens::radius::SMALL))
                    .border_1()
                    .border_color(rgb(box_border))
                    .bg(rgb(box_bg))
                    .when(!self.disabled, |el| {
                        el.hover(move |style| style.border_color(rgb(PRIMARY)))
                    })
                    .child(mark),
            )
            .when_some(content, |el, content| {
                el.child(div().ml(px(8.)).line_height(px(1.)).child(content))
            });
        let Some(id) = self.id else {
            return root.into_any_element();
        };
        // `Toggled::Mixed` carries the indeterminate state to assistive
        // technology, which GPUI's boolean checked attribute cannot express.
        let mut root = root
            .id(id)
            .role(gpui::Role::CheckBox)
            .aria_toggled(if self.indeterminate {
                Toggled::Mixed
            } else if self.checked {
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
                // Upstream outlines the box with the primary color while the
                // hidden input is focused; the native equivalent is a focus
                // ring on the checkbox itself.
                .focus_visible(move |s| {
                    s.shadow(vec![gpui::BoxShadow {
                        color: rgb(PRIMARY).into(),
                        offset: gpui::point(px(0.), px(0.)),
                        blur_radius: px(0.),
                        spread_radius: px(2.),
                        inset: false,
                    }])
                })
                // An indeterminate box checks on the next activation, and a
                // checked one unchecks; both are simply `!checked`.
                .on_click(move |_, window, cx| handler(!checked, window, cx));
        }
        root.into_any_element()
    }
}

/// Creates an empty checkbox group. Add checkboxes with
/// [`CheckboxGroup::checkbox`].
pub fn el_checkbox_group() -> CheckboxGroup {
    CheckboxGroup::default().id(ElementId::CodeLocation(*std::panic::Location::caller()))
}

/// Lays out related checkboxes and propagates shared settings to them.
///
/// Native differences: the group does not collect the checked values into an
/// array; each [`Checkbox`] reports its own next state through `onchange` and
/// the application keeps the set. Upstream's arrow-key navigation between
/// group items is not reproduced; native Tab order applies instead.
#[derive(Default, IntoElement)]
#[must_use]
pub struct CheckboxGroup {
    id: Option<ElementId>,
    direction: Option<Direction>,
    size: Option<CheckboxSize>,
    disabled: Option<bool>,
    border: Option<bool>,
    /// Upstream spaces siblings with `margin-right: 30px`.
    gap: Option<f32>,
    accessibility_label: Option<SharedString>,
    checkboxes: Vec<Checkbox>,
}

impl CheckboxGroup {
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

    /// Sets the size of every checkbox that does not choose its own.
    pub fn size(mut self, size: CheckboxSize) -> Self {
        self.size = Some(size);
        self
    }
    pub fn large(self) -> Self {
        self.size(CheckboxSize::Large)
    }
    pub fn small(self) -> Self {
        self.size(CheckboxSize::Small)
    }

    /// Disables every checkbox that does not choose its own.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }

    /// Draws a border around every checkbox that does not choose its own.
    pub fn border(mut self, border: bool) -> Self {
        self.border = Some(border);
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

    pub fn checkbox(mut self, checkbox: Checkbox) -> Self {
        self.checkboxes.push(checkbox);
        self
    }

    /// Applies the group's shared settings to every child that has not chosen
    /// its own. Called while rendering, so settings added after a child keep
    /// winning, like upstream's reactive propagation.
    fn apply_defaults(&mut self) {
        for checkbox in &mut self.checkboxes {
            if checkbox.size == CheckboxSize::Default
                && let Some(size) = self.size
            {
                checkbox.size = size;
            }
            if !checkbox.explicit_disabled
                && let Some(disabled) = self.disabled
            {
                checkbox.disabled = disabled;
            }
            if !checkbox.explicit_border
                && let Some(border) = self.border
            {
                checkbox.border = border;
            }
        }
    }

    pub fn checkboxes(mut self, checkboxes: impl IntoIterator<Item = Checkbox>) -> Self {
        self.checkboxes.extend(checkboxes);
        self
    }
}

impl RenderOnce for CheckboxGroup {
    fn render(mut self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let direction = self.direction.unwrap_or(Direction::Horizontal);
        let gap = self.gap.unwrap_or(30.);
        self.apply_defaults();
        let group = div()
            .flex()
            .flex_none()
            .when(direction == Direction::Vertical, |el| el.flex_col())
            .gap(px(gap))
            .children(self.checkboxes);
        // The accessibility role and label need an element ID, which the
        // `el_checkbox_group` constructor always supplies.
        match self.id {
            Some(id) => group
                .id(id)
                .role(gpui::Role::Group)
                .when_some(self.accessibility_label, |el, label| el.aria_label(label))
                .into_any_element(),
            None => group.into_any_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn sizes_map_the_theme_chalk_tokens() {
        let large = CheckboxSize::Large.metrics();
        assert_eq!(
            (large.height, large.font_size, large.bordered_box),
            (40., 14., 14.)
        );
        let default = CheckboxSize::Default.metrics();
        assert_eq!(
            (default.height, default.font_size, default.bordered_box),
            (32., 14., 12.)
        );
        let small = CheckboxSize::Small.metrics();
        assert_eq!(
            (small.height, small.font_size, small.bordered_box),
            (24., 12., 12.)
        );
        // The library's desktop radius scale (3/5) instead of Theme Chalk's
        // 2/4; the bordered small size is one step down, like upstream.
        assert_eq!(
            (
                default.padding_left,
                default.padding_right,
                default.radius,
                small.radius
            ),
            (10., 16., 5., 4.)
        );
    }

    #[test]
    fn fluent_settings_do_not_reset_other_dimensions() {
        let widget = el_checkbox()
            .label("Remember")
            .checked(true)
            .indeterminate(false)
            .small()
            .border(true);
        assert_eq!(widget.label.as_deref(), Some("Remember"));
        assert!(widget.checked);
        assert!(!widget.indeterminate);
        assert_eq!(widget.size, CheckboxSize::Small);
        assert!(widget.border);
        assert!(el_checkbox().checked(true).border(true).checked);
        assert!(el_checkbox().checked(true).small().checked);
    }

    #[test]
    fn group_defaults_do_not_override_explicit_settings() {
        let mut group = el_checkbox_group()
            .small()
            .disabled(true)
            .border(true)
            .checkbox(el_checkbox().id("a").large().disabled(false))
            .checkbox(el_checkbox().id("b"));
        group.apply_defaults();
        let first = &group.checkboxes[0];
        assert_eq!(first.size, CheckboxSize::Large);
        assert!(!first.disabled);
        let second = &group.checkboxes[1];
        assert_eq!(second.size, CheckboxSize::Small);
        assert!(second.disabled);
        assert!(second.border);
        // The 30 px sibling gap is the render default, not a stored setting.
        let vertical = el_checkbox_group().vertical();
        assert_eq!(vertical.direction, Some(Direction::Vertical));
    }

    struct Harness {
        toggles: std::rc::Rc<std::cell::Cell<usize>>,
        checked: std::rc::Rc<std::cell::Cell<bool>>,
        values: std::rc::Rc<std::cell::RefCell<HashSet<&'static str>>>,
    }

    impl gpui::Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
            let toggles = self.toggles.clone();
            let checked = self.checked.clone();
            let values = self.values.clone();
            let view = cx.entity();
            div()
                .id("checkbox-tests")
                .tab_group()
                .flex()
                .flex_col()
                .size_full()
                .child(option(
                    "standalone",
                    "Remember me",
                    self.checked.get(),
                    view.clone(),
                    move |now_checked| {
                        toggles.set(toggles.get() + 1);
                        checked.set(now_checked);
                    },
                ))
                .child(
                    el_checkbox_group()
                        .vertical()
                        .checkbox({
                            let values = values.clone();
                            let view = view.clone();
                            let checked_a = values.borrow().contains("A");
                            option("option-a", "A", checked_a, view, move |now_checked| {
                                let mut values = values.borrow_mut();
                                if now_checked {
                                    values.insert("A");
                                } else {
                                    values.remove("A");
                                }
                            })
                        })
                        .checkbox({
                            let values = values.clone();
                            let view = view.clone();
                            let checked_b = values.borrow().contains("B");
                            option("option-b", "B", checked_b, view, move |now_checked| {
                                let mut values = values.borrow_mut();
                                if now_checked {
                                    values.insert("B");
                                } else {
                                    values.remove("B");
                                }
                            })
                        }),
                )
                .child(
                    el_checkbox()
                        .id("blocked")
                        .label("Blocked")
                        .disabled(true)
                        .onchange(|_, _, _| ()),
                )
        }
    }

    fn option(
        id: &'static str,
        label: &'static str,
        checked: bool,
        view: gpui::Entity<Harness>,
        apply: impl Fn(bool) + 'static,
    ) -> Checkbox {
        el_checkbox()
            .id(id)
            .label(label)
            .checked(checked)
            .onchange(move |now_checked, _, cx| {
                apply(now_checked);
                view.update(cx, |_, cx| cx.notify());
            })
    }

    #[gpui::test]
    fn pointer_and_keyboard_toggle_once_per_activation(cx: &mut gpui::TestAppContext) {
        let toggles = std::rc::Rc::new(std::cell::Cell::new(0));
        let checked = std::rc::Rc::new(std::cell::Cell::new(false));
        let values = std::rc::Rc::new(std::cell::RefCell::new(HashSet::from(["A"])));
        let (_, view) = cx.add_window_view({
            let toggles = toggles.clone();
            let checked = checked.clone();
            let values = values.clone();
            move |_, cx| {
                cx.set_reduce_motion(true);
                Harness {
                    toggles,
                    checked,
                    values,
                }
            }
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        // Rows: 32 px standalone, then the vertical group (A, B), then the
        // disabled checkbox.
        view.simulate_click(gpui::point(px(8.), px(16.)), gpui::Modifiers::default());
        assert_eq!(toggles.get(), 1);
        assert!(checked.get());
        press_enter(view);
        assert_eq!(toggles.get(), 2);
        assert!(!checked.get());
        // A starts checked and unchecks, then checks again.
        view.simulate_click(gpui::point(px(8.), px(48.)), gpui::Modifiers::default());
        assert!(!values.borrow().contains("A"));
        view.simulate_click(gpui::point(px(8.), px(48.)), gpui::Modifiers::default());
        assert!(values.borrow().contains("A"));
        // The disabled checkbox sits below the group (30 px gaps).
        view.simulate_click(gpui::point(px(8.), px(142.)), gpui::Modifiers::default());
        assert_eq!(toggles.get(), 2);
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
