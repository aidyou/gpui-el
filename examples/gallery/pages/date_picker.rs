use gpui::{Context, Div, SharedString, div, prelude::*, px};
use gpui_el::{el_button, el_date_picker, el_icon};

use crate::{Gallery, copy::Copy, demo::row};

pub(crate) struct State {
    basic: Option<SharedString>,
    sized: Option<SharedString>,
    readonly: Option<SharedString>,
    disabled: Option<SharedString>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            basic: Some("2026-11-28".into()),
            sized: Some("2026-01-01".into()),
            readonly: Some("2026-06-15".into()),
            disabled: Some("2026-12-31".into()),
        }
    }
}

impl Gallery {
    pub(crate) fn date_pickers(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let view = cx.entity();
        let field = |slot: usize, id: gpui::ElementId, value: Option<SharedString>| {
            let view = view.clone();
            el_date_picker()
                .id(id)
                .value(value)
                .onchange(move |value, _, cx| {
                    view.update(cx, |this, cx| {
                        match slot {
                            0 => this.date_picker.basic = value,
                            1 => this.date_picker.sized = value,
                            2 => this.date_picker.readonly = value,
                            3 => this.date_picker.disabled = value,
                            _ => unreachable!("unknown date-picker demo slot"),
                        }
                        cx.notify();
                    })
                })
        };
        let basic = row().child(div().w(px(240.)).child(field(
            0,
            "date-picker-basic".into(),
            self.date_picker.basic.clone(),
        )));
        let sizes = row().children([
            div().w(px(240.)).child(
                field(
                    1,
                    "date-picker-large".into(),
                    self.date_picker.sized.clone(),
                )
                .large(),
            ),
            div().w(px(240.)).child(field(
                1,
                "date-picker-default".into(),
                self.date_picker.sized.clone(),
            )),
            div().w(px(240.)).child(
                field(
                    1,
                    "date-picker-small".into(),
                    self.date_picker.sized.clone(),
                )
                .small(),
            ),
        ]);
        let readonly = row().child(
            div().w(px(240.)).child(
                field(
                    2,
                    "date-picker-readonly".into(),
                    self.date_picker.readonly.clone(),
                )
                .readonly(true),
            ),
        );
        let disabled = row().child(
            div().w(px(240.)).child(
                field(
                    3,
                    "date-picker-disabled".into(),
                    self.date_picker.disabled.clone(),
                )
                .disabled(true),
            ),
        );
        let custom = row().child(
            div().w(px(240.)).child(
                el_date_picker()
                    .id("date-picker-custom")
                    .value(Some("2026-11-28".into()))
                    .prefix_icon(el_icon("calendar"))
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
                "date-picker-basic",
                &s.date_picker_basic,
                || "el_date_picker().value(Some(\"2026-11-28\".into()));".into(),
                basic.into_any_element(),
            ))
            .child(self.section(
                cx,
                "date-picker-sizes",
                &s.date_picker_sizes,
                || "el_date_picker().large();\nel_date_picker();\nel_date_picker().small();".into(),
                sizes.into_any_element(),
            ))
            .child(self.section(
                cx,
                "date-picker-states",
                &s.date_picker_states,
                || "el_date_picker().readonly(true);\nel_date_picker().disabled(true);".into(),
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
                "date-picker-custom",
                &s.date_picker_custom,
                || "el_date_picker().prefix_icon(el_icon(\"calendar\")).editable(false).clearable(false);".into(),
                custom.into_any_element(),
            ))
            .child(
                div().flex().items_center().gap(px(12.)).child(
                    el_button()
                        .id("date-picker-reset")
                        .small()
                        .label(s.tag_reset.as_str())
                        .icon(el_icon("refresh-left"))
                        .onclick(cx.listener(|this, _, _, cx| {
                            this.date_picker = Default::default();
                            cx.notify();
                        })),
                ),
            )
    }
}
