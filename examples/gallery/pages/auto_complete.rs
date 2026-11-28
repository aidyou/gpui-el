use gpui::{Context, Div, SharedString, div, prelude::*, px};
use gpui_el::{AutoCompleteOption, el_auto_complete, el_button, el_icon};

use crate::{Gallery, copy::Copy, demo::row};

pub(crate) struct State {
    basic: SharedString,
    remote: SharedString,
    clearable: SharedString,
    unmatched: SharedString,
}

impl Default for State {
    fn default() -> Self {
        Self {
            basic: "gp".into(),
            remote: "rust".into(),
            clearable: "element".into(),
            unmatched: "custom".into(),
        }
    }
}

#[cfg(test)]
pub(crate) fn basic_value(state: &State) -> &SharedString {
    &state.basic
}

fn options<'a>(items: impl IntoIterator<Item = (&'a str, &'a str)>) -> Vec<AutoCompleteOption> {
    items
        .into_iter()
        .map(|(value, label)| AutoCompleteOption::new(value, label))
        .collect()
}

impl Gallery {
    pub(crate) fn auto_completes(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let view = cx.entity();
        let field = |slot: usize, id: gpui::ElementId, value: SharedString| {
            let view = view.clone();
            el_auto_complete()
                .id(id)
                .value(value)
                .options(options([
                    ("rust", s.auto_complete_rust.as_str()),
                    ("gpui", s.auto_complete_gpui.as_str()),
                    ("element", s.auto_complete_element.as_str()),
                ]))
                .oninput(move |value, _, cx| {
                    view.update(cx, |this, cx| {
                        match slot {
                            0 => this.auto_complete.basic = value,
                            1 => this.auto_complete.remote = value,
                            2 => this.auto_complete.clearable = value,
                            3 => this.auto_complete.unmatched = value,
                            _ => unreachable!("unknown autocomplete demo slot"),
                        }
                        cx.notify();
                    })
                })
        };
        let basic = row().child(
            div()
                .w(px(280.))
                .debug_selector(|| "auto-complete-basic".into())
                .child(field(
                    0,
                    "auto-complete-basic".into(),
                    self.auto_complete.basic.clone(),
                )),
        );
        let remote = row().child(
            div().w(px(280.)).child(
                field(
                    1,
                    "auto-complete-remote".into(),
                    self.auto_complete.remote.clone(),
                )
                .fetch_method(|_, _, _| {}),
            ),
        );
        let clearable = row().child(
            div().w(px(280.)).child(
                field(
                    2,
                    "auto-complete-clearable".into(),
                    self.auto_complete.clearable.clone(),
                )
                .clearable(true),
            ),
        );
        let unmatched = row().child(
            div().w(px(280.)).child(
                field(
                    3,
                    "auto-complete-unmatched".into(),
                    self.auto_complete.unmatched.clone(),
                )
                .select_when_unmatched(true),
            ),
        );
        let loading = row().child(
            div()
                .w(px(280.))
                .child(el_auto_complete().id("auto-complete-loading").loading(true)),
        );
        let disabled = row().child(
            div().w(px(280.)).child(
                el_auto_complete()
                    .id("auto-complete-disabled")
                    .value("rust")
                    .disabled(true)
                    .options(options([("rust", s.auto_complete_rust.as_str())])),
            ),
        );
        let custom = row().child(
            div().w(px(280.)).child(
                el_auto_complete()
                    .id("auto-complete-custom")
                    .options(options([
                        ("rust", s.auto_complete_rust.as_str()),
                        ("gpui", s.auto_complete_gpui.as_str()),
                    ]))
                    .prefix(el_icon("search").size(14.))
                    .suffix(div().text_size(px(12.)).child("API"))
                    .header(
                        div()
                            .text_size(px(12.))
                            .child(s.auto_complete_header.as_str()),
                    )
                    .footer(
                        div()
                            .text_size(px(12.))
                            .child(s.auto_complete_footer.as_str()),
                    ),
            ),
        );

        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "auto-complete-basic",
                &s.auto_complete_basic,
                || "el_auto_complete().value(\"gp\").options([\n    AutoCompleteOption::new(\"gpui\", \"GPUI\"),\n]);".into(),
                basic.into_any_element(),
            ))
            .child(self.section(
                cx,
                "auto-complete-remote",
                &s.auto_complete_remote,
                || "el_auto_complete().fetch_method(|query, _, cx| { /* update options */ });".into(),
                remote.into_any_element(),
            ))
            .child(self.section(
                cx,
                "auto-complete-clearable",
                &s.auto_complete_clearable,
                || "el_auto_complete().clearable(true);".into(),
                clearable.into_any_element(),
            ))
            .child(self.section(
                cx,
                "auto-complete-unmatched",
                &s.auto_complete_unmatched,
                || "el_auto_complete().select_when_unmatched(true);".into(),
                unmatched.into_any_element(),
            ))
            .child(self.section(
                cx,
                "auto-complete-loading",
                &s.auto_complete_loading,
                || "el_auto_complete().loading(true);".into(),
                loading.into_any_element(),
            ))
            .child(self.section(
                cx,
                "auto-complete-disabled",
                &s.auto_complete_disabled,
                || "el_auto_complete().disabled(true);".into(),
                disabled.into_any_element(),
            ))
            .child(self.section(
                cx,
                "auto-complete-custom",
                &s.auto_complete_custom,
                || "el_auto_complete()\n    .prefix(el_icon(\"search\").size(14.))\n    .suffix(\"API\")\n    .header(\"Header\")\n    .footer(\"Footer\");".into(),
                custom.into_any_element(),
            ))
            .child(
                div().flex().items_center().gap(px(12.)).child(
                    el_button()
                        .id("auto-complete-reset")
                        .small()
                        .label(s.tag_reset.as_str())
                        .icon(el_icon("refresh-left"))
                        .onclick(cx.listener(|this, _, _, cx| {
                            this.auto_complete = Default::default();
                            cx.notify();
                        })),
                ),
            )
    }
}
