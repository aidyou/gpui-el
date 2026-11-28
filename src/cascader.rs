//! Element Plus Cascader.
//!
//! Follows `cascader.md`, `cascader.vue`, `cascader.ts`, and Theme Chalk
//! `cascader.scss`: hierarchical options, path values, column expansion,
//! single/multiple leaf selection, filtering, and clearable display.
//!
//! Native differences: this implementation covers static options and leaf or
//! strict parent selection; lazy loading, collapse-tag tooltips, custom
//! rendering, virtual scrolling, and imperative exposes are not mapped.

use std::{rc::Rc, sync::Arc};

use gpui::{
    AnyElement, App, Bounds, ElementId, IntoElement, Pixels, RenderOnce, SharedString, Window,
    anchored, deferred, div, point, prelude::*, px, rgb,
};

use crate::{InputSize, Locale, Theme, el_icon, el_input};

/// One node in a Cascader tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CascaderOption {
    value: SharedString,
    label: SharedString,
    disabled: bool,
    children: Vec<CascaderOption>,
}

impl CascaderOption {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
            children: Vec::new(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = CascaderOption>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub(crate) fn value(&self) -> &SharedString {
        &self.value
    }

    pub(crate) fn label(&self) -> &SharedString {
        &self.label
    }

    pub(crate) fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub(crate) fn child_options(&self) -> &[CascaderOption] {
        &self.children
    }
}

/// Controlled Cascader selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CascaderSelection {
    Single(Vec<SharedString>),
    Multiple(Vec<Vec<SharedString>>),
}

impl Default for CascaderSelection {
    fn default() -> Self {
        Self::Single(Vec::new())
    }
}

type ChangeHandler = Rc<dyn Fn(CascaderSelection, &mut Window, &mut App)>;
type VisibleHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type FocusHandler = Rc<dyn Fn(&mut Window, &mut App)>;
type FilterMethod = Rc<dyn Fn(&CascaderOption, &str) -> bool>;

#[derive(Clone)]
struct CascaderProps {
    selection: CascaderSelection,
    options: Vec<CascaderOption>,
    multiple: bool,
    filterable: bool,
    check_strictly: bool,
    filter_method: Option<FilterMethod>,
    disabled: bool,
}

#[derive(Clone, Default)]
struct CascaderHandlers {
    onchange: Option<ChangeHandler>,
    onvisiblechange: Option<VisibleHandler>,
    onclear: Option<FocusHandler>,
    onfocus: Option<FocusHandler>,
    onblur: Option<FocusHandler>,
}

struct CascaderState {
    open: bool,
    query: String,
    expanded: Vec<SharedString>,
    active: (usize, usize),
    bounds: Option<Bounds<Pixels>>,
    popup_bounds: Option<Bounds<Pixels>>,
    props: CascaderProps,
    handlers: CascaderHandlers,
    last_selection: CascaderSelection,
}

impl Default for CascaderState {
    fn default() -> Self {
        Self {
            open: false,
            query: String::new(),
            expanded: Vec::new(),
            active: (0, 0),
            bounds: None,
            popup_bounds: None,
            props: CascaderProps {
                selection: CascaderSelection::default(),
                options: Vec::new(),
                multiple: false,
                filterable: false,
                check_strictly: false,
                filter_method: None,
                disabled: false,
            },
            handlers: CascaderHandlers::default(),
            last_selection: CascaderSelection::default(),
        }
    }
}

impl CascaderState {
    fn sync_selection(&mut self, selection: CascaderSelection) {
        if self.last_selection == selection {
            return;
        }
        self.last_selection = selection;
        if !self.open {
            self.query.clear();
        }
    }

    fn set_open(&mut self, open: bool, window: &mut Window, cx: &mut App) {
        if self.props.disabled || self.open == open {
            return;
        }
        self.open = open;
        if open {
            self.query.clear();
            let path = match &self.props.selection {
                CascaderSelection::Single(path) if !path.is_empty() => path.clone(),
                CascaderSelection::Multiple(paths) => paths.first().cloned().unwrap_or_default(),
                _ => Vec::new(),
            };
            self.expanded = path
                .iter()
                .take(path.len().saturating_sub(1))
                .cloned()
                .collect();
            self.active = (self.expanded.len(), 0);
        } else {
            self.query.clear();
        }
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
        self.active = (0, 0);
    }

