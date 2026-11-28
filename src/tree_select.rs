//! Element Plus TreeSelect.
//!
//! Follows `tree-select.md`, `tree-select.vue`, `tree-select/src`, and Theme
//! Chalk `tree-select.scss`: a Select-like trigger over an expandable tree,
//! single/multiple values, disabled nodes, filtering, and clearable display.
//!
//! Native differences: this implementation covers static trees and node-key
//! values; lazy loading, cache-data, custom node rendering, virtual scrolling,
//! and imperative tree/select exposes are not mapped.

use std::{collections::BTreeSet, rc::Rc, sync::Arc};

use gpui::{
    AnyElement, App, Bounds, ElementId, IntoElement, Pixels, RenderOnce, SharedString, Window,
    anchored, deferred, div, point, prelude::*, px, rgb,
};

use crate::{CascaderOption, InputSize, Locale, Theme, el_icon, el_input};

/// Controlled TreeSelect value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TreeSelectValue {
    Single(Option<SharedString>),
    Multiple(Vec<SharedString>),
}

impl Default for TreeSelectValue {
    fn default() -> Self {
        Self::Single(None)
    }
}

type ChangeHandler = Rc<dyn Fn(TreeSelectValue, &mut Window, &mut App)>;
type VisibleHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type FocusHandler = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Clone)]
struct TreeSelectProps {
    value: TreeSelectValue,
    options: Vec<CascaderOption>,
    multiple: bool,
    check_strictly: bool,
    filterable: bool,
    disabled: bool,
}

#[derive(Clone, Default)]
struct TreeSelectHandlers {
    onchange: Option<ChangeHandler>,
    onvisiblechange: Option<VisibleHandler>,
    onclear: Option<FocusHandler>,
    onfocus: Option<FocusHandler>,
    onblur: Option<FocusHandler>,
}

struct TreeSelectState {
    open: bool,
    query: String,
    expanded: BTreeSet<SharedString>,
    active: usize,
    bounds: Option<Bounds<Pixels>>,
    popup_bounds: Option<Bounds<Pixels>>,
    props: TreeSelectProps,
    handlers: TreeSelectHandlers,
    last_value: TreeSelectValue,
}

impl Default for TreeSelectState {
    fn default() -> Self {
        Self {
            open: false,
            query: String::new(),
            expanded: BTreeSet::new(),
            active: 0,
            bounds: None,
            popup_bounds: None,
            props: TreeSelectProps {
                value: TreeSelectValue::default(),
                options: Vec::new(),
                multiple: false,
                check_strictly: false,
                filterable: false,
                disabled: false,
            },
            handlers: TreeSelectHandlers::default(),
            last_value: TreeSelectValue::default(),
        }
    }
}

impl TreeSelectState {
    fn sync_value(&mut self, value: TreeSelectValue) {
        if self.last_value == value {
            return;
        }
        self.last_value = value;
        if !self.open {
            self.query.clear();
        }
    }

    fn set_open(&mut self, open: bool, window: &mut Window, cx: &mut App) {
        if self.props.disabled || self.open == open {
            return;
        }
        self.open = open;
        self.active = 0;
        self.query.clear();
        if let Some(handler) = self.handlers.onvisiblechange.clone() {
            handler(open, window, cx);
        }
    }

    fn accept_input(&mut self, query: SharedString) {
        if self.props.disabled || !self.props.filterable {
            return;
        }
        self.query = query.to_string();
        self.open = true;
        self.active = 0;
    }

    fn toggle_expand(&mut self, value: SharedString) {
        if self.props.disabled {
            return;
        }
        if self.expanded.contains(&value) {
            self.expanded.remove(&value);
        } else {
            self.expanded.insert(value);
        }
    }

    fn click_row(&mut self, row: TreeRow, window: &mut Window, cx: &mut App) {
        if self.props.disabled || row.disabled {
            return;
        }
        if row.has_children && !self.props.check_strictly {
            self.toggle_expand(row.value);
        } else {
            self.select_value(row.value, window, cx);
        }
    }

