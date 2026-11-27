//! Element Plus AutoComplete.
//!
//! Follows `autocomplete.md`, `autocomplete.vue`, and Theme Chalk
//! `autocomplete.scss`: an Input with a suggestion popup, local or caller-
//! driven filtering, loading state, keyboard highlighting, and select events.
//!
//! Native differences: suggestions are typed `AutoCompleteOption` values
//! instead of arbitrary records; custom suggestion rendering, debounce,
//! placement/popper configuration, and imperative exposes are not mapped.

use std::{collections::BTreeMap, rc::Rc, sync::Arc};

use gpui::{
    AnyElement, App, Bounds, ElementId, IntoElement, Pixels, RenderOnce, SharedString, Window,
    anchored, deferred, div, point, prelude::*, px, rgb,
};

use crate::{InputSize, Locale, Theme, el_icon, el_input};

/// One autocomplete suggestion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutoCompleteOption {
    value: SharedString,
    label: SharedString,
}

impl AutoCompleteOption {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

type InputHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;
type SelectHandler = Rc<dyn Fn(AutoCompleteOption, &mut Window, &mut App)>;
type FocusHandler = Rc<dyn Fn(&mut Window, &mut App)>;
type VisibleHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type FetchMethod = Arc<dyn Fn(&str, &mut Window, &mut App)>;

#[derive(Clone)]
struct AutoCompleteProps {
    value: SharedString,
    options: Vec<AutoCompleteOption>,
    disabled: bool,
    trigger_on_focus: bool,
    select_when_unmatched: bool,
    loop_navigation: bool,
    highlight_first: bool,
    fetch_method: Option<FetchMethod>,
}

impl Default for AutoCompleteProps {
    fn default() -> Self {
        Self {
            value: String::new().into(),
            options: Vec::new(),
            disabled: false,
            trigger_on_focus: true,
            select_when_unmatched: false,
            loop_navigation: true,
            highlight_first: false,
            fetch_method: None,
        }
    }
}

#[derive(Clone, Default)]
struct AutoCompleteHandlers {
    oninput: Option<InputHandler>,
    onchange: Option<InputHandler>,
    onselect: Option<SelectHandler>,
    onclear: Option<FocusHandler>,
    onfocus: Option<FocusHandler>,
    onblur: Option<FocusHandler>,
    onvisiblechange: Option<VisibleHandler>,
}

#[derive(Default)]
struct AutoCompleteState {
    open: bool,
    active: Option<usize>,
    focused: bool,
    composing: bool,
    bounds: Option<Bounds<Pixels>>,
    popup_bounds: Option<Bounds<Pixels>>,
    option_bounds: BTreeMap<SharedString, Bounds<Pixels>>,
    props: AutoCompleteProps,
    handlers: AutoCompleteHandlers,
}

impl AutoCompleteState {
    fn set_open(&mut self, open: bool, window: &mut Window, cx: &mut App) {
        if self.props.disabled || self.open == open {
            return;
        }
        self.open = open;
        if let Some(handler) = self.handlers.onvisiblechange.clone() {
            handler(open, window, cx);
        }
    }

    fn fetch(&self, window: &mut Window, cx: &mut App) {
        if let Some(handler) = self.props.fetch_method.clone() {
            handler(self.props.value.as_ref(), window, cx);
        }
    }

    fn accept_input(&mut self, value: SharedString, window: &mut Window, cx: &mut App) {
        if self.props.disabled {
            return;
        }
        self.props.value = value;
        if let Some(handler) = self.handlers.oninput.clone() {
            handler(self.props.value.clone(), window, cx);
        }
        if !self.props.value.is_empty() || self.props.trigger_on_focus {
            self.set_open(true, window, cx);
            self.active = self.props.highlight_first.then_some(0);
            self.fetch(window, cx);
        } else {
            self.set_open(false, window, cx);
        }
    }

    fn change(&mut self, value: SharedString, window: &mut Window, cx: &mut App) {
        self.props.value = value;
        if let Some(handler) = self.handlers.onchange.clone() {
            handler(self.props.value.clone(), window, cx);
        }
    }

