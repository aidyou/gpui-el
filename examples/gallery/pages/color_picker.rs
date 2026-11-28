use gpui::{Context, Div, SharedString, div, prelude::*, px};
use gpui_el::{ColorFormat, el_button, el_color_picker, el_icon};

use crate::{Gallery, copy::Copy, demo::row};

pub(crate) struct State {
    basic: Option<SharedString>,
    alpha: Option<SharedString>,
    rgb: Option<SharedString>,
    sized: Option<SharedString>,
    disabled: Option<SharedString>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            basic: Some("#409eff".into()),
            alpha: Some("#409eff80".into()),
            rgb: Some("#67c23a".into()),
            sized: Some("#e6a23c".into()),
            disabled: Some("#f56c6c".into()),
        }
    }
}

fn presets() -> Vec<SharedString> {
    [
        "#409eff", "#67c23a", "#e6a23c", "#f56c6c", "#909399", "#303133", "#ffffff", "#000000",
    ]
    .into_iter()
    .map(SharedString::from)
    .collect()
}

impl Gallery {
    pub(crate) fn color_pickers(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let view = cx.entity();
        let field = |slot: usize, id: gpui::ElementId, value: Option<SharedString>| {
            let view = view.clone();
            el_color_picker()
                .id(id)
                .value(value)
                .predefine(presets())
                .onchange(move |value, _, cx| {
                    view.update(cx, |this, cx| {
                        match slot {
                            0 => this.color_picker.basic = value,
                            1 => this.color_picker.alpha = value,
                            2 => this.color_picker.rgb = value,
                            3 => this.color_picker.sized = value,
                            4 => this.color_picker.disabled = value,
                            _ => unreachable!("unknown color-picker demo slot"),
                        }
                        cx.notify();
                    })
                })
        };
        let basic = row().child(
            div()
                .size(px(32.))
                .debug_selector(|| "color-picker-basic".into())
                .child(field(
                    0,
                    "color-picker-basic".into(),
                    self.color_picker.basic.clone(),
                )),
        );
        let alpha = row().child(
            field(
                1,
                "color-picker-alpha".into(),
                self.color_picker.alpha.clone(),
            )
            .show_alpha(true),
        );
        let rgb = row().child(
            field(2, "color-picker-rgb".into(), self.color_picker.rgb.clone())
                .format(ColorFormat::Rgb),
        );
        let sizes = row().children([
            field(
                3,
                "color-picker-large".into(),
                self.color_picker.sized.clone(),
            )
            .large(),
            field(
                3,
                "color-picker-default".into(),
                self.color_picker.sized.clone(),
            ),
            field(
                3,
                "color-picker-small".into(),
                self.color_picker.sized.clone(),
            )
            .small(),
        ]);
        let disabled = row().child(
            field(
                4,
                "color-picker-disabled".into(),
                self.color_picker.disabled.clone(),
            )
            .disabled(true),
        );
        let no_clear = row().child(
            el_color_picker()
                .id("color-picker-no-clear")
                .value(Some("#909399".into()))
                .clearable(false)
                .predefine(presets()),
        );

        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "color-picker-basic",
                &s.color_picker_basic,
                || "el_color_picker().value(Some(\"#409eff\".into()));".into(),
                basic.into_any_element(),
            ))
            .child(self.section(
                cx,
                "color-picker-alpha",
                &s.color_picker_alpha,
                || "el_color_picker().show_alpha(true);".into(),
                alpha.into_any_element(),
            ))
            .child(self.section(
                cx,
                "color-picker-format",
                &s.color_picker_format,
                || "el_color_picker().format(ColorFormat::Rgb);".into(),
                rgb.into_any_element(),
            ))
            .child(self.section(
                cx,
                "color-picker-sizes",
                &s.color_picker_sizes,
                || {
                    "el_color_picker().large();\nel_color_picker();\nel_color_picker().small();"
                        .into()
                },
                sizes.into_any_element(),
            ))
            .child(self.section(
                cx,
                "color-picker-disabled",
                &s.color_picker_disabled,
                || "el_color_picker().disabled(true);".into(),
                disabled.into_any_element(),
            ))
            .child(self.section(
                cx,
                "color-picker-clear",
                &s.color_picker_clear,
                || "el_color_picker().clearable(false);".into(),
                no_clear.into_any_element(),
            ))
            .child(
                div().flex().items_center().gap(px(12.)).child(
                    el_button()
                        .id("color-picker-reset")
                        .small()
                        .label(s.tag_reset.as_str())
                        .icon(el_icon("refresh-left"))
                        .onclick(cx.listener(|this, _, _, cx| {
                            this.color_picker = Default::default();
                            cx.notify();
                        })),
                ),
            )
    }
}