    fn select_value(&mut self, value: SharedString, window: &mut Window, cx: &mut App) {
        if self.props.disabled {
            return;
        }
        let selection = if self.props.multiple {
            let mut values = match &self.props.value {
                TreeSelectValue::Multiple(values) => values.clone(),
                _ => Vec::new(),
            };
            if let Some(index) = values.iter().position(|selected| selected == &value) {
                values.remove(index);
            } else {
                values.push(value);
            }
            TreeSelectValue::Multiple(values)
        } else {
            self.query.clear();
            self.set_open(false, window, cx);
            TreeSelectValue::Single(Some(value))
        };
        self.props.value = selection.clone();
        self.last_value = selection.clone();
        if let Some(handler) = self.handlers.onchange.clone() {
            handler(selection, window, cx);
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut App) {
        if self.props.disabled {
            return;
        }
        let selection = if self.props.multiple {
            TreeSelectValue::Multiple(Vec::new())
        } else {
            TreeSelectValue::Single(None)
        };
        self.props.value = selection.clone();
        self.last_value = selection.clone();
        if let Some(handler) = self.handlers.onchange.clone() {
            handler(selection, window, cx);
        }
        if let Some(handler) = self.handlers.onclear.clone() {
            handler(window, cx);
        }
        self.set_open(false, window, cx);
    }

    fn rows(&self) -> Vec<TreeRow> {
        if self.query.is_empty() {
            let mut rows = Vec::new();
            flatten_visible(&self.props.options, &self.expanded, 0, &[], &mut rows);
            rows
        } else {
            let query = self.query.to_lowercase();
            let mut rows = Vec::new();
            flatten_all(&self.props.options, 0, &[], &mut rows);
            rows.retain(|row| row.label.to_lowercase().contains(&query));
            rows
        }
    }

    fn key(&mut self, key: &str, window: &mut Window, cx: &mut App) -> bool {
        if self.props.disabled {
            return false;
        }
        let rows = self.rows();
        match key {
            "down" | "up" => {
                if !self.open {
                    self.set_open(true, window, cx);
                }
                if !rows.is_empty() {
                    self.active = (self.active as i32 + if key == "down" { 1 } else { -1 })
                        .rem_euclid(rows.len() as i32) as usize;
                }
                true
            }
            "right" => {
                if let Some(row) = rows.get(self.active)
                    && row.has_children
                    && !row.disabled
                {
                    self.expanded.insert(row.value.clone());
                }
                true
            }
            "left" => {
                if let Some(row) = rows.get(self.active) {
                    if row.has_children && self.expanded.contains(&row.value) {
                        self.expanded.remove(&row.value);
                    } else if let Some(parent) = row
                        .path
                        .get(..row.path.len().saturating_sub(1))
                        .and_then(|path| path.last())
                    {
                        self.expanded.remove(parent);
                        self.active = rows
                            .iter()
                            .position(|candidate| candidate.value == *parent)
                            .unwrap_or(self.active);
                    }
                }
                true
            }
            "enter" => {
                if !self.open {
                    self.set_open(true, window, cx);
                } else if let Some(row) = rows.get(self.active) {
                    self.click_row(row.clone(), window, cx);
                }
                true
            }
            "escape" => {
                self.query.clear();
                self.set_open(false, window, cx);
                true
            }
            _ => false,
        }
    }
}

#[derive(Clone)]
struct TreeRow {
    value: SharedString,
    label: SharedString,
    disabled: bool,
    has_children: bool,
    depth: usize,
    path: Vec<SharedString>,
}

fn flatten_visible(
    options: &[CascaderOption],
    expanded: &BTreeSet<SharedString>,
    depth: usize,
    prefix: &[SharedString],
    out: &mut Vec<TreeRow>,
) {
    for option in options {
        let mut path = prefix.to_vec();
        path.push(option.value().clone());
        out.push(TreeRow {
            value: option.value().clone(),
            label: option.label().clone(),
            disabled: option.is_disabled(),
            has_children: !option.child_options().is_empty(),
            depth,
            path: path.clone(),
        });
        if !option.child_options().is_empty() && expanded.contains(option.value()) {
            flatten_visible(option.child_options(), expanded, depth + 1, &path, out);
        }
    }
}

fn flatten_all(
    options: &[CascaderOption],
    depth: usize,
    prefix: &[SharedString],
    out: &mut Vec<TreeRow>,
) {
    for option in options {
        let mut path = prefix.to_vec();
        path.push(option.value().clone());
        out.push(TreeRow {
            value: option.value().clone(),
            label: option.label().clone(),
            disabled: option.is_disabled(),
            has_children: !option.child_options().is_empty(),
            depth,
            path: path.clone(),
        });
        flatten_all(option.child_options(), depth + 1, &path, out);
    }
}

fn find_node<'a>(
    options: &'a [CascaderOption],
    value: &SharedString,
) -> Option<&'a CascaderOption> {
    options.iter().find_map(|option| {
        if option.value() == value {
            Some(option)
        } else {
            find_node(option.child_options(), value)
        }
    })
}

