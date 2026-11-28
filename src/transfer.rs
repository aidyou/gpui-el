//! Element Plus Transfer.
//!
//! Follows `transfer.md`, `transfer.vue`, `transfer.ts`, and Theme Chalk
//! `transfer.scss`: two checkable lists, controlled target keys, batch movement,
//! filtering, disabled items, counts, and change/check-change events.
//!
//! Native differences: this implementation covers static data and text labels;
//! prop aliases, custom item rendering, virtual scrolling, and imperative
//! panel/query exposes are not mapped.

use std::{collections::BTreeSet, rc::Rc, sync::Arc};

use gpui::{
    AnyElement, App, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px,
    rgb,
};

use crate::{Locale, Theme, el_button, el_checkbox, el_icon, el_input};

/// One Transfer data item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferItem {
    key: SharedString,
    label: SharedString,
    disabled: bool,
}

impl TransferItem {
    pub fn new(key: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn key(&self) -> &SharedString {
        &self.key
    }

    pub fn label(&self) -> &SharedString {
        &self.label
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

/// Direction of a Transfer move.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferDirection {
    Left,
    Right,
}

/// Ordering of newly moved target items.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TransferTargetOrder {
    #[default]
    Original,
    Push,
    Unshift,
}

type ChangeHandler =
    Rc<dyn Fn(Vec<SharedString>, TransferDirection, Vec<SharedString>, &mut Window, &mut App)>;
type CheckHandler = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut App)>;
type FilterMethod = Rc<dyn Fn(&str, &TransferItem) -> bool>;

#[derive(Clone)]
struct TransferProps {
    data: Vec<TransferItem>,
    value: Vec<SharedString>,
    target_order: TransferTargetOrder,
    titles: [SharedString; 2],
    button_texts: [SharedString; 2],
    filter_method: Option<FilterMethod>,
}

#[derive(Clone, Default)]
struct TransferHandlers {
    onchange: Option<ChangeHandler>,
    onleftcheckchange: Option<CheckHandler>,
    onrightcheckchange: Option<CheckHandler>,
}

struct TransferState {
    left_checked: BTreeSet<SharedString>,
    right_checked: BTreeSet<SharedString>,
    left_query: String,
    right_query: String,
    props: TransferProps,
    handlers: TransferHandlers,
    last_value: Vec<SharedString>,
}

impl Default for TransferState {
    fn default() -> Self {
        Self {
            left_checked: BTreeSet::new(),
            right_checked: BTreeSet::new(),
            left_query: String::new(),
            right_query: String::new(),
            props: TransferProps {
                data: Vec::new(),
                value: Vec::new(),
                target_order: TransferTargetOrder::Original,
                titles: ["List".into(), "Selected".into()],
                button_texts: ["Move right".into(), "Move left".into()],
                filter_method: None,
            },
            handlers: TransferHandlers::default(),
            last_value: Vec::new(),
        }
    }
}

impl TransferState {
    fn sync_value(&mut self, value: Vec<SharedString>) {
        if self.last_value == value {
            return;
        }
        self.last_value = value.clone();
        self.left_checked.retain(|key| !value.contains(key));
        self.right_checked.retain(|key| value.contains(key));
    }

    fn visible(&self, right: bool) -> Vec<TransferItem> {
        let query = if right {
            &self.right_query
        } else {
            &self.left_query
        };
        self.props
            .data
            .iter()
            .filter(|item| self.props.value.contains(&item.key) == right)
            .filter(|item| {
                query.is_empty()
                    || self
                        .props
                        .filter_method
                        .as_ref()
                        .is_some_and(|method| method(query, item))
                    || item.label.to_lowercase().contains(&query.to_lowercase())
            })
            .cloned()
            .collect()
    }

    fn checked(&self, right: bool) -> &BTreeSet<SharedString> {
        if right {
            &self.right_checked
        } else {
            &self.left_checked
        }
    }

    fn checked_mut(&mut self, right: bool) -> &mut BTreeSet<SharedString> {
        if right {
            &mut self.right_checked
        } else {
            &mut self.left_checked
        }
    }

