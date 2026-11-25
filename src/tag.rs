//! Element Plus Tag and CheckTag.
//!
//! Tokens from Theme Chalk `tag.scss`, `check-tag.scss`, and `common/var.scss`.
//! State colors are derived from the same semantic accents as Button; the tag
//! itself has no hover state upstream, only its close button does.

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, IntoElement, RenderOnce, SharedString,
    Window, div, prelude::*, px, rgb,
};

use crate::{
    Locale, Theme,
    icon::Icon,
    theme::{DANGER, INFO, PRIMARY, SUCCESS, WARNING, WHITE},
    tokens,
};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
type ChangeHandler = Box<dyn Fn(bool, &mut Window, &mut App)>;

/// Element Plus semantic tag color. Tag has no neutral variant.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TagType {
    #[default]
    Primary,
    Success,
    Info,
    Warning,
    Danger,
}

/// Tag background theme, independent of type, size and shape.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TagEffect {
    #[default]
    Light,
    Dark,
    Plain,
}

/// Tag outer height and insets. The font size is 12 px at every size.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TagSize {
    Large,
    #[default]
    Default,
    Small,
}

struct Metrics {
    height: f32,
    padding_x: f32,
    /// Close button hit area, from `--el-tag-icon-size`.
    icon_size: f32,
    /// Space between the content and the close button (`$tag-icon-span-gap`).
    close_gap: f32,
    /// Drawn close glyph: `icon-size - 2px`, scaled to 0.8 at the small size.
    close_icon: f32,
}

impl TagSize {
    fn metrics(self) -> Metrics {
        match self {
            Self::Small => Metrics {
                height: 20.,
                padding_x: 7.,
                icon_size: 12.,
                close_gap: 4.,
                close_icon: 8.,
            },
            Self::Default => Metrics {
                height: 24.,
                padding_x: 9.,
                icon_size: 14.,
                close_gap: 6.,
                close_icon: 12.,
            },
            Self::Large => Metrics {
                height: 32.,
                padding_x: 11.,
                icon_size: 16.,
                close_gap: 8.,
                close_icon: 14.,
            },
        }
    }
}

#[derive(Clone, Copy)]
struct Paint {
    text: u32,
    bg: u32,
    border: u32,
    /// Background of the close button on hover; its glyph turns white.
    close_hover_bg: u32,
}

fn accent(kind: TagType) -> u32 {
    match kind {
        TagType::Primary => PRIMARY,
        TagType::Success => SUCCESS,
        TagType::Info => INFO,
        TagType::Warning => WARNING,
        TagType::Danger => DANGER,
    }
}

fn paint(theme: &Theme, kind: TagType, effect: TagEffect) -> Paint {
    let base = accent(kind);
    let tint = |n| theme.tint(base, n);
    match effect {
        TagEffect::Light => Paint {
            text: base,
            bg: tint(9),
            border: tint(8),
            close_hover_bg: base,
        },
        TagEffect::Dark => Paint {
            text: WHITE,
            bg: base,
            border: base,
            close_hover_bg: tint(3),
        },
        TagEffect::Plain => Paint {
            text: base,
            // `--el-fill-color-blank`: white in light theme, the dark background otherwise.
            bg: theme.background(),
            border: tint(5),
            close_hover_bg: base,
        },
    }
}

/// Creates a Tag with the default Element Plus type, effect and size.
///
/// A call-site ID is provided by default. Set an explicit [`Tag::id`] for
/// repeated tags created at the same call site (for example in a loop).
#[track_caller]
pub fn el_tag() -> Tag {
    Tag::default().id(ElementId::CodeLocation(*std::panic::Location::caller()))
}

