//! A compact single-selection capsule for forms and dense popovers.
//!
//! The geometry follows the ChatSpeed automation form's execution-frequency
//! control: a gray, hairline-bordered well with edge-to-edge options, 2 px
//! gaps, capsule corners, wrapping, and a 30 px default height. Colors are
//! resolved through this crate's Element Plus theme rather than ChatSpeed's
//! application palette.

use std::{rc::Rc, sync::Arc};

use gpui::{
    App, ElementId, FocusHandle, IntoElement, RenderOnce, SharedString, Window, div, prelude::*,
    px, rgb,
};

use crate::Theme;

/// One selectable capsule option.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapsuleOption {
    value: SharedString,
    label: SharedString,
}

impl CapsuleOption {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

type SelectHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

#[derive(Default)]
struct CapsuleState {
    focuses: Vec<FocusHandle>,
}

/// A connected, wrapping single-selection capsule.
#[derive(IntoElement)]
#[must_use]
pub struct CapsuleSegments {
    id: ElementId,
    options: Vec<CapsuleOption>,
    selected: Option<SharedString>,
    height: f32,
    font_size: Option<f32>,
    disabled: bool,
    accessibility_label: Option<SharedString>,
    onselect: Option<SelectHandler>,
}

/// Creates a capsule control.
#[track_caller]
pub fn el_capsule_segments() -> CapsuleSegments {
    CapsuleSegments::new()
}

impl Default for CapsuleSegments {
    fn default() -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            options: Vec::new(),
            selected: None,
            height: 30.,
            font_size: None,
            disabled: false,
            accessibility_label: None,
            onselect: None,
        }
    }
}

