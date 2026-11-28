use gpui::{Context, Div, div, prelude::*, px};
use gpui_el::{FeedbackEffect, FeedbackType, el_alert};

use crate::{Gallery, copy::Copy};

impl Gallery {
    pub(crate) fn alerts(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let basic = el_alert()
            .id("alert-basic-demo")
            .title(s.alert_basic.as_str())
            .description(s.alert_description.as_str())
            .show_icon(true);
        let types = div().flex().flex_col().gap(px(12.)).children(
            [
                FeedbackType::Primary,
                FeedbackType::Success,
                FeedbackType::Info,
                FeedbackType::Warning,
                FeedbackType::Error,
            ]
            .into_iter()
            .enumerate()
            .map(|(index, kind)| {
                let title = match kind {
                    FeedbackType::Primary => s.primary.as_str(),
                    FeedbackType::Success => s.success.as_str(),
                    FeedbackType::Info => s.info.as_str(),
                    FeedbackType::Warning => s.warning.as_str(),
                    FeedbackType::Error => s.danger.as_str(),
                };
                el_alert()
                    .id(("alert-type", index))
                    .title(title)
                    .kind(kind)
                    .show_icon(true)
                    .closable(false)
            }),
        );
        let dark = el_alert()
            .id("alert-dark-demo")
            .title(s.alert_dark.as_str())
            .description(s.alert_description.as_str())
            .kind(FeedbackType::Success)
            .effect(FeedbackEffect::Dark)
            .show_icon(true)
            .closable(false);

        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "alert-basic",
                &s.alert_basic,
                || "el_alert().title(\"Saved\").description(\"Your changes are stored.\").show_icon(true);".into(),
                basic.into_any_element(),
            ))
            .child(self.section(
                cx,
                "alert-types",
                &s.alert_types,
                || "el_alert().kind(FeedbackType::Success).show_icon(true);".into(),
                types.into_any_element(),
            ))
            .child(self.section(
                cx,
                "alert-dark",
                &s.alert_dark,
                || "el_alert().effect(FeedbackEffect::Dark).kind(FeedbackType::Success);".into(),
                dark.into_any_element(),
            ))
    }
}