    fn click_node(
        &mut self,
        path: Vec<SharedString>,
        node: CascaderOption,
        window: &mut Window,
        cx: &mut App,
    ) {
        if self.props.disabled || node.disabled {
            return;
        }
        if node.children.is_empty() || self.props.check_strictly {
            self.select_path(path, window, cx);
        } else {
            self.expanded = path;
            self.active = (self.expanded.len(), 0);
        }
    }

    fn select_path(&mut self, path: Vec<SharedString>, window: &mut Window, cx: &mut App) {
        if self.props.disabled {
            return;
        }
        let selection = if self.props.multiple {
            let mut paths = match &self.props.selection {
                CascaderSelection::Multiple(paths) => paths.clone(),
                _ => Vec::new(),
            };
            if let Some(index) = paths.iter().position(|selected| selected == &path) {
                paths.remove(index);
            } else {
                paths.push(path);
            }
            CascaderSelection::Multiple(paths)
        } else {
            self.query.clear();
            self.set_open(false, window, cx);
            CascaderSelection::Single(path)
        };
        self.props.selection = selection.clone();
        self.last_selection = selection.clone();
        if let Some(handler) = self.handlers.onchange.clone() {
            handler(selection, window, cx);
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut App) {
        if self.props.disabled {
            return;
        }
        let selection = if self.props.multiple {
            CascaderSelection::Multiple(Vec::new())
        } else {
            CascaderSelection::Single(Vec::new())
        };
        self.props.selection = selection.clone();
        self.last_selection = selection.clone();
        if let Some(handler) = self.handlers.onchange.clone() {
            handler(selection, window, cx);
        }
        if let Some(handler) = self.handlers.onclear.clone() {
            handler(window, cx);
        }
        self.set_open(false, window, cx);
    }

    fn navigate(&mut self, direction: i32) {
        let nodes = column_nodes(&self.props.options, &self.expanded, self.active.0);
        if nodes.is_empty() {
            self.active.1 = 0;
            return;
        }
        let mut index = self.active.1.min(nodes.len() - 1);
        for _ in 0..nodes.len() {
            index = (index as i32 + direction).rem_euclid(nodes.len() as i32) as usize;
            if nodes.get(index).is_some_and(|(_, node)| !node.disabled) {
                self.active.1 = index;
                return;
            }
        }
    }

    fn key(&mut self, key: &str, window: &mut Window, cx: &mut App) -> bool {
        if self.props.disabled {
            return false;
        }
        if !self.query.is_empty() {
            let paths = filtered_paths(
                &self.props.options,
                &self.query,
                self.props.check_strictly,
                self.props.filter_method.as_ref(),
            );
            match key {
                "up" | "down" => {
                    if !paths.is_empty() {
                        self.active.1 = (self.active.1 as i32 + if key == "down" { 1 } else { -1 })
                            .rem_euclid(paths.len() as i32)
                            as usize;
                    }
                    return true;
                }
                "enter" => {
                    if let Some(path) = paths.get(self.active.1) {
                        self.select_path(path.clone(), window, cx);
                    }
                    return true;
                }
                "escape" => {
                    self.query.clear();
                    self.set_open(false, window, cx);
                    return true;
                }
                _ => return false,
            }
        }
        match key {
            "down" | "up" => {
                if !self.open {
                    self.set_open(true, window, cx);
                }
                self.navigate(if key == "down" { 1 } else { -1 });
                true
            }
            "left" => {
                if self.active.0 > 0 {
                    self.active.0 -= 1;
                    self.expanded.truncate(self.active.0);
                    self.active.1 = 0;
                }
                true
            }
            "right" => {
                let nodes = column_nodes(&self.props.options, &self.expanded, self.active.0);
                if let Some((path, node)) = nodes.get(self.active.1)
                    && !node.children.is_empty()
                    && !node.disabled
                {
                    self.expanded = path.clone();
                    self.active = (self.active.0 + 1, 0);
                }
                true
            }
            "enter" => {
                if !self.open {
                    self.set_open(true, window, cx);
                } else if let Some((path, node)) =
                    column_nodes(&self.props.options, &self.expanded, self.active.0)
                        .get(self.active.1)
                {
                    self.click_node(path.clone(), node.clone(), window, cx);
                }
                true
            }
            "escape" => {
                self.set_open(false, window, cx);
                true
            }
            _ => false,
        }
    }
}

fn column_nodes(
    options: &[CascaderOption],
    expanded: &[SharedString],
    column: usize,
) -> Vec<(Vec<SharedString>, CascaderOption)> {
    let mut nodes: Vec<_> = options
        .iter()
        .cloned()
        .map(|node| (vec![node.value.clone()], node))
        .collect();
    let mut path = Vec::new();
    for value in expanded.iter().take(column) {
        path.push(value.clone());
        let Some((_, node)) = nodes.iter().find(|(_, node)| node.value == *value) else {
            return Vec::new();
        };
        nodes = node
            .children
            .iter()
            .cloned()
            .map(|child| {
                let mut child_path = path.clone();
                child_path.push(child.value.clone());
                (child_path, child)
            })
            .collect();
    }
    nodes
}

fn leaf_paths(
    options: &[CascaderOption],
    prefix: &[SharedString],
    out: &mut Vec<Vec<SharedString>>,
) {
    for option in options {
        let mut path = prefix.to_vec();
        path.push(option.value.clone());
        if option.children.is_empty() {
            out.push(path);
        } else {
            leaf_paths(&option.children, &path, out);
        }
    }
}

fn filtered_paths(
    options: &[CascaderOption],
    query: &str,
    check_strictly: bool,
    filter_method: Option<&FilterMethod>,
) -> Vec<Vec<SharedString>> {
    let query = query.to_lowercase();
    let mut paths = Vec::new();
    if check_strictly {
        all_paths(options, &[], &mut paths);
    } else {
        leaf_paths(options, &[], &mut paths);
    }
    paths.retain(|path| {
        path.iter().enumerate().any(|(index, _)| {
            path.get(..=index)
                .and_then(|prefix| node_for_path(options, prefix))
                .is_some_and(|node| {
                    filter_method.is_some_and(|method| method(node, &query))
                        || node.label.to_lowercase().contains(&query)
                })
        })
    });
    paths
}

fn all_paths(
    options: &[CascaderOption],
    prefix: &[SharedString],
    out: &mut Vec<Vec<SharedString>>,
) {
    for option in options {
        let mut path = prefix.to_vec();
        path.push(option.value.clone());
        out.push(path.clone());
        all_paths(&option.children, &path, out);
    }
}

fn node_for_path<'a>(
    options: &'a [CascaderOption],
    path: &[SharedString],
) -> Option<&'a CascaderOption> {
    let mut nodes = options;
    let mut selected = None;
    for value in path {
        selected = nodes.iter().find(|node| node.value == *value);
        let node = selected?;
        nodes = &node.children;
    }
    selected
}

