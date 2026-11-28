//! Element Plus Dialog.
//!
//! Follows `dialog.md`, `components/dialog/src`, and Theme Chalk
//! `dialog.scss`: a controlled modal surface with title/body/footer regions,
//! backdrop and Escape closing, focus entry/restoration, and basic dragging.

use std::{rc::Rc, sync::Arc};

use gpui::{
    AnyElement, App, Bounds, ElementId, FocusHandle, IntoElement, MouseButton, Pixels, Point,
    RenderOnce, SharedString, Window, canvas, deferred, div, point, prelude::*, px, rgb,
};

use crate::{
    Locale, Theme, TransitionEffect, el_button, el_icon, el_transition, theme::popup_priority,
};

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;
type BeforeClose = Rc<dyn Fn(&mut Window, &mut App) -> bool>;

/// An Element Plus dialog.
#[derive(IntoElement)]
#[must_use]
pub struct Dialog {
    id: ElementId,
    open: bool,
    title: Option<SharedString>,
    header: Option<AnyElement>,
    content: AnyElement,
    footer: Option<AnyElement>,
    width: Option<f32>,
    top: Option<f32>,
    fullscreen: bool,
    align_center: bool,
    center: bool,
    modal: bool,
    close_on_click_modal: bool,
    close_on_press_escape: bool,
    show_close: bool,
    draggable: bool,
    overflow: bool,
    accessibility_label: Option<SharedString>,
    before_close: Option<BeforeClose>,
    onopen: Option<Handler>,
    onclose: Option<Handler>,
}

/// Creates a controlled dialog.
#[track_caller]
pub fn el_dialog() -> Dialog {
    Dialog::new()
}

impl Default for Dialog {
    fn default() -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            open: false,
            title: None,
            header: None,
            content: div().into_any_element(),
            footer: None,
            width: None,
            top: None,
            fullscreen: false,
            align_center: false,
            center: false,
            modal: true,
            close_on_click_modal: true,
            close_on_press_escape: true,
            show_close: true,
            draggable: false,
            overflow: false,
            accessibility_label: None,
            before_close: None,
            onopen: None,
            onclose: None,
        }
    }
}

impl Dialog {
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

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(0.));
        self
    }

    pub fn top(mut self, top: f32) -> Self {
        self.top = Some(top.max(0.));
        self
    }

    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    pub fn align_center(mut self, align: bool) -> Self {
        self.align_center = align;
        self
    }

    pub fn center(mut self, center: bool) -> Self {
        self.center = center;
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

    pub fn draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }

    pub fn overflow(mut self, overflow: bool) -> Self {
        self.overflow = overflow;
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

struct DialogState {
    focus: FocusHandle,
    previous_focus: Option<FocusHandle>,
    last_open: bool,
    bounds: Option<Bounds<Pixels>>,
    drag: Option<(Point<Pixels>, Bounds<Pixels>)>,
    offset: Point<Pixels>,
}

impl DialogState {
    fn new(_window: &mut Window, cx: &mut gpui::Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle(),
            previous_focus: None,
            last_open: false,
            bounds: None,
            drag: None,
            offset: point(px(0.), px(0.)),
        }
    }
}

fn request_close(
    state: gpui::Entity<DialogState>,
    before_close: Option<BeforeClose>,
    onclose: Option<Handler>,
    window: &mut Window,
    cx: &mut App,
) {
    let allowed = before_close
        .as_ref()
        .is_none_or(|handler| handler(window, cx));
    if allowed {
        if let Some(handler) = onclose {
            handler(window, cx);
        }
        state.update(cx, |state, _| state.drag = None);
        window.refresh();
    }
}

