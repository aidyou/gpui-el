use gpui::{Context, Div, div, prelude::*, px, rgb};
use gpui_el::{ButtonType, ButtonVariant, Direction, el_button, el_button_group, el_icon};

use crate::{
    Gallery,
    copy::Copy,
    demo::{row, source},
};

#[derive(Default)]
pub(crate) struct State {
    clicks: usize,
}

impl Gallery {
    pub(crate) fn buttons(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let section = |id, title, controls| {
            self.section(
                cx,
                id,
                title,
                || source(include_str!("button.rs")),
                controls,
            )
        };
        let themed = |id, title, variant| {
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
            .child(section("text-samples", &s.text_title, text))
            .child(section("sizes", &s.size_title, sizes))
            .child(section("icons", &s.icon_title, icons))
            .child(section("activation", &s.activation_title, activation))
            .child(section(
                "button-group",
                &s.button_group,
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
                row()
                    .child(el_button().label(s.primary.as_str()).color(rgb(0x626aef)))
                    .child(
                        el_button()
                            .label(s.primary.as_str())
                            .color(rgb(0x626aef))
                            .plain(),
                    )
                    .child(
                        el_button()
                            .label(s.button_confirm.as_str())
                            .auto_insert_space(true),
                    ),
            ))
    }
}