    fn toggle(&mut self, right: bool, key: SharedString, window: &mut Window, cx: &mut App) {
        let checked = self.checked_mut(right);
        if checked.contains(&key) {
            checked.remove(&key);
        } else {
            checked.insert(key);
        }
        let values = self.checked(right).iter().cloned().collect::<Vec<_>>();
        let handler = if right {
            self.handlers.onrightcheckchange.clone()
        } else {
            self.handlers.onleftcheckchange.clone()
        };
        if let Some(handler) = handler {
            handler(values, window, cx);
        }
    }

    fn toggle_all(&mut self, right: bool, checked: bool, window: &mut Window, cx: &mut App) {
        let keys = self
            .visible(right)
            .into_iter()
            .filter(|item| !item.disabled)
            .map(|item| item.key)
            .collect::<Vec<_>>();
        let selected = self.checked_mut(right);
        for key in keys {
            if checked {
                selected.insert(key);
            } else {
                selected.remove(&key);
            }
        }
        let values = self.checked(right).iter().cloned().collect::<Vec<_>>();
        let handler = if right {
            self.handlers.onrightcheckchange.clone()
        } else {
            self.handlers.onleftcheckchange.clone()
        };
        if let Some(handler) = handler {
            handler(values, window, cx);
        }
    }

    fn move_to(&mut self, right: bool, window: &mut Window, cx: &mut App) {
        let source_checked = self.checked(!right).clone();
        let moved = self
            .props
            .data
            .iter()
            .filter(|item| source_checked.contains(&item.key) && !item.disabled)
            .map(|item| item.key.clone())
            .collect::<Vec<_>>();
        if moved.is_empty() {
            return;
        }
        let mut value = if right {
            let mut value = self.props.value.clone();
            match self.props.target_order {
                TransferTargetOrder::Original => {
                    value.extend(moved.iter().cloned());
                    value.sort_by_key(|key| {
                        self.props
                            .data
                            .iter()
                            .position(|item| item.key == *key)
                            .unwrap_or(usize::MAX)
                    });
                    value
                }
                TransferTargetOrder::Push => {
                    value.extend(moved.iter().cloned());
                    value
                }
                TransferTargetOrder::Unshift => {
                    let mut next = moved.clone();
                    next.extend(value);
                    next
                }
            }
        } else {
            self.props
                .value
                .iter()
                .filter(|key| !moved.contains(key))
                .cloned()
                .collect()
        };
        value.dedup();
        self.props.value = value.clone();
        self.last_value = value.clone();
        self.checked_mut(!right).clear();
        let direction = if right {
            TransferDirection::Right
        } else {
            TransferDirection::Left
        };
        if let Some(handler) = self.handlers.onchange.clone() {
            handler(value, direction, moved, window, cx);
        }
    }
}

/// An Element Plus two-list transfer control.
#[derive(IntoElement)]
#[must_use]
pub struct Transfer {
    id: ElementId,
    data: Vec<TransferItem>,
    value: Vec<SharedString>,
    filterable: bool,
    target_order: TransferTargetOrder,
    titles: Option<[SharedString; 2]>,
    button_texts: Option<[SharedString; 2]>,
    filter_placeholder: Option<SharedString>,
    filter_method: Option<FilterMethod>,
    left_footer: Option<AnyElement>,
    right_footer: Option<AnyElement>,
    left_empty: Option<AnyElement>,
    right_empty: Option<AnyElement>,
    onchange: Option<ChangeHandler>,
    onleftcheckchange: Option<CheckHandler>,
    onrightcheckchange: Option<CheckHandler>,
}

/// Creates a transfer control.
#[track_caller]
pub fn el_transfer() -> Transfer {
    Transfer::new()
}

impl Default for Transfer {
    fn default() -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            data: Vec::new(),
            value: Vec::new(),
            filterable: false,
            target_order: TransferTargetOrder::Original,
            titles: None,
            button_texts: None,
            filter_placeholder: None,
            filter_method: None,
            left_footer: None,
            right_footer: None,
            left_empty: None,
            right_empty: None,
            onchange: None,
            onleftcheckchange: None,
            onrightcheckchange: None,
        }
    }
}