fn path_labels(path: &[SharedString], options: &[CascaderOption]) -> Vec<SharedString> {
    let mut nodes = options;
    let mut labels = Vec::new();
    for value in path {
        let Some(node) = nodes.iter().find(|node| node.value == *value) else {
            break;
        };
        labels.push(node.label.clone());
        nodes = &node.children;
    }
    labels
}

fn selection_paths(selection: &CascaderSelection) -> Vec<&Vec<SharedString>> {
    match selection {
        CascaderSelection::Single(path) => (!path.is_empty()).then_some(path).into_iter().collect(),
        CascaderSelection::Multiple(paths) => paths.iter().collect(),
    }
}

/// An Element Plus hierarchical selector.
#[derive(IntoElement)]
#[must_use]
pub struct Cascader {
    id: ElementId,
    selection: CascaderSelection,
    options: Vec<CascaderOption>,
    multiple: bool,
    filterable: bool,
    check_strictly: bool,
    filter_method: Option<FilterMethod>,
    show_all_levels: bool,
    separator: SharedString,
    size: InputSize,
    disabled: bool,
    clearable: bool,
    placeholder: Option<SharedString>,
    accessibility_label: Option<SharedString>,
    header: Option<AnyElement>,
    footer: Option<AnyElement>,
    onchange: Option<ChangeHandler>,
    onvisiblechange: Option<VisibleHandler>,
    onclear: Option<FocusHandler>,
    onfocus: Option<FocusHandler>,
    onblur: Option<FocusHandler>,
}