fn selected_values(value: &TreeSelectValue) -> Vec<&SharedString> {
    match value {
        TreeSelectValue::Single(value) => value.iter().collect(),
        TreeSelectValue::Multiple(values) => values.iter().collect(),
    }
}

/// An Element Plus tree selector.
#[derive(IntoElement)]
#[must_use]
pub struct TreeSelect {
    id: ElementId,
    value: TreeSelectValue,
    options: Vec<CascaderOption>,
    multiple: bool,
    check_strictly: bool,
    filterable: bool,
    size: InputSize,
    disabled: bool,
    clearable: bool,
    placeholder: Option<SharedString>,
    accessibility_label: Option<SharedString>,
    onchange: Option<ChangeHandler>,
    onvisiblechange: Option<VisibleHandler>,
    onclear: Option<FocusHandler>,
    onfocus: Option<FocusHandler>,
    onblur: Option<FocusHandler>,
}

/// Creates a tree selector.
#[track_caller]
pub fn el_tree_select() -> TreeSelect {
    TreeSelect::new()
}

impl Default for TreeSelect {
    fn default() -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            value: TreeSelectValue::default(),
            options: Vec::new(),
            multiple: false,
            check_strictly: false,
            filterable: false,
            size: InputSize::Default,
            disabled: false,
            clearable: false,
            placeholder: None,
            accessibility_label: None,
            onchange: None,
            onvisiblechange: None,
            onclear: None,
            onfocus: None,
            onblur: None,
        }
    }
}

impl TreeSelect {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn value(mut self, value: TreeSelectValue) -> Self {
        self.value = value;
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = CascaderOption>) -> Self {
        self.options = options.into_iter().collect();
        self
    }

    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    pub fn check_strictly(mut self, strict: bool) -> Self {
        self.check_strictly = strict;
        self
    }

