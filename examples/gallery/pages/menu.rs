use crate::{Gallery, copy::Copy};
use gpui::{Context, Div, div, prelude::*, px};
use gpui_el::{
    MenuEvent, MenuMode, MenuTrigger, el_button, el_icon, el_menu, el_menu_item, el_sub_menu,
};

#[derive(Default)]
pub(crate) struct State {
    collapsed: bool,
    selection: Option<String>,
}

#[cfg(test)]
impl State {
    pub(crate) fn with_collapse(collapsed: bool) -> Self {
        Self {
            collapsed,
            ..Self::default()
        }
    }
}

impl Gallery {
    pub(crate) fn menus(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let submenu = || {
            el_sub_menu("settings", s.menu_second.as_str())
                .icon(el_icon("Setting"))
                .entry(el_menu_item("language", s.menu_language.as_str()))
                .entry(el_menu_item("theme", s.menu_theme.as_str()))
                .entry(el_menu_item("disabled", s.menu_disabled.as_str()).disabled(true))
        };
        let home = || el_menu_item("overview", s.menu_first.as_str()).icon(el_icon("Grid"));
        let entries = r#"    .default_active("overview")
    .entry(el_menu_item("overview", "Overview").icon(el_icon("Grid")))
    .entry(el_sub_menu("settings", "Settings")
        .icon(el_icon("Setting"))
        .entry(el_menu_item("language", "Language"))
        .entry(el_menu_item("theme", "Theme"))
        .entry(el_menu_item("disabled", "Unavailable").disabled(true)))"#;
        div()
            .flex()
            .flex_col()
            .gap(px(24.))
            .child(
                self.section(
                    cx,
                    "menu-vertical",
                    &s.vertical_menu,
                    || {
                        format!(
                            r#"el_button().id("collapse-menu").small()
    .label(if collapsed {{ "Expand menu" }} else {{ "Collapse menu" }})
    .onclick(cx.listener(|this, _, _, cx| {{
        this.menu.collapsed = !this.menu.collapsed;
        cx.notify();
    }}));

el_menu().id("vertical-demo").collapse(collapsed)
{entries};"#
                        )
                    },
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(12.))
                        .items_start()
                        .child(
                            el_button()
                                .id("collapse-menu")
                                .small()
                                .label(if self.menu.collapsed {
                                    s.menu_expand.as_str()
                                } else {
                                    s.menu_collapse.as_str()
                                })
                                .onclick(cx.listener(|this, _, _, cx| {
                                    this.menu.collapsed = !this.menu.collapsed;
                                    cx.notify();
                                })),
                        )
                        .child(
                            div().w(px(224.)).max_w_full().child(
                                el_menu()
                                    .id("vertical-demo")
                                    .collapse(self.menu.collapsed)
                                    .default_active("overview")
                                    .entry(home())
                                    .entry(submenu()),
                            ),
                        ),
                ),
            )
            .child(
                self.section(
                    cx,
                    "menu-horizontal",
                    &s.horizontal_menu,
                    || {
                        format!(
                            r#"el_menu().id("horizontal-demo")
    .mode(MenuMode::Horizontal)
    .menu_trigger(MenuTrigger::Click)
    .close_on_click_outside(true)
{entries}
    .entry(el_menu_item("motion", "Reduce motion"))
    .onselect(cx.listener(|this, event: &MenuEvent, _, cx| {{
        this.menu.selection = Some(
            event.index_path.iter().map(|part| part.as_ref()).collect::<Vec<_>>().join(" / "),
        );
        cx.notify();
    }}));
// The line below mirrors the selected index path."#
                        )
                    },
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(12.))
                        .child(
                            el_menu()
                                .id("horizontal-demo")
                                .mode(MenuMode::Horizontal)
                                .menu_trigger(MenuTrigger::Click)
                                .close_on_click_outside(true)
                                .default_active("overview")
                                .entry(home())
                                .entry(submenu())
                                .entry(el_menu_item("motion", s.reduce_motion.as_str()))
                                .onselect(cx.listener(|this, event: &MenuEvent, _, cx| {
                                    this.menu.selection = Some(
                                        event
                                            .index_path
                                            .iter()
                                            .map(|part| part.as_ref())
                                            .collect::<Vec<_>>()
                                            .join(" / "),
                                    );
                                    cx.notify();
                                })),
                        )
                        .child(s.menu_index_path.replace(
                            "%{path}",
                            self.menu.selection.as_deref().unwrap_or("overview"),
                        )),
                ),
            )
    }
}