impl Transfer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn data(mut self, data: impl IntoIterator<Item = TransferItem>) -> Self {
        self.data = data.into_iter().collect();
        self
    }

    pub fn value(mut self, value: impl IntoIterator<Item = SharedString>) -> Self {
        self.value = value.into_iter().collect();
        self
    }

    pub fn filterable(mut self, filterable: bool) -> Self {
        self.filterable = filterable;
        self
    }

    pub fn target_order(mut self, order: TransferTargetOrder) -> Self {
        self.target_order = order;
        self
    }

    pub fn titles(mut self, titles: [SharedString; 2]) -> Self {
        self.titles = Some(titles);
        self
    }

    pub fn button_texts(mut self, texts: [SharedString; 2]) -> Self {
        self.button_texts = Some(texts);
        self
    }

    pub fn filter_placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.filter_placeholder = Some(placeholder.into());
        self
    }

    pub fn filter_method(mut self, method: impl Fn(&str, &TransferItem) -> bool + 'static) -> Self {
        self.filter_method = Some(Rc::new(method));
        self
    }

    pub fn left_footer(mut self, footer: impl IntoElement) -> Self {
        self.left_footer = Some(footer.into_any_element());
        self
    }

    pub fn right_footer(mut self, footer: impl IntoElement) -> Self {
        self.right_footer = Some(footer.into_any_element());
        self
    }

    pub fn left_empty(mut self, empty: impl IntoElement) -> Self {
        self.left_empty = Some(empty.into_any_element());
        self
    }

    pub fn right_empty(mut self, empty: impl IntoElement) -> Self {
        self.right_empty = Some(empty.into_any_element());
        self
    }

    #[track_caller]
    pub fn onchange(
        mut self,
        handler: impl Fn(Vec<SharedString>, TransferDirection, Vec<SharedString>, &mut Window, &mut App)
        + 'static,
    ) -> Self {
        self.onchange = Some(Rc::new(handler));
        self
    }

    #[track_caller]
    pub fn onleftcheckchange(
        mut self,
        handler: impl Fn(Vec<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onleftcheckchange = Some(Rc::new(handler));
        self
    }

    #[track_caller]
    pub fn onrightcheckchange(
        mut self,
        handler: impl Fn(Vec<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onrightcheckchange = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Transfer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Transfer {
            id,
            data,
            value,
            filterable,
            target_order,
            titles,
            button_texts,
            filter_placeholder,
            filter_method,
            mut left_footer,
            mut right_footer,
            mut left_empty,
            mut right_empty,
            onchange,
            onleftcheckchange,
            onrightcheckchange,
        } = self;
        let locale = Locale::current(cx);
        let props = TransferProps {
            data,
            value: value.clone(),
            target_order,
            titles: titles.unwrap_or_else(|| {
                [
                    locale.transfer_left_title().into(),
                    locale.transfer_right_title().into(),
                ]
            }),
            button_texts: button_texts.unwrap_or_else(|| {
                [
                    locale.transfer_to_right().into(),
                    locale.transfer_to_left().into(),
                ]
            }),
            filter_method,
        };
        let handlers = TransferHandlers {
            onchange,
            onleftcheckchange,
            onrightcheckchange,
        };
        let state = window.use_keyed_state(id.clone(), cx, |_, _| TransferState::default());
        state.update(cx, |state, _| {
            state.props = props.clone();
            state.handlers = handlers;
        });
        state.update(cx, |state, _| state.sync_value(value));
        let theme = Theme::current(cx);
        let mut panel = |right: bool| {
            let side = if right { "right" } else { "left" };
            let items = state.read(cx).visible(right);
            let checked = state.read(cx).checked(right).clone();
            let all_checked = !items.is_empty()
                && items
                    .iter()
                    .filter(|item| !item.disabled)
                    .all(|item| checked.contains(&item.key));
            let mut root = div()
                .id(ElementId::NamedChild(Arc::new(id.clone()), side.into()))
                .w(px(200.))
                .flex_col()
                .rounded(px(4.))
                .border_1()
                .border_color(rgb(theme.border_lighter()))
                .bg(rgb(theme.overlay_background()))
                .overflow_hidden();
            let title = if right {
                &state.read(cx).props.titles[1]
            } else {
                &state.read(cx).props.titles[0]
            };
            root = root.child(
                div()
                    .h(px(40.))
                    .flex()
                    .items_center()
                    .px(px(12.))
                    .bg(rgb(theme.fill(crate::theme::Fill::Light)))
                    .child(
                        el_checkbox()
                            .id(ElementId::NamedChild(
                                Arc::new(id.clone()),
                                format!("{side}-all").into(),
                            ))
                            .checked(all_checked)
                            .label(title.clone())
                            .onchange({
                                let state = state.clone();
                                move |checked, window, cx| {
                                    state.update(cx, |state, cx| {
                                        state.toggle_all(right, checked, window, cx)
                                    });
                                    window.refresh();
                                }
                            }),
                    )
                    .child(
                        div()
                            .ml_auto()
                            .text_size(px(12.))
                            .text_color(rgb(theme.secondary_text()))
                            .child(items.len().to_string()),
                    ),
            );
            if filterable {
                let query = if right {
                    state.read(cx).right_query.clone()
                } else {
                    state.read(cx).left_query.clone()
                };
                root = root.child(
                    div().p(px(8.)).child(
                        el_input()
                            .id(ElementId::NamedChild(
                                Arc::new(id.clone()),
                                format!("{side}-filter").into(),
                            ))
                            .value(query)
                            .small()
                            .radius_small()
                            .placeholder(
                                filter_placeholder
                                    .clone()
                                    .unwrap_or_else(|| locale.transfer_filter_placeholder().into()),
                            )
                            .prefix(el_icon("search").size(12.))
                            .oninput({
                                let state = state.clone();
                                move |query, _, cx| {
                                    state.update(cx, |state, _| {
                                        if right {
                                            state.right_query = query.to_string();
                                        } else {
                                            state.left_query = query.to_string();
                                        }
                                    });
                                }
                            }),
                    ),
                );
            }
            let mut list = div()
                .id(ElementId::NamedChild(
                    Arc::new(id.clone()),
                    format!("{side}-list").into(),
                ))
                .h(px(246.))
                .py(px(6.))
                .overflow_y_scroll();
            if items.is_empty() {
                let empty = if right {
                    right_empty.take()
                } else {
                    left_empty.take()
                };
                let has_empty = empty.is_some();
                list = list.child(
                    div()
                        .h_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(13.))
                        .text_color(rgb(theme.secondary_text()))
                        .when_some(empty, |el, empty| el.child(empty))
                        .when(!has_empty, |el| el.child(locale.transfer_no_data())),
                );
            }
            for (index, item) in items.into_iter().enumerate() {
                list = list.child(
                    div()
                        .id(ElementId::NamedChild(
                            Arc::new(id.clone()),
                            format!("{side}-item-{index}").into(),
                        ))
                        .h(px(30.))
                        .flex()
                        .items_center()
                        .px(px(12.))
                        .child(
                            el_checkbox()
                                .id(ElementId::NamedChild(
                                    Arc::new(id.clone()),
                                    format!("{side}-check-{index}").into(),
                                ))
                                .checked(checked.contains(&item.key))
                                .disabled(item.disabled)
                                .label(item.label.clone())
                                .onchange({
                                    let state = state.clone();
                                    let key = item.key.clone();
                                    move |_, window, cx| {
                                        state.update(cx, |state, cx| {
                                            state.toggle(right, key.clone(), window, cx)
                                        });
                                        window.refresh();
                                    }
                                }),
                        ),
                );
            }
            root = root.child(list);
            let footer = if right {
                right_footer.take()
            } else {
                left_footer.take()
            };
            if let Some(footer) = footer {
                root = root.child(
                    div()
                        .px(px(12.))
                        .py(px(6.))
                        .border_t_1()
                        .text_size(px(12.))
                        .text_color(rgb(theme.secondary_text()))
                        .child(footer),
                );
            }
            root = root.child(
                div()
                    .h(px(30.))
                    .flex()
                    .items_center()
                    .px(px(12.))
                    .border_t_1()
                    .text_size(px(12.))
                    .text_color(rgb(theme.secondary_text()))
                    .child(
                        locale
                            .transfer_checked()
                            .replace("{count}", &checked.len().to_string()),
                    ),
            );
            root.into_any_element()
        };
        let can_right = state.read(cx).checked(false).iter().any(|key| {
            state
                .read(cx)
                .props
                .data
                .iter()
                .any(|item| item.key == *key && !item.disabled)
        });
        let can_left = state.read(cx).checked(true).iter().any(|key| {
            state
                .read(cx)
                .props
                .data
                .iter()
                .any(|item| item.key == *key && !item.disabled)
        });
        div()
            .id(id.clone())
            .flex()
            .items_center()
            .child(panel(false))
            .child(
                div().flex().flex_col().gap(px(10.)).px(px(20.)).children([
                    el_button()
                        .id(ElementId::NamedChild(
                            Arc::new(id.clone()),
                            "to-right".into(),
                        ))
                        .disabled(!can_right)
                        .label(state.read(cx).props.button_texts[0].clone())
                        .icon(el_icon("arrow-right"))
                        .onclick({
                            let state = state.clone();
                            move |_, window, cx| {
                                state.update(cx, |state, cx| state.move_to(true, window, cx));
                                window.refresh();
                            }
                        }),
                    el_button()
                        .id(ElementId::NamedChild(
                            Arc::new(id.clone()),
                            "to-left".into(),
                        ))
                        .disabled(!can_left)
                        .label(state.read(cx).props.button_texts[1].clone())
                        .icon(el_icon("arrow-left"))
                        .onclick({
                            let state = state.clone();
                            move |_, window, cx| {
                                state.update(cx, |state, cx| state.move_to(false, window, cx));
                                window.refresh();
                            }
                        }),
                ]),
            )
            .child(panel(true))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data() -> Vec<TransferItem> {
        vec![
            TransferItem::new("a", "Alpha"),
            TransferItem::new("b", "Beta"),
            TransferItem::new("c", "Gamma").disabled(true),
        ]
    }

    #[test]
    fn target_order_controls_new_item_positions() {
        let items = data();
        let moved = vec![SharedString::from("b")];
        let mut original = vec![SharedString::from("a")];
        original.extend(moved.iter().cloned());
        original.sort_by_key(|key| {
            items
                .iter()
                .position(|item| item.key == *key)
                .unwrap_or(usize::MAX)
        });
        assert_eq!(original, ["a", "b"]);
        let mut unshift = moved.clone();
        unshift.extend([SharedString::from("a")]);
        assert_eq!(unshift, ["b", "a"]);
        let mut push = vec![SharedString::from("a")];
        push.extend(moved);
        assert_eq!(push, ["a", "b"]);
    }

    #[gpui::test]
    fn moving_checked_items_updates_target_and_emits(cx: &mut gpui::TestAppContext) {
        struct Harness {
            state: gpui::Entity<TransferState>,
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
                let state = cx.new(|_| TransferState::default());
                state.update(cx, |state, _| {
                    state.props.data = data();
                    state.handlers.onchange = Some(Rc::new({
                        let changes = changes.clone();
                        move |value, direction, moved, _, _| {
                            changes.borrow_mut().push((value, direction, moved))
                        }
                    }));
                });
                Harness { state }
            }
        });
        view.update(|window, cx| {
            let state = harness.read(cx).state.clone();
            state.update(cx, |state, _| {
                state.left_checked.insert("a".into());
                state.left_checked.insert("c".into());
            });
            state.update(cx, |state, cx| state.move_to(true, window, cx));
            assert_eq!(state.read(cx).props.value, ["a"]);
            assert!(state.read(cx).left_checked.is_empty());
        });
        assert_eq!(
            *changes.borrow(),
            [(
                vec![SharedString::from("a")],
                TransferDirection::Right,
                vec![SharedString::from("a")],
            )]
        );
    }
}
