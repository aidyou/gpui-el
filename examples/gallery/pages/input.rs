use gpui::{Context, Div, div, prelude::*, px};
use gpui_el::{el_button, el_icon, el_input};

use crate::{Gallery, copy::Copy, demo::row};

pub(crate) struct State {
    basic: String,
    sized: String,
    limited: String,
    clearing: String,
    password: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            basic: "Element Plus".into(),
            sized: "Composable".into(),
            limited: "Type up to ten characters".into(),
            clearing: "Hover or focus to clear".into(),
            password: "secret".into(),
        }
    }
}

impl Gallery {
    pub(crate) fn inputs(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let view = cx.entity();
        let field = |slot: usize, id: gpui::ElementId, value: &str| {
            let view = view.clone();
            el_input()
                .id(id)
                .value(value)
                .oninput(move |reported, _, cx| {
                    view.update(cx, |this, cx| {
                        let target = match slot {
                            0 => &mut this.input.basic,
                            1 => &mut this.input.sized,
                            2 => &mut this.input.limited,
                            3 => &mut this.input.clearing,
                            _ => &mut this.input.password,
                        };
                        *target = reported.to_string();
                        cx.notify();
                    })
                })
        };
        let basic = row().child(
            field(0, "input-basic".into(), &self.input.basic)
                .placeholder(s.input_basic_placeholder.as_str()),
        );
        let sizes = row().children((0..3u32).map(|index| {
            field(1, ("input-sized", index).into(), &self.input.sized)
                .when(index == 0, |el| el.large())
                .when(index == 2, |el| el.small())
        }));
        let limited = row().child(
            field(2, "input-limited".into(), &self.input.limited)
                .maxlength(10)
                .show_word_limit(true),
        );
        let clearing = row().child(
            field(3, "input-clearing".into(), &self.input.clearing)
                .clearable(true)
                .placeholder(s.input_clearable_placeholder.as_str()),
        );
        let password = row().child(
            field(4, "input-password".into(), &self.input.password)
                .password(true)
                .show_password(true),
        );
        let prefixed = row().child(
            el_input()
                .id("input-prefixed")
                .placeholder(s.input_search_placeholder.as_str())
                .prefix(div().child(el_icon("search").size(14.))),
        );
        let readonly = row().child(
            el_input()
                .id("input-readonly")
                .value("Read-only, but the caret and selection still work")
                .readonly(true),
        );
        let disabled = row().child(
            el_input()
                .id("input-disabled")
                .value("Disabled")
                .placeholder("Disabled")
                .disabled(true),
        );

        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "input-basic",
                &s.input_basic,
                || "el_input().value(\"Element Plus\");".into(),
                basic.into_any_element(),
            ))
            .child(self.section(
                cx,
                "input-sizes",
                &s.input_sizes,
                || "el_input().large();\nel_input();\nel_input().small();".into(),
                sizes.into_any_element(),
            ))
            .child(self.section(
                cx,
                "input-limited",
                &s.input_limited,
                || "el_input().maxlength(10).show_word_limit(true);".into(),
                limited.into_any_element(),
            ))
            .child(self.section(
                cx,
                "input-clearable",
                &s.input_clearable,
                || "el_input().clearable(true);".into(),
                clearing.into_any_element(),
            ))
            .child(self.section(
                cx,
                "input-password",
                &s.input_password,
                || "el_input().password(true).show_password(true);".into(),
                password.into_any_element(),
            ))
            .child(self.section(
                cx,
                "input-prefix",
                &s.input_prefix,
                || "el_input().prefix(el_icon(\"search\").size(14.));".into(),
                prefixed.into_any_element(),
            ))
            .child(self.section(
                cx,
                "input-readonly",
                &s.input_readonly,
                || "el_input().value(\"...\").readonly(true);".into(),
                readonly.into_any_element(),
            ))
            .child(self.section(
                cx,
                "input-disabled",
                &s.input_disabled,
                || "el_input().value(\"Disabled\").disabled(true);".into(),
                disabled.into_any_element(),
            ))
            .child(
                div().flex().items_center().gap(px(12.)).child(
                    el_button()
                        .id("input-reset")
                        .small()
                        .label(s.tag_reset.as_str())
                        .icon(el_icon("refresh-left"))
                        .onclick(cx.listener(|this, _, _, cx| {
                            this.input = Default::default();
                            cx.notify();
                        })),
                ),
            )
    }
}
