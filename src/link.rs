//! Element Plus Link.
//!
//! Tokens from Theme Chalk `link.scss` and `common/var.scss`. The underline
//! is reproduced with a reserved 1 px bottom border so hovering never shifts
//! the layout, unlike the upstream out-of-flow `::after` pseudo-element.

use gpui::{
    AnyElement, App, ClickEvent, ElementId, Hsla, IntoElement, RenderOnce, SharedString, Window,
    div, prelude::*, px, rgb,
};

use crate::{
    Theme,
    theme::{DANGER, INFO, PRIMARY, SUCCESS, WARNING, mix},
    tokens,
};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// Element Plus semantic link color. `default` uses the regular text color.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LinkType {
    #[default]
    Default,
    Primary,
    Success,
    Info,
    Warning,
    Danger,
}

/// When the underline appears. The upstream boolean form is deprecated and
/// not reproduced: `true` meant `Hover` and `false` meant `Never`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LinkUnderline {
    Always,
    #[default]
    Hover,
    Never,
}

#[derive(Clone, Copy)]
struct Paint {
    text: u32,
    /// Reserved 1 px bottom border; transparent hides the underline without
    /// changing the outer size. Upstream draws it with the current text color.
    underline: Hsla,
}

fn paint(theme: &Theme, kind: LinkType) -> (Paint, Paint, Paint) {
    let transparent = Hsla::transparent_black();
    let base = match kind {
        LinkType::Default => None,
        LinkType::Primary => Some(PRIMARY),
        LinkType::Success => Some(SUCCESS),
        LinkType::Info => Some(INFO),
        LinkType::Warning => Some(WARNING),
        LinkType::Danger => Some(DANGER),
    };
    match base {
        Some(base) => {
            let disabled = theme.tint(base, 5);
            (
                Paint {
                    text: base,
                    underline: transparent,
                },
                Paint {
                    text: theme.tint(base, 3),
                    underline: transparent,
                },
                Paint {
                    text: disabled,
                    underline: transparent,
                },
            )
        }
        None => {
            // `$link` defaults: regular text, primary hover, placeholder disabled.
            let placeholder = if theme.dark {
                mix(theme.background(), 0xf0f5ff, 55)
            } else {
                0xa8abb2
            };
            (
                Paint {
                    text: theme.regular_text(),
                    underline: transparent,
                },
                Paint {
                    text: PRIMARY,
                    underline: transparent,
                },
                Paint {
                    text: placeholder,
                    underline: transparent,
                },
            )
        }
    }
}

/// Creates a Link with the default Element Plus type and underline behavior.
///
/// A call-site ID is provided by default. Set an explicit [`Link::id`] for
/// repeated links created at the same call site (for example in a loop).
#[track_caller]
pub fn el_link() -> Link {
    Link::default().id(ElementId::CodeLocation(*std::panic::Location::caller()))
}

/// An Element Plus text hyperlink that can be placed directly in the GPUI
/// element tree.
///
/// Native differences: `href` opens the URL through the platform handler
/// instead of navigating the current view, and `target` is not reproduced
/// because a desktop application has no frames to target. The deprecated
/// boolean form of `underline` is not reproduced. The upstream anchor sets
/// `outline: none`; this link is keyboard-focusable and therefore shows a
/// visible focus ring.
#[derive(Default, IntoElement)]
#[must_use]
pub struct Link {
    id: Option<ElementId>,
    label: Option<SharedString>,
    accessibility_label: Option<SharedString>,
    content: Option<AnyElement>,
    icon: Option<AnyElement>,
    trailing_icon: Option<AnyElement>,
    href: Option<SharedString>,
    kind: LinkType,
    underline: LinkUnderline,
    disabled: bool,
    onclick: Option<ClickHandler>,
}

