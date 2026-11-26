use gpui::{Context, Div, div, prelude::*, px, rgb};
use gpui_el::{ButtonType, ButtonVariant, Direction, el_button, el_button_group, el_icon};

use crate::{Gallery, copy::Copy, demo::row};

#[derive(Default)]
pub(crate) struct State {
    clicks: usize,
}

impl Gallery {
    pub(crate) fn buttons(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let section =
            |id, title, code: &str, controls| self.section(cx, id, title, || code.into(), controls);
        let themed = |id, title, variant| {
            let suffix = match variant {
                ButtonVariant::Plain => ".plain()",
                ButtonVariant::Dashed => ".dashed()",
                _ => "",
            };
            let code = [
                ("", "Default"),
                (".primary()", "Primary"),
                (".success()", "Success"),
                (".warning()", "Warning"),
                (".danger()", "Danger"),
                (".info()", "Info"),
            ]
            .map(|(kind, label)| format!("el_button(){kind}{suffix}.label({label:?});"))
            .join("\n");
            let types = [
                (ButtonType::Default, s.default.as_str()),
                (ButtonType::Primary, &s.primary),
                (ButtonType::Success, &s.success),
                (ButtonType::Warning, &s.warning),
                (ButtonType::Danger, &s.danger),
                (ButtonType::Info, &s.info),
            ];
            section(
                id,
                title,
                &code,
                row().children(types.into_iter().enumerate().map(|(i, (kind, label))| {
                    el_button()
                        .id((id, i as u32 + 2))
                        .label(label)
                        .when(kind != ButtonType::Default, |button| button.kind(kind))
                        .when(variant != ButtonVariant::Solid, |button| {
                            button.variant(variant)
                        })
                })),
            )
        };
        let text = row()
            .child(
                el_button()
                    .id("text")
                    .label(s.text.as_str())
                    .primary()
                    .text(),
            )
            .child(
                el_button()
                    .id("text-bg")
                    .label(s.text_bg.as_str())
                    .primary()
                    .text_with_background(),
            )
            .child(
                el_button()
                    .id("link")
                    .label(s.link.as_str())
                    .primary()
                    .link(),
            );
        let sizes = row()
            .child(el_button().id("large").label(s.large.as_str()).large())
            .child(
                el_button()
                    .id("round")
                    .label(s.compact.as_str())
                    .round()
                    .small()
                    .font_size(13.0),
            );
        let icons = row()
            .child(
                el_button()
                    .id("circle")
                    .circle()
                    .primary()
                    .icon(el_icon("plus"))
                    .accessibility_label(s.add.as_str()),
            )
            .child(
                el_button()
                    .id("icon")
                    .label(s.add.as_str())
                    .icon(el_icon("plus")),
            );
        let activation = row()
            .child(
                el_button()
                    .id("count")
                    .label(s.count_increment.as_str())
                    .primary()
                    .onclick(cx.listener(|this, _, _, cx| {
                        this.button.clicks += 1;
                        cx.notify();
                    })),
            )
            .child(
                el_button()
                    .id("disabled")
                    .label(s.disabled.as_str())
                    .danger()
                    .disabled(true),
            )
            .child(
                el_button()
                    .id("loading-demo")
                    .label(s.saving.as_str())
                    .primary()
                    .loading(true),
            )
            .child(
                s.activation_help
                    .replace("%{count}", &self.button.clicks.to_string()),
            );
        div()
            .flex()
            .flex_col()
            .gap(px(24.))
            .child(themed("solid", &s.solid_title, ButtonVariant::Solid))
            .child(themed("plain", &s.plain_title, ButtonVariant::Plain))
            .child(themed("dashed", &s.dashed_title, ButtonVariant::Dashed))
            .child(section(
                "text-samples",
                &s.text_title,
                r#"el_button().primary().text().label("Text button");
el_button().primary().text_with_background().label("With background");
el_button().primary().link().label("View details");"#,
                text,
            ))
            .child(section(
                "sizes",
                &s.size_title,
                r#"el_button().large().label("Large");
el_button().round().small().font_size(13.).label("Small and round");"#,
                sizes,
            ))
            .child(section(
                "icons",
                &s.icon_title,
                r#"el_button().circle().primary().icon(el_icon("plus"))
    .accessibility_label("Add");
el_button().icon(el_icon("plus")).label("Add");"#,
                icons,
            ))
            .child(section(
                "activation",
                &s.activation_title,
                r#"el_button().primary().label("Count +1")
    .onclick(cx.listener(|this, _, _, cx| {
        this.button.clicks += 1;
        cx.notify();
    }));
el_button().danger().disabled(true).label("Disabled");
el_button().primary().loading(true).label("Saving");"#,
                activation,
            ))
            .child(section(
                "button-group",
                &s.button_group,
                r#"el_button_group().kind(ButtonType::Primary).buttons([
    el_button().label("Default"),
    el_button().label("Primary"),
]);
el_button_group().direction(Direction::Vertical).buttons([
    el_button().success().label("Success"),
    el_button().info().label("Info"),
]);"#,
                row()
                    .child(el_button_group().kind(ButtonType::Primary).buttons([
                        el_button().label(s.default.as_str()),
                        el_button().label(s.primary.as_str()),
                    ]))
                    .child(el_button_group().direction(Direction::Vertical).buttons([
                        el_button().label(s.success.as_str()).success(),
                        el_button().label(s.info.as_str()),
                    ])),
            ))
            .child(section(
                "button-custom",
                &s.button_custom,
                r#"el_button().color(rgb(0x626aef)).label("Primary");
el_button().color(rgb(0x626aef)).plain().label("Primary");
// Two Han characters get the extra space; caller labels are never translated.
el_button().auto_insert_space(true).label("确定");"#,
                row()
                    .child(el_button().label(s.primary.as_str()).color(rgb(0x626aef)))
                    .child(
                        el_button()
                            .label(s.primary.as_str())
                            .color(rgb(0x626aef))
                            .plain(),
                    )
                    .child(el_button().label("确定").auto_insert_space(true)),
            ))
    }
}
