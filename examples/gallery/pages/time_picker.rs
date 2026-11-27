use gpui::{Context, Div, SharedString, div, prelude::*, px};
use gpui_el::{TimeFormat, el_button, el_icon, el_time_picker};

use crate::{Gallery, copy::Copy, demo::row};

pub(crate) struct State {
    basic: Option<SharedString>,
    seconds: Option<SharedString>,
    sized: Option<SharedString>,
    readonly: Option<SharedString>,
    disabled: Option<SharedString>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            basic: Some("09:30".into()),
            seconds: Some("18:45:12".into()),
            sized: Some("12:00".into()),
            readonly: Some("08:15".into()),
            disabled: Some("23:59".into()),
        }
    }
}

impl Gallery {
    pub(crate) fn time_pickers(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let view = cx.entity();
        let field = |slot: usize, id: gpui::ElementId, value: Option<SharedString>| {
            let view = view.clone();
            el_time_picker()
                .id(id)
                .value(value)
                .onchange(move |value, _, cx| {
                    view.update(cx, |this, cx| {
                        match slot {
                            0 => this.time_picker.basic = value,
                            1 => this.time_picker.seconds = value,
                            2 => this.time_picker.sized = value,
                            3 => this.time_picker.readonly = value,
                            4 => this.time_picker.disabled = value,
                            _ => unreachable!("unknown time-picker demo slot"),
                        }
                        cx.notify();
                    })
                })
        };
        let basic = row().child(div().w(px(240.)).child(field(
            0,
            "time-picker-basic".into(),
            self.time_picker.basic.clone(),
        )));
        let seconds = row().child(
            div().w(px(240.)).child(
                field(
                    1,
                    "time-picker-seconds".into(),
                    self.time_picker.seconds.clone(),
                )
                .format(TimeFormat::HourMinuteSecond),
            ),
        );
        let sizes = row().children([
            div().w(px(240.)).child(
                field(
                    2,
                    "time-picker-large".into(),
                    self.time_picker.sized.clone(),
                )
                .large(),
            ),
            div().w(px(240.)).child(field(
                2,
                "time-picker-default".into(),
                self.time_picker.sized.clone(),
            )),
            div().w(px(240.)).child(
                field(
                    2,
                    "time-picker-small".into(),
                    self.time_picker.sized.clone(),
                )
                .small(),
            ),
        ]);
        let readonly = row().child(
            div().w(px(240.)).child(
                field(
                    3,
                    "time-picker-readonly".into(),
                    self.time_picker.readonly.clone(),
                )
                .readonly(true),
            ),
        );
        let disabled = row().child(
            div().w(px(240.)).child(
                field(
                    4,
                    "time-picker-disabled".into(),
                    self.time_picker.disabled.clone(),
                )
                .disabled(true),
            ),
        );
        let custom = row().child(
            div().w(px(240.)).child(
                el_time_picker()
                    .id("time-picker-custom")
                    .value(Some("10:20".into()))
                    .prefix_icon(el_icon("timer"))
                    .editable(false)
                    .clearable(false),
            ),
        );

        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "time-picker-basic",
                &s.time_picker_basic,
                || "el_time_picker().value(Some(\"09:30\".into()));".into(),
                basic.into_any_element(),
            ))
            .child(self.section(
                cx,
                "time-picker-seconds",
                &s.time_picker_seconds,
                || "el_time_picker().format(TimeFormat::HourMinuteSecond);".into(),
                seconds.into_any_element(),
            ))
            .child(self.section(
                cx,
                "time-picker-sizes",
                &s.time_picker_sizes,
                || "el_time_picker().large();\nel_time_picker();\nel_time_picker().small();".into(),
                sizes.into_any_element(),
            ))
            .child(self.section(
                cx,
                "time-picker-states",
                &s.time_picker_states,
                || "el_time_picker().readonly(true);\nel_time_picker().disabled(true);".into(),
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
                "time-picker-custom",
                &s.time_picker_custom,
                || "el_time_picker().prefix_icon(el_icon(\"timer\")).editable(false).clearable(false);".into(),
                custom.into_any_element(),
            ))
            .child(
                div().flex().items_center().gap(px(12.)).child(
                    el_button()
                        .id("time-picker-reset")
                        .small()
                        .label(s.tag_reset.as_str())
                        .icon(el_icon("refresh-left"))
                        .onclick(cx.listener(|this, _, _, cx| {
                            this.time_picker = Default::default();
                            cx.notify();
                        })),
                ),
            )
    }
}