/// An Element Plus tag that can be placed directly in the GPUI element tree.
///
/// Native differences: the upstream `el-zoom-in-center` mount/unmount
/// transition is not reproduced, so tags appear and disappear immediately;
/// there is therefore no `disable_transitions` property. The `color` property
/// overrides only the background, exactly like the upstream inline style.
#[derive(Default, IntoElement)]
#[must_use]
pub struct Tag {
    id: Option<ElementId>,
    label: Option<SharedString>,
    accessibility_label: Option<SharedString>,
    content: Option<AnyElement>,
    kind: TagType,
    effect: TagEffect,
    size: TagSize,
    color: Option<gpui::Rgba>,
    hit: bool,
    round: bool,
    closable: bool,
    on_click: Option<ClickHandler>,
    onclose: Option<ClickHandler>,
}

impl Tag {
    /// Creates a default tag with a stable GPUI element ID.
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

    /// Names icon-only and custom-content tags for assistive technology.
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

    pub fn kind(mut self, kind: TagType) -> Self {
        self.kind = kind;
        self
    }
    pub fn primary(self) -> Self {
        self.kind(TagType::Primary)
    }
    pub fn success(self) -> Self {
        self.kind(TagType::Success)
    }
    pub fn info(self) -> Self {
        self.kind(TagType::Info)
    }
    pub fn warning(self) -> Self {
        self.kind(TagType::Warning)
    }
    pub fn danger(self) -> Self {
        self.kind(TagType::Danger)
    }
    pub fn effect(mut self, effect: TagEffect) -> Self {
        self.effect = effect;
        self
    }
    /// Selects the dark background theme (`effect="dark"`).
    pub fn dark(self) -> Self {
        self.effect(TagEffect::Dark)
    }
    /// Selects the plain background theme (`effect="plain"`).
    pub fn plain(self) -> Self {
        self.effect(TagEffect::Plain)
    }
    pub fn size(mut self, size: TagSize) -> Self {
        self.size = size;
        self
    }
    pub fn small(self) -> Self {
        self.size(TagSize::Small)
    }
    pub fn large(self) -> Self {
        self.size(TagSize::Large)
    }

    /// Selects the capsule shape; the radius is derived from the outer height.
    pub fn round(mut self, round: bool) -> Self {
        self.round = round;
        self
    }

    /// Overrides only the background with an opaque sRGB color.
    pub fn color(mut self, color: gpui::Rgba) -> Self {
        self.color = Some(color);
        self
    }

    /// Replaces the border with the full-strength type color.
    pub fn hit(mut self, hit: bool) -> Self {
        self.hit = hit;
        self
    }

    /// Shows the close button. Uses the tag ID, or the call site when unset.
    #[track_caller]
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self.ensure_id();
        self
    }

    /// Handles native pointer and keyboard activation of the whole tag.
    #[track_caller]
    pub fn onclick(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self.ensure_id();
        self
    }

    /// Handles activation of the close button. The tag itself is not removed;
    /// the application updates its own state, as with upstream's `close` event.
    #[track_caller]
    pub fn onclose(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onclose = Some(Box::new(handler));
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

impl RenderOnce for Tag {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let metrics = self.size.metrics();
        let palette = paint(&theme, self.kind, self.effect);
        let border = if self.hit {
            accent(self.kind)
        } else {
            palette.border
        };
        let bg = self.color.map_or(palette.bg, |color| {
            let byte = |c: f32| (c.clamp(0., 1.) * 255.).round() as u32;
            (byte(color.r) << 16) | (byte(color.g) << 8) | byte(color.b)
        });
        let radius = if self.round { metrics.height / 2. } else { 4. };
        let accessibility_label = self.accessibility_label.or_else(|| self.label.clone());
        let content = self
            .content
            .or_else(|| self.label.map(|label| label.into_any_element()));
        let id = self.id;
        let root = div()
            .flex()
            .flex_none()
            .items_center()
            .h(px(metrics.height))
            .pl(px(metrics.padding_x))
            .pr(px(if self.closable {
                metrics.close_gap - 1.
            } else {
                metrics.padding_x
            }))
            .when(self.closable, |el| el.gap(px(metrics.close_gap)))
            .text_size(px(12.))
            .line_height(px(12.))
            .font_weight(FontWeight::NORMAL)
            .whitespace_nowrap()
            .border_1()
            .border_color(rgb(border))
            .rounded(px(radius))
            .bg(rgb(bg))
            .text_color(rgb(palette.text))
            .when_some(theme.resolved_font(cx), |el, family| el.font_family(family))
            .children(content)
            .when(self.closable, |el| {
                el.child(close_button(
                    &metrics,
                    palette,
                    id.clone(),
                    Locale::current(cx),
                    self.onclose,
                ))
            });
        let Some(id) = id else {
            return root.into_any_element();
        };
        let mut root = root.id(id);
        if let Some(label) = accessibility_label {
            root = root.aria_label(label);
        }
        if let Some(handler) = self.on_click {
            root = root
                .role(gpui::Role::Button)
                .cursor_pointer()
                .focusable()
                .tab_stop(true)
                .on_click(move |event, window, cx| handler(event, window, cx));
        }
        root.into_any_element()
    }
}

