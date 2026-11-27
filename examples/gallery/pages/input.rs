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
    prefixed: String,
    radius: [String; 5],
}

fn field_target(state: &mut State, slot: usize) -> &mut String {
    match slot {
        0 => &mut state.basic,
        1 => &mut state.sized,
        2 => &mut state.limited,
        3 => &mut state.clearing,
        4 => &mut state.password,
        5 => &mut state.textarea,
        6 => &mut state.styled,
        7 => &mut state.prefixed,
        8..=12 => &mut state.radius[slot - 8],
        _ => unreachable!("unknown input demo slot"),
    }
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
            prefixed: String::new(),
            radius: [
                "Off · 0 px".into(),
                "Small · 4 px".into(),
                "Default · 8 px".into(),
                "Large · 12 px".into(),
                "Round · capsule".into(),
            ],
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
                        let target = field_target(&mut this.input, slot);
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
                field(7, "input-prefixed".into(), &self.input.prefixed)
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
                width()
                    .child(field(8, "input-radius-off".into(), &self.input.radius[0]).radius_off()),
                width().child(
                    field(9, "input-radius-small".into(), &self.input.radius[1]).radius_small(),
                ),
                width().child(
                    field(10, "input-radius-default".into(), &self.input.radius[2])
                        .radius(InputRadius::Default),
                ),
                width().child(
                    field(11, "input-radius-large".into(), &self.input.radius[3]).radius_large(),
                ),
                width()
                    .child(field(12, "input-radius-round".into(), &self.input.radius[4]).round()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_editable_demo_has_its_own_controlled_state_slot() {
        let mut state = State::default();
        for slot in 0..=12 {
            *field_target(&mut state, slot) = format!("slot {slot}");
        }
        assert_eq!(state.basic, "slot 0");
        assert_eq!(state.sized, "slot 1");
        assert_eq!(state.limited, "slot 2");
        assert_eq!(state.clearing, "slot 3");
        assert_eq!(state.password, "slot 4");
        assert_eq!(state.textarea, "slot 5");
        assert_eq!(state.styled, "slot 6");
        assert_eq!(state.prefixed, "slot 7");
        assert_eq!(
            state.radius,
            ["slot 8", "slot 9", "slot 10", "slot 11", "slot 12"]
        );
    }
}
