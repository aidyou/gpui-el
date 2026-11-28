use gpui::{Context, Div, div, prelude::*, px};
use gpui_el::{BreadcrumbItem, Icon, el_breadcrumb, el_icon};

use crate::{Gallery, copy::Copy};

impl Gallery {
    pub(crate) fn breadcrumbs(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "breadcrumb-basic",
                &s.breadcrumb_basic,
                || "el_breadcrumb().item(BreadcrumbItem::new(\"Home\")).item(BreadcrumbItem::new(\"Current\"));".into(),
                el_breadcrumb()
                    .id("breadcrumb-basic-demo")
                    .item(BreadcrumbItem::new(s.breadcrumb_home.as_str()).href("https://element-plus.org"))
                    .item(BreadcrumbItem::new(s.breadcrumb_components.as_str()).href("https://element-plus.org/en-US/component"))
                    .item(BreadcrumbItem::new(s.breadcrumb_current.as_str())),
            ))
            .child(self.section(
                cx,
                "breadcrumb-icon",
                &s.breadcrumb_icon,
                || "el_breadcrumb().separator_icon(Icon::named(\"arrow-right\"));".into(),
                el_breadcrumb()
                    .id("breadcrumb-icon-demo")
                    .separator_icon(Icon::named("arrow-right"))
                    .item(BreadcrumbItem::new(s.breadcrumb_home.as_str()))
                    .item(BreadcrumbItem::new(s.breadcrumb_components.as_str()))
                    .item(BreadcrumbItem::new(s.breadcrumb_current.as_str())),
            ))
            .child(self.section(
                cx,
                "breadcrumb-separator",
                &s.breadcrumb_separator,
                || "el_breadcrumb().separator(\"›\");".into(),
                div().flex().flex_col().gap(px(12.)).children([
                    el_breadcrumb().id("breadcrumb-slash").separator("/").item(BreadcrumbItem::new("Home")).item(BreadcrumbItem::new("Library")).item(BreadcrumbItem::new("Data")),
                    el_breadcrumb().id("breadcrumb-chevron").separator("›").item(BreadcrumbItem::new("Home")).item(BreadcrumbItem::new("Library")).item(BreadcrumbItem::new("Data")),
                    el_breadcrumb().id("breadcrumb-icon-inline").separator_icon(el_icon("arrow-right")).item(BreadcrumbItem::new("Home")).item(BreadcrumbItem::new("Library")).item(BreadcrumbItem::new("Data")),
                ]),
            ))
    }
}
