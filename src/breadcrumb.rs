//! Element Plus Breadcrumb.
//!
//! Follows `breadcrumb.md`, `breadcrumb.vue`, `breadcrumb-item.vue`, and
//! Theme Chalk `breadcrumb.scss` / `breadcrumb-item.scss`: 14 px inline
//! navigation, bold primary links, a muted separator, and a regular current
//! page item.

use std::{rc::Rc, sync::Arc};

use gpui::{
    App, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px, rgb,
};

use crate::{Icon, Locale, Theme, tokens};

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// One breadcrumb level.
#[derive(Clone)]
pub struct BreadcrumbItem {
    label: SharedString,
    href: Option<SharedString>,
    onclick: Option<Handler>,
}

impl BreadcrumbItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            href: None,
            onclick: None,
        }
    }

    pub fn href(mut self, href: impl Into<SharedString>) -> Self {
        self.href = Some(href.into());
        self
    }

    pub fn onclick(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.onclick = Some(Rc::new(handler));
        self
    }
}

/// An Element Plus breadcrumb trail.
#[derive(IntoElement)]
#[must_use]
pub struct Breadcrumb {
    id: ElementId,
    items: Vec<BreadcrumbItem>,
    separator: SharedString,
    separator_icon: Option<Icon>,
    accessibility_label: Option<SharedString>,
}

/// Creates an empty breadcrumb and adds levels with [`Breadcrumb::item`].
#[track_caller]
pub fn el_breadcrumb() -> Breadcrumb {
    Breadcrumb::new()
}

impl Default for Breadcrumb {
    fn default() -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            items: Vec::new(),
            separator: "/".into(),
            separator_icon: None,
            accessibility_label: None,
        }
    }
}

impl Breadcrumb {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn item(mut self, item: BreadcrumbItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn separator(mut self, separator: impl Into<SharedString>) -> Self {
        self.separator = separator.into();
        self
    }

    pub fn separator_icon(mut self, icon: Icon) -> Self {
        self.separator_icon = Some(icon);
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }
}

impl RenderOnce for Breadcrumb {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let locale = Locale::current(cx);
        let label = self
            .accessibility_label
            .unwrap_or_else(|| locale.breadcrumb_label().into());
        let count = self.items.len();
        let separator_icon = self.separator_icon.clone();
        let mut root = div()
            .id(self.id.clone())
            .debug_selector(|| "breadcrumb".into())
            .flex()
            .items_center()
            .text_size(px(14.))
            .line_height(px(14.))
            .text_color(rgb(theme.regular_text()))
            .role(gpui::Role::Navigation)
            .aria_label(label);
        for (index, item) in self.items.into_iter().enumerate() {
            let last = index + 1 == count;
            let clickable = item.href.is_some() || item.onclick.is_some();
            let href = item.href;
            let onclick = item.onclick;
            let mut inner = div()
                .id(ElementId::NamedChild(
                    Arc::new(self.id.clone()),
                    format!("item-{index}").into(),
                ))
                .debug_selector(|| "breadcrumb-item".into())
                .flex()
                .items_center()
                .child(
                    div()
                        .id(ElementId::NamedChild(
                            Arc::new(self.id.clone()),
                            format!("label-{index}").into(),
                        ))
                        .debug_selector(move || format!("breadcrumb-label-{index}"))
                        .when(!last, |el| el.font_weight(tokens::FONT_WEIGHT_PRIMARY))
                        .text_color(rgb(if last {
                            theme.regular_text()
                        } else {
                            theme.primary_text()
                        }))
                        .when(clickable, |el| {
                            el.role(gpui::Role::Link)
                                .cursor_pointer()
                                .focusable()
                                .tab_stop(true)
                                .hover(|style| style.text_color(rgb(crate::theme::PRIMARY)))
                                .on_click(move |_, window, cx| {
                                    if let Some(handler) = onclick.clone() {
                                        handler(window, cx);
                                    }
                                    if let Some(href) = href.as_ref() {
                                        cx.open_url(href);
                                    }
                                    cx.stop_propagation();
                                })
                        })
                        .child(item.label),
                );
            if !last {
                inner = inner.child(if let Some(icon) = separator_icon.clone() {
                    div()
                        .mx(px(6.))
                        .text_color(rgb(theme.disabled_text()))
                        .child(icon.size(14.))
                        .into_any_element()
                } else {
                    div()
                        .mx(px(9.))
                        .font_weight(tokens::FONT_WEIGHT_PRIMARY)
                        .text_color(rgb(theme.disabled_text()))
                        .child(self.separator.clone())
                        .into_any_element()
                });
            }
            root = root.child(inner.aria_selected(last));
        }
        root.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[gpui::test]
    fn breadcrumb_click_reports_once_and_keeps_current_item_regular(cx: &mut gpui::TestAppContext) {
        struct Harness {
            clicks: Rc<Cell<usize>>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let clicks = self.clicks.clone();
                div().p(px(20.)).child(
                    el_breadcrumb()
                        .id("breadcrumb")
                        .item(
                            BreadcrumbItem::new("Home")
                                .onclick(move |_, _| clicks.set(clicks.get() + 1)),
                        )
                        .item(BreadcrumbItem::new("Projects"))
                        .item(BreadcrumbItem::new("Current")),
                )
            }
        }
        let clicks = Rc::new(Cell::new(0));
        let (_, view) = cx.add_window_view({
            let clicks = clicks.clone();
            move |_, _| Harness { clicks }
        });
        view.simulate_resize(gpui::size(px(800.), px(600.)));
        view.update(|window, cx| window.draw(cx).clear(cx));
        let item = view
            .debug_bounds("breadcrumb-label-0")
            .expect("breadcrumb label");
        view.simulate_click(item.center(), gpui::Modifiers::default());
        assert_eq!(clicks.get(), 1);
    }
}