    fn focus(&mut self, window: &mut Window, cx: &mut App) {
        self.focused = true;
        if let Some(handler) = self.handlers.onfocus.clone() {
            handler(window, cx);
        }
        if self.props.trigger_on_focus && !self.props.disabled {
            self.set_open(true, window, cx);
            self.active = self.props.highlight_first.then_some(0);
            self.fetch(window, cx);
        }
    }

    fn blur(&mut self, window: &mut Window, cx: &mut App) {
        self.focused = false;
        if let Some(handler) = self.handlers.onblur.clone() {
            handler(window, cx);
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut App) {
        if self.props.disabled {
            return;
        }
        self.props.value = String::new().into();
        self.set_open(false, window, cx);
        if let Some(handler) = self.handlers.onclear.clone() {
            handler(window, cx);
        }
    }

    fn navigate(&mut self, direction: i32) {
        let count = filtered_indices(&self.props).len();
        if count == 0 {
            self.active = None;
            return;
        }
        let next = match (self.active, direction) {
            (None, value) if value < 0 => count - 1,
            (None, _) => 0,
            (Some(current), value) if value < 0 => {
                if current == 0 {
                    if self.props.loop_navigation {
                        count - 1
                    } else {
                        0
                    }
                } else {
                    current - 1
                }
            }
            (Some(current), _) => {
                if current + 1 >= count {
                    if self.props.loop_navigation {
                        0
                    } else {
                        count - 1
                    }
                } else {
                    current + 1
                }
            }
        };
        self.active = Some(next);
    }

    fn select_active(&mut self, window: &mut Window, cx: &mut App) {
        let Some(active) = self.active else {
            return;
        };
        let Some(option) = filtered_indices(&self.props)
            .get(active)
            .and_then(|index| self.props.options.get(*index))
            .cloned()
        else {
            return;
        };
        self.select_option(option, window, cx);
    }

    fn select_option(&mut self, option: AutoCompleteOption, window: &mut Window, cx: &mut App) {
        if self.props.disabled {
            return;
        }
        self.props.value = option.value.clone();
        if let Some(handler) = self.handlers.oninput.clone() {
            handler(option.value.clone(), window, cx);
        }
        if let Some(handler) = self.handlers.onselect.clone() {
            handler(option, window, cx);
        }
        self.active = None;
        self.set_open(false, window, cx);
    }

    fn key(&mut self, key: &str, window: &mut Window, cx: &mut App) -> bool {
        if self.props.disabled {
            return false;
        }
        match key {
            "up" | "down" => {
                if !self.open {
                    self.set_open(true, window, cx);
                    self.active = Some(0);
                } else {
                    self.navigate(if key == "down" { 1 } else { -1 });
                }
                true
            }
            "enter" => {
                if self.open && self.active.is_some() {
                    self.select_active(window, cx);
                } else if self.props.select_when_unmatched {
                    let value = self.props.value.clone();
                    self.select_option(AutoCompleteOption::new(value.clone(), value), window, cx);
                } else {
                    self.set_open(true, window, cx);
                    self.fetch(window, cx);
                }
                true
            }
            "escape" | "tab" => {
                self.set_open(false, window, cx);
                true
            }
            "home" if self.open => {
                self.active = Some(0);
                true
            }
            "end" if self.open => {
                let count = filtered_indices(&self.props).len();
                self.active = count.checked_sub(1);
                true
            }
            _ => false,
        }
    }
}

fn filtered_indices(props: &AutoCompleteProps) -> Vec<usize> {
    if props.fetch_method.is_some() || props.value.is_empty() {
        return (0..props.options.len()).collect();
    }
    let query = props.value.to_lowercase();
    props
        .options
        .iter()
        .enumerate()
        .filter(|(_, option)| {
            option.label.to_lowercase().contains(&query)
                || option.value.to_lowercase().contains(&query)
        })
        .map(|(index, _)| index)
        .collect()
}

/// An Element Plus autocomplete input.
#[derive(IntoElement)]
#[must_use]
pub struct AutoComplete {
    id: ElementId,
    #[cfg(test)]
    state: Option<gpui::Entity<AutoCompleteState>>,
    value: SharedString,
    options: Vec<AutoCompleteOption>,
    size: InputSize,
    disabled: bool,
    clearable: bool,
    loading: bool,
    trigger_on_focus: bool,
    select_when_unmatched: bool,
    loop_navigation: bool,
    highlight_first: bool,
    placeholder: SharedString,
    accessibility_label: Option<SharedString>,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
    header: Option<AnyElement>,
    footer: Option<AnyElement>,
    loading_text: Option<SharedString>,
    fetch_method: Option<FetchMethod>,
    oninput: Option<InputHandler>,
    onchange: Option<InputHandler>,
    onselect: Option<SelectHandler>,
    onclear: Option<FocusHandler>,
    onfocus: Option<FocusHandler>,
    onblur: Option<FocusHandler>,
    onvisiblechange: Option<VisibleHandler>,
}

/// Creates an autocomplete input.
#[track_caller]
pub fn el_auto_complete() -> AutoComplete {
    AutoComplete::new()
}

impl Default for AutoComplete {
    fn default() -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            #[cfg(test)]
            state: None,
            value: String::new().into(),
            options: Vec::new(),
            size: InputSize::Default,
            disabled: false,
            clearable: false,
            loading: false,
            trigger_on_focus: true,
            select_when_unmatched: false,
            loop_navigation: true,
            highlight_first: false,
            placeholder: String::new().into(),
            accessibility_label: None,
            prefix: None,
            suffix: None,
            header: None,
            footer: None,
            loading_text: None,
            fetch_method: None,
            oninput: None,
            onchange: None,
            onselect: None,
            onclear: None,
            onfocus: None,
            onblur: None,
            onvisiblechange: None,
        }
    }
}