impl RenderOnce for Dialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, DialogState::new);
        let was_open = state.read(cx).last_open;
        if was_open != self.open {
            state.update(cx, |state, _| state.last_open = self.open);
            if self.open {
                let previous = window.focused(cx);
                let focus = state.read(cx).focus.clone();
                state.update(cx, |state, _| state.previous_focus = previous);
                window.focus(&focus, cx);
                if let Some(handler) = self.onopen.clone() {
                    handler(window, cx);
                }
            } else {
                let previous = state.read(cx).previous_focus.clone();
                state.update(cx, |state, _| {
                    state.drag = None;
                    state.offset = point(px(0.), px(0.));
                });
                if let Some(previous) = previous {
                    window.focus(&previous, cx);
                }
            }
        }
        let theme = Theme::current(cx);
        let locale = Locale::current(cx);
        let viewport = window.viewport_size();
        let width = if self.fullscreen {
            viewport.width
        } else {
            self.width
                .map(px)
                .unwrap_or_else(|| viewport.width * 0.5)
                .min(viewport.width - px(32.))
        };
        let top = self.top.map(px).unwrap_or_else(|| viewport.height * 0.15);
        let mut panel = div()
            .id(ElementId::NamedChild(
                Arc::new(self.id.clone()),
                "panel".into(),
            ))
            .debug_selector(|| "dialog-panel".into())
            .relative()
            .flex()
            .flex_col()
            .w(width)
            .max_h(viewport.height - px(32.))
            .p(px(16.))
            .rounded(if self.fullscreen { px(0.) } else { px(4.) })
            .bg(rgb(theme.background()))
            .shadow(theme.overlay_shadow())
            .text_color(rgb(theme.primary_text()))
            .ml(state.read(cx).offset.x)
            .mt(state.read(cx).offset.y)
            .track_focus(&state.read(cx).focus)
            .tab_stop(true)
            .occlude()
            .on_key_down({
                let state = state.clone();
                let close = self.close_on_press_escape;
                let before_close = self.before_close.clone();
                let onclose = self.onclose.clone();
                move |event, window, cx| {
                    if close && event.keystroke.key == "escape" {
                        request_close(
                            state.clone(),
                            before_close.clone(),
                            onclose.clone(),
                            window,
                            cx,
                        );
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
        if let Some(header) = self.header {
            panel = panel.child(
                div()
                    .pb(px(16.))
                    .when(self.show_close, |el| el.pr(px(48.)))
                    .when(self.center, |el| el.text_center())
                    .child(header),
            );
        } else if let Some(title) = self.title.filter(|title| !title.is_empty()) {
            panel = panel.child(
                div()
                    .pb(px(16.))
                    .when(self.show_close, |el| el.pr(px(48.)))
                    .when(self.center, |el| el.text_center())
                    .text_size(px(18.))
                    .line_height(px(24.))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(title),
            );
        }
        panel = panel.child(
            div()
                .text_size(px(14.))
                .text_color(rgb(theme.regular_text()))
                .child(self.content),
        );
        if let Some(footer) = self.footer {
            panel = panel.child(
                div()
                    .pt(px(16.))
                    .flex()
                    .justify_end()
                    .when(self.center, |el| el.justify_center())
                    .child(footer),
            );
        }
        if self.show_close {
            panel = panel.child(
                div()
                    .debug_selector(|| "dialog-close".into())
                    .absolute()
                    .top(px(0.))
                    .right(px(0.))
                    .w(px(48.))
                    .h(px(48.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        el_button()
                            .id(ElementId::NamedChild(
                                Arc::new(self.id.clone()),
                                "close".into(),
                            ))
                            .text()
                            .small()
                            .icon(el_icon("close").size(16.))
                            .accessibility_label(locale.tag_close())
                            .onclick({
                                let state = state.clone();
                                let before_close = self.before_close.clone();
                                let onclose = self.onclose.clone();
                                move |_, window, cx| {
                                    request_close(
                                        state.clone(),
                                        before_close.clone(),
                                        onclose.clone(),
                                        window,
                                        cx,
                                    );
                                }
                            }),
                    ),
            );
        }
        if self.draggable {
            panel = panel.on_mouse_down(MouseButton::Left, {
                let state = state.clone();
                move |event, _, cx| {
                    if let Some(bounds) = state.read(cx).bounds {
                        state.update(cx, |state, _| {
                            state.drag = Some((event.position, bounds));
                        });
                    }
                }
            });
        }
        let mut overlay = div()
            .id(ElementId::NamedChild(
                Arc::new(self.id.clone()),
                "overlay".into(),
            ))
            .debug_selector(|| "dialog-overlay".into())
            .absolute()
            .inset_0()
            .flex()
            .when(self.align_center, |el| el.items_center().justify_center())
            .when(!self.align_center, |el| {
                el.items_start().justify_center().pt(top)
            })
            .when(self.modal, |el| el.bg(Theme::modal_mask()).occlude())
            .on_mouse_move({
                let state = state.clone();
                let overflow = self.overflow;
                move |event, window, cx| {
                    if event.pressed_button != Some(MouseButton::Left) {
                        return;
                    }
                    let drag = state.read(cx).drag;
                    let Some((origin, bounds)) = drag else {
                        return;
                    };
                    let mut next = gpui::Bounds::new(
                        point(
                            bounds.left() + event.position.x - origin.x,
                            bounds.top() + event.position.y - origin.y,
                        ),
                        bounds.size,
                    );
                    if !overflow {
                        let viewport = window.viewport_size();
                        next.origin.x = next
                            .origin
                            .x
                            .clamp(px(0.), (viewport.width - next.size.width).max(px(0.)));
                        next.origin.y = next
                            .origin
                            .y
                            .clamp(px(0.), (viewport.height - next.size.height).max(px(0.)));
                    }
                    let current = state.read(cx).bounds.unwrap_or(bounds);
                    state.update(cx, |state, _| {
                        state.offset = point(
                            state.offset.x + next.left() - current.left(),
                            state.offset.y + next.top() - current.top(),
                        );
                    });
                    window.refresh();
                }
            })
            .on_mouse_up(MouseButton::Left, {
                let state = state.clone();
                move |_, _, cx| state.update(cx, |state, _| state.drag = None)
            })
            .on_mouse_down(MouseButton::Left, {
                let state = state.clone();
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
                        request_close(
                            state.clone(),
                            before_close.clone(),
                            onclose.clone(),
                            window,
                            cx,
                        );
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
    fn escape_and_backdrop_request_close(cx: &mut gpui::TestAppContext) {
        struct Harness {
            open: bool,
            closes: Rc<std::cell::Cell<usize>>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let closes = self.closes.clone();
                div().size_full().child(
                    el_dialog()
                        .id("dialog")
                        .open(self.open)
                        .title("Confirm")
                        .child("Body")
                        .onclose(move |_, _| closes.set(closes.get() + 1)),
                )
            }
        }
        let closes = Rc::new(std::cell::Cell::new(0));
        let (harness, view) = cx.add_window_view({
            let closes = closes.clone();
            move |_, _| Harness { open: true, closes }
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        view.simulate_keystrokes("escape");
        assert_eq!(closes.get(), 1);
        let overlay = view.debug_bounds("dialog-overlay").expect("dialog overlay");
        view.simulate_click(
            point(overlay.left(), overlay.top()),
            gpui::Modifiers::default(),
        );
        assert_eq!(closes.get(), 2);
        harness.update(cx, |harness, cx| {
            harness.open = false;
            cx.notify();
        });
    }
}
