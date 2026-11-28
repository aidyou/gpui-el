//! Element Plus Dropdown.
//!
//! Follows `dropdown.md`, `dropdown.vue`, `dropdown.ts`, and Theme Chalk
//! `dropdown.scss`: hover/click/contextmenu activation, 150 ms delays, light
//! popper surface, three menu sizes, disabled/divided items, command events,
//! hide-on-click, keyboard navigation, and a split-button trigger.

use std::{rc::Rc, sync::Arc, time::Duration};

use gpui::{
    AnyElement, App, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px,
    rgb,
};

use crate::{
    Button, ButtonSize, ButtonType, Icon, Locale, PopupPlacement, Theme, Tooltip, TooltipEffect,
    el_button, el_icon,
};

/// Dropdown activation method.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DropdownTrigger {
    #[default]
    Hover,
    Click,
    ContextMenu,
}

/// One dropdown menu item.
#[derive(Clone)]
pub struct DropdownItem {
    command: SharedString,
    label: SharedString,
    icon: Option<Icon>,
    disabled: bool,
    divided: bool,
}

impl DropdownItem {
    pub fn new(command: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            command: command.into(),
            label: label.into(),
            icon: None,
            disabled: false,
            divided: false,
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn divided(mut self, divided: bool) -> Self {
        self.divided = divided;
        self
    }
}

type CommandHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;
type VisibleHandler = Rc<dyn Fn(bool, &mut App)>;
type ClickHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// An Element Plus dropdown menu.
#[derive(IntoElement)]
#[must_use]
pub struct Dropdown {
    id: ElementId,
    child: AnyElement,
    items: Vec<DropdownItem>,
    placement: PopupPlacement,
    effect: TooltipEffect,
    trigger: DropdownTrigger,
    hide_on_click: bool,
    show_arrow: bool,
    show_timeout: Duration,
    hide_timeout: Duration,
    max_height: Option<f32>,
    disabled: bool,
    size: ButtonSize,
    split_button: bool,
    button_type: ButtonType,
    label: Option<SharedString>,
    visible: Option<bool>,
    accessibility_label: Option<SharedString>,
    oncommand: Option<CommandHandler>,
    onvisible_change: Option<VisibleHandler>,
    onclick: Option<ClickHandler>,
}

/// Creates a dropdown and adds its trigger with [`Dropdown::child`].
#[track_caller]
pub fn el_dropdown() -> Dropdown {
    Dropdown::new()
}

impl Default for Dropdown {
    fn default() -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            child: div().into_any_element(),
            items: Vec::new(),
            placement: PopupPlacement::Bottom,
            effect: TooltipEffect::Light,
            trigger: DropdownTrigger::Hover,
            hide_on_click: true,
            show_arrow: true,
            show_timeout: Duration::from_millis(150),
            hide_timeout: Duration::from_millis(150),
            max_height: None,
            disabled: false,
            size: ButtonSize::Default,
            split_button: false,
            button_type: ButtonType::Default,
            label: None,
            visible: None,
            accessibility_label: None,
            oncommand: None,
            onvisible_change: None,
            onclick: None,
        }
    }
}

impl Dropdown {
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

    pub fn item(mut self, item: DropdownItem) -> Self {
        self.items.push(item);
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

    pub fn trigger(mut self, trigger: DropdownTrigger) -> Self {
        self.trigger = trigger;
        self
    }

    pub fn hide_on_click(mut self, hide: bool) -> Self {
        self.hide_on_click = hide;
        self
    }

    pub fn show_arrow(mut self, show: bool) -> Self {
        self.show_arrow = show;
        self
    }

    pub fn show_timeout(mut self, delay: Duration) -> Self {
        self.show_timeout = delay;
        self
    }

    pub fn hide_timeout(mut self, delay: Duration) -> Self {
        self.hide_timeout = delay;
        self
    }

    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height.max(0.));
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn split_button(mut self, split: bool) -> Self {
        self.split_button = split;
        self
    }

