//! Element Plus Text.
//!
//! Tokens from Theme Chalk `text.scss` and `common/var.scss`. Truncation uses
//! GPUI's own ellipsis and line-clamp layout instead of the CSS properties, so
//! a truncated text still needs a definite width from its container.

use gpui::{
    AnyElement, App, FontWeight, IntoElement, RenderOnce, SharedString, Window, div, prelude::*,
    px, rgb,
};

use crate::{
    Theme,
    theme::{DANGER, INFO, PRIMARY, SUCCESS, WARNING},
    tokens,
};

/// Element Plus semantic text color. `default` uses the regular text color.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextType {
    #[default]
    Default,
    Primary,
    Success,
    Info,
    Warning,
    Danger,
}

/// Element Plus text sizes, mapped to the desktop token scale.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextSize {
    Large,
    #[default]
    Default,
    Small,
}

fn color(theme: &Theme, kind: TextType) -> u32 {
    match kind {
        TextType::Default => theme.regular_text(),
        TextType::Primary => PRIMARY,
        TextType::Success => SUCCESS,
        TextType::Info => INFO,
        TextType::Warning => WARNING,
        TextType::Danger => DANGER,
    }
}

fn size(kind: TextSize) -> f32 {
    match kind {
        TextSize::Large => tokens::font_size::MEDIUM,
        TextSize::Default => tokens::font_size::BASE,
        TextSize::Small => tokens::font_size::SMALL,
    }
}

/// Creates a Text with the default Element Plus type and size.
pub fn el_text() -> Text {
    Text::default()
}

/// Element Plus text in any semantic color and size, with optional ellipsis.
///
/// Native differences: the `tag` property is not reproduced because GPUI has
/// no element tags to override, and `isTruncated` has no ref equivalent: the
/// ellipsis is a layout effect of [`Text::truncated`] under a definite width.
/// Decorations use GPUI's own underline and strikethrough, so they cannot be
/// combined into a single `text-decoration` shorthand.
#[derive(Default, IntoElement)]
#[must_use]
pub struct Text {
    label: Option<SharedString>,
    content: Option<AnyElement>,
    kind: TextType,
    size: TextSize,
    truncated: bool,
    line_clamp: Option<usize>,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
}

impl Text {
    /// Sets the text content, replacing any custom content.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self.content = None;
        self
    }

    /// Replaces the label with custom GPUI content, for example a
    /// [`gpui::StyledText`] with mixed runs.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self.label = None;
        self
    }

    pub fn kind(mut self, kind: TextType) -> Self {
        self.kind = kind;
        self
    }
    pub fn primary(self) -> Self {
        self.kind(TextType::Primary)
    }
    pub fn success(self) -> Self {
        self.kind(TextType::Success)
    }
    pub fn info(self) -> Self {
        self.kind(TextType::Info)
    }
    pub fn warning(self) -> Self {
        self.kind(TextType::Warning)
    }
    pub fn danger(self) -> Self {
        self.kind(TextType::Danger)
    }

    pub fn size(mut self, size: TextSize) -> Self {
        self.size = size;
        self
    }
    pub fn large(self) -> Self {
        self.size(TextSize::Large)
    }
    pub fn small(self) -> Self {
        self.size(TextSize::Small)
    }

    /// Renders an ellipsis when the text exceeds the width available from
    /// its container, like the upstream `truncated` property.
    pub fn truncated(mut self, truncated: bool) -> Self {
        self.truncated = truncated;
        self
    }

    /// Renders an ellipsis after the given number of lines, like the
    /// upstream `line-clamp` property. Zero is treated as unset.
    pub fn line_clamp(mut self, lines: usize) -> Self {
        self.line_clamp = (lines > 0).then_some(lines);
        self
    }

    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }
    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }
    pub fn underline(mut self, underline: bool) -> Self {
        self.underline = underline;
        self
    }
    pub fn strikethrough(mut self, strikethrough: bool) -> Self {
        self.strikethrough = strikethrough;
        self
    }
}

impl RenderOnce for Text {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let content = self
            .content
            .or_else(|| self.label.map(|label| label.into_any_element()));
        div()
            .text_size(px(size(self.size)))
            .text_color(rgb(color(&theme, self.kind)))
            .font_weight(if self.bold {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            })
            .when_some(theme.resolved_font(cx), |el, family| el.font_family(family))
            .when(self.italic, |el| el.italic())
            .when(self.underline, |el| el.underline())
            .when(self.strikethrough, |el| el.line_through())
            // The clamps need a bounded box to measure against, and a flex
            // child would otherwise grow to the intrinsic text width.
            .when(self.truncated, |el| {
                el.text_ellipsis()
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .max_w_full()
                    .min_w_0()
            })
            .when_some(self.line_clamp, |el, lines| {
                el.line_clamp(lines).overflow_hidden().min_h_0()
            })
            .children(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_text_uses_regular_color_and_base_size() {
        let theme = Theme::light();
        assert_eq!(color(&theme, TextType::Default), theme.regular_text());
        assert_eq!(size(TextSize::Default), tokens::font_size::BASE);
        assert_eq!(size(TextSize::Large), tokens::font_size::MEDIUM);
        assert_eq!(size(TextSize::Small), tokens::font_size::SMALL);
    }

    #[test]
    fn typed_text_uses_the_accent_color_in_both_themes() {
        let light = Theme::light();
        let dark = Theme::dark();
        assert_eq!(color(&light, TextType::Primary), PRIMARY);
        assert_eq!(color(&dark, TextType::Danger), DANGER);
        assert_eq!(color(&dark, TextType::Default), dark.regular_text());
    }

    #[test]
    fn fluent_settings_do_not_reset_other_dimensions() {
        let text = el_text()
            .label("Hello")
            .danger()
            .large()
            .truncated(true)
            .line_clamp(2)
            .bold(true)
            .italic(true);
        assert_eq!(text.label.as_deref(), Some("Hello"));
        assert_eq!(text.kind, TextType::Danger);
        assert_eq!(text.size, TextSize::Large);
        assert!(text.truncated);
        assert_eq!(text.line_clamp, Some(2));
        assert!(text.bold);
        assert!(text.italic);
        assert!(!text.underline);
    }

    #[test]
    fn zero_line_clamp_is_unset() {
        assert_eq!(el_text().line_clamp(0).line_clamp, None);
        assert_eq!(el_text().line_clamp(3).line_clamp, Some(3));
    }

    #[test]
    fn custom_content_replaces_the_label() {
        let text = el_text().label("First").content(div().child("Second"));
        assert!(text.label.is_none());
        assert!(text.content.is_some());
    }

    #[gpui::test]
    fn text_renders_with_every_option(cx: &mut gpui::TestAppContext) {
        struct Harness;
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let long = "Element Plus is a Vue 3 component library with native GPUI ports.";
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .w(px(220.))
                    .child(el_text().label("Default"))
                    .child(el_text().primary().large().label("Primary large"))
                    .child(el_text().success().small().label("Success small"))
                    .child(
                        el_text()
                            .warning()
                            .truncated(true)
                            .label(long),
                    )
                    .child(el_text().info().line_clamp(2).label(long))
                    .child(
                        el_text()
                            .danger()
                            .bold(true)
                            .italic(true)
                            .underline(true)
                            .strikethrough(true)
                            .label("Decorated"),
                    )
                    .child(el_text().content(div().child("Custom content")))
            }
        }
        let (_, view) = cx.add_window_view(|_, _| Harness);
        view.update(|window, cx| window.draw(cx).clear(cx));
    }
}