/// Creates a cascader.
#[track_caller]
pub fn el_cascader() -> Cascader {
    Cascader::new()
}

impl Default for Cascader {
    fn default() -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            selection: CascaderSelection::default(),
            options: Vec::new(),
            multiple: false,
            filterable: false,
            check_strictly: false,
            filter_method: None,
            show_all_levels: true,
            separator: " / ".into(),
            size: InputSize::Default,
            disabled: false,
            clearable: false,
            placeholder: None,
            accessibility_label: None,
            header: None,
            footer: None,
            onchange: None,
            onvisiblechange: None,
            onclear: None,
            onfocus: None,
            onblur: None,
        }
    }
}

impl Cascader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn selection(mut self, selection: CascaderSelection) -> Self {
        self.selection = selection;
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

    pub fn filterable(mut self, filterable: bool) -> Self {
        self.filterable = filterable;
        self
    }

    /// Allows parent nodes to be selected directly, like `check-strictly`.
    pub fn check_strictly(mut self, strict: bool) -> Self {
        self.check_strictly = strict;
        self
    }

    /// Overrides the default label/path filtering.
    pub fn filter_method(
        mut self,
        method: impl Fn(&CascaderOption, &str) -> bool + 'static,
    ) -> Self {
        self.filter_method = Some(Rc::new(method));
        self
    }

    pub fn show_all_levels(mut self, show: bool) -> Self {
        self.show_all_levels = show;
        self
    }