    pub fn button_type(mut self, kind: ButtonType) -> Self {
        self.button_type = kind;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = Some(visible);
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    pub fn oncommand(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.oncommand = Some(Rc::new(handler));
        self
    }

    pub fn onvisible_change(mut self, handler: impl Fn(bool, &mut App) + 'static) -> Self {
        self.onvisible_change = Some(Rc::new(handler));
        self
    }

    pub fn onclick(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.onclick = Some(Rc::new(handler));
        self
    }
}

#[derive(Default)]
struct DropdownState {
    open: bool,
    active: Option<usize>,
}

fn button_with_type(mut button: Button, kind: ButtonType) -> Button {
    button = match kind {
        ButtonType::Default => button,
        ButtonType::Primary => button.primary(),
        ButtonType::Success => button.success(),
        ButtonType::Warning => button.warning(),
        ButtonType::Danger => button.danger(),
        ButtonType::Info => button.info(),
    };
    button
}

fn button_with_size(button: Button, size: ButtonSize) -> Button {
    match size {
        ButtonSize::Small => button.small(),
        ButtonSize::Default => button,
        ButtonSize::Large => button.large(),
    }
}

impl RenderOnce for Dropdown {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| DropdownState::default());
        let open = self.visible.unwrap_or_else(|| state.read(cx).open);
        if self.visible.is_some() && state.read(cx).open != open {
            state.update(cx, |state, _| state.open = open);
        }
        let theme = Theme::current(cx);
        let locale = Locale::current(cx);
        let items = self.items.clone();
        let command_items = items.clone();
        let first_enabled = items.iter().position(|item| !item.disabled);
        let oncommand = self.oncommand.clone();
        let hide_on_click = self.hide_on_click;
        let command_state = state.clone();
        let command = Rc::new(move |index: usize, window: &mut Window, cx: &mut App| {
            let Some(item) = command_items.get(index) else {
                return;
            };
            if item.disabled {
                return;
            }
            if let Some(handler) = oncommand.clone() {
                handler(item.command.clone(), window, cx);
            }
            if hide_on_click {
                command_state.update(cx, |state, _| state.open = false);
                window.refresh();
            }
        });
        let menu_state = state.clone();
        let mut menu = div()
            .id(ElementId::NamedChild(
                Arc::new(self.id.clone()),
                "menu".into(),
            ))
            .debug_selector(|| "dropdown-menu".into())
            .flex()
            .flex_col()
            .py(px(match self.size {
                ButtonSize::Large => 8.,
                ButtonSize::Default => 6.,
                ButtonSize::Small => 4.,
            }))
            .when_some(self.max_height, |el, height| {
                el.max_h(px(height)).overflow_y_scroll()
            })
            .role(gpui::Role::Menu)
            .on_key_down({
                let state = menu_state.clone();
                let command = command.clone();
                move |event, window, cx| {
                    let current = state.read(cx).active;
                    let next = match event.keystroke.key.as_ref() {
                        "down" => Some(current.map_or(0, |index| index + 1)),
                        "up" => Some(current.map_or(0, |index| index.saturating_sub(1))),
                        "home" => Some(0),
                        "end" => Some(items.len().saturating_sub(1)),
                        "enter" => current,
                        "escape" => {
                            state.update(cx, |state, _| state.open = false);
                            window.refresh();
                            None
                        }
                        _ => None,
                    };
                    if let Some(index) = next {
                        let index = index.min(items.len().saturating_sub(1));
                        if event.keystroke.key == "enter" {
                            command(index, window, cx);
                        } else {
                            state.update(cx, |state, _| state.active = Some(index));
                            window.refresh();
                        }
                        cx.stop_propagation();
                    }
                }
            });
        for (index, item) in self.items.iter().cloned().enumerate() {
            let active = state.read(cx).active == Some(index);
            let command = command.clone();
            let row_state = state.clone();
            let mut row = div()
                .id(ElementId::NamedChild(
                    Arc::new(self.id.clone()),
                    format!("item-{index}").into(),
                ))
                .debug_selector(move || format!("dropdown-item-{index}"))
                .flex()
                .items_center()
                .h(px(match self.size {
                    ButtonSize::Large => 36.,
                    ButtonSize::Default => 32.,
                    ButtonSize::Small => 24.,
                }))
                .px(px(match self.size {
                    ButtonSize::Large => 20.,
                    ButtonSize::Default => 16.,
                    ButtonSize::Small => 12.,
                }))
                .text_size(px(match self.size {
                    ButtonSize::Large | ButtonSize::Default => 14.,
                    ButtonSize::Small => 12.,
                }))
                .text_color(rgb(if item.disabled {
                    theme.disabled_text()
                } else {
                    theme.regular_text()
                }))
                .role(gpui::Role::MenuItem)
                .aria_selected(active)
                .when(item.divided, |el| {
                    el.mt(px(match self.size {
                        ButtonSize::Large => 8.,
                        ButtonSize::Default => 6.,
                        ButtonSize::Small => 4.,
                    }))
                    .border_t_1()
                    .border_color(rgb(theme.border_lighter()))
                })
                .when(!item.disabled, |el| {
                    el.cursor_pointer()
                        .hover(|style| style.bg(rgb(theme.fill(crate::theme::Fill::Light))))
                        .on_hover(move |_, window, cx| {
                            row_state.update(cx, |state, _| state.active = Some(index));
                            window.refresh();
                        })
                        .on_click(move |_, window, cx| command(index, window, cx))
                });
            if let Some(icon) = item.icon {
                row = row.child(div().mr(px(5.)).child(icon.size(14.)));
            }
            row = row.child(item.label);
            menu = menu.child(row);
        }
        let visible_handler = self.onvisible_change.clone();
        let tooltip = Tooltip::new()
            .id(self.id.clone())
            .content(menu)
            .placement(self.placement)
            .effect(self.effect)
            .trigger(match self.trigger {
                DropdownTrigger::Hover => crate::TooltipTrigger::Hover,
                DropdownTrigger::Click => crate::TooltipTrigger::Click,
                DropdownTrigger::ContextMenu => crate::TooltipTrigger::ContextMenu,
            })
            .visible(open)
            .disabled(self.disabled)
            .offset(0.)
            .padding(0.)
            .show_arrow(self.show_arrow)
            .shadow(true)
            .show_after(if self.trigger == DropdownTrigger::Hover {
                self.show_timeout
            } else {
                Duration::ZERO
            })
            .hide_after(if self.trigger == DropdownTrigger::Hover {
                self.hide_timeout
            } else {
                Duration::ZERO
            })
            .onshow({
                let state = state.clone();
                let onvisible = visible_handler.clone();
                move |app| {
                    state.update(app, |state, cx| {
                        state.open = true;
                        state.active = first_enabled;
                        cx.notify();
                    });
                    if let Some(handler) = onvisible.clone() {
                        handler(true, app);
                    }
                    app.refresh_windows();
                }
            })
            .onhide({
                let state = state.clone();
                let onvisible = visible_handler.clone();
                move |app| {
                    state.update(app, |state, cx| {
                        state.open = false;
                        state.active = None;
                        cx.notify();
                    });
                    if let Some(handler) = onvisible.clone() {
                        handler(false, app);
                    }
                    app.refresh_windows();
                }
            });
        let tooltip = if let Some(label) = self.accessibility_label {
            tooltip.accessibility_label(label)
        } else {
            tooltip.accessibility_label(locale.dropdown_toggle())
        };
        if self.split_button {
            let main_handler = self.onclick.clone();
            let main = button_with_size(
                button_with_type(
                    el_button()
                        .id(ElementId::NamedChild(
                            Arc::new(self.id.clone()),
                            "main".into(),
                        ))
                        .label(self.label.unwrap_or_else(|| "Actions".into())),
                    self.button_type,
                )
                .disabled(self.disabled)
                .onclick(move |_, window, cx| {
                    if let Some(handler) = main_handler.clone() {
                        handler(window, cx);
                    }
                }),
                self.size,
            );
            let caret = button_with_size(
                el_button()
                    .id(ElementId::NamedChild(
                        Arc::new(self.id.clone()),
                        "caret".into(),
                    ))
                    .icon(el_icon("arrow-down").size(14.))
                    .accessibility_label(locale.dropdown_toggle())
                    .disabled(self.disabled),
                self.size,
            );
            div()
                .flex()
                .items_center()
                .child(main)
                .child(tooltip.child(caret))
                .into_any_element()
        } else {
            tooltip.child(self.child).into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[gpui::test]
    fn hover_command_closes_and_disabled_items_do_not_fire(cx: &mut gpui::TestAppContext) {
        struct Harness {
            commands: Rc<Cell<usize>>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let commands = self.commands.clone();
                div().p(px(20.)).child(
                    div().debug_selector(|| "dropdown-area".into()).child(
                        el_dropdown()
                            .id("dropdown")
                            .item(DropdownItem::new("save", "Save"))
                            .item(DropdownItem::new("delete", "Delete").disabled(true))
                            .child(div().w(px(140.)).h(px(40.)))
                            .oncommand(move |command, _, _| {
                                if command == "save" {
                                    commands.set(commands.get() + 1);
                                }
                            }),
                    ),
                )
            }
        }
        let commands = Rc::new(Cell::new(0));
        let (_, view) = cx.add_window_view({
            let commands = commands.clone();
            move |_, _| Harness { commands }
        });
        view.simulate_resize(gpui::size(px(800.), px(600.)));
        view.update(|window, cx| window.draw(cx).clear(cx));
        let trigger = view
            .debug_bounds("dropdown-area")
            .expect("dropdown trigger");
        view.simulate_mouse_move(trigger.center(), None, Default::default());
        view.run_until_parked();
        view.executor().advance_clock(Duration::from_millis(170));
        view.run_until_parked();
        view.update(|window, cx| window.draw(cx).clear(cx));
        assert!(view.debug_bounds("dropdown-menu").is_some());
        let disabled = view.debug_bounds("dropdown-item-1").expect("disabled item");
        view.simulate_click(disabled.center(), gpui::Modifiers::default());
        assert_eq!(commands.get(), 0);
        let save = view.debug_bounds("dropdown-item-0").expect("save item");
        view.simulate_click(save.center(), gpui::Modifiers::default());
        view.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(commands.get(), 1);
        assert!(view.debug_bounds("dropdown-menu").is_none());
    }

    #[gpui::test]
    fn hide_on_click_false_keeps_the_menu_open(cx: &mut gpui::TestAppContext) {
        struct Harness;
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                div().p(px(20.)).child(
                    div().debug_selector(|| "dropdown-area".into()).child(
                        el_dropdown()
                            .id("dropdown")
                            .trigger(DropdownTrigger::Click)
                            .hide_on_click(false)
                            .item(DropdownItem::new("save", "Save"))
                            .child(div().w(px(140.)).h(px(40.))),
                    ),
                )
            }
        }
        let (_, view) = cx.add_window_view(|_, _| Harness);
        view.simulate_resize(gpui::size(px(800.), px(600.)));
        view.update(|window, cx| window.draw(cx).clear(cx));
        let trigger = view
            .debug_bounds("dropdown-area")
            .expect("dropdown trigger");
        view.simulate_click(trigger.center(), gpui::Modifiers::default());
        view.update(|window, cx| window.draw(cx).clear(cx));
        let save = view.debug_bounds("dropdown-item-0").expect("save item");
        view.simulate_click(save.center(), gpui::Modifiers::default());
        view.update(|window, cx| window.draw(cx).clear(cx));
        assert!(view.debug_bounds("dropdown-menu").is_some());
    }
}
