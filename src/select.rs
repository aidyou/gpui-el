//! Element Plus Select.
//!
//! Follows `select.md`, `select.vue`, `option.vue`, `useSelect.ts`, and Theme
//! Chalk `select.scss`/`option.scss`: a 100%-width field, 32/40/24 px sizes,
//! filterable text, single and multiple selection, clearable values, disabled
//! options, a 274 px popup, and keyboard navigation. The field reuses
//! [`Input`] so filtering keeps native caret, clipboard, and IME behavior.
//!
//! Native differences: multiple selections are summarized as comma-separated
//! labels instead of removable Tag chips; `Backspace` removes the last
//! enabled selection and `onremove` reports it. Remote search is caller-driven
//! through `filter_method`; debounce, option groups, custom option rendering,
//! collapse-tag tooltips, and imperative focus/blur exposes are not mapped.

use std::{collections::BTreeMap, rc::Rc, sync::Arc};

use gpui::{
    AnyElement, App, Bounds, ElementId, IntoElement, Pixels, RenderOnce, SharedString, Window,
    anchored, deferred, div, point, prelude::*, px, rgb,
};

use crate::{InputSize, Locale, Theme, el_icon, el_input};

/// One Select option.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectOption {
    value: SharedString,
    label: SharedString,
    disabled: bool,
}

impl SelectOption {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// The controlled value reported by Select.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectValue {
    Single(Option<SharedString>),
    Multiple(Vec<SharedString>),
}

impl Default for SelectValue {
    fn default() -> Self {
        Self::Single(None)
    }
}

impl SelectValue {
    fn has_value(&self) -> bool {
        match self {
            Self::Single(value) => value.is_some(),
            Self::Multiple(values) => !values.is_empty(),
        }
    }
}

type ChangeHandler = Rc<dyn Fn(SelectValue, &mut Window, &mut App)>;
type VisibleHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type ValueHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;
type FocusHandler = Rc<dyn Fn(&mut Window, &mut App)>;
type FilterMethod = Arc<dyn Fn(&str, &mut Window, &mut App)>;

#[derive(Clone)]
struct SelectProps {
    value: SelectValue,
    options: Vec<SelectOption>,
    multiple: bool,
    multiple_limit: usize,
    disabled: bool,
    filterable: bool,
    remote: bool,
    reserve_keyword: bool,
    value_on_clear: Option<SharedString>,
    filter_method: Option<FilterMethod>,
}

impl Default for SelectProps {
    fn default() -> Self {
        Self {
            value: SelectValue::default(),
            options: Vec::new(),
            multiple: false,
            multiple_limit: 0,
            disabled: false,
            filterable: false,
            remote: false,
            reserve_keyword: true,
            value_on_clear: None,
            filter_method: None,
        }
    }
}

#[derive(Clone, Default)]
struct SelectHandlers {
    onchange: Option<ChangeHandler>,
    onvisiblechange: Option<VisibleHandler>,
    onremove: Option<ValueHandler>,
    onclear: Option<FocusHandler>,
    onfocus: Option<FocusHandler>,
    onblur: Option<FocusHandler>,
}

#[derive(Default)]
struct SelectState {
    open: bool,
    query: String,
    active: Option<usize>,
    focused: bool,
    hovered: bool,
    composing: bool,
    bounds: Option<Bounds<Pixels>>,
    popup_bounds: Option<Bounds<Pixels>>,
    option_bounds: BTreeMap<SharedString, Bounds<Pixels>>,
    props: SelectProps,
    handlers: SelectHandlers,
    last_value: SelectValue,
}

impl SelectState {
    fn sync_value(&mut self, value: SelectValue) {
        if self.last_value != value && !self.focused {
            self.query.clear();
        }
        self.last_value = value;
    }

    fn set_open(&mut self, open: bool, window: &mut Window, cx: &mut App) {
        if self.props.disabled || self.open == open {
            return;
        }
        self.open = open;
        if let Some(handler) = self.handlers.onvisiblechange.clone() {
            handler(open, window, cx);
        }
    }

