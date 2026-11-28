//! Element Plus Alert.
//!
//! Follows `alert.md`, `components/alert/src`, and Theme Chalk `alert.scss`:
//! semantic types, light/dark effects, optional icon and description, centered
//! text, and a dismiss action.

use std::{rc::Rc, sync::Arc};

use gpui::{
    AnyElement, App, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px,
    rgb,
};

use crate::{FeedbackEffect, FeedbackType, Locale, Theme, el_button, el_icon};

type CloseHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// An Element Plus alert.
#[derive(IntoElement)]
#[must_use]
pub struct Alert {
    id: ElementId,
    title: SharedString,
    description: Option<SharedString>,
    content: Option<AnyElement>,
    kind: FeedbackType,
    effect: FeedbackEffect,
    closable: bool,
    close_text: Option<SharedString>,
    show_icon: bool,
    center: bool,
    accessibility_label: Option<SharedString>,
    onclose: Option<CloseHandler>,
}

/// Creates an alert.
#[track_caller]
pub fn el_alert() -> Alert {
    Alert::new()
}

impl Default for Alert {
    fn default() -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            title: String::new().into(),
            description: None,
            content: None,
            kind: FeedbackType::Info,
            effect: FeedbackEffect::Light,
            closable: true,
            close_text: None,
            show_icon: false,
            center: false,
            accessibility_label: None,
            onclose: None,
        }
    }
}

impl Alert {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Replaces the text description with arbitrary native content.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self.description = None;
        self
    }

    pub fn kind(mut self, kind: FeedbackType) -> Self {
        self.kind = kind;
        self
    }

    pub fn effect(mut self, effect: FeedbackEffect) -> Self {
        self.effect = effect;
        self
    }

    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    pub fn close_text(mut self, text: impl Into<SharedString>) -> Self {
        self.close_text = Some(text.into());
        self
    }

    pub fn show_icon(mut self, show: bool) -> Self {
        self.show_icon = show;
        self
    }

    pub fn center(mut self, center: bool) -> Self {
        self.center = center;
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    pub fn onclose(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.onclose = Some(Rc::new(handler));
        self
    }
}

#[derive(Clone, Copy, Default)]
struct AlertState {
    visible: bool,
}

impl RenderOnce for Alert {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state =
            window.use_keyed_state(self.id.clone(), cx, |_, _| AlertState { visible: true });
        if !state.read(cx).visible {
            return div().into_any_element();
        }
        let theme = Theme::current(cx);
        let locale = Locale::current(cx);
        let color = self.kind.color();
        let dark = self.effect == FeedbackEffect::Dark;
        let foreground = if dark { 0xffffff } else { color };
        let background = if dark {
            color
        } else {
            self.kind.tint(&theme, 9)
        };
        let has_description = self.description.is_some() || self.content.is_some();
        let mut content = div().flex().flex_col().gap(px(4.)).min_w_0();
        if !self.title.is_empty() {
            content = content.child(
                div()
                    .text_size(px(if has_description { 16. } else { 14. }))
                    .line_height(px(24.))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(self.title.clone()),
            );
        }
        if let Some(description) = self.description {
            content = content.child(
                div()
                    .text_size(px(14.))
                    .text_color(rgb(foreground))
                    .child(description),
            );
        } else if let Some(custom) = self.content {
            content = content.child(custom);
        }
        let mut root = div()
            .id(self.id.clone())
            .debug_selector(|| "alert".into())
            .w_full()
            .relative()
            .flex()
            .items_center()
            .px(px(16.))
            .py(px(8.))
            .rounded(px(4.))
            .bg(rgb(background))
            .text_color(rgb(foreground))
            .when(self.center, |el| el.justify_center())
            .when_some(self.accessibility_label.clone(), |el, label| {
                el.aria_label(label)
            });
        if self.show_icon {
            root = root.child(
                div()
                    .mr(if has_description { px(12.) } else { px(8.) })
                    .child(el_icon(self.kind.icon()).size(if has_description { 28. } else { 16. })),
            );
        }
        root = root.child(content);
        if self.closable {
            let state = state.clone();
            let handler = self.onclose.clone();
            let close_text = self.close_text.clone();
            let close_label = close_text
                .clone()
                .unwrap_or_else(|| locale.tag_close().into());
            let mut close = el_button()
                .id(ElementId::NamedChild(
                    Arc::new(self.id.clone()),
                    "close".into(),
                ))
                .text()
                .small()
                .label(close_text.unwrap_or_default())
                .accessibility_label(close_label)
                .onclick(move |_, window, cx| {
                    state.update(cx, |state, _| state.visible = false);
                    if let Some(handler) = handler.clone() {
                        handler(window, cx);
                    }
                    window.refresh();
                });
            if self.close_text.is_none() {
                close = close.icon(el_icon("close").size(16.));
            }
            root = root.child(
                div()
                    .debug_selector(|| "alert-close".into())
                    .absolute()
                    .top(px(if self.close_text.is_some() { 8. } else { 12. }))
                    .right(px(16.))
                    .child(close),
            );
        }
        root.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feedback_types_cover_upstream_semantics() {
        assert_eq!(FeedbackType::default(), FeedbackType::Info);
        assert_eq!(FeedbackType::Success.icon(), "success-filled");
        assert_eq!(FeedbackType::Error.icon(), "circle-close-filled");
        for kind in [
            FeedbackType::Primary,
            FeedbackType::Info,
            FeedbackType::Success,
            FeedbackType::Warning,
            FeedbackType::Error,
        ] {
            assert!(
                crate::IconPack::ElementPlus
                    .names()
                    .any(|name| name == kind.icon())
            );
        }
    }

    #[gpui::test]
    fn close_hides_the_alert_and_reports_once(cx: &mut gpui::TestAppContext) {
        struct Harness {
            closed: Rc<std::cell::Cell<usize>>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let closed = self.closed.clone();
                div().child(
                    el_alert()
                        .id("alert")
                        .title("Saved")
                        .onclose(move |_, _| closed.set(closed.get() + 1)),
                )
            }
        }
        let closed = Rc::new(std::cell::Cell::new(0));
        let (_, view) = cx.add_window_view({
            let closed = closed.clone();
            move |_, _| Harness { closed }
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        let close = view
            .debug_bounds("alert-close")
            .expect("alert close button");
        view.simulate_click(close.center(), gpui::Modifiers::default());
        view.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(closed.get(), 1);
        assert!(view.debug_bounds("alert").is_none());
    }
}
