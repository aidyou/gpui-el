use gpui::{Context, Div, SharedString, div, prelude::*, px};
use gpui_el::{SelectOption, SelectOptionGroup, SelectValue, el_button, el_icon, el_select};

use crate::{Gallery, copy::Copy, demo::row};

pub(crate) struct State {
    basic: Option<SharedString>,
    filterable: Option<SharedString>,
    grouped: Option<SharedString>,
    created: Option<SharedString>,
    multiple: Vec<SharedString>,
    clearable: Option<SharedString>,
    sized: Option<SharedString>,
    disabled: Option<SharedString>,
    empty: Option<SharedString>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            basic: Some("daily".into()),
            filterable: Some("interval".into()),
            grouped: Some("daily".into()),
            created: Some("daily".into()),
            multiple: vec!["daily".into(), "once".into()],
            clearable: Some("daily".into()),
            sized: Some("default".into()),
            disabled: Some("once".into()),
            empty: None,
        }
    }
}

#[cfg(test)]
pub(crate) fn basic_value(state: &State) -> Option<&SharedString> {
    state.basic.as_ref()
}

#[cfg(test)]
pub(crate) fn created_value(state: &State) -> Option<&SharedString> {
    state.created.as_ref()
}

fn options<'a>(items: impl IntoIterator<Item = (&'a str, &'a str)>) -> Vec<SelectOption> {
    items
        .into_iter()
        .map(|(value, label)| SelectOption::new(value, label))
        .collect()
}