impl AutoComplete {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    #[cfg(test)]
    fn state(mut self, state: gpui::Entity<AutoCompleteState>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = value.into();
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = AutoCompleteOption>) -> Self {
        self.options = options.into_iter().collect();
        self
    }

    pub fn size(mut self, size: InputSize) -> Self {
        self.size = size;
        self
    }

    pub fn large(self) -> Self {
        self.size(InputSize::Large)
    }

    pub fn small(self) -> Self {
        self.size(InputSize::Small)
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn clearable(mut self, clearable: bool) -> Self {
        self.clearable = clearable;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn trigger_on_focus(mut self, trigger: bool) -> Self {
        self.trigger_on_focus = trigger;
        self
    }

    pub fn select_when_unmatched(mut self, select: bool) -> Self {
        self.select_when_unmatched = select;
        self
    }

    pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
        self
    }

    pub fn highlight_first(mut self, highlight: bool) -> Self {
        self.highlight_first = highlight;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    pub fn prefix(mut self, prefix: impl IntoElement) -> Self {
        self.prefix = Some(prefix.into_any_element());
        self
    }

    pub fn suffix(mut self, suffix: impl IntoElement) -> Self {
        self.suffix = Some(suffix.into_any_element());
        self
    }

    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    pub fn loading_text(mut self, text: impl Into<SharedString>) -> Self {
        self.loading_text = Some(text.into());
        self
    }

    pub fn fetch_method(mut self, method: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.fetch_method = Some(Arc::new(method));
        self
    }

    #[track_caller]
    pub fn oninput(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.oninput = Some(Rc::new(handler));
        self
    }

    #[track_caller]
    pub fn onchange(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onchange = Some(Rc::new(handler));
        self
    }

    #[track_caller]
    pub fn onselect(
        mut self,
        handler: impl Fn(AutoCompleteOption, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onselect = Some(Rc::new(handler));
        self
    }

    #[track_caller]
    pub fn onclear(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.onclear = Some(Rc::new(handler));
        self
    }

    pub fn onfocus(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.onfocus = Some(Rc::new(handler));
        self
    }

    pub fn onblur(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.onblur = Some(Rc::new(handler));
        self
    }

    #[track_caller]
    pub fn onvisiblechange(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onvisiblechange = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for AutoComplete {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let AutoComplete {
            id,
            #[cfg(test)]
                state: provided_state,
            value,
            options,
            size,
            disabled,
            clearable,
            loading,
            trigger_on_focus,
            select_when_unmatched,
            loop_navigation,
            highlight_first,
            placeholder,
            accessibility_label,
            prefix,
            suffix,
            header,
            footer,
            loading_text,
            fetch_method,
            oninput,
            onchange,
            onselect,
            onclear,
            onfocus,
            onblur,
            onvisiblechange,
        } = self;
        let props = AutoCompleteProps {
            value: value.clone(),
            options: options.clone(),
            disabled,
            trigger_on_focus,
            select_when_unmatched,
            loop_navigation,
            highlight_first,
            fetch_method,
        };
        let handlers = AutoCompleteHandlers {
            oninput,
            onchange,
            onselect,
            onclear,
            onfocus,
            onblur,
            onvisiblechange,
        };
        #[cfg(test)]
        let state = provided_state.unwrap_or_else(|| {
            window.use_keyed_state(id.clone(), cx, |_, _| AutoCompleteState::default())
        });
        #[cfg(not(test))]
        let state = window.use_keyed_state(id.clone(), cx, |_, _| AutoCompleteState::default());
        state.update(cx, |state, _| {
            state.props = props.clone();
            state.handlers = handlers;
        });
        let filtered = filtered_indices(&props);
        state.update(cx, |state, _| {
            if state.active.is_none_or(|active| active >= filtered.len()) {
                state.active = None;
            }
            state
                .option_bounds
                .retain(|value, _| options.iter().any(|option| option.value == *value));
        });
        let open = state.read(cx).open && !disabled && (loading || !filtered.is_empty());
        let locale = Locale::current(cx);
        let theme = Theme::current(cx);
        let mut suffix_items = Vec::new();
        if let Some(suffix) = suffix {
            suffix_items.push(suffix);
        }
        suffix_items.push(
            div()
                .flex()
                .items_center()
                .text_color(rgb(theme.disabled_text()))
                .child(
                    el_icon("arrow-down")
                        .size(size.font_size())
                        .rotate(if open { std::f32::consts::PI } else { 0. }),
                )
                .into_any_element(),
        );
        let mut input = el_input()
            .id(ElementId::NamedChild(Arc::new(id.clone()), "input".into()))
            .value(value)
            .placeholder(placeholder)
            .size(size)
            .radius_small()
            .disabled(disabled)
            .clearable(clearable)
            .suffix(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .children(suffix_items),
            )
            .oninput({
                let state = state.clone();
                move |reported, window, cx| {
                    state.update(cx, |state, cx| state.accept_input(reported, window, cx));
                    window.refresh();
                }
            })
            .onchange({
                let state = state.clone();
                move |reported, window, cx| {
                    state.update(cx, |state, cx| state.change(reported, window, cx));
                }
            })
            .onfocus({
                let state = state.clone();
                move |window, cx| {
                    state.update(cx, |state, cx| state.focus(window, cx));
                    window.refresh();
                }
            })
            .onblur({
                let state = state.clone();
                move |window, cx| {
                    state.update(cx, |state, cx| state.blur(window, cx));
                    window.refresh();
                }
            })
            .onclear({
                let state = state.clone();
                move |_, window, cx| {
                    state.update(cx, |state, cx| state.clear(window, cx));
                    window.refresh();
                }
            })
            .on_key_down_filter({
                let state = state.clone();
                move |event, window, cx| {
                    let handled =
                        state.update(cx, |state, cx| state.key(&event.keystroke.key, window, cx));
                    if handled {
                        window.refresh();
                    }
                    handled
                }
            })
            .oncompositionstart({
                let state = state.clone();
                move |_window, cx| state.update(cx, |state, _| state.composing = true)
            })
            .oncompositionend({
                let state = state.clone();
                move |_, _window, cx| state.update(cx, |state, _| state.composing = false)
            });
        if let Some(label) = accessibility_label {
            input = input.aria_label(label);
        }
        if let Some(prefix) = prefix {
            input = input.prefix(prefix);
        }
        let measure = state.clone();
        let mut root = div()
            .id(id.clone())
            .relative()
            .w_full()
            .child(input)
            .child(
                gpui::canvas(
                    move |bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App| {
                        if measure.read(cx).bounds != Some(bounds) {
                            measure.update(cx, |state, _| state.bounds = Some(bounds));
                            window.refresh();
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            )
            .on_mouse_down_out({
                let state = state.clone();
                move |event, window, cx| {
                    let inside_popup = state
                        .read(cx)
                        .popup_bounds
                        .is_some_and(|bounds| bounds.contains(&event.position))
                        || state.read(cx).bounds.is_some_and(|target| {
                            gpui::Bounds::new(
                                point(target.left(), target.bottom() + px(4.)),
                                gpui::size(target.size.width, px(280.)),
                            )
                            .contains(&event.position)
                        });
                    if !inside_popup {
                        state.update(cx, |state, cx| state.set_open(false, window, cx));
                        window.refresh();
                    }
                }
            });
        if open && let Some(target) = state.read(cx).bounds {
            let viewport = window.viewport_size();
            let below = viewport.height - target.bottom() >= px(284.) || target.top() < px(284.);
            let mut popup = div()
                .id(ElementId::NamedChild(Arc::new(id.clone()), "popup".into()))
                .flex()
                .flex_col()
                .py(px(10.))
                .w(target.size.width)
                .max_h(px(280.))
                .rounded(px(4.))
                .border_1()
                .border_color(rgb(theme.border_lighter()))
                .bg(rgb(theme.overlay_background()))
                .shadow(theme.overlay_shadow())
                .overflow_y_scroll()
                .occlude()
                .on_scroll(|_, _, cx| cx.stop_propagation());
            if let Some(header) = header {
                popup = popup.child(
                    div()
                        .px(px(10.))
                        .py(px(8.))
                        .border_b_1()
                        .border_color(rgb(theme.border_lighter()))
                        .child(header),
                );
            }
            if loading {
                popup = popup.child(
                    div()
                        .h(px(100.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(20.))
                        .text_color(rgb(theme.secondary_text()))
                        .child(
                            el_icon("loading")
                                .size(20.)
                                .spin(crate::LoadingMotion::Uniform),
                        )
                        .child(
                            loading_text.unwrap_or_else(|| locale.auto_complete_loading().into()),
                        ),
                );
            } else {
                for (index, option_index) in filtered.iter().copied().enumerate() {
                    let Some(option) = options.get(option_index) else {
                        continue;
                    };
                    let active = state.read(cx).active == Some(index);
                    let mut row = div()
                        .id(ElementId::NamedChild(
                            Arc::new(id.clone()),
                            format!("option-{option_index}").into(),
                        ))
                        .relative()
                        .h(px(34.))
                        .flex()
                        .items_center()
                        .px(px(20.))
                        .text_size(px(14.))
                        .whitespace_nowrap()
                        .truncate()
                        .text_color(rgb(theme.regular_text()))
                        .aria_selected(active)
                        .aria_label(option.label.clone())
                        .child(option.label.clone())
                        .cursor_pointer()
                        .when(active, |el| {
                            el.bg(rgb(theme.fill(crate::theme::Fill::Light)))
                        })
                        .hover(|style| style.bg(rgb(theme.fill(crate::theme::Fill::Light))))
                        .on_hover({
                            let state = state.clone();
                            move |_, window, cx| {
                                state.update(cx, |state, _| state.active = Some(index));
                                window.refresh();
                            }
                        })
                        .on_click({
                            let state = state.clone();
                            let option = option.clone();
                            move |_, window, cx| {
                                cx.stop_propagation();
                                state.update(cx, |state, cx| {
                                    state.select_option(option.clone(), window, cx)
                                });
                                window.refresh();
                            }
                        });
                    let measure = state.clone();
                    let value_key = option.value.clone();
                    row = row.child(
                        gpui::canvas(
                            move |bounds: Bounds<Pixels>, _, cx: &mut App| {
                                if measure.read(cx).option_bounds.get(&value_key) != Some(&bounds) {
                                    measure.update(cx, |state, _| {
                                        state.option_bounds.insert(value_key.clone(), bounds);
                                    });
                                }
                            },
                            |_, _, _, _| {},
                        )
                        .absolute()
                        .inset_0(),
                    );
                    popup = popup.child(row);
                }
            }
            if let Some(footer) = footer {
                popup = popup.child(
                    div()
                        .px(px(10.))
                        .py(px(8.))
                        .border_t_1()
                        .border_color(rgb(theme.border_lighter()))
                        .child(footer),
                );
            }
            let measure = state.clone();
            popup = popup.child(
                gpui::canvas(
                    move |bounds: Bounds<Pixels>, _, cx: &mut App| {
                        if measure.read(cx).popup_bounds != Some(bounds) {
                            measure.update(cx, |state, _| state.popup_bounds = Some(bounds));
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            );
            root = root.child(
                deferred(
                    anchored()
                        .attach(
                            target,
                            if below {
                                gpui::Anchor::BottomLeft
                            } else {
                                gpui::Anchor::TopLeft
                            },
                        )
                        .offset(point(px(0.), if below { px(4.) } else { px(-4.) }))
                        .snap_to_window_with_margin(px(4.))
                        .child(popup),
                )
                .with_priority(crate::theme::popup_priority(0)),
            );
        } else {
            state.update(cx, |state, _| state.popup_bounds = None);
        }
        root.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options() -> Vec<AutoCompleteOption> {
        vec![
            AutoCompleteOption::new("rust", "Rust"),
            AutoCompleteOption::new("gpui", "GPUI"),
            AutoCompleteOption::new("element", "Element Plus"),
        ]
    }

    #[test]
    fn filters_static_suggestions_case_insensitively() {
        let mut props = AutoCompleteProps {
            options: options(),
            ..Default::default()
        };
        assert_eq!(filtered_indices(&props), [0, 1, 2]);
        props.value = "r".into();
        assert_eq!(filtered_indices(&props), [0]);
        props.value = "plus".into();
        assert_eq!(filtered_indices(&props), [2]);
        props.fetch_method = Some(Arc::new(|_, _, _| {}));
        assert_eq!(filtered_indices(&props), [0, 1, 2]);
    }

    #[gpui::test]
    fn rendered_autocomplete_supports_keyboard_and_pointer_selection(
        cx: &mut gpui::TestAppContext,
    ) {
        use std::cell::RefCell;
        struct Harness {
            value: Rc<RefCell<SharedString>>,
            selected: Rc<RefCell<Vec<SharedString>>>,
            state: gpui::Entity<AutoCompleteState>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let value = self.value.clone();
                let input_value = self.value.clone();
                let selected = self.selected.clone();
                div().p(px(20.)).w(px(240.)).child(
                    el_auto_complete()
                        .id("rendered-autocomplete")
                        .state(self.state.clone())
                        .value(value.borrow().clone())
                        .options(options())
                        .oninput(move |reported, _, _| *input_value.borrow_mut() = reported)
                        .onselect(move |option, _, _| selected.borrow_mut().push(option.value)),
                )
            }
        }
        let value = Rc::new(RefCell::new(String::new().into()));
        let selected = Rc::new(RefCell::new(Vec::new()));
        let (harness, view) = cx.add_window_view({
            let value = value.clone();
            let selected = selected.clone();
            move |_, cx| Harness {
                value,
                selected,
                state: cx.new(|_| AutoCompleteState::default()),
            }
        });
        let state = view.update(|_, cx| harness.read(cx).state.clone());
        view.simulate_click(gpui::point(px(100.), px(36.)), gpui::Modifiers::default());
        press_key(view, "down");
        press_key(view, "enter");
        assert_eq!(selected.borrow().as_slice(), ["rust".to_string()]);
        assert_eq!(value.borrow().as_str(), "rust");
        press_key(view, "down");
        view.update(|window, cx| window.draw(cx).clear(cx));
        let option = view.update(|_, cx| {
            cx.read_entity(&state, |state, _| {
                state
                    .option_bounds
                    .get(&SharedString::from("rust"))
                    .copied()
                    .map(|bounds| bounds.center())
            })
        });
        let Some(option) = option else {
            panic!("Rust option bounds were not measured");
        };
        view.simulate_click(option, gpui::Modifiers::default());
        assert_eq!(selected.borrow().len(), 2);
        assert_eq!(value.borrow().as_str(), "rust");
    }

    fn press_key(view: &mut gpui::VisualTestContext, key: &str) {
        let keystroke = view
            .update(|_, cx| gpui::Keystroke::parse_with_os(key, cx.platform_info().os).unwrap());
        view.simulate_event(gpui::KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        view.simulate_event(gpui::KeyUpEvent { keystroke });
    }
}
