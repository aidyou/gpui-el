//! Element Plus Tabs.
//!
//! Follows `tabs.md`, `tabs.tsx`, `tab-pane.ts`, and Theme Chalk `tabs.scss`:
//! line/card/border-card styles, four positions, stretch, controlled or
//! default selection, synchronous before-leave, keyboard navigation,
//! closable/addable/editable tabs, and lazy panes.

use std::{collections::HashSet, rc::Rc, sync::Arc};

use gpui::{
    AnyElement, App, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px,
    rgb,
};

use crate::{Locale, Theme, el_icon, theme::PRIMARY};

/// A tab name, matching Element Plus string/number pane names.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TabValue {
    String(SharedString),
    Number(i64),
}

impl From<&str> for TabValue {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl From<String> for TabValue {
    fn from(value: String) -> Self {
        Self::String(value.into())
    }
}

impl From<SharedString> for TabValue {
    fn from(value: SharedString) -> Self {
        Self::String(value)
    }
}

impl From<i32> for TabValue {
    fn from(value: i32) -> Self {
        Self::Number(value as i64)
    }
}

impl From<i64> for TabValue {
    fn from(value: i64) -> Self {
        Self::Number(value)
    }
}

impl From<usize> for TabValue {
    fn from(value: usize) -> Self {
        Self::Number(value as i64)
    }
}

/// Tabs visual treatment.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TabsType {
    #[default]
    Default,
    Card,
    BorderCard,
}

/// Tabs header position.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TabsPosition {
    #[default]
    Top,
    Right,
    Bottom,
    Left,
}

impl TabsPosition {
    fn vertical(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
}

/// One tab pane.
pub struct TabPane {
    name: TabValue,
    label: SharedString,
    content: AnyElement,
    disabled: bool,
    closable: Option<bool>,
    lazy: bool,
}

impl TabPane {
    pub fn new(name: impl Into<TabValue>, label: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            content: div().into_any_element(),
            disabled: false,
            closable: None,
            lazy: false,
        }
    }

    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = content.into_any_element();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = Some(closable);
        self
    }

    pub fn lazy(mut self, lazy: bool) -> Self {
        self.lazy = lazy;
        self
    }
}

type BeforeLeave = Rc<dyn Fn(TabValue, TabValue) -> bool>;
type TabHandler = Rc<dyn Fn(TabValue, &mut Window, &mut App)>;
type ChangeHandler = Rc<dyn Fn(TabValue, &mut App)>;
type AddHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// An Element Plus tabs component.
#[derive(IntoElement)]
#[must_use]
pub struct Tabs {
    id: ElementId,
    panes: Vec<TabPane>,
    active: Option<TabValue>,
    default_value: Option<TabValue>,
    kind: TabsType,
    position: TabsPosition,
    stretch: bool,
    closable: bool,
    addable: bool,
    editable: bool,
    before_leave: Option<BeforeLeave>,
    ontabclick: Option<TabHandler>,
    onchange: Option<ChangeHandler>,
    onremove: Option<TabHandler>,
    onadd: Option<AddHandler>,
}

/// Creates empty tabs and adds panes with [`Tabs::pane`].
#[track_caller]
pub fn el_tabs() -> Tabs {
    Tabs::new()
}

impl Default for Tabs {
    fn default() -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            panes: Vec::new(),
            active: None,
            default_value: None,
            kind: TabsType::Default,
            position: TabsPosition::Top,
            stretch: false,
            closable: false,
            addable: false,
            editable: false,
            before_leave: None,
            ontabclick: None,
            onchange: None,
            onremove: None,
            onadd: None,
        }
    }
}

