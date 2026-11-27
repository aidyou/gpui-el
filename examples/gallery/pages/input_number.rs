use gpui::{Context, Div, div, prelude::*, px};
use gpui_el::{InputNumberAlign, InputNumberValueOnClear, el_button, el_icon, el_input_number};

use crate::{Gallery, copy::Copy, demo::row};

pub(crate) struct State {
    basic: Option<f64>,
    step: Option<f64>,
    strict: Option<f64>,
    precision: Option<f64>,
    sizes: Option<f64>,
    right: Option<f64>,
    plain: Option<f64>,
    readonly: Option<f64>,
    disabled: Option<f64>,
    formatted: Option<f64>,
    clear: Option<f64>,
    custom: Option<f64>,
    align: Option<f64>,
}

fn field_target(state: &mut State, slot: usize) -> &mut Option<f64> {
    match slot {
        0 => &mut state.basic,
        1 => &mut state.step,
        2 => &mut state.strict,
        3 => &mut state.precision,
        4 => &mut state.sizes,
        5 => &mut state.right,
        6 => &mut state.plain,
        7 => &mut state.readonly,
        8 => &mut state.disabled,
        9 => &mut state.formatted,
        10 => &mut state.clear,
        11 => &mut state.custom,
        12 => &mut state.align,
        _ => unreachable!("unknown input-number demo slot"),
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            basic: Some(5.),
            step: Some(2.),
            strict: Some(3.),
            precision: Some(3.12345),
            sizes: Some(8.),
            right: Some(4.),
            plain: Some(7.),
            readonly: Some(9.),
            disabled: Some(10.),
            formatted: Some(1234.5),
            clear: Some(1.),
            custom: Some(6.),
            align: Some(42.),
        }
    }
}

impl Gallery {
    pub(crate) fn input_numbers(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let view = cx.entity();
        let field = |slot: usize, id: gpui::ElementId, value: Option<f64>| {
            let view = view.clone();
            el_input_number()
                .id(id)
                .value(value)
                .oninput(move |reported, _, cx| {
                    view.update(cx, |this, cx| {
                        *field_target(&mut this.input_number, slot) = reported;
                        cx.notify();
                    })
                })
        };
        let basic = row().child(field(
            0,
            "input-number-basic".into(),
            self.input_number.basic,
        ));
        let steps = row().child(
            div()
                .flex()
                .flex_col()
                .gap(px(12.))
                .child(field(1, "input-number-step".into(), self.input_number.step).step(2.))
                .child(
                    field(2, "input-number-strict".into(), self.input_number.strict)
                        .step(2.)
                        .step_strictly(true),
                ),
        );
        let precision = row().child(
            field(
                3,
                "input-number-precision".into(),
                self.input_number.precision,
            )
            .min(0.)
            .max(10.)
            .precision(2),
        );
        let sizes = row().children((0..3u32).map(|index| {
            field(
                4,
                ("input-number-sized", index).into(),
                self.input_number.sizes,
            )
            .when(index == 0, |el| el.large())
            .when(index == 2, |el| el.small())
        }));
        let right = row().child(
            field(5, "input-number-right".into(), self.input_number.right).controls_right(true),
        );
        let plain = row()
            .child(field(6, "input-number-plain".into(), self.input_number.plain).controls(false));
        let readonly = row().child(
            field(
                7,
                "input-number-readonly".into(),
                self.input_number.readonly,
            )
            .readonly(true),
        );
        let disabled = row().child(
            field(
                8,
                "input-number-disabled".into(),
                self.input_number.disabled,
            )
            .disabled(true),
        );
        let formatted = row().child(
            field(
                9,
                "input-number-formatted".into(),
                self.input_number.formatted,
            )
            .formatter(|value| format!("${value:.2}"))
            .parser(|raw| raw.chars().filter(|ch| !matches!(ch, '$' | ',')).collect()),
        );
        let clear = row().child(
            field(10, "input-number-clear".into(), self.input_number.clear)
                .min(0.)
                .max(20.)
                .value_on_clear(InputNumberValueOnClear::Min),
        );
        let custom = row().child(
            field(11, "input-number-custom".into(), self.input_number.custom)
                .prefix(div().child("$"))
                .suffix(div().child("USD"))
                .decrease_icon(el_icon("minus"))
                .increase_icon(el_icon("plus")),
        );
        let align = row().children([
            field(12, "input-number-left".into(), self.input_number.align)
                .align(InputNumberAlign::Left),
            field(12, "input-number-center".into(), self.input_number.align)
                .align(InputNumberAlign::Center),
            field(12, "input-number-right".into(), self.input_number.align)
                .align(InputNumberAlign::Right),
        ]);

        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "input-number-basic",
                &s.input_number_basic,
                || "el_input_number().value(5);".into(),
                basic.into_any_element(),
            ))
            .child(self.section(
                cx,
                "input-number-steps",
                &s.input_number_steps,
                || "el_input_number().step(2.);\nel_input_number().step(2.).step_strictly(true);".into(),
                steps.into_any_element(),
            ))
            .child(self.section(
                cx,
                "input-number-precision",
                &s.input_number_precision,
                || "el_input_number().min(0.).max(10.).precision(2);".into(),
                precision.into_any_element(),
            ))
            .child(self.section(
                cx,
                "input-number-sizes",
                &s.input_number_sizes,
                || "el_input_number().large();\nel_input_number();\nel_input_number().small();".into(),
                sizes.into_any_element(),
            ))
            .child(self.section(
                cx,
                "input-number-controls",
                &s.input_number_controls,
                || "el_input_number().controls_right(true);\nel_input_number().controls(false);".into(),
                div()
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .child(right)
                    .child(plain)
                    .into_any_element(),
            ))
            .child(self.section(
                cx,
                "input-number-states",
                &s.input_number_states,
                || "el_input_number().readonly(true);\nel_input_number().disabled(true);".into(),
                div()
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .child(readonly)
                    .child(disabled)
                    .into_any_element(),
            ))
            .child(self.section(
                cx,
                "input-number-formatted",
                &s.input_number_formatted,
                || "el_input_number()\n    .formatter(|value| format!(\"${value:.2}\"))\n    .parser(|raw| raw.replace(['$', ','], \"\"));".into(),
                formatted.into_any_element(),
            ))
            .child(self.section(
                cx,
                "input-number-clear",
                &s.input_number_clear,
                || "el_input_number()\n    .min(0.)\n    .max(20.)\n    .value_on_clear(InputNumberValueOnClear::Min);".into(),
                clear.into_any_element(),
            ))
            .child(self.section(
                cx,
                "input-number-custom",
                &s.input_number_custom,
                || "el_input_number()\n    .prefix(\"$\")\n    .suffix(\"USD\")\n    .decrease_icon(el_icon(\"minus\"))\n    .increase_icon(el_icon(\"plus\"));".into(),
                custom.into_any_element(),
            ))
            .child(self.section(
                cx,
                "input-number-align",
                &s.input_number_align,
                || "el_input_number().align(InputNumberAlign::Left);\nel_input_number().align(InputNumberAlign::Center);\nel_input_number().align(InputNumberAlign::Right);".into(),
                align.into_any_element(),
            ))
            .child(
                div().flex().items_center().gap(px(12.)).child(
                    el_button()
                        .id("input-number-reset")
                        .small()
                        .label(s.tag_reset.as_str())
                        .icon(el_icon("refresh-left"))
                        .onclick(cx.listener(|this, _, _, cx| {
                            this.input_number = Default::default();
                            cx.notify();
                        })),
                ),
            )
    }
}