impl Link {
    /// Creates a default link with a stable GPUI element ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self::default().id(id)
    }

    /// Enables pointer and keyboard activation for this ID.
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

    /// Names icon-only and custom-content links for assistive technology.
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

    /// Sets the leading icon, five pixels ahead of the content.
    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    /// Sets the trailing icon, five pixels after the content.
    pub fn trailing_icon(mut self, icon: impl IntoElement) -> Self {
        self.trailing_icon = Some(icon.into_any_element());
        self
    }

    /// Opens this URL through the platform handler on activation. Ignored
    /// while disabled; the application is responsible for validating the URL.
    pub fn href(mut self, href: impl Into<SharedString>) -> Self {
        self.href = Some(href.into());
        self
    }

    pub fn kind(mut self, kind: LinkType) -> Self {
        self.kind = kind;
        self
    }
    pub fn primary(self) -> Self {
        self.kind(LinkType::Primary)
    }
    pub fn success(self) -> Self {
        self.kind(LinkType::Success)
    }
    pub fn info(self) -> Self {
        self.kind(LinkType::Info)
    }
    pub fn warning(self) -> Self {
        self.kind(LinkType::Warning)
    }
    pub fn danger(self) -> Self {
        self.kind(LinkType::Danger)
    }

    /// Sets when the underline appears; the default is on hover.
    pub fn underline(mut self, underline: LinkUnderline) -> Self {
        self.underline = underline;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Handles native pointer and keyboard activation. Fires before the
    /// `href` is opened, and not at all while disabled.
    #[track_caller]
    pub fn onclick(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onclick = Some(Box::new(handler));
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

impl RenderOnce for Link {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let (normal, hover, disabled) = paint(&theme, self.kind);
        // The underline always uses the current text color, and is hidden with
        // a transparent border so hovering never shifts the layout.
        let shown = |paint: Paint, shown: bool| Paint {
            underline: if shown {
                rgb(paint.text).into()
            } else {
                Hsla::transparent_black()
            },
            ..paint
        };
        let always = self.underline == LinkUnderline::Always;
        let never = self.underline == LinkUnderline::Never;
        let normal = shown(normal, always);
        let hover = shown(hover, !never);
        let disabled = shown(disabled, always);
        let accessibility_label = self.accessibility_label.or_else(|| self.label.clone());
        let content = self
            .content
            .or_else(|| self.label.map(|label| label.into_any_element()));
        let has_icon = self.icon.is_some() || self.trailing_icon.is_some();
        let root = div()
            .flex()
            .flex_none()
            .items_center()
            // `utils-inline-flex-center` plus the upstream 5 px icon gap.
            .when(has_icon, |el| el.gap(px(5.)))
            .text_size(px(tokens::font_size::BASE))
            .font_weight(tokens::FONT_WEIGHT_PRIMARY)
            .whitespace_nowrap()
            // Reserve the underline space so hovering never moves the text.
            .border_b_1()
            .border_color(normal.underline)
            .text_color(rgb(normal.text))
            .when_some(theme.resolved_font(cx), |el, family| el.font_family(family))
            .children(self.icon)
            .children(content)
            .children(self.trailing_icon);
        let Some(id) = self.id else {
            return root.into_any_element();
        };
        let mut root = root.id(id);
        if let Some(label) = accessibility_label {
            root = root.aria_label(label);
        }
        if self.disabled {
            return root
                .cursor_not_allowed()
                .text_color(rgb(disabled.text))
                .border_color(disabled.underline)
                .into_any_element();
        }
        let href = self.href;
        let onclick = self.onclick;
        root = root
            .role(gpui::Role::Link)
            .cursor_pointer()
            .focusable()
            .tab_stop(true)
            .hover(move |style| {
                style
                    .text_color(rgb(hover.text))
                    .border_color(hover.underline)
            })
            .focus_visible(move |style| {
                style.shadow(vec![gpui::BoxShadow {
                    color: rgb(PRIMARY).into(),
                    offset: gpui::point(px(0.), px(0.)),
                    blur_radius: px(0.),
                    spread_radius: px(2.),
                    inset: false,
                }])
            })
            .on_click(move |event, window, cx| {
                if let Some(handler) = &onclick {
                    handler(event, window, cx);
                }
                if let Some(href) = href.as_ref() {
                    cx.open_url(href);
                }
            });
        root.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_link_uses_text_tokens_and_primary_hover() {
        let theme = Theme::light();
        let (normal, hover, disabled) = paint(&theme, LinkType::Default);
        assert_eq!(normal.text, 0x606266);
        assert_eq!(hover.text, PRIMARY);
        assert_eq!(disabled.text, 0xa8abb2);
    }

    #[test]
    fn typed_link_derives_states_from_the_accent() {
        let theme = Theme::light();
        let (normal, hover, disabled) = paint(&theme, LinkType::Success);
        assert_eq!(normal.text, SUCCESS);
        assert_eq!(hover.text, theme.tint(SUCCESS, 3));
        assert_eq!(disabled.text, theme.tint(SUCCESS, 5));
    }

    #[test]
    fn dark_theme_mixes_toward_the_dark_background() {
        let theme = Theme::dark();
        let (normal, _, disabled) = paint(&theme, LinkType::Default);
        assert_eq!(normal.text, theme.regular_text());
        assert_eq!(disabled.text, mix(theme.background(), 0xf0f5ff, 55));
        let (normal, _, _) = paint(&theme, LinkType::Danger);
        assert_eq!(normal.text, DANGER);
    }

    #[test]
    fn fluent_settings_do_not_reset_other_dimensions() {
        let link = el_link()
            .label("Details")
            .danger()
            .underline(LinkUnderline::Always)
            .disabled(true);
        assert_eq!(link.kind, LinkType::Danger);
        assert_eq!(link.underline, LinkUnderline::Always);
        assert!(link.disabled);
        assert_eq!(el_link().underline, LinkUnderline::Hover);
    }

    struct Harness {
        clicks: std::rc::Rc<std::cell::Cell<usize>>,
        opened: std::rc::Rc<std::cell::Cell<usize>>,
    }

    impl gpui::Render for Harness {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            let clicks = self.clicks.clone();
            let opened = self.opened.clone();
            // Fixed 28 px rows keep the pointer targets deterministic.
            let row = |link: Link| div().h(px(28.)).child(link);
            let plain_clicks = clicks.clone();
            let opened_href = opened.clone();
            let opened_blocked = opened.clone();
            div()
                .id("link-tests")
                .tab_group()
                .flex()
                .flex_col()
                .size_full()
                .child(row(el_link().id("plain").label("Plain").onclick(
                    move |_, _, _| plain_clicks.set(plain_clicks.get() + 1),
                )))
                .child(row(el_link()
                    .id("with-href")
                    .label("Element Plus")
                    .href("https://element-plus.org")
                    .onclick(move |_, _, _| {
                        opened_href.set(opened_href.get() + 1)
                    })))
                .child(row(el_link()
                    .id("blocked")
                    .label("Blocked")
                    .disabled(true)
                    .href("https://element-plus.org")
                    .onclick(move |_, _, _| {
                        opened_blocked.set(opened_blocked.get() + 1)
                    })))
        }
    }

    #[gpui::test]
    fn click_fires_and_disabled_blocks_both_event_and_url(cx: &mut gpui::TestAppContext) {
        let clicks = std::rc::Rc::new(std::cell::Cell::new(0));
        let opened = std::rc::Rc::new(std::cell::Cell::new(0));
        let (_, view) = cx.add_window_view({
            let clicks = clicks.clone();
            let opened = opened.clone();
            move |_, cx| {
                cx.set_reduce_motion(true);
                Harness { clicks, opened }
            }
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        view.simulate_click(gpui::point(px(12.), px(14.)), gpui::Modifiers::default());
        assert_eq!(clicks.get(), 1);
        // The pointer focused the first link; keyboard activation works too.
        press_enter(view);
        assert_eq!(clicks.get(), 2);
        view.simulate_click(gpui::point(px(12.), px(42.)), gpui::Modifiers::default());
        assert_eq!(opened.get(), 1);
        // The disabled link blocks both the event and the URL.
        view.simulate_click(gpui::point(px(12.), px(70.)), gpui::Modifiers::default());
        assert_eq!(opened.get(), 1);
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