impl Tabs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn pane(mut self, pane: TabPane) -> Self {
        self.panes.push(pane);
        self
    }

    pub fn active(mut self, value: impl Into<TabValue>) -> Self {
        self.active = Some(value.into());
        self
    }

    pub fn default_value(mut self, value: impl Into<TabValue>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    pub fn kind(mut self, kind: TabsType) -> Self {
        self.kind = kind;
        self
    }

    pub fn position(mut self, position: TabsPosition) -> Self {
        self.position = position;
        self
    }

    pub fn stretch(mut self, stretch: bool) -> Self {
        self.stretch = stretch;
        self
    }

    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    pub fn addable(mut self, addable: bool) -> Self {
        self.addable = addable;
        self
    }

    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    pub fn before_leave(mut self, handler: impl Fn(TabValue, TabValue) -> bool + 'static) -> Self {
        self.before_leave = Some(Rc::new(handler));
        self
    }

    pub fn ontabclick(
        mut self,
        handler: impl Fn(TabValue, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.ontabclick = Some(Rc::new(handler));
        self
    }

    pub fn onchange(mut self, handler: impl Fn(TabValue, &mut App) + 'static) -> Self {
        self.onchange = Some(Rc::new(handler));
        self
    }

    pub fn onremove(mut self, handler: impl Fn(TabValue, &mut Window, &mut App) + 'static) -> Self {
        self.onremove = Some(Rc::new(handler));
        self
    }

    pub fn onadd(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.onadd = Some(Rc::new(handler));
        self
    }
}

#[derive(Default)]
struct TabsState {
    active: Option<TabValue>,
    visited: HashSet<TabValue>,
}

impl RenderOnce for Tabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| TabsState::default());
        if state.read(cx).active.is_none() {
            let initial = self
                .active
                .clone()
                .or_else(|| self.default_value.clone())
                .or_else(|| self.panes.first().map(|pane| pane.name.clone()))
                .unwrap_or(TabValue::Number(0));
            state.update(cx, |state, _| state.active = Some(initial));
        }
        if let Some(active) = self.active.clone()
            && state.read(cx).active.as_ref() != Some(&active)
        {
            state.update(cx, |state, _| state.active = Some(active));
        }
        let current = state
            .read(cx)
            .active
            .clone()
            .or_else(|| self.panes.first().map(|pane| pane.name.clone()))
            .unwrap_or(TabValue::Number(0));
        state.update(cx, |state, _| {
            state.visited.insert(current.clone());
        });
        let theme = Theme::current(cx);
        let locale = Locale::current(cx);
        let panes: Vec<_> = self
            .panes
            .iter()
            .map(|pane| (pane.name.clone(), pane.disabled))
            .collect();
        let before_leave = self.before_leave.clone();
        let onchange = self.onchange.clone();
        let activate_state = state.clone();
        let activate_panes = panes.clone();
        let activate = Rc::new(move |index: usize, window: &mut Window, cx: &mut App| {
            let Some((name, disabled)) = activate_panes.get(index) else {
                return;
            };
            if *disabled {
                return;
            }
            let old = activate_state
                .read(cx)
                .active
                .clone()
                .unwrap_or(TabValue::Number(0));
            if let Some(handler) = before_leave.clone()
                && !handler(name.clone(), old.clone())
            {
                return;
            }
            if old != *name {
                activate_state.update(cx, |state, _| {
                    state.active = Some(name.clone());
                    state.visited.insert(name.clone());
                });
                if let Some(handler) = onchange.clone() {
                    handler(name.clone(), cx);
                }
                window.refresh();
            }
        });
        let vertical = self.position.vertical();
        let editable = self.editable;
        let addable = self.addable || editable;
        let globally_closable = self.closable || editable;
        let mut nav = div()
            .id(ElementId::NamedChild(
                Arc::new(self.id.clone()),
                "nav".into(),
            ))
            .debug_selector(|| "tabs-nav".into())
            .flex()
            .when(vertical, |el| el.flex_col())
            .when(self.stretch, |el| el.w_full())
            .on_key_down({
                let activate = activate.clone();
                let panes = panes.clone();
                let state = state.clone();
                move |event, window, cx| {
                    if panes.is_empty() {
                        return;
                    }
                    let index = panes
                        .iter()
                        .position(|(name, _)| Some(name) == state.read(cx).active.as_ref())
                        .unwrap_or(0);
                    let forward = matches!(event.keystroke.key.as_ref(), "right" | "down");
                    let backward = matches!(event.keystroke.key.as_ref(), "left" | "up");
                    let next = if forward || backward {
                        let mut candidate = index;
                        for _ in 0..panes.len() {
                            candidate = if forward {
                                (candidate + 1) % panes.len()
                            } else {
                                (candidate + panes.len() - 1) % panes.len()
                            };
                            if panes.get(candidate).is_some_and(|(_, disabled)| !disabled) {
                                break;
                            }
                        }
                        Some(candidate)
                    } else if event.keystroke.key == "home" {
                        panes.iter().position(|(_, disabled)| !disabled)
                    } else if event.keystroke.key == "end" {
                        panes.iter().rposition(|(_, disabled)| !disabled)
                    } else {
                        None
                    };
                    if let Some(next) = next {
                        activate(next, window, cx);
                        cx.stop_propagation();
                    }
                }
            });
        for (index, pane) in self.panes.iter().enumerate() {
            let pane_name = pane.name.clone();
            let active = pane.name == current;
            let disabled = pane.disabled;
            let closable = pane.closable.unwrap_or(globally_closable);
            let activate = activate.clone();
            let tabclick = self.ontabclick.clone();
            let remove = self.onremove.clone();
            let click_name = pane_name.clone();
            let close_name = pane_name.clone();
            let mut item = div()
                .id(ElementId::NamedChild(
                    Arc::new(self.id.clone()),
                    format!("tab-{index}").into(),
                ))
                .debug_selector(move || format!("tab-{index}"))
                .h(px(40.))
                .px(px(20.))
                .flex()
                .items_center()
                .justify_center()
                .gap(px(5.))
                .text_size(px(14.))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(rgb(if disabled {
                    theme.disabled_text()
                } else if active {
                    PRIMARY
                } else {
                    theme.primary_text()
                }))
                .role(gpui::Role::Tab)
                .aria_selected(active)
                .when(self.stretch, |el| el.flex_1())
                .when(self.kind == TabsType::Default, |el| {
                    let border_color = if active {
                        rgb(PRIMARY).into()
                    } else {
                        gpui::transparent_black()
                    };
                    match self.position {
                        TabsPosition::Top => el.border_b_2().border_color(border_color),
                        TabsPosition::Bottom => el.border_t_2().border_color(border_color),
                        TabsPosition::Left => el.border_r_2().border_color(border_color),
                        TabsPosition::Right => el.border_l_2().border_color(border_color),
                    }
                })
                .when(self.kind != TabsType::Default, |el| {
                    el.border_1()
                        .border_color(rgb(if active {
                            theme.border(false)
                        } else {
                            theme.border_lighter()
                        }))
                        .when(active, |el| el.bg(rgb(theme.overlay_background())))
                })
                .when(!disabled, |el| {
                    el.cursor_pointer()
                        .hover(|style| style.text_color(rgb(PRIMARY)))
                        .focusable()
                        .tab_stop(true)
                        .on_click(move |_, window, cx| {
                            if let Some(handler) = tabclick.clone() {
                                handler(click_name.clone(), window, cx);
                            }
                            activate(index, window, cx);
                        })
                });
            item = item.child(pane.label.clone());
            if closable && !disabled {
                item = item.child(
                    div()
                        .id(ElementId::NamedChild(
                            Arc::new(self.id.clone()),
                            format!("close-{index}").into(),
                        ))
                        .debug_selector(move || format!("tab-close-{index}"))
                        .w(px(14.))
                        .h(px(14.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(7.))
                        .cursor_pointer()
                        .aria_label(locale.tabs_close())
                        .on_click(move |_, window, cx| {
                            if let Some(handler) = remove.clone() {
                                handler(close_name.clone(), window, cx);
                            }
                            cx.stop_propagation();
                        })
                        .child(el_icon("close").size(12.)),
                );
            }
            nav = nav.child(item);
        }
        if addable {
            nav = nav.child(
                div()
                    .id(ElementId::NamedChild(
                        Arc::new(self.id.clone()),
                        "add".into(),
                    ))
                    .debug_selector(|| "tab-add".into())
                    .w(px(20.))
                    .h(px(20.))
                    .ml(px(10.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(3.))
                    .border_1()
                    .border_color(rgb(theme.border(false)))
                    .text_color(rgb(theme.primary_text()))
                    .cursor_pointer()
                    .aria_label(locale.tabs_add())
                    .focusable()
                    .tab_stop(true)
                    .on_click({
                        let onadd = self.onadd.clone();
                        move |_, window, cx| {
                            if let Some(handler) = onadd.clone() {
                                handler(window, cx);
                            }
                        }
                    })
                    .child(el_icon("plus").size(12.)),
            );
        }
        let mut header = div()
            .id(ElementId::NamedChild(
                Arc::new(self.id.clone()),
                "header".into(),
            ))
            .debug_selector(|| "tabs-header".into())
            .flex()
            .items_center()
            .when(vertical, |el| el.flex_col())
            .when(self.position == TabsPosition::Bottom, |el| el.mt(px(10.)))
            .when(self.position == TabsPosition::Left, |el| el.mr(px(10.)))
            .when(self.position == TabsPosition::Right, |el| el.ml(px(10.)))
            .child(nav);
        if self.kind == TabsType::BorderCard {
            header = header
                .bg(rgb(theme.fill(crate::theme::Fill::Light)))
                .border_b_1();
        }
        let mut content = div()
            .id(ElementId::NamedChild(
                Arc::new(self.id.clone()),
                "content".into(),
            ))
            .debug_selector(|| "tabs-content".into())
            .flex_1()
            .when(self.kind == TabsType::BorderCard, |el| el.p(px(15.)));
        for (index, pane) in self.panes.into_iter().enumerate() {
            let active = pane.name == current;
            if pane.lazy && !state.read(cx).visited.contains(&pane.name) {
                continue;
            }
            content = content.child(
                div()
                    .id(ElementId::NamedChild(
                        Arc::new(self.id.clone()),
                        format!("pane-{index}").into(),
                    ))
                    .debug_selector(move || {
                        if active {
                            format!("pane-active-{index}")
                        } else {
                            format!("pane-inactive-{index}")
                        }
                    })
                    .when(!active, |el| el.hidden())
                    .child(pane.content),
            );
        }
        let mut root = div()
            .id(self.id)
            .debug_selector(|| "tabs".into())
            .flex()
            .when(vertical, |el| el.flex_row())
            .when(!vertical, |el| el.flex_col())
            .when(self.kind == TabsType::BorderCard, |el| {
                el.rounded(px(4.))
                    .border_1()
                    .border_color(rgb(theme.border(false)))
                    .bg(rgb(theme.overlay_background()))
            });
        if matches!(self.position, TabsPosition::Bottom | TabsPosition::Right) {
            root = root.child(content).child(header);
        } else {
            root = root.child(header).child(content);
        }
        root.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[gpui::test]
    fn clicking_tabs_switches_content_and_respects_disabled(cx: &mut gpui::TestAppContext) {
        struct Harness;
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                div().p(px(20.)).child(
                    el_tabs()
                        .id("tabs")
                        .pane(TabPane::new("first", "First").content("First pane"))
                        .pane(TabPane::new("second", "Second").content("Second pane"))
                        .pane(
                            TabPane::new("disabled", "Disabled")
                                .disabled(true)
                                .content("Disabled pane"),
                        ),
                )
            }
        }
        let (_, view) = cx.add_window_view(|_, _| Harness);
        view.simulate_resize(gpui::size(px(800.), px(600.)));
        view.update(|window, cx| window.draw(cx).clear(cx));
        assert!(view.debug_bounds("pane-active-0").is_some());
        assert!(view.debug_bounds("pane-active-1").is_none());
        let second = view.debug_bounds("tab-1").expect("second tab");
        view.simulate_click(second.center(), gpui::Modifiers::default());
        view.update(|window, cx| window.draw(cx).clear(cx));
        assert!(view.debug_bounds("pane-active-0").is_none());
        assert!(view.debug_bounds("pane-active-1").is_some());
        let disabled = view.debug_bounds("tab-2").expect("disabled tab");
        view.simulate_click(disabled.center(), gpui::Modifiers::default());
        view.update(|window, cx| window.draw(cx).clear(cx));
        assert!(view.debug_bounds("pane-active-1").is_some());
    }

    #[gpui::test]
    fn before_leave_can_block_a_switch(cx: &mut gpui::TestAppContext) {
        struct Harness;
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                div().p(px(20.)).child(
                    el_tabs()
                        .id("tabs")
                        .before_leave(|_, _| false)
                        .pane(TabPane::new("first", "First").content("First pane"))
                        .pane(TabPane::new("second", "Second").content("Second pane")),
                )
            }
        }
        let (_, view) = cx.add_window_view(|_, _| Harness);
        view.simulate_resize(gpui::size(px(800.), px(600.)));
        view.update(|window, cx| window.draw(cx).clear(cx));
        let second = view.debug_bounds("tab-1").expect("second tab");
        view.simulate_click(second.center(), gpui::Modifiers::default());
        view.update(|window, cx| window.draw(cx).clear(cx));
        assert!(view.debug_bounds("pane-active-0").is_some());
        assert!(view.debug_bounds("pane-active-1").is_none());
    }

    #[gpui::test]
    fn editable_tabs_report_close_and_add(cx: &mut gpui::TestAppContext) {
        struct Harness {
            removed: Rc<Cell<usize>>,
            added: Rc<Cell<usize>>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let removed = self.removed.clone();
                let added = self.added.clone();
                div().p(px(20.)).child(
                    el_tabs()
                        .id("tabs")
                        .kind(TabsType::Card)
                        .editable(true)
                        .pane(TabPane::new("first", "First").content("First pane"))
                        .onremove(move |_, _, _| removed.set(removed.get() + 1))
                        .onadd(move |_, _| added.set(added.get() + 1)),
                )
            }
        }
        let removed = Rc::new(Cell::new(0));
        let added = Rc::new(Cell::new(0));
        let (_, view) = cx.add_window_view({
            let removed = removed.clone();
            let added = added.clone();
            move |_, _| Harness { removed, added }
        });
        view.simulate_resize(gpui::size(px(800.), px(600.)));
        view.update(|window, cx| window.draw(cx).clear(cx));
        let close = view.debug_bounds("tab-close-0").expect("close tab");
        view.simulate_click(close.center(), gpui::Modifiers::default());
        let add = view.debug_bounds("tab-add").expect("add tab");
        view.simulate_click(add.center(), gpui::Modifiers::default());
        assert_eq!(removed.get(), 1);
        assert_eq!(added.get(), 1);
    }
}
