//! Element Plus Popover.
//!
//! Follows `popover.md`, `components/popover/src`, and Theme Chalk
//! `popover.scss`: a light, titled 150 px popup built on the native Tooltip
//! positioning and interaction model.

use std::{rc::Rc, time::Duration};

use gpui::{
    AnyElement, App, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px,
    rgb,
};

use crate::{PopupPlacement, Theme, Tooltip, TooltipEffect, TooltipTrigger};

type Handler = Rc<dyn Fn(&mut App)>;

/// An Element Plus popover.
#[derive(IntoElement)]
#[must_use]
pub struct Popover {
    id: ElementId,
    child: AnyElement,
    title: Option<SharedString>,
    content: AnyElement,
    width: f32,
    placement: PopupPlacement,
    effect: TooltipEffect,
    trigger: TooltipTrigger,
    visible: Option<bool>,
    disabled: bool,
    offset: f32,
    show_arrow: bool,
    enterable: bool,
    show_after: Duration,
    hide_after: Duration,
    auto_close: Duration,
    accessibility_label: Option<SharedString>,
    onshow: Option<Handler>,
    onhide: Option<Handler>,
}

/// Creates a popover and adds its trigger with [`Popover::child`].
#[track_caller]
pub fn el_popover() -> Popover {
    Popover::new()
}

impl Default for Popover {
    fn default() -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            child: div().into_any_element(),
            title: None,
            content: String::new().into_any_element(),
            width: 150.,
            placement: PopupPlacement::Bottom,
            effect: TooltipEffect::Light,
            trigger: TooltipTrigger::Hover,
            visible: None,
            disabled: false,
            offset: 0.,
            show_arrow: true,
            enterable: true,
            show_after: Duration::ZERO,
            hide_after: Duration::from_millis(200),
            auto_close: Duration::ZERO,
            accessibility_label: None,
            onshow: None,
            onhide: None,
        }
    }
}

impl Popover {
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
        self.title = Some(title.into());
        self
    }

    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = content.into_any_element();
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

    pub fn show_arrow(mut self, show: bool) -> Self {
        self.show_arrow = show;
        self
    }

    pub fn enterable(mut self, enterable: bool) -> Self {
        self.enterable = enterable;
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

impl RenderOnce for Popover {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let mut content = div().flex().flex_col();
        if let Some(title) = self.title.filter(|title| !title.is_empty()) {
            content = content.child(
                div()
                    .debug_selector(|| "popover-title".into())
                    .mb(px(12.))
                    .text_size(px(16.))
                    .line_height(px(16.))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(theme.primary_text()))
                    .child(title),
            );
        }
        content = content.child(self.content);
        let mut tooltip = Tooltip::new()
            .id(self.id)
            .child(self.child)
            .content(content)
            .placement(self.placement)
            .effect(self.effect)
            .trigger(self.trigger)
            .disabled(self.disabled)
            .offset(self.offset)
            .width(self.width)
            .padding(12.)
            .show_arrow(self.show_arrow)
            .enterable(self.enterable)
            .shadow(true)
            .show_after(self.show_after)
            .hide_after(self.hide_after)
            .auto_close(self.auto_close);
        if let Some(visible) = self.visible {
            tooltip = tooltip.visible(visible);
        }
        if let Some(label) = self.accessibility_label {
            tooltip = tooltip.accessibility_label(label);
        }
        if let Some(handler) = self.onshow {
            tooltip = tooltip.onshow(move |cx| handler(cx));
        }
        if let Some(handler) = self.onhide {
            tooltip = tooltip.onhide(move |cx| handler(cx));
        }
        tooltip
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn hover_opens_the_titled_popover(cx: &mut gpui::TestAppContext) {
        struct Harness;
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                div().size_full().child(
                    el_popover()
                        .id("popover")
                        .title("Details")
                        .content("Native popover content")
                        .child(div().w(px(120.)).h(px(40.))),
                )
            }
        }
        let (_, view) = cx.add_window_view(|_, _| Harness);
        view.simulate_resize(gpui::size(px(800.), px(600.)));
        view.update(|window, cx| window.draw(cx).clear(cx));
        let trigger = view.debug_bounds("tooltip-root").expect("popover trigger");
        view.simulate_mouse_move(trigger.center(), None, Default::default());
        view.update(|window, cx| window.draw(cx).clear(cx));
        assert!(view.debug_bounds("tooltip-popup").is_some());
        assert!(view.debug_bounds("popover-title").is_some());
    }
}
