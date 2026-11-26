use gpui::{Context, Div, div, prelude::*, px, rgb};
use gpui_el::{Icon, IconPack, LoadingMotion, LoadingSpinner, el_button, el_loading};

use crate::{Gallery, copy::Copy};

#[derive(Default)]
pub(crate) struct State {
    pub(crate) masked: bool,
}

impl Gallery {
    pub(crate) fn loading(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let border = rgb(if self.dark { 0x414243 } else { 0xe4e7ed });
        let panel = |label| {
            div()
                .h(px(120.))
                .p(px(16.))
                .border_1()
                .border_color(border)
                .child(label)
        };
        self.section(
            cx,
            "mask",
            &s.loading_page,
            || {
                r#"el_button().id("toggle-mask").primary()
    .label(if masked { "Hide loading mask" } else { "Show loading mask" })
    .onclick(cx.listener(|this, _, _, cx| {
        this.loading.masked = !this.loading.masked;
        cx.notify();
    }));

el_loading().id("element-plus-mask").active(masked)
    .spinner(LoadingSpinner::Spokes)
    .v_text("Loading data")
    .child("Element Plus Loading icon · text below");

el_loading().id("dots-mask").active(masked)
    .h_text("Loading data")
    .spinner(LoadingSpinner::Dots)
    .motion(LoadingMotion::EaseIn)
    .mask_opacity(0.4)
    .child("Graduated dots · accelerating rotation · text to the right");

el_loading().id("custom-icon-mask").active(masked)
    .h_text("Loading data")
    .loading_icon(Icon::from_pack(IconPack::ElementPlus, "RefreshRight").size(28.))
    .mask_opacity(0.)
    .child("Custom rotating icon · transparent mask");"#
                    .into()
            },
            div()
                .flex()
                .flex_col()
                .gap(px(12.))
                .items_stretch()
                .child(
                    div().child(
                        el_button()
                            .id("toggle-mask")
                            .primary()
                            .label(if self.loading.masked {
                                s.hide_loading.as_str()
                            } else {
                                s.show_loading.as_str()
                            })
                            .onclick(cx.listener(|this, _, _, cx| {
                                this.loading.masked = !this.loading.masked;
                                cx.notify();
                            })),
                    ),
                )
                .child(
                    el_loading()
                        .id("element-plus-mask")
                        .spinner(LoadingSpinner::Spokes)
                        .v_text(s.loading_text.as_str())
                        .active(self.loading.masked)
                        .child(panel(s.loading_vertical.as_str())),
                )
                .child(
                    el_loading()
                        .id("dots-mask")
                        .h_text(s.loading_text.as_str())
                        .active(self.loading.masked)
                        .spinner(LoadingSpinner::Dots)
                        .motion(LoadingMotion::EaseIn)
                        .mask_opacity(0.40)
                        .child(panel(s.loading_horizontal.as_str())),
                )
                .child(
                    el_loading()
                        .id("custom-icon-mask")
                        .h_text(s.loading_text.as_str())
                        .active(self.loading.masked)
                        .loading_icon(
                            Icon::from_pack(IconPack::ElementPlus, "RefreshRight").size(28.),
                        )
                        .mask_opacity(0.)
                        .child(panel(s.loading_clear.as_str())),
                ),
        )
    }
}