    fn set_focused(&mut self, focused: bool, window: &mut Window, cx: &mut App) {
        self.focused = focused;
        let handler = if focused {
            self.handlers.onfocus.clone()
        } else {
            self.handlers.onblur.clone()
        };
        if let Some(handler) = handler {
            handler(window, cx);
        }
    }

    fn accept_query(&mut self, query: String, window: &mut Window, cx: &mut App) {
        if self.props.disabled {
            return;
        }
        self.query = query;
        self.set_open(true, window, cx);
        self.active = first_enabled(
            &filtered_indices(&self.props, &self.query),
            &self.props.options,
        );
        if let Some(handler) = self.props.filter_method.clone() {
            handler(&self.query, window, cx);
        }
    }

    fn navigate(&mut self, direction: i32) {
        let filtered = filtered_indices(&self.props, &self.query);
        if filtered.is_empty() {
            self.active = None;
            return;
        }
        let mut current = self.active.unwrap_or(0).min(filtered.len() - 1);
        for _ in 0..filtered.len() {
            current = (current as i32 + direction).rem_euclid(filtered.len() as i32) as usize;
            let Some(option_index) = filtered.get(current) else {
                self.active = None;
                return;
            };
            if !self
                .props
                .options
                .get(*option_index)
                .is_some_and(|option| option.disabled)
            {
                self.active = Some(current);
                return;
            }
        }
        self.active = None;
    }

    fn focus_edge(&mut self, end: bool) {
        let filtered = filtered_indices(&self.props, &self.query);
        self.active = if end {
            filtered.into_iter().rposition(|index| {
                !self
                    .props
                    .options
                    .get(index)
                    .is_some_and(|option| option.disabled)
            })
        } else {
            first_enabled(&filtered, &self.props.options)
        };
    }

    fn select_active(&mut self, window: &mut Window, cx: &mut App) {
        let Some(active) = self.active else {
            return;
        };
        let Some(option) = filtered_indices(&self.props, &self.query)
            .get(active)
            .and_then(|index| self.props.options.get(*index))
            .cloned()
        else {
            return;
        };
        self.select_option(option, window, cx);
    }

    fn select_option(&mut self, option: SelectOption, window: &mut Window, cx: &mut App) {
        if self.props.disabled || option.disabled {
            return;
        }
        let old = self.props.value.clone();
        let mut removed = None;
        let new = if self.props.multiple {
            let mut values = match &old {
                SelectValue::Multiple(values) => values.clone(),
                _ => Vec::new(),
            };
            if let Some(index) = values.iter().position(|value| value == &option.value) {
                removed = Some(values.remove(index));
            } else if self.props.multiple_limit == 0 || values.len() < self.props.multiple_limit {
                values.push(option.value.clone());
            }
            SelectValue::Multiple(values)
        } else {
            self.set_open(false, window, cx);
            SelectValue::Single(Some(option.value.clone()))
        };
        if new != old {
            self.props.value = new.clone();
            self.last_value = new.clone();
            if let Some(handler) = self.handlers.onchange.clone() {
                handler(new, window, cx);
            }
        }
        if let Some(value) = removed
            && let Some(handler) = self.handlers.onremove.clone()
        {
            handler(value, window, cx);
        }
        if self.props.multiple && !self.props.reserve_keyword {
            self.query.clear();
        }
    }

    fn remove_value(&mut self, value: SharedString, window: &mut Window, cx: &mut App) {
        if self.props.disabled {
            return;
        }
        let SelectValue::Multiple(values) = &self.props.value else {
            return;
        };
        if self
            .props
            .options
            .iter()
            .find(|option| option.value == value)
            .is_some_and(|option| option.disabled)
        {
            return;
        }
        let Some(index) = values.iter().position(|selected| selected == &value) else {
            return;
        };
        let mut values = values.clone();
        values.remove(index);
        let new = SelectValue::Multiple(values);
        self.props.value = new.clone();
        self.last_value = new.clone();
        if let Some(handler) = self.handlers.onchange.clone() {
            handler(new, window, cx);
        }
        if let Some(handler) = self.handlers.onremove.clone() {
            handler(value, window, cx);
        }
    }

