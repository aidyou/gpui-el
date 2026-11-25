use gpui::{Context, Div, div, prelude::*, px};

use crate::{Gallery, copy::Copy};

impl Gallery {
    pub(crate) fn initialization(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let standard = "use gpui::App;\nuse gpui_el::{el_init, IconPack, Locale};\n\nfn configure(cx: &mut App) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {\n    el_init!(cx, locale: Locale::SimplifiedChinese, icons: IconPack::ElementPlus)\n}";
        let custom = "use gpui::App;\nuse gpui_el::{el_init, IconPack, Locale};\n\nfn configure_with_font(cx: &mut App) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {\n    el_init!(cx, locale: Locale::English, icons: IconPack::Tabler,\n        fonts: [include_bytes!(\"assets/icons.ttf\")])\n}";
        self.section(
            cx,
            "init-example",
            &s.init_title,
            || format!("{standard}\n\n{custom}"),
            div()
                .flex()
                .flex_col()
                .gap(px(12.))
                .child(s.init_intro.as_str())
                .child(s.init_fonts.as_str())
                .child(div().font_family("monospace").child(standard))
                .child(div().font_family("monospace").child(custom)),
        )
    }
}