impl CapsuleSegments {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = CapsuleOption>) -> Self {
        self.options = options.into_iter().collect();
        self
    }

    /// Marks the controlled selection. Unknown or absent values select nothing.
    pub fn selected(mut self, value: impl Into<SharedString>) -> Self {
        self.selected = Some(value.into());
        self
    }

    pub fn clear_selection(mut self) -> Self {
        self.selected = None;
        self
    }

    /// Sets the shared option and track height. Invalid values restore 30 px.
    pub fn height(mut self, height: f32) -> Self {
        self.height = if height.is_finite() && height > 0. {
            height
        } else {
            30.
        };
        self
    }

    /// Overrides the label size; otherwise it follows the height, clamped to 10–13 px.
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = (size.is_finite() && size > 0.).then_some(size);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Reports every activated option, including an already selected one.
    #[track_caller]
    pub fn onselect(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onselect = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for CapsuleSegments {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let CapsuleSegments {
            id,
            options,
            selected,
            height,
            font_size,
            disabled,
            accessibility_label,
            onselect,
        } = self;
        let state = window.use_keyed_state(id.clone(), cx, |_, _| CapsuleState {
            focuses: Vec::new(),
        });
        state.update(cx, |state, cx| {
            while state.focuses.len() < options.len() {
                state.focuses.push(cx.focus_handle());
            }
            state.focuses.truncate(options.len());
        });
        let theme = Theme::current(cx);
        let font_size = font_size.unwrap_or_else(|| (height * 0.44).clamp(10., 13.));
        let track_border = rgb(theme.border(false));
        let track_bg = rgb(theme.fill(crate::theme::Fill::Base));
        let hover_bg = rgb(theme.fill(crate::theme::Fill::Light));
        let primary = rgb(crate::theme::PRIMARY);
        let regular = rgb(theme.regular_text());
        let disabled_color = rgb(theme.disabled_text());
        let mut track = div()
            .id(id.clone())
            .min_w_0()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(2.))
            .rounded(px(height / 2.))
            .border_1()
            .border_color(track_border)
            .bg(track_bg);
        if let Some(label) = accessibility_label {
            track = track.aria_label(label);
        }
        for (index, option) in options.into_iter().enumerate() {
            let value = option.value.clone();
            let is_selected = selected.as_ref() == Some(&option.value);
            let focused = state
                .read(cx)
                .focuses
                .get(index)
                .is_some_and(|focus| focus.is_focused(window));
            let Some(focus) = state.read(cx).focuses.get(index).cloned() else {
                continue;
            };
            let mut item = div()
                .id(ElementId::NamedChild(
                    Arc::new(id.clone()),
                    option.value.clone(),
                ))
                .flex()
                .items_center()
                .justify_center()
                .h(px(height))
                .px(px(8.))
                .pt(px(0.))
                .pb(px(2.))
                .line_height(px(font_size + 4.))
                .rounded(px(height / 2.))
                .border_1()
                .border_color(gpui::transparent_black())
                .text_color(if disabled {
                    disabled_color
                } else if is_selected {
                    rgb(0xffffff)
                } else {
                    regular
                })
                .track_focus(&focus)
                .tab_stop(!disabled)
                .aria_selected(is_selected)
                .aria_label(option.label.clone())
                .child(div().text_size(px(font_size)).child(option.label))
                .when(disabled, |el| el.cursor_not_allowed())
                .when(!disabled, |el| {
                    el.cursor_pointer().when(!is_selected, |el| {
                        el.hover(|style| style.bg(hover_bg).text_color(primary))
                    })
                })
                .when(is_selected && !disabled, |el| el.bg(primary))
                .when(focused && !disabled, |el| el.border_color(primary));
            if !disabled && let Some(handler) = onselect.clone() {
                let click_focus = focus.clone();
                item = item.on_click(move |_, window, cx| {
                    click_focus.focus(window, cx);
                    cx.stop_propagation();
                    handler(value.clone(), window, cx);
                });
            }
            track = track.child(item);
        }
        track.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_setters_are_independent_and_invalid_sizes_fall_back() {
        let capsule = el_capsule_segments()
            .height(24.)
            .font_size(11.)
            .selected("interval")
            .disabled(true);
        assert_eq!(capsule.height, 24.);
        assert_eq!(capsule.font_size, Some(11.));
        assert_eq!(capsule.selected.as_deref(), Some("interval"));
        assert!(capsule.disabled);
        let capsule = capsule.height(f32::NAN).font_size(0.);
        assert_eq!(capsule.height, 30.);
        assert_eq!(capsule.font_size, None);
    }

    #[gpui::test]
    fn pointer_and_keyboard_activation_report_values(cx: &mut gpui::TestAppContext) {
        use std::cell::RefCell;
        struct Harness {
            selected: Rc<RefCell<Option<SharedString>>>,
            events: Rc<RefCell<Vec<SharedString>>>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let selected = self.selected.clone();
                let events = self.events.clone();
                let current = selected.borrow().clone().unwrap_or_default();
                div().p(px(20.)).child(
                    el_capsule_segments()
                        .id("frequency")
                        .options([
                            CapsuleOption::new("daily", "Daily"),
                            CapsuleOption::new("interval", "Interval"),
                            CapsuleOption::new("once", "Once"),
                        ])
                        .selected(current)
                        .onselect(move |value, _, _| {
                            *selected.borrow_mut() = Some(value.clone());
                            events.borrow_mut().push(value);
                        }),
                )
            }
        }
        let selected = Rc::new(RefCell::new(None));
        let events = Rc::new(RefCell::new(Vec::new()));
        let (_, view) = cx.add_window_view({
            let selected = selected.clone();
            let events = events.clone();
            move |_, _| Harness { selected, events }
        });
        view.simulate_click(gpui::point(px(35.), px(35.)), gpui::Modifiers::default());
        assert_eq!(selected.borrow().as_deref(), Some("daily"));
        view.simulate_click(gpui::point(px(95.), px(35.)), gpui::Modifiers::default());
        assert_eq!(selected.borrow().as_deref(), Some("interval"));
        let keystroke = view.update(|_, cx| {
            gpui::Keystroke::parse_with_os("enter", cx.platform_info().os).unwrap()
        });
        view.simulate_event(gpui::KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        view.simulate_event(gpui::KeyUpEvent { keystroke });
        assert_eq!(events.borrow().len(), 3);
    }

    #[gpui::test]
    fn disabled_capsules_do_not_report_activation(cx: &mut gpui::TestAppContext) {
        use std::cell::RefCell;
        struct Harness {
            events: Rc<RefCell<usize>>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let events = self.events.clone();
                div().p(px(20.)).child(
                    el_capsule_segments()
                        .id("disabled-frequency")
                        .options([CapsuleOption::new("daily", "Daily")])
                        .disabled(true)
                        .onselect(move |_, _, _| *events.borrow_mut() += 1),
                )
            }
        }
        let events = Rc::new(RefCell::new(0));
        let (_, view) = cx.add_window_view({
            let events = events.clone();
            move |_, _| Harness { events }
        });
        view.simulate_click(gpui::point(px(35.), px(35.)), gpui::Modifiers::default());
        assert_eq!(*events.borrow(), 0);
    }
}
