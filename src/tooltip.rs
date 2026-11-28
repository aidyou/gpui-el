//! Element Plus Tooltip.
//!
//! Follows `tooltip.md`, `components/tooltip/src`, and Theme Chalk
//! `popper.scss`: twelve placements, dark/light effects, hover/click/focus/
//! contextmenu triggers, delayed show/hide, enterable content, and native
//! anchored popups with boundary fallback.

use std::{rc::Rc, sync::Arc, time::Duration};

use gpui::{
    Anchor, AnyElement, App, Bounds, ElementId, Entity, IntoElement, MouseButton, Pixels,
    RenderOnce, SharedString, Size, Window, anchored, canvas, deferred, div, prelude::*, px, rgb,
    size,
};

use crate::{
    Theme, TransitionEffect, el_transition,
    popup::{arrow, resolve_placement},
    theme::popup_priority,
};

pub use crate::popup::PopupPlacement;

/// Tooltip color treatment.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TooltipEffect {
    #[default]
    Dark,
    Light,
}

/// Tooltip activation method.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TooltipTrigger {
    #[default]
    Hover,
    Click,
    Focus,
    ContextMenu,
}

type Handler = Rc<dyn Fn(&mut App)>;

/// An Element Plus tooltip wrapper.
#[derive(IntoElement)]
#[must_use]
pub struct Tooltip {
    id: ElementId,
    child: AnyElement,
    content: AnyElement,
    placement: PopupPlacement,
    fallback_placements: Vec<PopupPlacement>,
    effect: TooltipEffect,
    trigger: TooltipTrigger,
    visible: Option<bool>,
    disabled: bool,
    offset: f32,
    width: Option<f32>,
    padding_x: f32,
    padding_y: f32,
    show_arrow: bool,
    enterable: bool,
    shadow: bool,
    show_after: Duration,
    hide_after: Duration,
    auto_close: Duration,
    accessibility_label: Option<SharedString>,
    onshow: Option<Handler>,
    onhide: Option<Handler>,
}

/// Creates a tooltip and adds its trigger with [`Tooltip::child`].
#[track_caller]
pub fn el_tooltip() -> Tooltip {
    Tooltip::new()
}

impl Default for Tooltip {
    fn default() -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            child: div().into_any_element(),
            content: String::new().into_any_element(),
            placement: PopupPlacement::Bottom,
            fallback_placements: Vec::new(),
            effect: TooltipEffect::Dark,
            trigger: TooltipTrigger::Hover,
            visible: None,
            disabled: false,
            offset: 12.,
            width: None,
            padding_x: 11.,
            padding_y: 5.,
            show_arrow: true,
            enterable: true,
            shadow: false,
            show_after: Duration::ZERO,
            hide_after: Duration::from_millis(200),
            auto_close: Duration::ZERO,
            accessibility_label: None,
            onshow: None,
            onhide: None,
        }
    }
}