    pub fn separator(mut self, separator: impl Into<SharedString>) -> Self {
        self.separator = separator.into();
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

    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    #[track_caller]
    pub fn onchange(
        mut self,
        handler: impl Fn(CascaderSelection, &mut Window, &mut App) + 'static,
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

fn measure(
    state: gpui::Entity<CascaderState>,
    popup: bool,
    control_height: Option<f32>,
) -> AnyElement {
    let canvas = gpui::canvas(
        move |bounds: Bounds<Pixels>, _, cx: &mut App| {
            let bounds = if let Some(height) = control_height {
                gpui::Bounds::new(bounds.origin, gpui::size(bounds.size.width, px(height)))
            } else {
                bounds
            };
            state.update(cx, |state, _| {
                if popup {
                    state.popup_bounds = Some(bounds);
                } else {
                    state.bounds = Some(bounds);
                }
            });
        },
        |_, _, _, _| {},
    );
    if popup {
        canvas.absolute().inset_0().into_any_element()
    } else {
        canvas.w_full().h(px(0.)).into_any_element()
    }
}

impl RenderOnce for Cascader {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Cascader {
            id,
            selection,
            options,
            multiple,
            filterable,
            check_strictly,
            filter_method,
            show_all_levels,
            separator,
            size,
            disabled,
            clearable,
            placeholder,
            accessibility_label,
            header,
            footer,
            onchange,
            onvisiblechange,
            onclear,
            onfocus,
            onblur,
        } = self;
        let props = CascaderProps {
            selection: selection.clone(),
            options: options.clone(),
            multiple,
            filterable,
            check_strictly,
            filter_method,
            disabled,
        };
        let handlers = CascaderHandlers {
            onchange,
            onvisiblechange,
            onclear,
            onfocus,
            onblur,
        };
        let state = window.use_keyed_state(id.clone(), cx, |_, _| CascaderState::default());
        state.update(cx, |state, _| {
            state.props = props.clone();
            state.handlers = handlers;
        });
        state.update(cx, |state, _| state.sync_selection(selection.clone()));
        let open = state.read(cx).open && !disabled;
        let query = state.read(cx).query.clone();
        let display = if !query.is_empty() {
            query.clone()
        } else {
            selection_paths(&selection)
                .iter()
                .map(|path| {
                    let labels = path_labels(path, &options);
                    if show_all_levels {
                        labels.join(separator.as_ref())
                    } else {
                        labels.last().cloned().unwrap_or_default().to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        };
        let locale = Locale::current(cx);
        let theme = Theme::current(cx);
        let mut input = el_input()
            .id(ElementId::NamedChild(Arc::new(id.clone()), "input".into()))
            .value(display)
            .placeholder(placeholder.unwrap_or_else(|| locale.cascader_placeholder().into()))
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
            .child(measure(state.clone(), false, Some(size.height())))
            .child(input)
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
                                gpui::size(px(480.), px(300.)),
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
            let mut panel = div()
                .id(ElementId::NamedChild(Arc::new(id.clone()), "panel".into()))
                .debug_selector(|| "cascader-panel".into())
                .flex()
                .rounded(px(4.))
                .border_1()
                .border_color(rgb(theme.border_lighter()))
                .bg(rgb(theme.overlay_background()))
                .shadow(theme.overlay_shadow())
                .occlude();
            if let Some(header) = header {
                panel = panel.child(div().px(px(10.)).py(px(8.)).border_b_1().child(header));
            }
            if query.is_empty() {
                let column_count = state.read(cx).expanded.len() + 1;
                for column in 0..column_count {
                    let nodes = column_nodes(&options, &state.read(cx).expanded, column);
                    let mut list = div()
                        .id(ElementId::NamedChild(
                            Arc::new(id.clone()),
                            format!("column-{column}").into(),
                        ))
                        .w(px(160.))
                        .max_h(px(274.))
                        .py(px(6.))
                        .overflow_y_scroll()
                        .on_scroll(|_, _, cx| cx.stop_propagation());
                    for (index, (path, node)) in nodes.into_iter().enumerate() {
                        let selected = selection_paths(&selection).iter().any(|selected| {
                            *selected == &path
                                || selected.starts_with(&path) && !node.children.is_empty()
                        });
                        let active = state.read(cx).active == (column, index);
                        let mut row = div()
                            .id(ElementId::NamedChild(
                                Arc::new(id.clone()),
                                format!("node-{column}-{index}").into(),
                            ))
                            .h(px(34.))
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .px(px(12.))
                            .text_size(px(14.))
                            .whitespace_nowrap()
                            .truncate()
                            .text_color(if node.disabled {
                                rgb(theme.disabled_text())
                            } else if selected {
                                rgb(crate::theme::PRIMARY)
                            } else {
                                rgb(theme.regular_text())
                            })
                            .when(active && !node.disabled, |el| {
                                el.bg(rgb(theme.fill(crate::theme::Fill::Light)))
                            })
                            .when(!node.disabled, |el| {
                                el.cursor_pointer().hover(|style| {
                                    style.bg(rgb(theme.fill(crate::theme::Fill::Light)))
                                })
                            })
                            .child(node.label.clone());
                        if !node.children.is_empty() {
                            row = row.child(div().flex_1()).child(
                                el_icon("arrow-right")
                                    .size(12.)
                                    .color(rgb(theme.secondary_text())),
                            );
                        } else if multiple && selected {
                            row = row.child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .justify_end()
                                    .child(el_icon("check").size(12.)),
                            );
                        }
                        row = row.on_click({
                            let state = state.clone();
                            let path = path.clone();
                            let node = node.clone();
                            move |_, window, cx| {
                                cx.stop_propagation();
                                state.update(cx, |state, cx| {
                                    state.click_node(path.clone(), node.clone(), window, cx)
                                });
                                window.refresh();
                            }
                        });
                        list = list.child(row);
                    }
                    panel = panel.child(list);
                }
            } else {
                let paths = filtered_paths(
                    &options,
                    &query,
                    props.check_strictly,
                    props.filter_method.as_ref(),
                );
                let mut list = div()
                    .id(ElementId::NamedChild(
                        Arc::new(id.clone()),
                        "results".into(),
                    ))
                    .w(px(280.))
                    .max_h(px(274.))
                    .py(px(6.))
                    .overflow_y_scroll()
                    .on_scroll(|_, _, cx| cx.stop_propagation());
                for (index, path) in paths.into_iter().enumerate() {
                    let labels = path_labels(&path, &options).join(separator.as_ref());
                    let active = state.read(cx).active.1 == index;
                    list = list.child(
                        div()
                            .id(ElementId::NamedChild(
                                Arc::new(id.clone()),
                                format!("result-{index}").into(),
                            ))
                            .h(px(34.))
                            .flex()
                            .items_center()
                            .px(px(12.))
                            .text_size(px(14.))
                            .whitespace_nowrap()
                            .truncate()
                            .cursor_pointer()
                            .when(active, |el| {
                                el.bg(rgb(theme.fill(crate::theme::Fill::Light)))
                            })
                            .child(labels)
                            .on_click({
                                let state = state.clone();
                                let path = path.clone();
                                move |_, window, cx| {
                                    cx.stop_propagation();
                                    state.update(cx, |state, cx| {
                                        state.select_path(path.clone(), window, cx)
                                    });
                                    window.refresh();
                                }
                            }),
                    );
                }
                panel = panel.child(list);
            }
            if let Some(footer) = footer {
                panel = panel.child(div().px(px(10.)).py(px(8.)).border_t_1().child(footer));
            }
            panel = panel.child(measure(state.clone(), true, None));
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
        vec![CascaderOption::new("guide", "Guide").children([
            CascaderOption::new("components", "Components").children([
                CascaderOption::new("button", "Button"),
                CascaderOption::new("input", "Input"),
            ]),
            CascaderOption::new("layout", "Layout"),
        ])]
    }

    #[test]
    fn columns_follow_the_expanded_path() {
        let roots = column_nodes(&options(), &[], 0);
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].1.label, "Guide");
        let children = column_nodes(&options(), &["guide".into()], 1);
        assert_eq!(children.len(), 2);
        assert_eq!(
            children[0].0,
            vec![
                SharedString::from("guide"),
                SharedString::from("components")
            ]
        );
        let leaves = column_nodes(&options(), &["guide".into(), "components".into()], 2);
        assert_eq!(leaves.len(), 2);
        assert_eq!(leaves[1].1.label, "Input");
    }

    #[test]
    fn filtering_matches_parent_labels_and_returns_leaf_paths() {
        assert_eq!(
            filtered_paths(&options(), "component", false, None),
            [
                vec![
                    SharedString::from("guide"),
                    SharedString::from("components"),
                    SharedString::from("button"),
                ],
                vec![
                    SharedString::from("guide"),
                    SharedString::from("components"),
                    SharedString::from("input"),
                ],
            ]
        );
        assert_eq!(
            filtered_paths(&options(), "input", false, None),
            [vec![
                SharedString::from("guide"),
                SharedString::from("components"),
                SharedString::from("input"),
            ]]
        );
    }

    #[test]
    fn path_labels_include_every_nested_level() {
        assert_eq!(
            path_labels(
                &["guide".into(), "components".into(), "input".into()],
                &options(),
            ),
            ["Guide", "Components", "Input"]
        );
    }

    #[test]
    fn strict_filtering_can_select_parents_and_use_custom_methods() {
        let strict = filtered_paths(&options(), "component", true, None);
        assert!(strict.contains(&vec!["guide".into(), "components".into()]));
        let method: FilterMethod = Rc::new(|node: &CascaderOption, query: &str| {
            node.value.as_ref() == "button" && query == "custom"
        });
        assert_eq!(
            filtered_paths(&options(), "custom", false, Some(&method)),
            [vec![
                SharedString::from("guide"),
                SharedString::from("components"),
                SharedString::from("button"),
            ]]
        );
    }

    #[gpui::test]
    fn leaf_selection_reports_the_full_path(cx: &mut gpui::TestAppContext) {
        struct Harness {
            state: gpui::Entity<CascaderState>,
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
                let state = cx.new(|_| CascaderState::default());
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
                state.click_node(
                    vec!["guide".into(), "components".into(), "input".into()],
                    CascaderOption::new("input", "Input"),
                    window,
                    cx,
                )
            });
            assert!(!state.read(cx).open);
        });
        assert_eq!(
            *changes.borrow(),
            [CascaderSelection::Single(vec![
                "guide".into(),
                "components".into(),
                "input".into(),
            ])]
        );
    }
}
