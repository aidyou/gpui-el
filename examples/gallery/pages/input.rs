use gpui::{Context, Div, div, prelude::*, px, rgb};
use gpui_el::{InputRadius, el_button, el_icon, el_input};

use crate::{Gallery, copy::Copy, demo::row};

pub(crate) struct State {
    basic: String,
    sized: String,
    limited: String,
    clearing: String,
    password: String,
    textarea: String,
    styled: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            basic: "Element Plus".into(),
            sized: "Composable".into(),
            limited: "Type up to ten characters".into(),
            clearing: "Hover or focus to clear".into(),
            password: "secret".into(),
            textarea: "First line\nSecond line\nThird line".into(),
            styled: "Custom focus, outline, and text color".into(),
        }
    }
}

impl Gallery {
    pub(crate) fn inputs(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let view = cx.entity();
        let width = || div().w(px(300.));
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
                            4 => &mut this.input.password,
                            5 => &mut this.input.textarea,
                            _ => &mut this.input.styled,
                        };
                        *target = reported.to_string();
                        cx.notify();
                    })
                })
        };
        let basic = row().child(
            width().child(
                field(0, "input-basic".into(), &self.input.basic)
                    .placeholder(s.input_basic_placeholder.as_str()),
            ),
        );
        let sizes = row().children((0..3u32).map(|index| {
            width().child(
                field(1, ("input-sized", index).into(), &self.input.sized)
                    .when(index == 0, |el| el.large())
                    .when(index == 2, |el| el.small()),
            )
        }));
        let limited = row().child(
            width().child(
                field(2, "input-limited".into(), &self.input.limited)
                    .maxlength(10)
                    .show_word_limit(true),
            ),
        );
        let clearing = row().child(
            width().child(
                field(3, "input-clearing".into(), &self.input.clearing)
                    .clearable(true)
                    .placeholder(s.input_clearable_placeholder.as_str()),
            ),
        );
        let password = row().child(
            width().child(
                field(4, "input-password".into(), &self.input.password)
                    .password(true)
                    .show_password(true),
            ),
        );
        let prefixed = row().child(
            width().child(
                el_input()
                    .id("input-prefixed")
                    .placeholder(s.input_search_placeholder.as_str())
                    .prefix(div().child(el_icon("search").size(14.))),
            ),
        );
        let readonly = row().child(
            width().child(
                el_input()
                    .id("input-readonly")
                    .value("Read-only, but the caret and selection still work")
                    .readonly(true),
            ),
        );
        let disabled = row().child(
            width().child(
                el_input()
                    .id("input-disabled")
                    .value("Disabled")
                    .placeholder("Disabled")
                    .disabled(true),
            ),
        );
        let radius = row().child(
            div().flex().flex_col().gap(px(12.)).children([
                width().child(
                    el_input()
                        .id("input-radius-off")
                        .value("Off · 0 px")
                        .radius_off(),
                ),
                width().child(
                    el_input()
                        .id("input-radius-small")
                        .value("Small · 4 px")
                        .radius_small(),
                ),
                width().child(
                    el_input()
                        .id("input-radius-default")
                        .value("Default · 8 px")
                        .radius(InputRadius::Default),
                ),
                width().child(
                    el_input()
                        .id("input-radius-large")
                        .value("Large · 12 px")
                        .radius_large(),
                ),
                width().child(
                    el_input()
                        .id("input-radius-round")
                        .value("Round · capsule")
                        .round(),
                ),
            ]),
        );
        let styled = row().child(
            width().child(
                field(6, "input-styled".into(), &self.input.styled)
                    .active_border_color(rgb(0x67c23a))
                    .outline(true)
                    .text_color(rgb(0x303133)),
            ),
        );
        let textarea = row().child(
            width().child(
                field(5, "input-textarea".into(), &self.input.textarea)
                    .textarea(true)
                    .rows(4)
                    .maxlength(120)
                    .show_word_limit(true)
                    .clearable(true),
            ),
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
                "input-radius",
                &s.input_radius,
                || "el_input().radius_off();\nel_input().radius_small();\nel_input(); // 8 px\nel_input().radius_large();\nel_input().round();".into(),
                radius.into_any_element(),
            ))
            .child(self.section(
                cx,
                "input-styled",
                &s.input_styled,
                || "el_input()\n    .active_border_color(rgb(0x67c23a))\n    .outline(true)\n    .text_color(rgb(0x303133));".into(),
                styled.into_any_element(),
            ))
            .child(self.section(
                cx,
                "input-textarea",
                &s.input_textarea,
                || "el_input()\n    .textarea(true)\n    .rows(4)\n    .maxlength(120)\n    .show_word_limit(true)\n    .clearable(true);".into(),
                textarea.into_any_element(),
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
