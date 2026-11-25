mod style;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, Hsla, IntoElement, RenderOnce, SharedString,
    StyleRefinement, Window, div, prelude::*, px, rgb,
};

use crate::{Direction, Locale, Theme, loading::inline_spinner, theme::Mask, tokens};
pub use style::{ButtonShape, ButtonSize, ButtonType, ButtonVariant};
use style::{Paint, Palette};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// Creates a Button with the default Element Plus type, size and shape.
///
/// A call-site ID is provided by default. Set an explicit [`Button::id`] for
/// repeated buttons created at the same call site (for example in a loop).
#[track_caller]
pub fn el_button() -> Button {
    Button::default().id(ElementId::CodeLocation(*std::panic::Location::caller()))
}

/// An Element Plus button that can be placed directly in the GPUI element tree.
///
/// HTML attributes such as `tag` and `native-type` have no native equivalent.
/// Connect app actions explicitly with [`Self::onclick`].
#[derive(Default, IntoElement)]
#[must_use]
pub struct Button {
    id: Option<ElementId>,
    label: Option<SharedString>,
    accessibility_label: Option<SharedString>,
    content: Option<AnyElement>,
    icon: Option<AnyElement>,
    loading_icon: Option<AnyElement>,
    kind: ButtonType,
    variant: ButtonVariant,
    shape: ButtonShape,
    size: ButtonSize,
    font_size: Option<f32>,
    color: Option<gpui::Rgba>,
    auto_insert_space: bool,
    disabled: bool,
    loading: bool,
    tooltip: Option<SharedString>,
    on_click: Option<ClickHandler>,
    group: Option<(Direction, bool, bool)>,
    explicit_size: bool,
    explicit_kind: bool,
}

/// Creates a connected group of buttons with shared size and semantic type.
pub fn el_button_group() -> ButtonGroup {
    ButtonGroup::default()
}

#[derive(Default, IntoElement)]
pub struct ButtonGroup {
    direction: Option<Direction>,
    size: Option<ButtonSize>,
    kind: Option<ButtonType>,
    buttons: Vec<Button>,
}

impl ButtonGroup {
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = Some(direction);
        self
    }
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = Some(size);
        self
    }
    pub fn kind(mut self, kind: ButtonType) -> Self {
        self.kind = Some(kind);
        self
    }
    pub fn button(mut self, button: Button) -> Self {
        self.buttons.push(button);
        self
    }
    pub fn buttons(mut self, buttons: impl IntoIterator<Item = Button>) -> Self {
        self.buttons.extend(buttons);
        self
    }
}

impl RenderOnce for ButtonGroup {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let direction = self.direction.unwrap_or(Direction::Horizontal);
        let count = self.buttons.len();
        div()
            .flex()
            .when(direction == Direction::Vertical, |el| {
                el.flex_col().items_stretch()
            })
            .children(self.buttons.into_iter().enumerate().map(|(i, mut button)| {
                if !button.explicit_kind
                    && let Some(kind) = self.kind
                {
                    button.kind = kind;
                }
                if !button.explicit_size
                    && let Some(size) = self.size
                {
                    button.size = size;
                }
                if count > 1 {
                    button.group = Some((direction, i == 0, i + 1 == count));
                }
                button
            }))
    }
}