impl Tooltip {
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

    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = content.into_any_element();
        self
    }

    pub fn placement(mut self, placement: PopupPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn fallback_placements(
        mut self,
        placements: impl IntoIterator<Item = PopupPlacement>,
    ) -> Self {
        self.fallback_placements = placements.into_iter().collect();
        self
    }

    pub fn effect(mut self, effect: TooltipEffect) -> Self {
        self.effect = effect;
        self
    }

    pub fn trigger(mut self, trigger: TooltipTrigger) -> Self {
        self.trigger = trigger;
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

    pub fn offset(mut self, offset: f32) -> Self {
        self.offset = offset.max(0.);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(0.));
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding_x = padding.max(0.);
        self.padding_y = padding.max(0.);
        self
    }

    pub fn show_arrow(mut self, show: bool) -> Self {
        self.show_arrow = show;
        self
    }

    pub fn enterable(mut self, enterable: bool) -> Self {
        self.enterable = enterable;
        self
    }

    pub(crate) fn shadow(mut self, shadow: bool) -> Self {
        self.shadow = shadow;
        self
    }

    pub fn show_after(mut self, delay: Duration) -> Self {
        self.show_after = delay;
        self
    }

    pub fn hide_after(mut self, delay: Duration) -> Self {
        self.hide_after = delay;
        self
    }

    pub fn auto_close(mut self, duration: Duration) -> Self {
        self.auto_close = duration;
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    pub fn onshow(mut self, handler: impl Fn(&mut App) + 'static) -> Self {
        self.onshow = Some(Rc::new(handler));
        self
    }

    pub fn onhide(mut self, handler: impl Fn(&mut App) + 'static) -> Self {
        self.onhide = Some(Rc::new(handler));
        self
    }
}

#[derive(Default)]
struct TooltipState {
    open: bool,
    target_bounds: Option<Bounds<Pixels>>,
    popup_bounds: Option<Bounds<Pixels>>,
    popup_size: Option<Size<Pixels>>,
    show_task: Option<gpui::Task<()>>,
    hide_task: Option<gpui::Task<()>>,
    auto_task: Option<gpui::Task<()>>,
}

fn apply_open(
    state: Entity<TooltipState>,
    open: bool,
    auto_close: Duration,
    onshow: Option<Handler>,
    onhide: Option<Handler>,
    cx: &mut App,
) {
    let changed = state.read(cx).open != open;
    state.update(cx, |state, _| {
        state.show_task.take();
        state.hide_task.take();
        state.auto_task.take();
        state.open = open;
    });
    if changed {
        if open {
            if let Some(handler) = onshow.clone() {
                handler(cx);
            }
        } else if let Some(handler) = onhide.clone() {
            handler(cx);
        }
    }
    if open && !auto_close.is_zero() {
        let task_state = state.clone();
        let task = cx.spawn(async move |cx| {
            cx.background_executor().timer(auto_close).await;
            task_state.update(cx, |state, cx| {
                state.auto_task.take();
                state.open = false;
                if let Some(handler) = onhide.clone() {
                    handler(cx);
                }
            });
        });
        state.update(cx, |state, _| state.auto_task = Some(task));
    }
}

fn schedule_show(
    state: Entity<TooltipState>,
    delay: Duration,
    auto_close: Duration,
    onshow: Option<Handler>,
    onhide: Option<Handler>,
    cx: &mut App,
) {
    state.update(cx, |state, _| {
        state.show_task.take();
        state.hide_task.take();
    });
    if delay.is_zero() {
        apply_open(state, true, auto_close, onshow, onhide, cx);
        return;
    }
    let task_state = state.clone();
    let task = cx.spawn(async move |cx| {
        cx.background_executor().timer(delay).await;
        task_state.update(cx, |state, cx| {
            state.show_task.take();
            state.open = true;
            if let Some(handler) = onshow.clone() {
                handler(cx);
            }
        });
    });
    state.update(cx, |state, _| state.show_task = Some(task));
}

fn schedule_hide(
    state: Entity<TooltipState>,
    delay: Duration,
    onshow: Option<Handler>,
    onhide: Option<Handler>,
    cx: &mut App,
) {
    state.update(cx, |state, _| {
        state.hide_task.take();
        state.auto_task.take();
    });
    if delay.is_zero() {
        apply_open(state, false, Duration::ZERO, onshow, onhide, cx);
        return;
    }
    let task_state = state.clone();
    let task = cx.spawn(async move |cx| {
        cx.background_executor().timer(delay).await;
        task_state.update(cx, |state, cx| {
            state.hide_task.take();
            state.open = false;
            if let Some(handler) = onhide.clone() {
                handler(cx);
            }
        });
    });
    state.update(cx, |state, _| state.hide_task = Some(task));
}

impl RenderOnce for Tooltip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| TooltipState::default());
        let controlled = self.visible.unwrap_or_else(|| state.read(cx).open);
        if (self.disabled && state.read(cx).open)
            || (self.visible.is_some() && state.read(cx).open != controlled)
        {
            apply_open(
                state.clone(),
                controlled && !self.disabled,
                self.auto_close,
                self.onshow.clone(),
                self.onhide.clone(),
                cx,
            );
        }
        let open = state.read(cx).open && !self.disabled;
        let theme = Theme::current(cx);
        let onhide = self.onhide.clone();
        let measure = state.clone();
        let mut root = div()
            .id(self.id.clone())
            .debug_selector(|| "tooltip-root".into())
            .relative()
            .child(
                canvas(
                    move |bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App| {
                        if measure.read(cx).target_bounds != Some(bounds) {
                            measure.update(cx, |state, _| state.target_bounds = Some(bounds));
                            window.refresh();
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            )
            .child(self.child)
            .on_mouse_down_out({
                let state = state.clone();
                let onhide = onhide.clone();
                move |event, window, cx| {
                    let inside = state
                        .read(cx)
                        .target_bounds
                        .is_some_and(|bounds| bounds.contains(&event.position))
                        || state
                            .read(cx)
                            .popup_bounds
                            .is_some_and(|bounds| bounds.contains(&event.position));
                    if !inside {
                        apply_open(
                            state.clone(),
                            false,
                            Duration::ZERO,
                            None,
                            onhide.clone(),
                            cx,
                        );
                        window.refresh();
                    }
                }
            })
            .on_key_down({
                let state = state.clone();
                let close = self.visible.is_none();
                move |event, window, cx| {
                    if close && event.keystroke.key == "escape" {
                        apply_open(state.clone(), false, Duration::ZERO, None, None, cx);
                        cx.stop_propagation();
                        window.refresh();
                    }
                }
            });
        if !self.disabled {
            root = match self.trigger {
                TooltipTrigger::Hover => root.on_hover({
                    let state = state.clone();
                    let enterable = self.enterable;
                    let show_after = self.show_after;
                    let hide_after = self.hide_after;
                    let auto_close = self.auto_close;
                    let onshow = self.onshow.clone();
                    let onhide = self.onhide.clone();
                    move |hovered, window, cx| {
                        if *hovered {
                            schedule_show(
                                state.clone(),
                                show_after,
                                auto_close,
                                onshow.clone(),
                                onhide.clone(),
                                cx,
                            );
                        } else if enterable {
                            schedule_hide(
                                state.clone(),
                                hide_after,
                                onshow.clone(),
                                onhide.clone(),
                                cx,
                            );
                        } else {
                            apply_open(
                                state.clone(),
                                false,
                                Duration::ZERO,
                                onshow.clone(),
                                onhide.clone(),
                                cx,
                            );
                        }
                        window.refresh();
                    }
                }),
                TooltipTrigger::Click => root.on_mouse_down(MouseButton::Left, {
                    let state = state.clone();
                    let show_after = self.show_after;
                    let auto_close = self.auto_close;
                    let onshow = self.onshow.clone();
                    let onhide = self.onhide.clone();
                    move |_, window, cx| {
                        if state.read(cx).open {
                            apply_open(
                                state.clone(),
                                false,
                                Duration::ZERO,
                                onshow.clone(),
                                onhide.clone(),
                                cx,
                            );
                        } else {
                            schedule_show(
                                state.clone(),
                                show_after,
                                auto_close,
                                onshow.clone(),
                                onhide.clone(),
                                cx,
                            );
                        }
                        window.refresh();
                    }
                }),
                TooltipTrigger::Focus => root.on_key_down({
                    let state = state.clone();
                    let show_after = self.show_after;
                    let auto_close = self.auto_close;
                    let onshow = self.onshow.clone();
                    let onhide = self.onhide.clone();
                    move |event, window, cx| {
                        if matches!(event.keystroke.key.as_ref(), "enter" | "space") {
                            schedule_show(
                                state.clone(),
                                show_after,
                                auto_close,
                                onshow.clone(),
                                onhide.clone(),
                                cx,
                            );
                            window.refresh();
                        }
                    }
                }),
                TooltipTrigger::ContextMenu => root.on_mouse_down(MouseButton::Right, {
                    let state = state.clone();
                    let show_after = self.show_after;
                    let auto_close = self.auto_close;
                    let onshow = self.onshow.clone();
                    let onhide = self.onhide.clone();
                    move |_, window, cx| {
                        schedule_show(
                            state.clone(),
                            show_after,
                            auto_close,
                            onshow.clone(),
                            onhide.clone(),
                            cx,
                        );
                        window.refresh();
                    }
                }),
            };
        }
        if open && let Some(target) = state.read(cx).target_bounds {
            let viewport = window.viewport_size();
            let popup_size = state.read(cx).popup_size;
            let (placement, origin) = resolve_placement(
                self.placement,
                &self.fallback_placements,
                target,
                popup_size,
                viewport,
                self.offset,
            );
            let dark = self.effect == TooltipEffect::Dark;
            let background = if dark {
                theme.primary_text()
            } else {
                theme.overlay_background()
            };
            let foreground = if dark {
                theme.background()
            } else {
                theme.regular_text()
            };
            let border = if dark {
                theme.primary_text()
            } else {
                theme.border_lighter()
            };
            let measured_size = popup_size.unwrap_or_else(|| size(px(150.), px(40.)));
            let mut popup = div()
                .id(ElementId::NamedChild(
                    Arc::new(self.id.clone()),
                    "popup".into(),
                ))
                .debug_selector(|| "tooltip-popup".into())
                .relative()
                .flex()
                .px(px(self.padding_x))
                .py(px(self.padding_y))
                .max_w(viewport.width - px(8.))
                .rounded(px(4.))
                .border_1()
                .border_color(rgb(border))
                .bg(rgb(background))
                .text_color(rgb(foreground))
                .text_size(px(12.))
                .line_height(px(20.))
                .when(self.width.is_some(), |el| {
                    el.w(px(self.width.unwrap_or(0.)))
                })
                .when(self.shadow, |el| el.shadow(theme.overlay_shadow()))
                .occlude()
                .on_key_down({
                    let state = state.clone();
                    move |event, window, cx| {
                        if event.keystroke.key == "escape" {
                            apply_open(state.clone(), false, Duration::ZERO, None, None, cx);
                            cx.stop_propagation();
                            window.refresh();
                        }
                    }
                })
                .on_hover({
                    let state = state.clone();
                    let enterable = self.enterable;
                    let hide_after = self.hide_after;
                    let onshow = self.onshow.clone();
                    let onhide = self.onhide.clone();
                    move |hovered, window, cx| {
                        if *hovered && enterable {
                            state.update(cx, |state, _| {
                                state.hide_task.take();
                                state.auto_task.take();
                            });
                        } else if !*hovered && enterable {
                            schedule_hide(
                                state.clone(),
                                hide_after,
                                onshow.clone(),
                                onhide.clone(),
                                cx,
                            );
                        }
                        window.refresh();
                    }
                })
                .child(self.content);
            if self.show_arrow {
                popup = popup.child(arrow(placement, measured_size, background));
            }
            let measure = state.clone();
            popup = popup.child(
                canvas(
                    move |bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App| {
                        let changed = measure.read(cx).popup_bounds != Some(bounds)
                            || measure.read(cx).popup_size != Some(bounds.size);
                        if changed {
                            measure.update(cx, |state, _| {
                                state.popup_bounds = Some(bounds);
                                state.popup_size = Some(bounds.size);
                            });
                            window.refresh();
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            );
            if let Some(label) = self.accessibility_label {
                popup = popup.aria_label(label);
            }
            root = root.child(
                deferred(
                    anchored()
                        .position(origin)
                        .anchor(Anchor::TopLeft)
                        .snap_to_window_with_margin(px(4.))
                        .child(
                            el_transition(
                                ElementId::NamedChild(
                                    Arc::new(self.id.clone()),
                                    "transition".into(),
                                ),
                                true,
                                TransitionEffect::FadeLinear,
                            )
                            .child(popup),
                        ),
                )
                .with_priority(popup_priority(10)),
            );
        } else {
            state.update(cx, |state, _| {
                state.popup_bounds = None;
                state.popup_size = None;
            });
        }
        root.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[gpui::test]
    fn hover_opens_and_delayed_hide_closes(cx: &mut gpui::TestAppContext) {
        struct Harness {
            shown: Rc<Cell<usize>>,
            hidden: Rc<Cell<usize>>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let shown = self.shown.clone();
                let hidden = self.hidden.clone();
                div().size_full().child(
                    el_tooltip()
                        .id("tooltip")
                        .content("Native tooltip")
                        .child(div().id("trigger").w(px(120.)).h(px(40.)))
                        .onshow(move |_| shown.set(shown.get() + 1))
                        .onhide(move |_| hidden.set(hidden.get() + 1)),
                )
            }
        }
        let shown = Rc::new(Cell::new(0));
        let hidden = Rc::new(Cell::new(0));
        let (_, view) = cx.add_window_view({
            let shown = shown.clone();
            let hidden = hidden.clone();
            move |_, _| Harness { shown, hidden }
        });
        view.simulate_resize(gpui::size(px(800.), px(600.)));
        view.update(|window, cx| window.draw(cx).clear(cx));
        let trigger = view.debug_bounds("tooltip-root").expect("tooltip trigger");
        view.simulate_mouse_move(trigger.center(), None, Default::default());
        view.update(|window, cx| window.draw(cx).clear(cx));
        assert!(view.debug_bounds("tooltip-popup").is_some());
        assert_eq!(shown.get(), 1);
        view.simulate_click(gpui::point(px(700.), px(500.)), gpui::Modifiers::default());
        view.update(|window, cx| window.draw(cx).clear(cx));
        assert!(view.debug_bounds("tooltip-popup").is_none());
        assert_eq!(hidden.get(), 1);
    }

    #[gpui::test]
    fn delayed_hide_uses_the_configured_timer(cx: &mut gpui::TestAppContext) {
        let state = cx.new(|_| TooltipState::default());
        let hidden = Rc::new(Cell::new(0));
        state.update(cx, |state, _| state.open = true);
        let handler = hidden.clone();
        cx.update(|cx| {
            schedule_hide(
                state.clone(),
                Duration::from_millis(50),
                None,
                Some(Rc::new(move |_| handler.set(handler.get() + 1))),
                cx,
            );
        });
        cx.executor().advance_clock(Duration::from_millis(60));
        cx.run_until_parked();
        assert!(!cx.update(|cx| state.read(cx).open));
        assert_eq!(hidden.get(), 1);
    }
}
