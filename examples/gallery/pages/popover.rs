use gpui::{Context, Div, div, prelude::*, px};
use gpui_el::{TooltipTrigger, el_button, el_popover};

use crate::{Gallery, copy::Copy};

impl Gallery {
    pub(crate) fn popovers(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "popover-basic",
                &s.popover_basic,
                || "el_popover().title(\"Details\").content(\"Native content\").child(el_button().label(\"Hover me\"));".into(),
                div()
                    .debug_selector(|| "popover-basic-area".into())
                    .child(
                        el_popover()
                            .id("popover-basic-demo")
                            .title(s.popover_title.as_str())
                            .content(s.popover_content.as_str())
                            .child(el_button().id("popover-basic-trigger").label(s.tooltip_hover.as_str())),
                    ),
            ))
            .child(self.section(
                cx,
                "popover-rich",
                &s.popover_rich,
                || "el_popover().content(div().child(\"Rich native content\"));".into(),
                el_popover()
                    .id("popover-rich-demo")
                    .title(s.popover_title.as_str())
                    .width(240.)
                    .content(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(s.popover_rich_body.as_str())
                            .child(el_button().id("popover-rich-action").small().label(s.popover_action.as_str())),
                    )
                    .child(el_button().label(s.popover_rich.as_str())),
            ))
            .child(self.section(
                cx,
                "popover-click",
                &s.popover_click,
                || "el_popover().trigger(TooltipTrigger::Click).content(\"Click content\");".into(),
                el_popover()
                    .id("popover-click-demo")
                    .trigger(TooltipTrigger::Click)
                    .title(s.popover_title.as_str())
                    .content(s.popover_content.as_str())
                    .child(el_button().id("popover-click-trigger").label(s.tooltip_click.as_str())),
            ))
    }
}
