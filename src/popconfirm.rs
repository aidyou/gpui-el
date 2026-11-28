//! Element Plus Popconfirm.
//!
//! Follows `popconfirm.md`, `components/popconfirm/src`, and Theme Chalk
//! `popconfirm.scss`: a click-triggered light popover with a warning icon,
//! title, cancel/confirm actions, and automatic close after either action.

use std::{rc::Rc, sync::Arc, time::Duration};

use gpui::{
    AnyElement, App, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px,
    rgb,
};

use crate::{
    Button, ButtonType, Locale, PopupPlacement, Tooltip, TooltipEffect, TooltipTrigger, el_button,
    el_icon,
};

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// An Element Plus popconfirm.
#[derive(IntoElement)]
#[must_use]
pub struct Popconfirm {
    id: ElementId,
    child: AnyElement,
    title: SharedString,
    confirm_text: Option<SharedString>,
    cancel_text: Option<SharedString>,
    confirm_type: ButtonType,
    cancel_type: ButtonType,
    icon_color: u32,
    hide_icon: bool,
    width: f32,
    placement: PopupPlacement,
    effect: TooltipEffect,
    visible: Option<bool>,
    disabled: bool,
    hide_after: Duration,
    accessibility_label: Option<SharedString>,
    onconfirm: Option<Handler>,
    oncancel: Option<Handler>,
}

/// Creates a popconfirm and adds its trigger with [`Popconfirm::child`].
#[track_caller]
pub fn el_popconfirm() -> Popconfirm {
    Popconfirm::new()
}

impl Default for Popconfirm {
    fn default() -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            child: div().into_any_element(),
            title: String::new().into(),
            confirm_text: None,
            cancel_text: None,
            confirm_type: ButtonType::Primary,
            cancel_type: ButtonType::Default,
            icon_color: 0xff9900,
            hide_icon: false,
            width: 150.,
            placement: PopupPlacement::Bottom,
            effect: TooltipEffect::Light,
            visible: None,
            disabled: false,
            hide_after: Duration::from_millis(200),
            accessibility_label: None,
            onconfirm: None,
            oncancel: None,
        }
    }
}

