use gpui::{Context, Div, div, prelude::*, px};
use gpui_el::{
    ButtonSize, ButtonType, DropdownItem, DropdownTrigger, Icon, el_button, el_dropdown,
};

use crate::{Gallery, copy::Copy};

impl Gallery {
    pub(crate) fn dropdowns(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "dropdown-basic",
                &s.dropdown_basic,
                || "el_dropdown().item(DropdownItem::new(\"save\", \"Save\")).child(el_button().label(\"Actions\"));".into(),
                div().debug_selector(|| "dropdown-basic-area".into()).child(
                    el_dropdown()
                        .id("dropdown-basic-demo")
                        .item(DropdownItem::new("edit", s.dropdown_edit.as_str()).icon(Icon::named("edit")))
                        .item(DropdownItem::new("copy", s.dropdown_copy.as_str()).icon(Icon::named("copy")))
                        .item(DropdownItem::new("delete", s.dropdown_delete.as_str()).icon(Icon::named("delete")).divided(true))
                        .child(el_button().id("dropdown-basic-trigger").label(s.dropdown_actions.as_str())),
                ),
            ))
            .child(self.section(
                cx,
                "dropdown-click",
                &s.dropdown_click,
                || "el_dropdown().trigger(DropdownTrigger::Click).hide_on_click(false);".into(),
                el_dropdown()
                    .id("dropdown-click-demo")
                    .trigger(DropdownTrigger::Click)
                    .hide_on_click(false)
                    .item(DropdownItem::new("save", s.dropdown_save.as_str()))
                    .item(DropdownItem::new("disabled", s.dropdown_disabled.as_str()).disabled(true))
                    .child(el_button().id("dropdown-click-trigger").label(s.dropdown_actions.as_str())),
            ))
            .child(self.section(
                cx,
                "dropdown-sizes",
                &s.dropdown_sizes,
                || "el_dropdown().size(ButtonSize::Small).max_height(120.);".into(),
                div().flex().flex_wrap().gap(px(12.)).children([
                    el_dropdown().id("dropdown-small").size(ButtonSize::Small).item(DropdownItem::new("small", s.dropdown_small.as_str())).child(el_button().small().label(s.dropdown_small.as_str())),
                    el_dropdown().id("dropdown-default").size(ButtonSize::Default).item(DropdownItem::new("default", s.dropdown_default.as_str())).child(el_button().label(s.dropdown_default.as_str())),
                    el_dropdown().id("dropdown-large").size(ButtonSize::Large).max_height(120.).item(DropdownItem::new("large", s.dropdown_large.as_str())).child(el_button().large().label(s.dropdown_large.as_str())),
                ]),
            ))
            .child(self.section(
                cx,
                "dropdown-split",
                &s.dropdown_split,
                || "el_dropdown().split_button(true).button_type(ButtonType::Primary).label(\"Actions\");".into(),
                el_dropdown()
                    .id("dropdown-split-demo")
                    .split_button(true)
                    .button_type(ButtonType::Primary)
                    .label(s.dropdown_actions.as_str())
                    .item(DropdownItem::new("save", s.dropdown_save.as_str()))
                    .item(DropdownItem::new("delete", s.dropdown_delete.as_str()).divided(true)),
            ))
    }
}