fn close_button(
    metrics: &Metrics,
    palette: Paint,
    id: Option<ElementId>,
    locale: Locale,
    handler: Option<ClickHandler>,
) -> AnyElement {
    let mut button = div()
        .id(match &id {
            Some(id) => (id.clone(), "close").into(),
            None => ElementId::CodeLocation(*std::panic::Location::caller()),
        })
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(px(metrics.icon_size))
        .rounded_full()
        .text_color(rgb(palette.text))
        .child(Icon::named("close").size(metrics.close_icon))
        .hover(move |style| style.text_color(rgb(WHITE)).bg(rgb(palette.close_hover_bg)))
        .focus_visible(move |style| {
            style.shadow(vec![gpui::BoxShadow {
                color: rgb(PRIMARY).into(),
                offset: gpui::point(px(0.), px(0.)),
                blur_radius: px(0.),
                spread_radius: px(2.),
                inset: false,
            }])
        })
        .role(gpui::Role::Button)
        .aria_label(locale.tag_close());
    if let Some(handler) = handler {
        button = button
            .cursor_pointer()
            .focusable()
            .tab_stop(true)
            // Upstream stops the click on the close button so the tag's own
            // click event is not emitted for the same interaction.
            .on_click(move |event, window, cx| {
                cx.stop_propagation();
                handler(event, window, cx);
            });
    }
    button.into_any_element()
}

/// Creates a CheckTag with the default Element Plus type.
///
/// A call-site ID is provided by default. Set an explicit [`CheckTag::id`] for
/// repeated check tags created at the same call site (for example in a loop).
#[track_caller]
pub fn el_check_tag() -> CheckTag {
    CheckTag::default().id(ElementId::CodeLocation(*std::panic::Location::caller()))
}

/// A toggle tag that reports its next checked state, like upstream ElCheckTag.
///
/// Native differences: the tag is focusable and keyboard activatable, which the
/// upstream `span` element is not; the `update:checked` Vue binding is replaced
/// by the application state that feeds [`CheckTag::checked`].
#[derive(Default, IntoElement)]
#[must_use]
pub struct CheckTag {
    id: Option<ElementId>,
    label: Option<SharedString>,
    accessibility_label: Option<SharedString>,
    content: Option<AnyElement>,
    checked: bool,
    disabled: bool,
    kind: TagType,
    onchange: Option<ChangeHandler>,
}

impl CheckTag {
    /// Creates a default check tag with a stable GPUI element ID.
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

    /// Names icon-only and custom-content check tags for assistive technology.
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn kind(mut self, kind: TagType) -> Self {
        self.kind = kind;
        self
    }
    pub fn primary(self) -> Self {
        self.kind(TagType::Primary)
    }
    pub fn success(self) -> Self {
        self.kind(TagType::Success)
    }
    pub fn info(self) -> Self {
        self.kind(TagType::Info)
    }
    pub fn warning(self) -> Self {
        self.kind(TagType::Warning)
    }
    pub fn danger(self) -> Self {
        self.kind(TagType::Danger)
    }