impl Button {
    /// Creates a default button with a stable GPUI element ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self::default().id(id)
    }

    /// Enables persistent hover, keyboard focus and click tracking for this ID.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the text content, replacing any custom content.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self.content = None;
        self
    }

    /// Names icon-only and custom-content buttons for assistive technology.
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

    /// Sets a leading icon. Set its size to the current text size if it does not inherit it.
    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    /// Replaces the default loading indicator.
    pub fn loading_icon(mut self, icon: impl IntoElement) -> Self {
        self.loading_icon = Some(icon.into_any_element());
        self
    }

    pub fn kind(mut self, kind: ButtonType) -> Self {
        self.kind = kind;
        self.explicit_kind = true;
        self
    }
    pub fn primary(self) -> Self {
        self.kind(ButtonType::Primary)
    }
    pub fn success(self) -> Self {
        self.kind(ButtonType::Success)
    }
    pub fn warning(self) -> Self {
        self.kind(ButtonType::Warning)
    }
    pub fn danger(self) -> Self {
        self.kind(ButtonType::Danger)
    }
    pub fn info(self) -> Self {
        self.kind(ButtonType::Info)
    }
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn plain(self) -> Self {
        self.variant(ButtonVariant::Plain)
    }
    pub fn dashed(self) -> Self {
        self.variant(ButtonVariant::Dashed)
    }
    pub fn text(self) -> Self {
        self.variant(ButtonVariant::Text)
    }
    pub fn text_with_background(self) -> Self {
        self.variant(ButtonVariant::TextWithBackground)
    }
    pub fn link(self) -> Self {
        self.variant(ButtonVariant::Link)
    }
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self.explicit_size = true;
        self
    }
    pub fn small(self) -> Self {
        self.size(ButtonSize::Small)
    }
    pub fn large(self) -> Self {
        self.size(ButtonSize::Large)
    }
    pub fn shape(mut self, shape: ButtonShape) -> Self {
        self.shape = shape;
        self
    }
    pub fn round(self) -> Self {
        self.shape(ButtonShape::Round)
    }
    pub fn circle(self) -> Self {
        self.shape(ButtonShape::Circle)
    }

    /// Overrides only the text size, in logical pixels. Non-positive and non-finite
    /// values clear the override. Does not change the height or insets.
    pub fn font_size(mut self, logical_px: f32) -> Self {
        self.font_size = (logical_px.is_finite() && logical_px > 0.).then_some(logical_px);
        self
    }

    /// Overrides the semantic accent with an opaque sRGB color.
    pub fn color(mut self, color: gpui::Rgba) -> Self {
        self.color = Some(color);
        self
    }

    /// Adds spacing to labels made up of exactly two Han characters.
    pub fn auto_insert_space(mut self, enabled: bool) -> Self {
        self.auto_insert_space = enabled;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Shows a native hover hint. Use a stable element ID.
    pub fn tooltip(mut self, text: impl Into<SharedString>) -> Self {
        self.tooltip = Some(text.into());
        self
    }

    /// Handles native pointer and keyboard activation. Use a stable ID for
    /// buttons created in a loop or other repeated call site.
    pub fn onclick(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    fn resolved_font_size(&self) -> f32 {
        self.font_size
            .unwrap_or_else(|| self.size.metrics(self.shape).font_size)
    }
}

fn paint(style: StyleRefinement, colors: Paint) -> StyleRefinement {
    style
        .text_color(rgb(colors.text))
        .bg(colors
            .bg
            .map_or(Hsla::transparent_black(), |c| rgb(c).into()))
        .border_color(
            colors
                .border
                .map_or(Hsla::transparent_black(), |c| rgb(c).into()),
        )
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let metrics = self.size.metrics(self.shape);
        let font_size = self.resolved_font_size();
        let palette = self.color.map_or_else(
            || Palette::resolve(&theme, self.kind, self.variant),
            |color| Palette::custom(&theme, self.variant, color),
        );
        let disabled = self.disabled;
        let loading = self.loading;
        let blocked = disabled || loading;
        let link = self.variant == ButtonVariant::Link;
        let padded = !matches!(
            self.variant,
            ButtonVariant::Text | ButtonVariant::TextWithBackground
        );
        let icon = if loading {
            Some(
                self.loading_icon
                    .unwrap_or_else(|| inline_spinner(font_size)),
            )
        } else {
            self.icon
        };
        let accessibility_label = self.accessibility_label.or_else(|| self.label.clone());
        let content = self.content.or_else(|| {
            self.label.map(|label| {
                if self.auto_insert_space {
                    let mut chars = label.chars();
                    let is_han = |ch| matches!(ch, '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}' | '\u{f900}'..='\u{faff}' | '\u{20000}'..='\u{2ffff}' | '\u{30000}'..='\u{323af}');
                    if let (Some(a), Some(b), None) = (chars.next(), chars.next(), chars.next())
                        && is_han(a) && is_han(b)
                    {
                        return format!("{a} {b}").into_any_element();
                    }
                }
                label.into_any_element()
            })
        });
        let circle = self.shape == ButtonShape::Circle && !link;
        let radius = px(metrics.radius);
        let group_shape = |el: gpui::Div| match self.group {
            Some((Direction::Horizontal, first, last)) => el
                .rounded_tl(if first { radius } else { px(0.) })
                .rounded_bl(if first { radius } else { px(0.) })
                .rounded_tr(if last { radius } else { px(0.) })
                .rounded_br(if last { radius } else { px(0.) }),
            Some((Direction::Vertical, first, last)) => el
                .rounded_tl(if first { radius } else { px(0.) })
                .rounded_tr(if first { radius } else { px(0.) })
                .rounded_bl(if last { radius } else { px(0.) })
                .rounded_br(if last { radius } else { px(0.) }),
            None => el.rounded(radius),
        };
        let root = div()
            .relative()
            .flex()
            .flex_none()
            .when(
                !matches!(self.group, Some((Direction::Vertical, ..))) || circle,
                |el| el.self_start(),
            )
            .when(
                matches!(self.group, Some((Direction::Horizontal, false, _))),
                |el| el.ml(px(-1.)),
            )
            .when(
                matches!(self.group, Some((Direction::Vertical, false, _))),
                |el| el.mt(px(-1.)),
            )
            .items_center()
            .justify_center()
            .gap(px(metrics.icon_gap))
            .map(group_shape)
            .text_size(px(font_size))
            .line_height(px(font_size))
            .font_weight(tokens::FONT_WEIGHT_PRIMARY)
            .whitespace_nowrap()
            .px(px(if link {
                2.
            } else if circle {
                0.
            } else {
                metrics.padding_x
            }))
            .when(link, |el| el.py(px(2.)))
            .when(!link, |el| el.h(px(metrics.height)))
            .when(circle, |el| el.w(px(metrics.height)))
            .when(padded, |el| el.border_1())
            .when(self.variant == ButtonVariant::Dashed, |el| {
                el.border_dashed()
            })
            .when_some(theme.resolved_font(cx), |el, family| el.font_family(family))
            .children(icon)
            .children(content)
            .when(loading, |el| {
                el.child(
                    div()
                        .absolute()
                        .inset_0()
                        .rounded(px(metrics.radius))
                        .bg(theme.mask(Mask::ExtraLight)),
                )
            });
        let base = if disabled {
            palette.disabled
        } else {
            palette.normal
        };
        let mut root = root;
        *root.style() = paint(root.style().clone(), base);
        // Dark Theme Chalk overrides only the neutral button's disabled foreground.
        let root = root.when(
            disabled && theme.dark && self.kind == ButtonType::Default,
            |el| el.text_color(Hsla::from(rgb(crate::theme::WHITE)).opacity(0.5)),
        );
        let id = self.id;
        let Some(id) = id else {
            return root
                .when(disabled, |el| el.cursor_not_allowed())
                .into_any_element();
        };
        let root = root
            .id(id)
            .role(gpui::Role::Button)
            .when_some(accessibility_label, |el, label| el.aria_label(label))
            .when(loading, |el| {
                el.aria_description(Locale::current(cx).button_loading())
            });
        if blocked {
            return root
                .when(disabled, |el| el.cursor_not_allowed())
                .into_any_element();
        }
        let outline = palette.outline;
        let mut root = root
            .cursor_pointer()
            .hover(move |s| paint(s, palette.hover))
            .focusable()
            .tab_stop(true)
            .active(move |s| paint(s, palette.active))
            .focus_visible(move |s| {
                s.shadow(vec![gpui::BoxShadow {
                    color: rgb(outline).into(),
                    offset: gpui::point(px(0.), px(0.)),
                    blur_radius: px(0.),
                    spread_radius: px(2.),
                    inset: false,
                }])
            });
        if let Some(text) = self.tooltip {
            let dark = theme.dark;
            root = root.tooltip(move |_, cx| cx.new(|_| ButtonHint(text.clone(), dark)).into());
        }
        if let Some(handler) = self.on_click {
            root = root.on_click(move |event, window, cx| {
                if event.standard_click() {
                    handler(event, window, cx);
                }
            });
        }
        root.into_any_element()
    }
}