impl Gallery {
    pub(crate) fn selects(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let view = cx.entity();
        let field = |slot: usize, id: gpui::ElementId, value: Option<SharedString>| {
            let view = view.clone();
            el_select()
                .id(id)
                .value(value)
                .options(options([
                    ("daily", s.select_daily.as_str()),
                    ("interval", s.select_interval.as_str()),
                    ("once", s.select_once.as_str()),
                ]))
                .onchange(move |value, _, cx| {
                    let SelectValue::Single(value) = value else {
                        return;
                    };
                    view.update(cx, |this, cx| {
                        match slot {
                            0 => this.select.basic = value,
                            1 => this.select.filterable = value,
                            2 => this.select.clearable = value,
                            3 => this.select.sized = value,
                            4 => this.select.disabled = value,
                            5 => this.select.empty = value,
                            _ => unreachable!("unknown select demo slot"),
                        }
                        cx.notify();
                    })
                })
        };
        let basic = row().child(
            div()
                .w(px(240.))
                .debug_selector(|| "select-basic".into())
                .child(field(0, "select-basic".into(), self.select.basic.clone())),
        );
        let filterable = row().child(
            div().w(px(240.)).child(
                field(
                    1,
                    "select-filterable".into(),
                    self.select.filterable.clone(),
                )
                .filterable(true),
            ),
        );
        let grouped = row().child(
            div().w(px(240.)).child(
                el_select()
                    .id("select-grouped")
                    .value(self.select.grouped.clone())
                    .option_groups([
                        SelectOptionGroup::new(
                            s.select_group_one.as_str(),
                            options([
                                ("daily", s.select_daily.as_str()),
                                ("interval", s.select_interval.as_str()),
                            ]),
                        ),
                        SelectOptionGroup::new(
                            s.select_group_two.as_str(),
                            options([("once", s.select_once.as_str())]),
                        ),
                    ])
                    .onchange({
                        let view = view.clone();
                        move |value, _, cx| {
                            if let SelectValue::Single(value) = value {
                                view.update(cx, |this, cx| {
                                    this.select.grouped = value;
                                    cx.notify();
                                });
                            }
                        }
                    }),
            ),
        );
        let created = row().child(
            div()
                .w(px(240.))
                .debug_selector(|| "select-create".into())
                .child(
                    el_select()
                        .id("select-create")
                        .value(self.select.created.clone())
                        .filterable(true)
                        .allow_create(true)
                        .default_first_option(true)
                        .options(options([
                            ("daily", s.select_daily.as_str()),
                            ("interval", s.select_interval.as_str()),
                            ("once", s.select_once.as_str()),
                        ]))
                        .onchange({
                            let view = view.clone();
                            move |value, _, cx| {
                                if let SelectValue::Single(value) = value {
                                    view.update(cx, |this, cx| {
                                        this.select.created = value;
                                        cx.notify();
                                    });
                                }
                            }
                        }),
                ),
        );
        let multiple = row().child(
            div().w(px(320.)).child(
                el_select()
                    .id("select-multiple")
                    .multiple(true)
                    .multiple_limit(3)
                    .values(self.select.multiple.clone())
                    .options(options([
                        ("daily", s.select_daily.as_str()),
                        ("interval", s.select_interval.as_str()),
                        ("once", s.select_once.as_str()),
                        ("weekdays", s.select_weekdays.as_str()),
                    ]))
                    .onchange({
                        let view = view.clone();
                        move |value, _, cx| {
                            if let SelectValue::Multiple(values) = value {
                                view.update(cx, |this, cx| {
                                    this.select.multiple = values;
                                    cx.notify();
                                });
                            }
                        }
                    }),
            ),
        );
        let clearable = row().child(
            div().w(px(240.)).child(
                field(2, "select-clearable".into(), self.select.clearable.clone())
                    .clearable(true)
                    .value_on_clear(Some("daily".into())),
            ),
        );
        let sizes = row().children((0..3u32).map(|index| {
            div().w(px(240.)).child(
                field(3, ("select-sized", index).into(), self.select.sized.clone())
                    .when(index == 0, |el| el.large())
                    .when(index == 2, |el| el.small()),
            )
        }));
        let disabled = row().child(
            div().w(px(240.)).child(
                el_select()
                    .id("select-disabled")
                    .value(self.select.disabled.clone())
                    .options([
                        SelectOption::new("daily", s.select_daily.as_str()),
                        SelectOption::new("interval", s.select_interval.as_str()),
                        SelectOption::new("once", s.select_once.as_str()).disabled(true),
                    ])
                    .onchange({
                        let view = view.clone();
                        move |value, _, cx| {
                            if let SelectValue::Single(value) = value {
                                view.update(cx, |this, cx| {
                                    this.select.disabled = value;
                                    cx.notify();
                                });
                            }
                        }
                    }),
            ),
        );
        let disabled_select = row().child(
            div().w(px(240.)).child(
                el_select()
                    .id("select-disabled-whole")
                    .selected("daily")
                    .disabled(true)
                    .options(options([
                        ("daily", s.select_daily.as_str()),
                        ("interval", s.select_interval.as_str()),
                    ])),
            ),
        );
        let loading = row().child(
            div().w(px(240.)).child(
                el_select()
                    .id("select-loading")
                    .loading(true)
                    .options(Vec::<SelectOption>::new()),
            ),
        );
        let empty = row().child(
            div().w(px(240.)).child(
                field(5, "select-empty".into(), self.select.empty.clone())
                    .options(Vec::<SelectOption>::new())
                    .no_data_text(s.select_no_data.as_str()),
            ),
        );
        let custom = row().child(
            div().w(px(240.)).child(
                el_select()
                    .id("select-custom")
                    .selected("interval")
                    .options(options([
                        ("daily", s.select_daily.as_str()),
                        ("interval", s.select_interval.as_str()),
                        ("once", s.select_once.as_str()),
                    ]))
                    .prefix(el_icon("timer").size(14.))
                    .suffix(div().text_size(px(12.)).child("UTC"))
                    .header(div().text_size(px(12.)).child(s.select_header.as_str()))
                    .footer(div().text_size(px(12.)).child(s.select_footer.as_str())),
            ),
        );

        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "select-basic",
                &s.select_basic,
                || "el_select().selected(\"daily\").options([\n    SelectOption::new(\"daily\", \"Daily\"),\n    SelectOption::new(\"interval\", \"Interval\"),\n]);".into(),
                basic.into_any_element(),
            ))
            .child(self.section(
                cx,
                "select-filterable",
                &s.select_filterable,
                || "el_select().filterable(true);".into(),
                filterable.into_any_element(),
            ))
            .child(self.section(
                cx,
                "select-grouped",
                &s.select_grouped,
                || "el_select().option_groups([\n    SelectOptionGroup::new(\"Group One\", [\n        SelectOption::new(\"daily\", \"Daily\"),\n        SelectOption::new(\"interval\", \"Interval\"),\n    ]),\n    SelectOptionGroup::new(\"Group Two\", [\n        SelectOption::new(\"once\", \"Once\"),\n    ]),\n]);".into(),
                grouped.into_any_element(),
            ))
            .child(self.section(
                cx,
                "select-create",
                &s.select_create,
                || "el_select().filterable(true).allow_create(true).default_first_option(true);".into(),
                created.into_any_element(),
            ))
            .child(self.section(
                cx,
                "select-multiple",
                &s.select_multiple,
                || "el_select().multiple(true).multiple_limit(3);".into(),
                multiple.into_any_element(),
            ))
            .child(self.section(
                cx,
                "select-clearable",
                &s.select_clearable,
                || "el_select().clearable(true).value_on_clear(Some(\"daily\".into()));".into(),
                clearable.into_any_element(),
            ))
            .child(self.section(
                cx,
                "select-sizes",
                &s.select_sizes,
                || "el_select().large();\nel_select();\nel_select().small();".into(),
                sizes.into_any_element(),
            ))
            .child(self.section(
                cx,
                "select-disabled",
                &s.select_disabled,
                || "SelectOption::new(\"once\", \"Once\").disabled(true);\nel_select().disabled(true);".into(),
                div()
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .child(disabled)
                    .child(disabled_select)
                    .into_any_element(),
            ))
            .child(self.section(
                cx,
                "select-loading",
                &s.select_loading,
                || "el_select().loading(true);".into(),
                loading.into_any_element(),
            ))
            .child(self.section(
                cx,
                "select-empty",
                &s.select_empty,
                || "el_select().options(Vec::<SelectOption>::new()).no_data_text(\"No data\");".into(),
                empty.into_any_element(),
            ))
            .child(self.section(
                cx,
                "select-custom",
                &s.select_custom,
                || "el_select()\n    .prefix(el_icon(\"timer\").size(14.))\n    .suffix(\"UTC\")\n    .header(\"Header\")\n    .footer(\"Footer\");".into(),
                custom.into_any_element(),
            ))
            .child(
                div().flex().items_center().gap(px(12.)).child(
                    el_button()
                        .id("select-reset")
                        .small()
                        .label(s.tag_reset.as_str())
                        .icon(el_icon("refresh-left"))
                        .onclick(cx.listener(|this, _, _, cx| {
                            this.select = Default::default();
                            cx.notify();
                        })),
                ),
            )
    }
}
