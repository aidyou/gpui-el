use gpui::{Context, Div, div, prelude::*, px};
use gpui_el::{CapsuleOption, el_button, el_capsule_segments, el_icon};

use crate::{Gallery, copy::Copy, demo::row};

pub(crate) struct State {
    frequency: String,
    sized: String,
    disabled: String,
    wrapped: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            frequency: "daily".into(),
            sized: "default".into(),
            disabled: "daily".into(),
            wrapped: "interval".into(),
        }
    }
}

fn options<'a>(items: impl IntoIterator<Item = (&'a str, &'a str)>) -> Vec<CapsuleOption> {
    items
        .into_iter()
        .map(|(value, label)| CapsuleOption::new(value, label))
        .collect()
}

impl Gallery {
    pub(crate) fn capsules(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let view = cx.entity();
        let field = |slot: usize, id: gpui::ElementId, selected: &str| {
            let view = view.clone();
            el_capsule_segments()
                .id(id)
                .selected(selected)
                .onselect(move |value, _, cx| {
                    view.update(cx, |this, cx| {
                        match slot {
                            0 => this.capsule.frequency = value.to_string(),
                            1 => this.capsule.sized = value.to_string(),
                            2 => this.capsule.disabled = value.to_string(),
                            3 => this.capsule.wrapped = value.to_string(),
                            _ => unreachable!("unknown capsule demo slot"),
                        }
                        cx.notify();
                    })
                })
        };
        let frequency = row().child(
            field(0, "capsule-frequency".into(), &self.capsule.frequency)
                .accessibility_label(s.capsule_frequency.as_str())
                .options(options([
                    ("daily", s.capsule_daily.as_str()),
                    ("interval", s.capsule_interval.as_str()),
                    ("once", s.capsule_once.as_str()),
                ])),
        );
        let sizes = row().children([
            field(1, "capsule-large".into(), &self.capsule.sized)
                .height(36.)
                .options(options([("small", "Small"), ("default", "Default")])),
            field(1, "capsule-default".into(), &self.capsule.sized)
                .options(options([("small", "Small"), ("default", "Default")])),
            field(1, "capsule-small".into(), &self.capsule.sized)
                .height(24.)
                .options(options([("small", "Small"), ("default", "Default")])),
        ]);
        let disabled = row().child(
            field(2, "capsule-disabled".into(), &self.capsule.disabled)
                .disabled(true)
                .options(options([
                    ("daily", s.capsule_daily.as_str()),
                    ("interval", s.capsule_interval.as_str()),
                    ("once", s.capsule_once.as_str()),
                ])),
        );
        let wrapped = row().child(div().w(px(240.)).child(
            field(3, "capsule-wrapped".into(), &self.capsule.wrapped).options(options([
                ("daily", s.capsule_daily.as_str()),
                ("interval", s.capsule_interval.as_str()),
                ("once", s.capsule_once.as_str()),
                ("weekdays", s.capsule_weekdays.as_str()),
                ("monthly", s.capsule_monthly.as_str()),
            ])),
        ));

        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "capsule-frequency",
                &s.capsule_basic,
                || "el_capsule_segments()\n    .selected(\"daily\")\n    .options([\n        CapsuleOption::new(\"daily\", \"Daily\"),\n        CapsuleOption::new(\"interval\", \"Interval\"),\n        CapsuleOption::new(\"once\", \"Once\"),\n    ]);".into(),
                frequency.into_any_element(),
            ))
            .child(self.section(
                cx,
                "capsule-sizes",
                &s.capsule_sizes,
                || "el_capsule_segments().height(36.);\nel_capsule_segments();\nel_capsule_segments().height(24.);".into(),
                sizes.into_any_element(),
            ))
            .child(self.section(
                cx,
                "capsule-disabled",
                &s.capsule_disabled,
                || "el_capsule_segments().disabled(true);".into(),
                disabled.into_any_element(),
            ))
            .child(self.section(
                cx,
                "capsule-wrapped",
                &s.capsule_wrapped,
                || "el_capsule_segments().options([/* ... */]);".into(),
                wrapped.into_any_element(),
            ))
            .child(
                div().flex().items_center().gap(px(12.)).child(
                    el_button()
                        .id("capsule-reset")
                        .small()
                        .label(s.tag_reset.as_str())
                        .icon(el_icon("refresh-left"))
                        .onclick(cx.listener(|this, _, _, cx| {
                            this.capsule = Default::default();
                            cx.notify();
                        })),
                ),
            )
    }
}
