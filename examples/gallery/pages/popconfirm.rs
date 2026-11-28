use gpui::{Context, Div, div, prelude::*, px};
use gpui_el::{ButtonType, el_button, el_popconfirm};

use crate::{Gallery, copy::Copy};

impl Gallery {
    pub(crate) fn popconfirms(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "popconfirm-basic",
                &s.popconfirm_basic,
                || "el_popconfirm().title(\"Delete this item?\").child(el_button().label(\"Delete\"));".into(),
                div()
                    .debug_selector(|| "popconfirm-basic-area".into())
                    .child(
                        el_popconfirm()
                            .id("popconfirm-basic-demo")
                            .title(s.popconfirm_title.as_str())
                            .child(el_button().id("popconfirm-basic-trigger").label(s.popconfirm_delete.as_str())),
                    ),
            ))
            .child(self.section(
                cx,
                "popconfirm-danger",
                &s.popconfirm_danger,
                || "el_popconfirm().confirm_type(ButtonType::Danger).title(\"Delete permanently?\");".into(),
                el_popconfirm()
                    .id("popconfirm-danger-demo")
                    .title(s.popconfirm_danger_title.as_str())
                    .confirm_type(ButtonType::Danger)
                    .child(el_button().id("popconfirm-danger-trigger").label(s.popconfirm_delete.as_str())),
            ))
            .child(self.section(
                cx,
                "popconfirm-custom",
                &s.popconfirm_custom,
                || "el_popconfirm().hide_icon(true).confirm_text(\"Yes\").cancel_text(\"No\");".into(),
                el_popconfirm()
                    .id("popconfirm-custom-demo")
                    .title(s.popconfirm_title.as_str())
                    .hide_icon(true)
                    .confirm_text(s.popconfirm_yes.as_str())
                    .cancel_text(s.popconfirm_no.as_str())
                    .child(el_button().id("popconfirm-custom-trigger").label(s.popconfirm_delete.as_str())),
            ))
    }
}