    pub fn filterable(mut self, filterable: bool) -> Self {
        self.filterable = filterable;
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

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    #[track_caller]
    pub fn onchange(
        mut self,
        handler: impl Fn(TreeSelectValue, &mut Window, &mut App) + 'static,
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

fn measure(state: gpui::Entity<TreeSelectState>, popup: bool) -> AnyElement {
    gpui::canvas(
        move |bounds: Bounds<Pixels>, _, cx: &mut App| {
            state.update(cx, |state, _| {
                if popup {
                    state.popup_bounds = Some(bounds);
                } else {
                    state.bounds = Some(bounds);
                }
            });
        },
        |_, _, _, _| {},
    )
    .absolute()
    .inset_0()
    .into_any_element()
}

impl RenderOnce for TreeSelect {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let TreeSelect {
            id,
            value,
            options,
            multiple,
            check_strictly,
            filterable,
            size,
            disabled,
            clearable,
            placeholder,
            accessibility_label,
            onchange,
            onvisiblechange,
            onclear,
            onfocus,
            onblur,
        } = self;
        let props = TreeSelectProps {
            value: value.clone(),
            options: options.clone(),
            multiple,
            check_strictly,
            filterable,
            disabled,
        };
        let handlers = TreeSelectHandlers {
            onchange,
            onvisiblechange,
            onclear,
            onfocus,
            onblur,
        };
        let state = window.use_keyed_state(id.clone(), cx, |_, _| TreeSelectState::default());
        state.update(cx, |state, _| {
            state.props = props.clone();
            state.handlers = handlers;
        });
        state.update(cx, |state, _| state.sync_value(value.clone()));
        let open = state.read(cx).open && !disabled;
        let query = state.read(cx).query.clone();
        let display = if !query.is_empty() {
            query.clone()
        } else {
            selected_values(&value)
                .iter()
                .filter_map(|value| find_node(&options, value).map(|node| node.label().clone()))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let locale = Locale::current(cx);
        let theme = Theme::current(cx);
        let mut input = el_input()
            .id(ElementId::NamedChild(Arc::new(id.clone()), "input".into()))
            .value(display)
            .placeholder(placeholder.unwrap_or_else(|| locale.tree_select_placeholder().into()))
            .size(size)
            .radius_small()
            .disabled(disabled)
            .readonly(!filterable)
            .clearable(clearable)
            .suffix(
                div()
                    .flex()
                    .items_center()
                    .text_color(rgb(theme.disabled_text()))
                    .child(
                        el_icon("arrow-down")
                            .size(size.font_size())
                            .rotate(if open { std::f32::consts::PI } else { 0. }),
                    ),
            )
            .oninput({
                let state = state.clone();
                move |reported, _, cx| state.update(cx, |state, _| state.accept_input(reported))
            })
            .on_mouse_down_filter({
                let state = state.clone();
                move |_, window, cx| {
                    if !disabled {
                        let open = !state.read(cx).open;
                        state.update(cx, |state, cx| state.set_open(open, window, cx));
                        window.refresh();
                    }
                    false
                }
            })
            .onfocus({
                let state = state.clone();
                move |window, cx| {
                    state.update(cx, |state, cx| {
                        if let Some(handler) = state.handlers.onfocus.clone() {
                            handler(window, cx);
                        }
                    });
                }
            })
            .onblur({
                let state = state.clone();
                move |window, cx| {
                    state.update(cx, |state, cx| {
                        if let Some(handler) = state.handlers.onblur.clone() {
                            handler(window, cx);
                        }
                    });
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
            });
        if let Some(label) = accessibility_label {
            input = input.aria_label(label);
        }
        let mut root = div()
            .id(id.clone())
            .relative()
            .w_full()
            .child(input)
            .child(measure(state.clone(), false))
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
                                gpui::size(px(280.), px(300.)),
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
            let below = viewport.height - target.bottom() >= px(304.) || target.top() < px(304.);
            let rows = state.read(cx).rows();
            let mut panel = div()
                .id(ElementId::NamedChild(Arc::new(id.clone()), "panel".into()))
                .w(px(280.))
                .max_h(px(300.))
                .py(px(6.))
                .rounded(px(4.))
                .border_1()
                .border_color(rgb(theme.border_lighter()))
                .bg(rgb(theme.overlay_background()))
                .shadow(theme.overlay_shadow())
                .overflow_y_scroll()
                .occlude()
                .on_scroll(|_, _, cx| cx.stop_propagation());
            for (index, row) in rows.into_iter().enumerate() {
                let selected = selected_values(&value).contains(&&row.value);
                let active = state.read(cx).active == index;
                let expanded = state.read(cx).expanded.contains(&row.value);
                let mut node = div()
                    .id(ElementId::NamedChild(
                        Arc::new(id.clone()),
                        format!("node-{index}").into(),
                    ))
                    .h(px(28.))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .pl(px(12. + row.depth as f32 * 16.))
                    .pr(px(12.))
                    .text_size(px(14.))
                    .whitespace_nowrap()
                    .truncate()
                    .text_color(if row.disabled {
                        rgb(theme.disabled_text())
                    } else if selected {
                        rgb(crate::theme::PRIMARY)
                    } else {
                        rgb(theme.regular_text())
                    })
                    .when(active && !row.disabled, |el| {
                        el.bg(rgb(theme.fill(crate::theme::Fill::Light)))
                    })
                    .when(!row.disabled, |el| {
                        el.cursor_pointer()
                            .hover(|style| style.bg(rgb(theme.fill(crate::theme::Fill::Light))))
                    })
                    .child(row.label.clone());
                if row.has_children {
                    node = node.child(
                        div()
                            .id(ElementId::NamedChild(
                                Arc::new(id.clone()),
                                format!("expand-{index}").into(),
                            ))
                            .w(px(16.))
                            .h(px(16.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .child(
                                el_icon(if expanded {
                                    "arrow-down"
                                } else {
                                    "arrow-right"
                                })
                                .size(12.)
                                .color(rgb(theme.secondary_text())),
                            )
                            .on_click({
                                let state = state.clone();
                                let value = row.value.clone();
                                move |_, window, cx| {
                                    cx.stop_propagation();
                                    state.update(cx, |state, _| state.toggle_expand(value.clone()));
                                    window.refresh();
                                }
                            }),
                    );
                } else if multiple && selected {
                    node = node.child(
                        div()
                            .flex_1()
                            .flex()
                            .justify_end()
                            .child(el_icon("check").size(12.)),
                    );
                }
                node = node.on_click({
                    let state = state.clone();
                    let row = row.clone();
                    move |_, window, cx| {
                        cx.stop_propagation();
                        state.update(cx, |state, cx| state.click_row(row.clone(), window, cx));
                        window.refresh();
                    }
                });
                panel = panel.child(node);
            }
            panel = panel.child(measure(state.clone(), true));
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
                        .child(panel),
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

    fn options() -> Vec<CascaderOption> {
        vec![CascaderOption::new("root", "Root").children([
            CascaderOption::new("branch", "Branch").children([CascaderOption::new("leaf", "Leaf")]),
            CascaderOption::new("disabled", "Disabled").disabled(true),
        ])]
    }

    #[test]
    fn visible_rows_follow_expanded_nodes() {
        let mut expanded = BTreeSet::new();
        let mut rows = Vec::new();
        flatten_visible(&options(), &expanded, 0, &[], &mut rows);
        assert_eq!(rows.len(), 1);
        expanded.insert("root".into());
        rows.clear();
        flatten_visible(&options(), &expanded, 0, &[], &mut rows);
        assert_eq!(rows.len(), 3);
        expanded.insert("branch".into());
        rows.clear();
        flatten_visible(&options(), &expanded, 0, &[], &mut rows);
        assert_eq!(rows.len(), 4);
        assert_eq!(rows[2].depth, 2);
        assert_eq!(rows[2].label, "Leaf");
    }

    #[test]
    fn node_lookup_and_selection_values_are_stable() {
        assert_eq!(
            find_node(&options(), &"leaf".into()).unwrap().label(),
            "Leaf"
        );
        assert!(find_node(&options(), &"missing".into()).is_none());
        let value = TreeSelectValue::Multiple(vec!["root".into(), "leaf".into()]);
        assert_eq!(selected_values(&value), ["root", "leaf"]);
    }

    #[gpui::test]
    fn selecting_a_leaf_reports_and_closes(cx: &mut gpui::TestAppContext) {
        struct Harness {
            state: gpui::Entity<TreeSelectState>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                div()
            }
        }
        let changes = Rc::new(std::cell::RefCell::new(Vec::new()));
        let (harness, view) = cx.add_window_view({
            let changes = changes.clone();
            move |_, cx| {
                let state = cx.new(|_| TreeSelectState::default());
                state.update(cx, |state, _| {
                    state.props.options = options();
                    state.handlers.onchange = Some(Rc::new({
                        let changes = changes.clone();
                        move |value, _, _| changes.borrow_mut().push(value)
                    }));
                });
                Harness { state }
            }
        });
        view.update(|window, cx| {
            let state = harness.read(cx).state.clone();
            state.update(cx, |state, cx| state.set_open(true, window, cx));
            state.update(cx, |state, cx| {
                state.select_value("leaf".into(), window, cx)
            });
            assert!(!state.read(cx).open);
        });
        assert_eq!(
            *changes.borrow(),
            [TreeSelectValue::Single(Some("leaf".into()))]
        );
    }
}