    /// Reports the next checked state on native pointer or keyboard activation.
    /// Not emitted while disabled.
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

impl RenderOnce for CheckTag {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let base = accent(self.kind);
        let tint = |n| theme.tint(base, n);
        // Upstream keys every unchecked state off `info`, except that a
        // disabled unchecked non-primary check tag uses its own type fill.
        let unchecked_bg = if self.disabled && self.kind != TagType::Primary {
            tint(9)
        } else {
            theme.tint(INFO, 9)
        };
        let disabled_text = if theme.dark {
            crate::theme::mix(theme.background(), WHITE, 50)
        } else {
            0xc0c4cc
        };
        let (bg, text) = if self.checked {
            (tint(8), if self.disabled { disabled_text } else { base })
        } else {
            (
                unchecked_bg,
                if self.disabled { disabled_text } else { INFO },
            )
        };
        let hover_bg = if self.disabled {
            bg
        } else if self.checked {
            tint(7)
        } else {
            theme.tint(INFO, 7)
        };
        let accessibility_label = self.accessibility_label.or_else(|| self.label.clone());
        let content = self
            .content
            .or_else(|| self.label.map(|label| label.into_any_element()));
        let root = div()
            .flex()
            .flex_none()
            .items_center()
            .py(px(7.))
            .px(px(15.))
            .rounded(px(tokens::radius::BASE))
            .text_size(px(tokens::font_size::BASE))
            .line_height(px(tokens::font_size::BASE))
            .font_weight(FontWeight::BOLD)
            .whitespace_nowrap()
            .bg(rgb(bg))
            .text_color(rgb(text))
            .when(!self.disabled, |el| {
                el.hover(move |style| style.bg(rgb(hover_bg)))
            })
            .when_some(theme.resolved_font(cx), |el, family| el.font_family(family))
            .children(content);
        let Some(id) = self.id else {
            return root.into_any_element();
        };
        let mut root = root.id(id);
        if let Some(label) = accessibility_label {
            root = root.aria_label(label);
        }
        if self.disabled {
            return root.cursor_not_allowed().into_any_element();
        }
        let checked = self.checked;
        if let Some(handler) = self.onchange {
            root = root
                .role(gpui::Role::Button)
                .cursor_pointer()
                .focusable()
                .tab_stop(true)
                .on_click(move |_, window, cx| handler(!checked, window, cx));
        }
        root.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_effect_state_colors() {
        let theme = Theme::light();
        let palette = paint(&theme, TagType::Primary, TagEffect::Light);
        assert_eq!(palette.text, 0x409eff);
        assert_eq!(palette.bg, 0xecf5ff);
        assert_eq!(palette.border, 0xd9ecff);
        assert_eq!(palette.close_hover_bg, 0x409eff);
    }

    #[test]
    fn dark_and_plain_effect_colors() {
        let theme = Theme::light();
        let dark = paint(&theme, TagType::Success, TagEffect::Dark);
        assert_eq!((dark.text, dark.bg, dark.border), (WHITE, SUCCESS, SUCCESS));
        assert_eq!(dark.close_hover_bg, theme.tint(SUCCESS, 3));
        let plain = paint(&theme, TagType::Warning, TagEffect::Plain);
        assert_eq!((plain.text, plain.bg), (WARNING, WHITE));
        assert_eq!(plain.border, theme.tint(WARNING, 5));
    }

    #[test]
    fn dark_theme_tints_toward_the_dark_background() {
        let theme = Theme::dark();
        let palette = paint(&theme, TagType::Primary, TagEffect::Light);
        assert_eq!(palette.bg, crate::theme::mix(PRIMARY, 0x141414, 90));
        assert_eq!(paint(&theme, TagType::Info, TagEffect::Plain).bg, 0x141414);
    }

    #[test]
    fn sizes_keep_twelve_pixel_text() {
        assert_eq!(TagSize::Small.metrics().height, 20.);
        assert_eq!(TagSize::Default.metrics().height, 24.);
        assert_eq!(TagSize::Large.metrics().height, 32.);
        for size in [TagSize::Small, TagSize::Default, TagSize::Large] {
            let metrics = size.metrics();
            assert_eq!(metrics.padding_x, metrics.close_gap + 3.);
        }
    }

