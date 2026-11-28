//! Element Plus Drawer.
//!
//! Follows `drawer.md`, `components/drawer/src`, and Theme Chalk
//! `drawer.scss`: a controlled edge-anchored modal surface with four
//! directions, percentage/pixel sizing, header/body/footer regions, backdrop
//! and Escape closing, and focus entry/restoration.

use std::{rc::Rc, sync::Arc};

use gpui::{
    AnyElement, App, Bounds, ElementId, FocusHandle, IntoElement, MouseButton, Pixels, RenderOnce,
    SharedString, Window, canvas, deferred, div, prelude::*, px, rgb,
};

use crate::{
    Locale, Theme, TransitionEffect, el_button, el_icon, el_transition, theme::popup_priority,
};

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;
type BeforeClose = Rc<dyn Fn(&mut Window, &mut App) -> bool>;

/// Drawer opening direction.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DrawerDirection {
    Left,
    #[default]
    Right,
    Top,
    Bottom,
}

/// An Element Plus drawer.
#[derive(IntoElement)]
#[must_use]
pub struct Drawer {
    id: ElementId,
    open: bool,
    title: Option<SharedString>,
    header: Option<AnyElement>,
    content: AnyElement,
    footer: Option<AnyElement>,
    direction: DrawerDirection,
    size: Option<f32>,
    with_header: bool,
    modal: bool,
    close_on_click_modal: bool,
    close_on_press_escape: bool,
    show_close: bool,
    accessibility_label: Option<SharedString>,
    before_close: Option<BeforeClose>,
    onopen: Option<Handler>,
    onclose: Option<Handler>,
}

/// Creates a controlled drawer.
#[track_caller]
pub fn el_drawer() -> Drawer {
    Drawer::new()
}

impl Default for Drawer {
    fn default() -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            open: false,
            title: None,
            header: None,
            content: div().into_any_element(),
            footer: None,
            direction: DrawerDirection::Right,
            size: None,
            with_header: true,
            modal: true,
            close_on_click_modal: true,
            close_on_press_escape: true,
            show_close: true,
            accessibility_label: None,
            before_close: None,
            onopen: None,
            onclose: None,
        }
    }
}

impl Drawer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    pub fn child(mut self, content: impl IntoElement) -> Self {
        self.content = content.into_any_element();
        self
    }

    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    pub fn direction(mut self, direction: DrawerDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Sets the width for left/right drawers or height for top/bottom drawers.
    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size.max(0.));
        self
    }

    pub fn with_header(mut self, with_header: bool) -> Self {
        self.with_header = with_header;
        self
    }

    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    pub fn close_on_click_modal(mut self, close: bool) -> Self {
        self.close_on_click_modal = close;
        self
    }

    pub fn close_on_press_escape(mut self, close: bool) -> Self {
        self.close_on_press_escape = close;
        self
    }

    pub fn show_close(mut self, show: bool) -> Self {
        self.show_close = show;
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    pub fn before_close(
        mut self,
        handler: impl Fn(&mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.before_close = Some(Rc::new(handler));
        self
    }

    pub fn onopen(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.onopen = Some(Rc::new(handler));
        self
    }

    pub fn onclose(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.onclose = Some(Rc::new(handler));
        self
    }
}

struct DrawerState {
    focus: FocusHandle,
    previous_focus: Option<FocusHandle>,
    last_open: bool,
    bounds: Option<Bounds<Pixels>>,
}

impl DrawerState {
    fn new(_window: &mut Window, cx: &mut gpui::Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle(),
            previous_focus: None,
            last_open: false,
            bounds: None,
        }
    }
}

fn request_close(
    before_close: Option<BeforeClose>,
    onclose: Option<Handler>,
    window: &mut Window,
    cx: &mut App,
) {
    if before_close
        .as_ref()
        .is_none_or(|handler| handler(window, cx))
        && let Some(handler) = onclose
    {
        handler(window, cx);
    }
    window.refresh();
}

