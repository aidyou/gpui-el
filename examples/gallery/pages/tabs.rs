use gpui::{Context, Div, div, prelude::*, px};
use gpui_el::{TabPane, TabsPosition, TabsType, el_tabs};

use crate::{Gallery, copy::Copy};

impl Gallery {
    pub(crate) fn tabs(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "tabs-basic",
                &s.tabs_basic,
                || "el_tabs().pane(TabPane::new(\"first\", \"First\").content(\"First pane\"));".into(),
                el_tabs()
                    .id("tabs-basic-demo")
                    .pane(TabPane::new("first", s.tabs_first.as_str()).content(s.tabs_first_content.as_str()))
                    .pane(TabPane::new("second", s.tabs_second.as_str()).content(s.tabs_second_content.as_str()))
                    .pane(TabPane::new("disabled", s.tabs_disabled.as_str()).disabled(true).content(s.tabs_disabled_content.as_str())),
            ))
            .child(self.section(
                cx,
                "tabs-card",
                &s.tabs_card,
                || "el_tabs().kind(TabsType::Card).stretch(true);".into(),
                el_tabs()
                    .id("tabs-card-demo")
                    .kind(TabsType::Card)
                    .stretch(true)
                    .pane(TabPane::new("first", s.tabs_first.as_str()).content(s.tabs_first_content.as_str()))
                    .pane(TabPane::new("second", s.tabs_second.as_str()).content(s.tabs_second_content.as_str())),
            ))
            .child(self.section(
                cx,
                "tabs-border-card",
                &s.tabs_border_card,
                || "el_tabs().kind(TabsType::BorderCard);".into(),
                el_tabs()
                    .id("tabs-border-card-demo")
                    .kind(TabsType::BorderCard)
                    .pane(TabPane::new("first", s.tabs_first.as_str()).content(s.tabs_first_content.as_str()))
                    .pane(TabPane::new("second", s.tabs_second.as_str()).content(s.tabs_second_content.as_str())),
            ))
            .child(self.section(
                cx,
                "tabs-position",
                &s.tabs_position,
                || "el_tabs().position(TabsPosition::Left);".into(),
                div().flex().flex_wrap().gap(px(16.)).children([
                    el_tabs().id("tabs-left").position(TabsPosition::Left).pane(TabPane::new("first", s.tabs_first.as_str()).content(s.tabs_first_content.as_str())).pane(TabPane::new("second", s.tabs_second.as_str()).content(s.tabs_second_content.as_str())),
                    el_tabs().id("tabs-right").position(TabsPosition::Right).pane(TabPane::new("first", s.tabs_first.as_str()).content(s.tabs_first_content.as_str())).pane(TabPane::new("second", s.tabs_second.as_str()).content(s.tabs_second_content.as_str())),
                    el_tabs().id("tabs-bottom").position(TabsPosition::Bottom).pane(TabPane::new("first", s.tabs_first.as_str()).content(s.tabs_first_content.as_str())).pane(TabPane::new("second", s.tabs_second.as_str()).content(s.tabs_second_content.as_str())),
                ]),
            ))
            .child(self.section(
                cx,
                "tabs-editable",
                &s.tabs_editable,
                || "el_tabs().kind(TabsType::Card).editable(true);".into(),
                el_tabs()
                    .id("tabs-editable-demo")
                    .kind(TabsType::Card)
                    .editable(true)
                    .pane(TabPane::new("first", s.tabs_first.as_str()).content(s.tabs_first_content.as_str()))
                    .pane(TabPane::new("second", s.tabs_second.as_str()).lazy(true).content(s.tabs_second_content.as_str())),
            ))
    }
}
