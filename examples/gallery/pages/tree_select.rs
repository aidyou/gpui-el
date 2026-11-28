use gpui::{Context, Div, div, prelude::*, px};
use gpui_el::{CascaderOption, TreeSelectValue, el_button, el_icon, el_tree_select};

use crate::{Gallery, copy::Copy, demo::row};

pub(crate) struct State {
    basic: TreeSelectValue,
    multiple: TreeSelectValue,
    strict: TreeSelectValue,
    filterable: TreeSelectValue,
    disabled: TreeSelectValue,
}

impl Default for State {
    fn default() -> Self {
        Self {
            basic: TreeSelectValue::Single(Some("leaf".into())),
            multiple: TreeSelectValue::Multiple(vec!["leaf".into(), "other".into()]),
            strict: TreeSelectValue::Single(Some("branch".into())),
            filterable: TreeSelectValue::Single(Some("leaf".into())),
            disabled: TreeSelectValue::Single(Some("other".into())),
        }
    }
}

fn options() -> Vec<CascaderOption> {
    vec![CascaderOption::new("root", "Root").children([
        CascaderOption::new("branch", "Branch").children([
            CascaderOption::new("leaf", "Leaf"),
            CascaderOption::new("disabled-leaf", "Disabled leaf").disabled(true),
        ]),
        CascaderOption::new("other", "Other"),
    ])]
}

impl Gallery {
    pub(crate) fn tree_selects(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let view = cx.entity();
        let field = |slot: usize, id: gpui::ElementId, value: TreeSelectValue| {
            let view = view.clone();
            el_tree_select()
                .id(id)
                .value(value)
                .options(options())
                .onchange(move |value, _, cx| {
                    view.update(cx, |this, cx| {
                        match slot {
                            0 => this.tree_select.basic = value,
                            1 => this.tree_select.multiple = value,
                            2 => this.tree_select.strict = value,
                            3 => this.tree_select.filterable = value,
                            4 => this.tree_select.disabled = value,
                            _ => unreachable!("unknown tree-select demo slot"),
                        }
                        cx.notify();
                    })
                })
        };
        let basic = row().child(
            div()
                .w(px(300.))
                .debug_selector(|| "tree-select-basic".into())
                .child(field(
                    0,
                    "tree-select-basic".into(),
                    self.tree_select.basic.clone(),
                )),
        );
        let multiple = row().child(
            div().w(px(340.)).child(
                field(
                    1,
                    "tree-select-multiple".into(),
                    self.tree_select.multiple.clone(),
                )
                .multiple(true)
                .clearable(true),
            ),
        );
        let strict = row().child(
            div().w(px(300.)).child(
                field(
                    2,
                    "tree-select-strict".into(),
                    self.tree_select.strict.clone(),
                )
                .check_strictly(true),
            ),
        );
        let filterable = row().child(
            div().w(px(300.)).child(
                field(
                    3,
                    "tree-select-filterable".into(),
                    self.tree_select.filterable.clone(),
                )
                .filterable(true)
                .clearable(true),
            ),
        );
        let disabled = row().child(
            div().w(px(300.)).child(
                field(
                    4,
                    "tree-select-disabled".into(),
                    self.tree_select.disabled.clone(),
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
                "tree-select-basic",
                &s.tree_select_basic,
                || "el_tree_select().value(TreeSelectValue::Single(Some(\"leaf\".into())));".into(),
                basic.into_any_element(),
            ))
            .child(self.section(
                cx,
                "tree-select-multiple",
                &s.tree_select_multiple,
                || "el_tree_select().multiple(true).clearable(true);".into(),
                multiple.into_any_element(),
            ))
            .child(self.section(
                cx,
                "tree-select-strict",
                &s.tree_select_strict,
                || "el_tree_select().check_strictly(true);".into(),
                strict.into_any_element(),
            ))
            .child(self.section(
                cx,
                "tree-select-filterable",
                &s.tree_select_filterable,
                || "el_tree_select().filterable(true).clearable(true);".into(),
                filterable.into_any_element(),
            ))
            .child(self.section(
                cx,
                "tree-select-disabled",
                &s.tree_select_disabled,
                || "el_tree_select().disabled(true);".into(),
                disabled.into_any_element(),
            ))
            .child(
                div().flex().items_center().gap(px(12.)).child(
                    el_button()
                        .id("tree-select-reset")
                        .small()
                        .label(s.tag_reset.as_str())
                        .icon(el_icon("refresh-left"))
                        .onclick(cx.listener(|this, _, _, cx| {
                            this.tree_select = Default::default();
                            cx.notify();
                        })),
                ),
            )
    }
}
