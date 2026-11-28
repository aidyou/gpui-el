use gpui::{Context, Div, div, prelude::*, px};
use gpui_el::{CascaderOption, CascaderSelection, el_button, el_cascader, el_icon};

use crate::{Gallery, copy::Copy, demo::row};

pub(crate) struct State {
    basic: CascaderSelection,
    multiple: CascaderSelection,
    filterable: CascaderSelection,
    strict: CascaderSelection,
    disabled: CascaderSelection,
}

impl Default for State {
    fn default() -> Self {
        Self {
            basic: CascaderSelection::Single(vec![
                "guide".into(),
                "components".into(),
                "button".into(),
            ]),
            multiple: CascaderSelection::Multiple(vec![
                vec!["guide".into(), "layout".into()],
                vec!["guide".into(), "components".into(), "input".into()],
            ]),
            filterable: CascaderSelection::Single(vec![
                "guide".into(),
                "components".into(),
                "input".into(),
            ]),
            strict: CascaderSelection::Single(vec!["guide".into()]),
            disabled: CascaderSelection::Single(vec![
                "guide".into(),
                "components".into(),
                "button".into(),
            ]),
        }
    }
}

fn options() -> Vec<CascaderOption> {
    vec![CascaderOption::new("guide", "Guide").children([
        CascaderOption::new("components", "Components").children([
            CascaderOption::new("button", "Button"),
            CascaderOption::new("input", "Input"),
            CascaderOption::new("select", "Select"),
        ]),
        CascaderOption::new("layout", "Layout"),
        CascaderOption::new("disabled", "Disabled").disabled(true),
    ])]
}

impl Gallery {
    pub(crate) fn cascaders(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let view = cx.entity();
        let field = |slot: usize, id: gpui::ElementId, selection: CascaderSelection| {
            let view = view.clone();
            el_cascader()
                .id(id)
                .selection(selection)
                .options(options())
                .onchange(move |selection, _, cx| {
                    view.update(cx, |this, cx| {
                        match slot {
                            0 => this.cascader.basic = selection,
                            1 => this.cascader.multiple = selection,
                            2 => this.cascader.filterable = selection,
                            3 => this.cascader.strict = selection,
                            4 => this.cascader.disabled = selection,
                            _ => unreachable!("unknown cascader demo slot"),
                        }
                        cx.notify();
                    })
                })
        };
        let basic = row().child(
            div()
                .w(px(320.))
                .debug_selector(|| "cascader-basic".into())
                .child(field(
                    0,
                    "cascader-basic".into(),
                    self.cascader.basic.clone(),
                )),
        );
        let multiple = row().child(
            div().w(px(360.)).child(
                field(
                    1,
                    "cascader-multiple".into(),
                    self.cascader.multiple.clone(),
                )
                .multiple(true)
                .clearable(true),
            ),
        );
        let filterable = row().child(
            div().w(px(320.)).child(
                field(
                    2,
                    "cascader-filterable".into(),
                    self.cascader.filterable.clone(),
                )
                .filterable(true)
                .clearable(true),
            ),
        );
        let last_level = row().child(
            div().w(px(320.)).child(
                field(0, "cascader-last-level".into(), self.cascader.basic.clone())
                    .show_all_levels(false)
                    .separator(" > "),
            ),
        );
        let strict = row().child(div().w(px(320.)).child(
            field(3, "cascader-strict".into(), self.cascader.strict.clone()).check_strictly(true),
        ));
        let disabled = row().child(
            div().w(px(320.)).child(
                field(
                    4,
                    "cascader-disabled".into(),
                    self.cascader.disabled.clone(),
                )
                .disabled(true),
            ),
        );

        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "cascader-basic",
                &s.cascader_basic,
                || "el_cascader().selection(CascaderSelection::Single(vec![\n    \"guide\".into(), \"components\".into(), \"button\".into(),\n]));".into(),
                basic.into_any_element(),
            ))
            .child(self.section(
                cx,
                "cascader-multiple",
                &s.cascader_multiple,
                || "el_cascader().multiple(true).clearable(true);".into(),
                multiple.into_any_element(),
            ))
            .child(self.section(
                cx,
                "cascader-filterable",
                &s.cascader_filterable,
                || "el_cascader().filterable(true).clearable(true);".into(),
                filterable.into_any_element(),
            ))
            .child(self.section(
                cx,
                "cascader-last-level",
                &s.cascader_last_level,
                || "el_cascader().show_all_levels(false).separator(\" > \");".into(),
                last_level.into_any_element(),
            ))
            .child(self.section(
                cx,
                "cascader-strict",
                &s.cascader_strict,
                || "el_cascader().check_strictly(true);".into(),
                strict.into_any_element(),
            ))
            .child(self.section(
                cx,
                "cascader-disabled",
                &s.cascader_disabled,
                || "el_cascader().disabled(true);".into(),
                disabled.into_any_element(),
            ))
            .child(
                div().flex().items_center().gap(px(12.)).child(
                    el_button()
                        .id("cascader-reset")
                        .small()
                        .label(s.tag_reset.as_str())
                        .icon(el_icon("refresh-left"))
                        .onclick(cx.listener(|this, _, _, cx| {
                            this.cascader = Default::default();
                            cx.notify();
                        })),
                ),
            )
    }
}