impl Popconfirm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = child.into_any_element();
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    pub fn confirm_text(mut self, text: impl Into<SharedString>) -> Self {
        self.confirm_text = Some(text.into());
        self
    }

    pub fn cancel_text(mut self, text: impl Into<SharedString>) -> Self {
        self.cancel_text = Some(text.into());
        self
    }

    pub fn confirm_type(mut self, kind: ButtonType) -> Self {
        self.confirm_type = kind;
        self
    }

    pub fn cancel_type(mut self, kind: ButtonType) -> Self {
        self.cancel_type = kind;
        self
    }

    pub fn icon_color(mut self, color: u32) -> Self {
        self.icon_color = color;
        self
    }

    pub fn hide_icon(mut self, hide: bool) -> Self {
        self.hide_icon = hide;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(150.);
        self
    }

    pub fn placement(mut self, placement: PopupPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn effect(mut self, effect: TooltipEffect) -> Self {
        self.effect = effect;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = Some(visible);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn hide_after(mut self, delay: Duration) -> Self {
        self.hide_after = delay;
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    pub fn onconfirm(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.onconfirm = Some(Rc::new(handler));
        self
    }

    pub fn oncancel(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.oncancel = Some(Rc::new(handler));
        self
    }
}

#[derive(Default)]
struct PopconfirmState {
    open: bool,
}

fn button_with_type(mut button: Button, kind: ButtonType) -> Button {
    button = match kind {
        ButtonType::Default => button.text(),
        ButtonType::Primary => button.primary(),
        ButtonType::Success => button.success(),
        ButtonType::Warning => button.warning(),
        ButtonType::Danger => button.danger(),
        ButtonType::Info => button.info(),
    };
    button.small()
}

impl RenderOnce for Popconfirm {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| PopconfirmState::default());
        let open = self.visible.unwrap_or_else(|| state.read(cx).open);
        if self.visible.is_some() && state.read(cx).open != open {
            state.update(cx, |state, _| state.open = open);
        }
        let locale = Locale::current(cx);
        let confirm_text = self
            .confirm_text
            .clone()
            .unwrap_or_else(|| locale.popconfirm_confirm().into());
        let cancel_text = self
            .cancel_text
            .clone()
            .unwrap_or_else(|| locale.popconfirm_cancel().into());
        let mut main = div().flex().items_center().gap(px(5.));
        if !self.hide_icon {
            main = main.child(
                el_icon("question-filled")
                    .size(16.)
                    .color(rgb(self.icon_color)),
            );
        }
        main = main.child(self.title);
        let cancel_handler = self.oncancel.clone();
        let confirm_handler = self.onconfirm.clone();
        let content = div()
            .debug_selector(|| "popconfirm-content".into())
            .flex()
            .flex_col()
            .child(main)
            .child(
                div()
                    .mt(px(8.))
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .child(
                        div().debug_selector(|| "popconfirm-cancel".into()).child(
                            button_with_type(
                                el_button()
                                    .id(ElementId::NamedChild(
                                        Arc::new(self.id.clone()),
                                        "cancel".into(),
                                    ))
                                    .label(cancel_text),
                                self.cancel_type,
                            )
                            .onclick({
                                let state = state.clone();
                                move |_, window, cx| {
                                    state.update(cx, |state, _| state.open = false);
                                    if let Some(handler) = cancel_handler.clone() {
                                        handler(window, cx);
                                    }
                                    window.refresh();
                                }
                            }),
                        ),
                    )
                    .child(
                        div().debug_selector(|| "popconfirm-confirm".into()).child(
                            button_with_type(
                                el_button()
                                    .id(ElementId::NamedChild(
                                        Arc::new(self.id.clone()),
                                        "confirm".into(),
                                    ))
                                    .label(confirm_text),
                                self.confirm_type,
                            )
                            .onclick({
                                let state = state.clone();
                                move |_, window, cx| {
                                    state.update(cx, |state, _| state.open = false);
                                    if let Some(handler) = confirm_handler.clone() {
                                        handler(window, cx);
                                    }
                                    window.refresh();
                                }
                            }),
                        ),
                    ),
            );
        let mut tooltip = Tooltip::new()
            .id(self.id)
            .child(self.child)
            .content(content)
            .placement(self.placement)
            .effect(self.effect)
            .trigger(TooltipTrigger::Click)
            .visible(open)
            .disabled(self.disabled)
            .offset(0.)
            .width(self.width)
            .padding(12.)
            .shadow(true)
            .hide_after(self.hide_after)
            .onshow({
                let state = state.clone();
                move |app| {
                    state.update(app, |state, cx| {
                        state.open = true;
                        cx.notify();
                    });
                    app.refresh_windows();
                }
            })
            .onhide({
                let state = state.clone();
                move |app| {
                    state.update(app, |state, cx| {
                        state.open = false;
                        cx.notify();
                    });
                    app.refresh_windows();
                }
            });
        if let Some(label) = self.accessibility_label {
            tooltip = tooltip.accessibility_label(label);
        }
        tooltip
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[gpui::test]
    fn click_opens_and_confirm_or_cancel_closes_once(cx: &mut gpui::TestAppContext) {
        struct Harness {
            confirmed: Rc<Cell<usize>>,
            cancelled: Rc<Cell<usize>>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let confirmed = self.confirmed.clone();
                let cancelled = self.cancelled.clone();
                div().size_full().child(
                    el_popconfirm()
                        .id("popconfirm")
                        .title("Delete this item?")
                        .child(el_button().id("trigger").label("Delete"))
                        .onconfirm(move |_, _| confirmed.set(confirmed.get() + 1))
                        .oncancel(move |_, _| cancelled.set(cancelled.get() + 1)),
                )
            }
        }
        let confirmed = Rc::new(Cell::new(0));
        let cancelled = Rc::new(Cell::new(0));
        let (_, view) = cx.add_window_view({
            let confirmed = confirmed.clone();
            let cancelled = cancelled.clone();
            move |_, _| Harness {
                confirmed,
                cancelled,
            }
        });
        view.simulate_resize(gpui::size(px(800.), px(600.)));
        view.update(|window, cx| window.draw(cx).clear(cx));
        let trigger = view
            .debug_bounds("tooltip-root")
            .expect("popconfirm trigger");
        view.simulate_click(trigger.center(), gpui::Modifiers::default());
        view.update(|window, cx| window.draw(cx).clear(cx));
        assert!(view.debug_bounds("tooltip-popup").is_some());
        let confirm = view
            .debug_bounds("popconfirm-confirm")
            .expect("confirm action");
        view.simulate_click(confirm.center(), gpui::Modifiers::default());
        view.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(confirmed.get(), 1);
        assert!(view.debug_bounds("tooltip-popup").is_none());

        view.simulate_click(trigger.center(), gpui::Modifiers::default());
        view.update(|window, cx| window.draw(cx).clear(cx));
        let cancel = view
            .debug_bounds("popconfirm-cancel")
            .expect("cancel action");
        view.simulate_click(cancel.center(), gpui::Modifiers::default());
        view.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(cancelled.get(), 1);
        assert!(view.debug_bounds("tooltip-popup").is_none());
    }
}