impl RenderOnce for Drawer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, DrawerState::new);
        if state.read(cx).last_open != self.open {
            state.update(cx, |state, _| state.last_open = self.open);
            if self.open {
                let previous = window.focused(cx);
                let focus = state.read(cx).focus.clone();
                state.update(cx, |state, _| state.previous_focus = previous);
                window.focus(&focus, cx);
                if let Some(handler) = self.onopen.clone() {
                    handler(window, cx);
                }
            } else if let Some(previous) = state.read(cx).previous_focus.clone() {
                window.focus(&previous, cx);
            }
        }
        let theme = Theme::current(cx);
        let locale = Locale::current(cx);
        let viewport = window.viewport_size();
        let horizontal = matches!(
            self.direction,
            DrawerDirection::Left | DrawerDirection::Right
        );
        let size = self.size.map(px).unwrap_or_else(|| {
            if horizontal {
                viewport.width * 0.3
            } else {
                viewport.height * 0.3
            }
        });
        let mut panel = div()
            .id(ElementId::NamedChild(
                Arc::new(self.id.clone()),
                "panel".into(),
            ))
            .debug_selector(|| "drawer-panel".into())
            .absolute()
            .flex()
            .flex_col()
            .bg(rgb(theme.background()))
            .shadow(theme.overlay_shadow())
            .text_color(rgb(theme.primary_text()))
            .track_focus(&state.read(cx).focus)
            .tab_stop(true)
            .occlude()
            .on_key_down({
                let close = self.close_on_press_escape;
                let before_close = self.before_close.clone();
                let onclose = self.onclose.clone();
                move |event, window, cx| {
                    if close && event.keystroke.key == "escape" {
                        request_close(before_close.clone(), onclose.clone(), window, cx);
                        cx.stop_propagation();
                    }
                }
            })
            .child(
                canvas(
                    {
                        let state = state.clone();
                        move |bounds: Bounds<Pixels>, _, cx| {
                            state.update(cx, |state, _| state.bounds = Some(bounds));
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            );
        match self.direction {
            DrawerDirection::Left => {
                panel = panel.left_0().top_0().bottom_0().w(size);
            }
            DrawerDirection::Right => {
                panel = panel.right_0().top_0().bottom_0().w(size);
            }
            DrawerDirection::Top => {
                panel = panel.top_0().left_0().right_0().h(size);
            }
            DrawerDirection::Bottom => {
                panel = panel.bottom_0().left_0().right_0().h(size);
            }
        }
        if self.with_header {
            let mut header = div()
                .mb(px(32.))
                .px(px(20.))
                .pt(px(20.))
                .flex()
                .items_center()
                .gap(px(12.));
            if let Some(custom) = self.header {
                header = header.child(custom);
            } else if let Some(title) = self.title.filter(|title| !title.is_empty()) {
                header = header.child(
                    div()
                        .flex_1()
                        .text_size(px(16.))
                        .line_height(px(24.))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .child(title),
                );
            } else {
                header = header.child(div().flex_1());
            }
            if self.show_close {
                header = header.child(
                    el_button()
                        .id(ElementId::NamedChild(
                            Arc::new(self.id.clone()),
                            "close".into(),
                        ))
                        .text()
                        .small()
                        .icon(el_icon("close").size(20.))
                        .accessibility_label(locale.tag_close())
                        .onclick({
                            let before_close = self.before_close.clone();
                            let onclose = self.onclose.clone();
                            move |_, window, cx| {
                                request_close(before_close.clone(), onclose.clone(), window, cx);
                            }
                        }),
                );
            }
            panel = panel.child(header);
        }
        panel = panel.child(
            div()
                .id(ElementId::NamedChild(
                    Arc::new(self.id.clone()),
                    "body".into(),
                ))
                .flex_1()
                .min_h_0()
                .p(px(20.))
                .overflow_y_scroll()
                .on_scroll(|_, _, cx| cx.stop_propagation())
                .child(self.content),
        );
        if let Some(footer) = self.footer {
            panel = panel.child(
                div()
                    .px(px(20.))
                    .pt(px(10.))
                    .pb(px(20.))
                    .flex()
                    .justify_end()
                    .child(footer),
            );
        }
        let mut overlay = div()
            .id(ElementId::NamedChild(
                Arc::new(self.id.clone()),
                "overlay".into(),
            ))
            .debug_selector(|| "drawer-overlay".into())
            .absolute()
            .inset_0()
            .when(self.modal, |el| el.bg(Theme::modal_mask()).occlude())
            .on_mouse_down(MouseButton::Left, {
                let close = self.modal && self.close_on_click_modal;
                let before_close = self.before_close.clone();
                let onclose = self.onclose.clone();
                move |event, window, cx| {
                    if close
                        && !state
                            .read(cx)
                            .bounds
                            .is_some_and(|bounds| bounds.contains(&event.position))
                    {
                        request_close(before_close.clone(), onclose.clone(), window, cx);
                        cx.stop_propagation();
                    }
                }
            })
            .child(panel);
        if let Some(label) = self.accessibility_label {
            overlay = overlay.aria_label(label);
        }
        deferred(
            el_transition(
                ElementId::NamedChild(Arc::new(self.id.clone()), "transition".into()),
                self.open,
                TransitionEffect::Fade,
            )
            .child(overlay),
        )
        .with_priority(popup_priority(20))
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn right_drawer_is_edge_anchored_and_closes_on_escape(cx: &mut gpui::TestAppContext) {
        struct Harness {
            closes: Rc<std::cell::Cell<usize>>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let closes = self.closes.clone();
                div().size_full().child(
                    el_drawer()
                        .id("drawer")
                        .open(true)
                        .title("Details")
                        .child("Body")
                        .onclose(move |_, _| closes.set(closes.get() + 1)),
                )
            }
        }
        let closes = Rc::new(std::cell::Cell::new(0));
        let (_, view) = cx.add_window_view({
            let closes = closes.clone();
            move |_, _| Harness { closes }
        });
        view.simulate_resize(gpui::size(px(800.), px(600.)));
        view.update(|window, cx| window.draw(cx).clear(cx));
        let panel = view.debug_bounds("drawer-panel").expect("drawer panel");
        assert!((panel.right() - px(800.)).abs() < px(1.));
        assert!((panel.size.width - px(240.)).abs() < px(1.));
        view.simulate_keystrokes("escape");
        assert_eq!(closes.get(), 1);
    }
}
