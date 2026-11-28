use std::time::Duration;

use gpui::{Context, Div, div, prelude::*, px};
use gpui_el::{PopupPlacement, TooltipEffect, TooltipTrigger, el_button, el_tooltip};

use crate::{Gallery, copy::Copy};

impl Gallery {
    pub(crate) fn tooltips(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let placements = [
            (PopupPlacement::Top, s.drawer_top.as_str()),
            (PopupPlacement::TopStart, s.tooltip_top_start.as_str()),
            (PopupPlacement::TopEnd, s.tooltip_top_end.as_str()),
            (PopupPlacement::Bottom, s.drawer_bottom.as_str()),
            (PopupPlacement::BottomStart, s.tooltip_bottom_start.as_str()),
            (PopupPlacement::BottomEnd, s.tooltip_bottom_end.as_str()),
            (PopupPlacement::Left, s.drawer_left.as_str()),
            (PopupPlacement::LeftStart, s.tooltip_left_start.as_str()),
            (PopupPlacement::LeftEnd, s.tooltip_left_end.as_str()),
            (PopupPlacement::Right, s.drawer_right.as_str()),
            (PopupPlacement::RightStart, s.tooltip_right_start.as_str()),
            (PopupPlacement::RightEnd, s.tooltip_right_end.as_str()),
        ];
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "tooltip-basic",
                &s.tooltip_basic,
                || "el_tooltip().content(\"Native tooltip\").child(el_button().label(\"Hover me\"));".into(),
                div()
                    .debug_selector(|| "tooltip-basic-area".into())
                    .child(
                        el_tooltip()
                            .id("tooltip-basic-demo")
                            .content(s.tooltip_content.as_str())
                            .child(el_button().id("tooltip-basic-trigger").label(s.tooltip_hover.as_str())),
                    ),
            ))
            .child(self.section(
                cx,
                "tooltip-placements",
                &s.tooltip_placements,
                || "el_tooltip().placement(PopupPlacement::TopStart).content(\"Aligned start\");".into(),
                div().flex().flex_wrap().gap(px(8.)).children(
                    placements.into_iter().enumerate().map(|(index, (placement, label))| {
                        el_tooltip()
                            .id(("tooltip-placement", index))
                            .placement(placement)
                            .content(label)
                            .child(el_button().id(("tooltip-placement-trigger", index)).small().label(label))
                    }),
                ),
            ))
            .child(self.section(
                cx,
                "tooltip-effects",
                &s.tooltip_effects,
                || "el_tooltip().effect(TooltipEffect::Light).content(\"Light tooltip\");".into(),
                div().flex().flex_wrap().gap(px(8.)).children([
                    el_tooltip().id("tooltip-dark").effect(TooltipEffect::Dark).content(s.dark.as_str()).child(el_button().label(s.dark.as_str())),
                    el_tooltip().id("tooltip-light").effect(TooltipEffect::Light).content(s.light.as_str()).child(el_button().label(s.light.as_str())),
                ]),
            ))
            .child(self.section(
                cx,
                "tooltip-triggers",
                &s.tooltip_triggers,
                || "el_tooltip().trigger(TooltipTrigger::Click).content(\"Click tooltip\");".into(),
                div().flex().flex_wrap().gap(px(8.)).children([
                    el_tooltip().id("tooltip-click").trigger(TooltipTrigger::Click).content(s.tooltip_click.as_str()).child(el_button().label(s.tooltip_click.as_str())),
                    el_tooltip().id("tooltip-focus").trigger(TooltipTrigger::Focus).content(s.tooltip_focus.as_str()).child(el_button().label(s.tooltip_focus.as_str())),
                    el_tooltip().id("tooltip-context").trigger(TooltipTrigger::ContextMenu).content(s.tooltip_context.as_str()).child(el_button().label(s.tooltip_context.as_str())),
                ]),
            ))
            .child(self.section(
                cx,
                "tooltip-delays",
                &s.tooltip_delays,
                || "el_tooltip().show_after(Duration::from_millis(300)).hide_after(Duration::from_millis(120));".into(),
                el_tooltip()
                    .id("tooltip-delay-demo")
                    .content(s.tooltip_delayed.as_str())
                    .show_after(Duration::from_millis(300))
                    .hide_after(Duration::from_millis(120))
                    .child(el_button().label(s.tooltip_delayed.as_str())),
            ))
    }
}