    #[test]
    fn fluent_settings_do_not_reset_other_dimensions() {
        let tag = el_tag()
            .label("Tag")
            .danger()
            .plain()
            .small()
            .round(true)
            .hit(true);
        assert_eq!(tag.kind, TagType::Danger);
        assert_eq!(tag.effect, TagEffect::Plain);
        assert_eq!(tag.size, TagSize::Small);
        assert!(tag.round);
        assert!(tag.hit);
        assert!(!el_tag().closable(true).round);
        let check = el_check_tag().label("Toggle").checked(true).success();
        assert_eq!(check.kind, TagType::Success);
        assert!(check.checked);
    }

    struct Harness {
        closes: std::rc::Rc<std::cell::Cell<usize>>,
        clicks: std::rc::Rc<std::cell::Cell<usize>>,
        checked: std::rc::Rc<std::cell::Cell<bool>>,
    }

    impl gpui::Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
            let closes = self.closes.clone();
            let clicks = self.clicks.clone();
            let checked = self.checked.clone();
            let view = cx.entity();
            let blocked = self.checked.clone();
            // No label: the 14 px close button then sits at x 9..23, y 5..19,
            // so the pointer test can hit it deterministically.
            div()
                .id("tag-tests")
                .tab_group()
                .flex()
                .flex_col()
                .size_full()
                .child(
                    el_tag()
                        .id("removable")
                        .closable(true)
                        .onclick(move |_, _, _| clicks.set(clicks.get() + 1))
                        .onclose(move |_, _, _| closes.set(closes.get() + 1)),
                )
                .child(
                    el_check_tag()
                        .id("toggle")
                        .label("Toggle")
                        .checked(checked.get())
                        .onchange(move |now_checked, _, cx| {
                            checked.set(now_checked);
                            view.update(cx, |_, cx| cx.notify());
                        }),
                )
                .child(
                    el_check_tag()
                        .id("blocked")
                        .label("Blocked")
                        .disabled(true)
                        .onchange(move |checked, _, _| blocked.set(checked)),
                )
        }
    }

    #[gpui::test]
    fn close_and_change_events_fire_once_per_activation(cx: &mut gpui::TestAppContext) {
        let closes = std::rc::Rc::new(std::cell::Cell::new(0));
        let clicks = std::rc::Rc::new(std::cell::Cell::new(0));
        let checked = std::rc::Rc::new(std::cell::Cell::new(false));
        let (_, view) = cx.add_window_view({
            let closes = closes.clone();
            let clicks = clicks.clone();
            let checked = checked.clone();
            move |_, cx| {
                cx.set_reduce_motion(true);
                Harness {
                    closes,
                    clicks,
                    checked,
                }
            }
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        // Rows: 24 px tag, 28 px check tag, 28 px disabled check tag.
        // Pointer activation of the close button does not bubble to the tag's
        // own click event, matching upstream's `@click.stop` on the button.
        view.simulate_click(gpui::point(px(16.), px(12.)), gpui::Modifiers::default());
        assert_eq!(closes.get(), 1);
        assert_eq!(clicks.get(), 0);
        // The pointer focused the close button; keyboard activation works too.
        press_enter(view);
        assert_eq!(closes.get(), 2);
        assert_eq!(clicks.get(), 0);
        // Activating the tag body reports its own click event.
        view.simulate_click(gpui::point(px(2.), px(12.)), gpui::Modifiers::default());
        assert_eq!(clicks.get(), 1);
        // Check tags toggle by pointer and keyboard; disabled ones do not.
        view.simulate_click(gpui::point(px(20.), px(36.)), gpui::Modifiers::default());
        assert!(checked.get());
        view.simulate_click(gpui::point(px(20.), px(68.)), gpui::Modifiers::default());
        assert!(checked.get());
        press_enter(view);
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