    fn remove_last(&mut self, window: &mut Window, cx: &mut App) {
        let SelectValue::Multiple(values) = &self.props.value else {
            return;
        };
        let value = values.iter().rev().find(|value| {
            self.props
                .options
                .iter()
                .find(|option| option.value == **value)
                .is_none_or(|option| !option.disabled)
        });
        if let Some(value) = value.cloned() {
            self.remove_value(value, window, cx);
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut App) {
        if self.props.disabled {
            return;
        }
        let old = self.props.value.clone();
        let new = if self.props.multiple {
            let values = match &old {
                SelectValue::Multiple(values) => values
                    .iter()
                    .filter(|value| {
                        self.props
                            .options
                            .iter()
                            .find(|option| option.value == **value)
                            .is_some_and(|option| option.disabled)
                    })
                    .cloned()
                    .collect(),
                _ => Vec::new(),
            };
            SelectValue::Multiple(values)
        } else {
            SelectValue::Single(self.props.value_on_clear.clone())
        };
        if new != old {
            self.props.value = new.clone();
            self.last_value = new.clone();
            if let Some(handler) = self.handlers.onchange.clone() {
                handler(new, window, cx);
            }
        }
        self.query.clear();
        self.set_open(false, window, cx);
        if let Some(handler) = self.handlers.onclear.clone() {
            handler(window, cx);
        }
    }

    fn key(&mut self, key: &str, window: &mut Window, cx: &mut App) -> bool {
        if self.props.disabled || self.composing {
            return false;
        }
        match key {
            "down" | "up" => {
                if !self.open {
                    self.set_open(true, window, cx);
                    self.focus_edge(false);
                } else {
                    self.navigate(if key == "down" { 1 } else { -1 });
                }
                true
            }
            "enter" => {
                if self.open {
                    self.select_active(window, cx);
                } else {
                    self.set_open(true, window, cx);
                    self.focus_edge(false);
                }
                true
            }
            "escape" => {
                if self.query.is_empty() {
                    self.set_open(false, window, cx);
                } else {
                    self.query.clear();
                    self.active = first_enabled(
                        &filtered_indices(&self.props, &self.query),
                        &self.props.options,
                    );
                }
                true
            }
            "home" if self.open => {
                self.focus_edge(false);
                true
            }
            "end" if self.open => {
                self.focus_edge(true);
                true
            }
            "backspace"
                if self.props.multiple && self.query.is_empty() && self.props.filterable =>
            {
                self.remove_last(window, cx);
                true
            }
            _ => false,
        }
    }
}

fn filtered_indices(props: &SelectProps, query: &str) -> Vec<usize> {
    if !props.filterable || query.trim().is_empty() || props.remote || props.filter_method.is_some()
    {
        return (0..props.options.len()).collect();
    }
    let query = query.to_lowercase();
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

fn first_enabled(indices: &[usize], options: &[SelectOption]) -> Option<usize> {
    indices
        .iter()
        .position(|index| !options.get(*index).is_some_and(|option| option.disabled))
}

/// An Element Plus select field.
#[derive(IntoElement)]
#[must_use]
pub struct Select {
    id: ElementId,
    state: Option<gpui::Entity<SelectState>>,
    value: Option<SharedString>,
    values: Vec<SharedString>,
    options: Vec<SelectOption>,
    multiple: bool,
    multiple_limit: usize,
    size: InputSize,
    disabled: bool,
    clearable: bool,
    filterable: bool,
    loading: bool,
    remote: bool,
    reserve_keyword: bool,
    value_on_clear: Option<SharedString>,
    placeholder: Option<SharedString>,
    accessibility_label: Option<SharedString>,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
    header: Option<AnyElement>,
    footer: Option<AnyElement>,
    loading_text: Option<SharedString>,
    no_data_text: Option<SharedString>,
    no_match_text: Option<SharedString>,
    filter_method: Option<FilterMethod>,
    onchange: Option<ChangeHandler>,
    onvisiblechange: Option<VisibleHandler>,
    onremove: Option<ValueHandler>,
    onclear: Option<FocusHandler>,
    onfocus: Option<FocusHandler>,
    onblur: Option<FocusHandler>,
}

/// Creates a select field.
#[track_caller]
pub fn el_select() -> Select {
    Select::new()
}

impl Default for Select {
    fn default() -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            state: None,
            value: None,
            values: Vec::new(),
            options: Vec::new(),
            multiple: false,
            multiple_limit: 0,
            size: InputSize::Default,
            disabled: false,
            clearable: false,
            filterable: false,
            loading: false,
            remote: false,
            reserve_keyword: true,
            value_on_clear: None,
            placeholder: None,
            accessibility_label: None,
            prefix: None,
            suffix: None,
            header: None,
            footer: None,
            loading_text: None,
            no_data_text: None,
            no_match_text: None,
            filter_method: None,
            onchange: None,
            onvisiblechange: None,
            onremove: None,
            onclear: None,
            onfocus: None,
            onblur: None,
        }
    }
}

