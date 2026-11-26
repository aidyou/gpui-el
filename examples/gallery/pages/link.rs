use gpui::{Context, Div, div, prelude::*, px};
use gpui_el::{LinkType, LinkUnderline, el_icon, el_link};

use crate::{Gallery, copy::Copy, demo::row};

#[derive(Default)]
pub(crate) struct State {
    clicks: usize,
}

impl Gallery {
    pub(crate) fn links(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let section =
            |id, title, code: &str, controls| self.section(cx, id, title, || code.into(), controls);
        let types = [
            (LinkType::Default, s.default.as_str()),
            (LinkType::Primary, s.primary.as_str()),
            (LinkType::Success, s.success.as_str()),
            (LinkType::Warning, s.warning.as_str()),
            (LinkType::Danger, s.danger.as_str()),
            (LinkType::Info, s.info.as_str()),
        ];
        let basic = row()
            .children(types.into_iter().enumerate().map(|(i, (kind, label))| {
                el_link()
                    .id(("link-basic", i as u32))
                    .label(label)
                    .when(i == 0, |link| link.href("https://element-plus.org"))
                    .when(kind != LinkType::Default, |link| link.kind(kind))
            }))
            .child(
                el_link()
                    .id("link-count")
                    .primary()
                    .label(s.count_increment.as_str())
                    .onclick(cx.listener(|this, _, _, cx| {
                        this.link.clicks += 1;
                        cx.notify();
                    })),
            )
            .child(
                s.activation_help
                    .replace("%{count}", &self.link.clicks.to_string()),
            );
        let disabled = row().children(types.into_iter().enumerate().map(|(i, (kind, label))| {
            el_link()
                .id(("link-disabled", i as u32))
                .label(label)
                .disabled(true)
                .when(kind != LinkType::Default, |link| link.kind(kind))
        }));
        let underline = row()
            .child(el_link().id("underline-default").label(s.default.as_str()))
            .child(
                el_link()
                    .id("underline-always")
                    .label(s.underline_always.as_str())
                    .underline(LinkUnderline::Always),
            )
            .child(
                el_link()
                    .id("underline-hover")
                    .label(s.underline_hover.as_str())
                    .underline(LinkUnderline::Hover),
            )
            .child(
                el_link()
                    .id("underline-never")
                    .label(s.underline_never.as_str())
                    .underline(LinkUnderline::Never),
            );
        let with_icon = row()
            .child(
                el_link()
                    .id("link-edit")
                    .icon(el_icon("edit"))
                    .label(s.link_edit.as_str()),
            )
            .child(
                el_link()
                    .id("link-check")
                    .label(s.link_check.as_str())
                    .trailing_icon(el_icon("view")),
            );
        div()
            .flex()
            .flex_col()
            .gap(px(24.))
            .child(section(
                "link-basic",
                &s.link_basic,
                r#"el_link().label("Default").href("https://element-plus.org");
el_link().primary().label("Primary");
el_link().success().label("Success");
el_link().warning().label("Warning");
el_link().danger().label("Danger");
el_link().info().label("Info");
el_link().primary().label("Count +1")
    .onclick(cx.listener(|this, _, _, cx| {
        this.link.clicks += 1;
        cx.notify();
    }));"#,
                basic.into_any_element(),
            ))
            .child(section(
                "link-disabled",
                &s.link_disabled,
                r#"el_link().disabled(true).label("Default");
el_link().primary().disabled(true).label("Primary");
el_link().success().disabled(true).label("Success");
el_link().warning().disabled(true).label("Warning");
el_link().danger().disabled(true).label("Danger");
el_link().info().disabled(true).label("Info");"#,
                disabled.into_any_element(),
            ))
            .child(section(
                "link-underline",
                &s.link_underline,
                r#"el_link().label("Default"); // Hover is the default.
el_link().underline(LinkUnderline::Always).label("Always");
el_link().underline(LinkUnderline::Hover).label("Hover");
el_link().underline(LinkUnderline::Never).label("Never");"#,
                underline.into_any_element(),
            ))
            .child(section(
                "link-with-icon",
                &s.link_with_icon,
                r#"el_link().icon(el_icon("edit")).label("Edit");
el_link().label("Check").trailing_icon(el_icon("view"));"#,
                with_icon.into_any_element(),
            ))
    }
}
