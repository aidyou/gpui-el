use gpui::{Context, Div, div, prelude::*, px};
use gpui_el::{StepItem, StepStatus, StepsDirection, el_steps};

use crate::{Gallery, copy::Copy};

impl Gallery {
    pub(crate) fn steps(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "steps-basic",
                &s.steps_basic,
                || "el_steps().active(1).step(StepItem::new(\"Account\")).step(StepItem::new(\"Profile\"));".into(),
                el_steps()
                    .id("steps-basic-demo")
                    .active(1)
                    .step(StepItem::new(s.steps_account.as_str()).description(s.steps_account_desc.as_str()))
                    .step(StepItem::new(s.steps_profile.as_str()).description(s.steps_profile_desc.as_str()))
                    .step(StepItem::new(s.steps_done.as_str()).description(s.steps_done_desc.as_str())),
            ))
            .child(self.section(
                cx,
                "steps-status",
                &s.steps_status,
                || "StepItem::new(\"Failed\").status(StepStatus::Error);".into(),
                el_steps()
                    .id("steps-status-demo")
                    .active(2)
                    .step(StepItem::new(s.steps_account.as_str()))
                    .step(StepItem::new(s.steps_profile.as_str()).status(StepStatus::Error))
                    .step(StepItem::new(s.steps_done.as_str()).status(StepStatus::Success)),
            ))
            .child(self.section(
                cx,
                "steps-centered",
                &s.steps_centered,
                || "el_steps().align_center(true).space(180.);".into(),
                el_steps()
                    .id("steps-centered-demo")
                    .active(1)
                    .align_center(true)
                    .space(180.)
                    .step(StepItem::new(s.steps_account.as_str()))
                    .step(StepItem::new(s.steps_profile.as_str()))
                    .step(StepItem::new(s.steps_done.as_str())),
            ))
            .child(self.section(
                cx,
                "steps-vertical",
                &s.steps_vertical,
                || "el_steps().direction(StepsDirection::Vertical).active(1);".into(),
                el_steps()
                    .id("steps-vertical-demo")
                    .direction(StepsDirection::Vertical)
                    .active(1)
                    .step(StepItem::new(s.steps_account.as_str()).description(s.steps_account_desc.as_str()))
                    .step(StepItem::new(s.steps_profile.as_str()).description(s.steps_profile_desc.as_str()))
                    .step(StepItem::new(s.steps_done.as_str()).description(s.steps_done_desc.as_str())),
            ))
            .child(self.section(
                cx,
                "steps-simple",
                &s.steps_simple,
                || "el_steps().simple(true).active(1);".into(),
                el_steps()
                    .id("steps-simple-demo")
                    .simple(true)
                    .active(1)
                    .step(StepItem::new(s.steps_account.as_str()))
                    .step(StepItem::new(s.steps_profile.as_str()))
                    .step(StepItem::new(s.steps_done.as_str())),
            ))
    }
}