impl Select {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    #[cfg(test)]
    fn state(mut self, state: gpui::Entity<SelectState>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn value(mut self, value: Option<SharedString>) -> Self {
        self.value = value;
        self
    }

    pub fn selected(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn values(mut self, values: impl IntoIterator<Item = SharedString>) -> Self {
        self.values = values.into_iter().collect();
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = SelectOption>) -> Self {
        self.options = options.into_iter().collect();
        self
    }

    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    pub fn multiple_limit(mut self, limit: usize) -> Self {
        self.multiple_limit = limit;
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

    pub fn filterable(mut self, filterable: bool) -> Self {
        self.filterable = filterable;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn remote(mut self, remote: bool) -> Self {
        self.remote = remote;
        self
    }

    pub fn reserve_keyword(mut self, reserve: bool) -> Self {
        self.reserve_keyword = reserve;
        self
    }

    pub fn value_on_clear(mut self, value: Option<SharedString>) -> Self {
        self.value_on_clear = value;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
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

    pub fn no_data_text(mut self, text: impl Into<SharedString>) -> Self {
        self.no_data_text = Some(text.into());
        self
    }

    pub fn no_match_text(mut self, text: impl Into<SharedString>) -> Self {
        self.no_match_text = Some(text.into());
        self
    }

    pub fn filter_method(mut self, method: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.filter_method = Some(Arc::new(method));
        self
    }

    #[track_caller]
    pub fn onchange(
        mut self,
        handler: impl Fn(SelectValue, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onchange = Some(Rc::new(handler));
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

    #[track_caller]
    pub fn onremove(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onremove = Some(Rc::new(handler));
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
}

fn selected_labels(value: &SelectValue, options: &[SelectOption]) -> Vec<SharedString> {
    match value {
        SelectValue::Single(value) => value
            .iter()
            .map(|value| label_for(value, options))
            .collect(),
        SelectValue::Multiple(values) => values
            .iter()
            .map(|value| label_for(value, options))
            .collect(),
    }
}

fn label_for(value: &SharedString, options: &[SelectOption]) -> SharedString {
    options
        .iter()
        .find(|option| option.value == *value)
        .map(|option| option.label.clone())
        .unwrap_or_else(|| value.clone())
}

impl RenderOnce for Select {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Select {
            id,
            state: provided_state,
            value,
            values,
            options,
            multiple,
            multiple_limit,
            size,
            disabled,
            clearable,
            filterable,
            loading,
            remote,
            reserve_keyword,
            value_on_clear,
            placeholder,
            accessibility_label,
            prefix,
            suffix,
            header,
            footer,
            loading_text,
            no_data_text,
            no_match_text,
            filter_method,
            onchange,
            onvisiblechange,
            onremove,
            onclear,
            onfocus,
            onblur,
        } = self;
        let value = if multiple {
            SelectValue::Multiple(values)
        } else {
            SelectValue::Single(value)
        };
        let props = SelectProps {
            value: value.clone(),
            options: options.clone(),
            multiple,
            multiple_limit,
            disabled,
            filterable,
            remote,
            reserve_keyword,
            value_on_clear,
            filter_method,
        };
        let handlers = SelectHandlers {
            onchange,
            onvisiblechange,
            onremove,
            onclear,
            onfocus,
            onblur,
        };
        let state = provided_state.unwrap_or_else(|| {
            window.use_keyed_state(id.clone(), cx, |_, _| SelectState::default())
        });
        state.update(cx, |state, _| {
            state.props = props.clone();
            state.handlers = handlers;
        });
        state.update(cx, |state, _| state.sync_value(value.clone()));
        let query = state.read(cx).query.clone();
        let filtered = filtered_indices(&props, &query);
        state.update(cx, |state, _| {
            if state.active.is_none_or(|active| {
                filtered
                    .get(active)
                    .and_then(|index| options.get(*index))
                    .is_none_or(|option| option.disabled)
            }) {
                state.active = first_enabled(&filtered, &options);
            }
            state
                .option_bounds
                .retain(|value, _| options.iter().any(|option| option.value == *value));
        });
        let open = state.read(cx).open && !disabled;
        let focused = state.read(cx).focused;
        let hovered = state.read(cx).hovered;
        let labels = selected_labels(&value, &options);
        let summary = labels
            .iter()
            .map(SharedString::as_ref)
            .collect::<Vec<_>>()
            .join(", ");
        let editing = filterable && (focused || open);
        let display = if editing {
            query.clone()
        } else {
            summary.clone()
        };
        let locale = Locale::current(cx);
        let placeholder = placeholder.unwrap_or_else(|| locale.select_placeholder().into());
        let placeholder = if editing && query.is_empty() && !summary.is_empty() {
            summary.into()
        } else {
            placeholder
        };
        let show_clear = clearable && value.has_value() && !disabled && (focused || hovered);
        let theme = Theme::current(cx);
        let primary = rgb(crate::theme::PRIMARY);
        let icon_color = rgb(theme.disabled_text());
        let mut suffix_items = Vec::new();
        if let Some(suffix) = suffix {
            suffix_items.push(suffix);
        }
        if show_clear {
            suffix_items.push(
                div()
                    .id(ElementId::NamedChild(Arc::new(id.clone()), "clear".into()))
                    .flex()
                    .items_center()
                    .cursor_pointer()
                    .aria_label(locale.select_clear())
                    .child(el_icon("circle-close").size(size.font_size()))
                    .text_color(icon_color)
                    .hover(|style| style.text_color(rgb(theme.secondary_text())))
                    .on_click({
                        let state = state.clone();
                        move |_, window, cx| {
                            cx.stop_propagation();
                            state.update(cx, |state, cx| state.clear(window, cx));
                            window.refresh();
                        }
                    })
                    .into_any_element(),
            );
        }
        if loading {
            suffix_items.push(
                el_icon("loading")
                    .size(size.font_size())
                    .spin(crate::LoadingMotion::Uniform)
                    .into_any_element(),
            );
        }
        suffix_items.push(
            div()
                .id(ElementId::NamedChild(Arc::new(id.clone()), "arrow".into()))
                .flex()
                .items_center()
                .cursor_pointer()
                .child(
                    el_icon("arrow-down")
                        .size(size.font_size())
                        .rotate(if open { std::f32::consts::PI } else { 0. }),
                )
                .text_color(icon_color)
                .into_any_element(),
        );
        let mut input = el_input()
            .id(ElementId::NamedChild(Arc::new(id.clone()), "input".into()))
            .value(display)
            .placeholder(placeholder)
            .size(size)
            .radius_small()
            .disabled(disabled)
            .readonly(!filterable)
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
                    state.update(cx, |state, cx| {
                        state.accept_query(reported.to_string(), window, cx)
                    });
                    window.refresh();
                }
            })
            .onfocus({
                let state = state.clone();
                move |window, cx| {
                    state.update(cx, |state, cx| state.set_focused(true, window, cx));
                    window.refresh();
                }
            })
            .onblur({
                let state = state.clone();
                move |window, cx| {
                    state.update(cx, |state, cx| state.set_focused(false, window, cx));
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
            .on_hover({
                let state = state.clone();
                move |hovered, window, cx| {
                    state.update(cx, |state, _| state.hovered = *hovered);
                    window.refresh();
                }
            })
            .on_click({
                let state = state.clone();
                move |_, window, cx| {
                    if disabled {
                        return;
                    }
                    cx.stop_propagation();
                    let open = !state.read(cx).open;
                    state.update(cx, |state, cx| state.set_open(open, window, cx));
                    window.refresh();
                }
            })
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
                                gpui::size(target.size.width, px(274.)),
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
            let below = viewport.height - target.bottom() >= px(278.) || target.top() < px(278.);
            let popup_id = ElementId::NamedChild(Arc::new(id.clone()), "popup".into());
            let mut popup = div()
                .id(popup_id)
                .flex()
                .flex_col()
                .py(px(6.))
                .w(target.size.width)
                .max_h(px(274.))
                .rounded(px(4.))
                .border_1()
                .border_color(rgb(theme.border_lighter()))
                .bg(rgb(theme.overlay_background()))
                .shadow(theme.overlay_shadow())
                .overflow_y_scroll()
                .occlude()
                .on_scroll(|_, _, cx| cx.stop_propagation());
            if let Some(header) = header {
                popup = popup.child(div().px(px(10.)).py(px(8.)).child(header));
            }
            if loading {
                popup = popup.child(
                    div()
                        .h(px(34.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap(px(6.))
                        .text_size(px(14.))
                        .text_color(rgb(theme.secondary_text()))
                        .child(
                            el_icon("loading")
                                .size(14.)
                                .spin(crate::LoadingMotion::Uniform),
                        )
                        .child(loading_text.unwrap_or_else(|| locale.select_loading().into())),
                );
            } else if filtered.is_empty() {
                let text = if query.is_empty() {
                    no_data_text.unwrap_or_else(|| locale.select_no_data().into())
                } else {
                    no_match_text.unwrap_or_else(|| locale.select_no_match().into())
                };
                popup = popup.child(
                    div()
                        .h(px(34.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(14.))
                        .text_color(rgb(theme.secondary_text()))
                        .child(text),
                );
            } else {
                for (active_index, option_index) in filtered.iter().copied().enumerate() {
                    let Some(option) = options.get(option_index) else {
                        continue;
                    };
                    let selected = match &value {
                        SelectValue::Single(selected) => selected.as_ref() == Some(&option.value),
                        SelectValue::Multiple(selected) => selected.contains(&option.value),
                    };
                    let active = state.read(cx).active == Some(active_index);
                    let option_id = ElementId::NamedChild(
                        Arc::new(id.clone()),
                        format!("option-{}", option_index).into(),
                    );
                    let mut row = div()
                        .id(option_id)
                        .relative()
                        .h(px(34.))
                        .flex()
                        .items_center()
                        .px(px(20.))
                        .pr(px(32.))
                        .text_size(px(14.))
                        .whitespace_nowrap()
                        .truncate()
                        .text_color(if option.disabled {
                            rgb(theme.disabled_text())
                        } else if selected {
                            primary
                        } else {
                            rgb(theme.regular_text())
                        })
                        .when(selected, |el| el.font_weight(gpui::FontWeight::MEDIUM))
                        .when(active && !option.disabled, |el| {
                            el.bg(rgb(theme.fill(crate::theme::Fill::Light)))
                        })
                        .aria_selected(selected)
                        .aria_label(option.label.clone())
                        .child(option.label.clone());
                    if multiple && selected {
                        row = row.child(
                            div()
                                .absolute()
                                .right(px(20.))
                                .text_color(if option.disabled {
                                    rgb(theme.disabled_text())
                                } else {
                                    primary
                                })
                                .child(el_icon("check").size(12.)),
                        );
                    }
                    if option.disabled {
                        row = row.cursor_not_allowed();
                    } else {
                        row = row
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(theme.fill(crate::theme::Fill::Light))))
                            .on_hover({
                                let state = state.clone();
                                move |_, window, cx| {
                                    state.update(cx, |state, _| state.active = Some(active_index));
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
                    }
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
                popup = popup.child(div().px(px(10.)).py(px(8.)).child(footer));
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

    fn options() -> Vec<SelectOption> {
        vec![
            SelectOption::new("daily", "Daily"),
            SelectOption::new("interval", "Interval"),
            SelectOption::new("once", "Once").disabled(true),
        ]
    }

    #[test]
    fn filters_labels_case_insensitively_and_skips_no_options() {
        let mut props = SelectProps {
            options: options(),
            filterable: true,
            ..Default::default()
        };
        assert_eq!(filtered_indices(&props, ""), [0, 1, 2]);
        props.options[0].label = "Daily report".into();
        assert_eq!(filtered_indices(&props, "report"), [0]);
        assert_eq!(filtered_indices(&props, "REPORT"), [0]);
        assert_eq!(filtered_indices(&props, "missing"), Vec::<usize>::new());
    }

    #[test]
    fn selected_labels_fall_back_to_values() {
        let value = SelectValue::Multiple(vec!["daily".into(), "missing".into()]);
        assert_eq!(
            selected_labels(&value, &options()),
            vec![SharedString::from("Daily"), SharedString::from("missing")]
        );
    }

    #[gpui::test]
    fn single_selection_reports_change_and_visibility(cx: &mut gpui::TestAppContext) {
        struct Harness {
            state: gpui::Entity<SelectState>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                div()
            }
        }
        let changes = Rc::new(std::cell::RefCell::new(Vec::new()));
        let visible = Rc::new(std::cell::RefCell::new(Vec::new()));
        let (harness, view) = cx.add_window_view({
            let changes = changes.clone();
            let visible = visible.clone();
            move |_, cx| {
                let state = cx.new(|_| SelectState::default());
                state.update(cx, |state, _| {
                    state.props.options = options();
                    state.handlers.onchange = Some(Rc::new({
                        let changes = changes.clone();
                        move |value, _, _| changes.borrow_mut().push(value)
                    }));
                    state.handlers.onvisiblechange = Some(Rc::new({
                        let visible = visible.clone();
                        move |open, _, _| visible.borrow_mut().push(open)
                    }));
                });
                Harness { state }
            }
        });
        view.update(|window, cx| {
            let state = harness.read(cx).state.clone();
            state.update(cx, |state, cx| state.set_open(true, window, cx));
            state.update(cx, |state, cx| {
                state.select_option(options()[0].clone(), window, cx)
            });
            assert!(!state.read(cx).open);
        });
        assert_eq!(*visible.borrow(), [true, false]);
        assert_eq!(
            *changes.borrow(),
            [SelectValue::Single(Some("daily".into()))]
        );
    }

    #[gpui::test]
    fn multiple_selection_toggles_and_clear_keeps_disabled_values(cx: &mut gpui::TestAppContext) {
        struct Harness {
            state: gpui::Entity<SelectState>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                div()
            }
        }
        let changes = Rc::new(std::cell::RefCell::new(Vec::new()));
        let removed = Rc::new(std::cell::RefCell::new(Vec::new()));
        let (harness, view) = cx.add_window_view({
            let changes = changes.clone();
            let removed = removed.clone();
            move |_, cx| {
                let state = cx.new(|_| SelectState::default());
                state.update(cx, |state, _| {
                    state.props.multiple = true;
                    state.props.multiple_limit = 2;
                    state.props.options = options();
                    state.props.value = SelectValue::Multiple(vec!["daily".into()]);
                    state.handlers.onchange = Some(Rc::new({
                        let changes = changes.clone();
                        move |value, _, _| changes.borrow_mut().push(value)
                    }));
                    state.handlers.onremove = Some(Rc::new({
                        let removed = removed.clone();
                        move |value, _, _| removed.borrow_mut().push(value)
                    }));
                });
                Harness { state }
            }
        });
        view.update(|window, cx| {
            let state = harness.read(cx).state.clone();
            state.update(cx, |state, cx| {
                state.select_option(options()[1].clone(), window, cx)
            });
            state.update(cx, |state, cx| {
                state.select_option(options()[2].clone(), window, cx)
            });
            state.update(cx, |state, cx| {
                state.select_option(options()[0].clone(), window, cx)
            });
            assert_eq!(
                state.read(cx).props.value,
                SelectValue::Multiple(vec!["interval".into()])
            );
            state.update(cx, |state, _| {
                state.props.value = SelectValue::Multiple(vec!["interval".into(), "once".into()]);
            });
            state.update(cx, |state, cx| state.clear(window, cx));
            assert_eq!(
                state.read(cx).props.value,
                SelectValue::Multiple(vec!["once".into()])
            );
        });
        assert_eq!(*removed.borrow(), [SharedString::from("daily")]);
        assert_eq!(changes.borrow().len(), 3);
    }

    #[gpui::test]
    fn rendered_select_supports_keyboard_and_pointer_selection(cx: &mut gpui::TestAppContext) {
        use std::cell::RefCell;
        struct Harness {
            selected: Rc<RefCell<Option<SharedString>>>,
            visible: Rc<RefCell<Vec<bool>>>,
            state: gpui::Entity<SelectState>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let selected = self.selected.clone();
                let visible = self.visible.clone();
                let current = selected.borrow().clone();
                div().p(px(20.)).w(px(240.)).child(
                    el_select()
                        .id("rendered-select")
                        .state(self.state.clone())
                        .value(current)
                        .options(options())
                        .onchange(move |value, _, _| {
                            if let SelectValue::Single(value) = value {
                                *selected.borrow_mut() = value;
                            }
                        })
                        .onvisiblechange(move |open, _, _| visible.borrow_mut().push(open)),
                )
            }
        }
        let selected = Rc::new(RefCell::new(None));
        let visible = Rc::new(RefCell::new(Vec::new()));
        let (harness, view) = cx.add_window_view({
            let selected = selected.clone();
            let visible = visible.clone();
            move |_, cx| Harness {
                selected,
                visible,
                state: cx.new(|_| SelectState::default()),
            }
        });
        let harness_state = view.update(|_, cx| harness.read(cx).state.clone());
        assert!(
            view.update(|_, cx| cx.read_entity(&harness_state, |state, _| state.bounds.is_some())),
            "select bounds were not measured"
        );
        view.simulate_click(gpui::point(px(100.), px(36.)), gpui::Modifiers::default());
        press_key(view, "down");
        press_key(view, "enter");
        assert_eq!(selected.borrow().as_deref(), Some("daily"));
        press_key(view, "down");
        press_key(view, "down");
        press_key(view, "enter");
        assert_eq!(selected.borrow().as_deref(), Some("interval"));
        press_key(view, "down");
        assert!(
            visible.borrow().last() == Some(&true),
            "visibility events: {:?}",
            visible.borrow()
        );
        view.update(|window, cx| window.draw(cx).clear(cx));
        let option = view.update(|_, cx| {
            cx.read_entity(&harness_state, |state, _| {
                state
                    .option_bounds
                    .get(&SharedString::from("daily"))
                    .copied()
                    .map(|bounds| bounds.center())
            })
        });
        let Some(option) = option else {
            panic!(
                "daily option bounds were not measured: root={:?}, popup={:?}, options={:?}",
                view.update(|_, cx| cx.read_entity(&harness_state, |state, _| state.bounds)),
                view.update(|_, cx| cx.read_entity(&harness_state, |state, _| state.popup_bounds)),
                view.update(
                    |_, cx| cx.read_entity(&harness_state, |state, _| state.option_bounds.len())
                ),
            );
        };
        view.simulate_click(option, gpui::Modifiers::default());
        assert_eq!(selected.borrow().as_deref(), Some("daily"));
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