struct ButtonHint(SharedString, bool);

impl gpui::Render for ButtonHint {
    fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
        div()
            .px(px(10.))
            .py(px(5.))
            .rounded(px(crate::tokens::radius::BASE))
            .text_size(px(12.))
            .bg(rgb(if self.1 { 0xe5eaf3 } else { 0x303133 }))
            .text_color(rgb(if self.1 { 0x141414 } else { 0xffffff }))
            .child(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fluent_settings_do_not_reset_other_dimensions() {
        let button = el_button()
            .font_size(13.)
            .round()
            .large()
            .small()
            .success()
            .plain();
        assert_eq!(button.size, ButtonSize::Small);
        assert_eq!(button.shape, ButtonShape::Round);
        assert_eq!(button.kind, ButtonType::Success);
        assert_eq!(button.variant, ButtonVariant::Plain);
        assert_eq!(button.resolved_font_size(), 13.);
        assert_eq!(el_button().small().resolved_font_size(), 12.);
        assert_eq!(button.font_size(f32::NAN).resolved_font_size(), 12.);
    }

    struct Harness(std::rc::Rc<std::cell::Cell<usize>>);

    impl gpui::Render for Harness {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            let button = |id| {
                let count = self.0.clone();
                Button::new(id)
                    .label("Activate")
                    .onclick(move |_, _, _| count.set(count.get() + 1))
            };
            div()
                .id("button-tests")
                .tab_group()
                .flex()
                .flex_col()
                .size_full()
                .child(button("enabled"))
                .child(button("disabled").disabled(true))
                .child(button("loading").loading(true))
        }
    }

    #[gpui::test]
    fn pointer_and_keyboard_activation_skip_blocked_buttons(cx: &mut gpui::TestAppContext) {
        let count = std::rc::Rc::new(std::cell::Cell::new(0));
        let (_, view) = cx.add_window_view({
            let count = count.clone();
            move |_, cx| {
                cx.set_reduce_motion(true);
                Harness(count)
            }
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        for y in [48., 80., 16.] {
            view.simulate_click(gpui::point(px(20.), px(y)), gpui::Modifiers::default());
        }
        assert_eq!(count.get(), 1);
        view.update(|window, cx| {
            let focused = window.focused(cx).expect("button focused by pointer");
            window.focus_next(cx);
            assert_eq!(window.focused(cx), Some(focused));
            window.draw(cx).clear(cx);
        });
        for key in ["enter", "space"] {
            let keystroke = view.update(|_, cx| {
                gpui::Keystroke::parse_with_os(key, cx.platform_info().os).unwrap()
            });
            view.simulate_event(gpui::KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            });
            view.simulate_event(gpui::KeyUpEvent { keystroke });
        }
        assert_eq!(count.get(), 3);
    }
}
