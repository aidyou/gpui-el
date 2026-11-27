use gpui::{Context, Div, div, prelude::*, px, rgb};
use gpui_el::{Icon, RateSize, el_button, el_icon, el_rate};

use crate::{Gallery, copy::Copy, demo::row};

pub(crate) struct State {
    basic: f32,
    half: f32,
    sized: f32,
    text: f32,
    colors: f32,
    icons: f32,
    clearable: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            basic: 3.,
            half: 2.5,
            sized: 4.,
            text: 3.,
            colors: 4.,
            icons: 3.,
            clearable: 3.,
        }
    }
}

impl Gallery {
    pub(crate) fn rates(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let view = cx.entity();
        // One slot per demo, so a single helper wires every rate back to its
        // own state. The sizes demo passes an indexed ID per star row.
        let rate = |slot: usize, id: gpui::ElementId, value: f32| {
            let view = view.clone();
            el_rate()
                .id(id)
                .value(value)
                .onchange(move |picked, _, cx| {
                    view.update(cx, |this, cx| {
                        let target = match slot {
                            0 => &mut this.rate.basic,
                            1 => &mut this.rate.half,
                            2 => &mut this.rate.sized,
                            3 => &mut this.rate.text,
                            4 => &mut this.rate.colors,
                            5 => &mut this.rate.icons,
                            _ => &mut this.rate.clearable,
                        };
                        *target = picked;
                        cx.notify();
                    })
                })
        };
        let basic = row().child(
            rate(0, "rate-basic".into(), self.rate.basic)
                .show_score(true)
                .score_template("{value} points"),
        );
        let half = row().child(
            rate(1, "rate-half".into(), self.rate.half)
                .allow_half(true)
                .show_score(true),
        );
        let sized = self.rate.sized;
        let sizes = row().children((0..3).map(|index| {
            let size = [RateSize::Large, RateSize::Default, RateSize::Small][index];
            rate(2, ("rate-sized", index).into(), sized).size(size)
        }));
        let text = row().child(
            rate(3, "rate-text".into(), self.rate.text)
                .show_text(true)
                .texts([
                    s.rate_bad.as_str(),
                    s.rate_poor.as_str(),
                    s.rate_fair.as_str(),
                    s.rate_good.as_str(),
                    s.rate_excellent.as_str(),
                ]),
        );
        let colors = row().child(rate(4, "rate-colors".into(), self.rate.colors).colors([
            rgb(0xf56c6c),
            rgb(0xe6a23c),
            rgb(0x67c23a),
        ]));
        let icons = row().child(
            rate(5, "rate-icons".into(), self.rate.icons)
                .void_color(rgb(0xcdd0d6))
                .icon(
                    Icon::from_pack(gpui_el::IconPack::ElementPlus, "StarFilled")
                        .color(rgb(0xf7ba2a)),
                )
                .void_icon(
                    Icon::from_pack(gpui_el::IconPack::ElementPlus, "Star").color(rgb(0x909399)),
                ),
        );
        let readonly = row()
            .child(
                el_rate()
                    .id("rate-readonly")
                    .value(3.7)
                    .show_score(true)
                    .disabled(true),
            )
            .child(
                el_rate()
                    .id("rate-readonly-half")
                    .value(2.5)
                    .allow_half(true)
                    .disabled(true),
            );
        let clearable = row().child(
            rate(6, "rate-clearable".into(), self.rate.clearable)
                .clearable(true)
                .show_text(true)
                .texts([
                    s.rate_bad.as_str(),
                    s.rate_poor.as_str(),
                    s.rate_fair.as_str(),
                    s.rate_good.as_str(),
                    s.rate_excellent.as_str(),
                ]),
        );
        let section =
            |id, title, code: &str, controls| self.section(cx, id, title, || code.into(), controls);
        div()
            .flex()
            .flex_col()
            .gap(px(24.))
            .child(section(
                "rate-basic",
                &s.rate_basic,
                r#"let view = cx.entity();
el_rate().id("rate-basic").value(self.rate.basic)
    .show_score(true).score_template("{value} points")
    .onchange(move |picked, _, cx| {
        view.update(cx, |this, cx| {
            this.rate.basic = picked;
            cx.notify();
        });
    });"#,
                basic.into_any_element(),
            ))
            .child(section(
                "rate-half",
                &s.rate_half,
                r#"el_rate().allow_half(true).value(2.5).show_score(true);"#,
                half.into_any_element(),
            ))
            .child(section(
                "rate-sizes",
                &s.rate_sizes,
                r#"el_rate().large().value(4.);
el_rate().value(4.);
el_rate().small().value(4.);"#,
                sizes.into_any_element(),
            ))
            .child(section(
                "rate-text",
                &s.rate_text,
                r#"el_rate().show_text(true).texts(["Bad", "Poor", "Fair", "Good", "Excellent"]);"#,
                text.into_any_element(),
            ))
            .child(section(
                "rate-colors",
                &s.rate_colors,
                r#"// One color per score level, low to high.
el_rate().colors([rgb(0xf56c6c), rgb(0xe6a23c), rgb(0x67c23a)]);"#,
                colors.into_any_element(),
            ))
            .child(section(
                "rate-icons",
                &s.rate_icons,
                r#"el_rate()
    .icon(Icon::from_pack(IconPack::ElementPlus, "StarFilled").color(rgb(0xf7ba2a)))
    .void_icon(Icon::from_pack(IconPack::ElementPlus, "Star").color(rgb(0x909399)));"#,
                icons.into_any_element(),
            ))
            .child(section(
                "rate-readonly",
                &s.rate_readonly,
                r#"// Read-only rates keep fractional display: 3.7 clips the fourth star
// to 70 %, and 2.5 shows a half star.
el_rate().value(3.7).show_score(true).disabled(true);
el_rate().value(2.5).allow_half(true).disabled(true);"#,
                readonly.into_any_element(),
            ))
            .child(section(
                "rate-clearable",
                &s.rate_clearable,
                r#"el_rate().clearable(true).show_text(true)
    .texts(["Bad", "Poor", "Fair", "Good", "Excellent"]);"#,
                clearable.into_any_element(),
            ))
            .child(
                div().flex().items_center().gap(px(12.)).child(
                    el_button()
                        .id("rate-reset")
                        .small()
                        .label(s.tag_reset.as_str())
                        .icon(el_icon("refresh-left"))
                        .onclick(cx.listener(|this, _, _, cx| {
                            this.rate = Default::default();
                            cx.notify();
                        })),
                ),
            )
    }
}
