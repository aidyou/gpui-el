use gpui::{Context, Div, div, prelude::*, px};

use crate::{Gallery, copy::Copy};

impl Gallery {
    pub(crate) fn initialization(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let standard =
            r#"el_init!(cx, locale: Locale::SimplifiedChinese, icons: IconPack::ElementPlus)?;"#;
        let custom = r#"el_init!(cx, locale: Locale::English, icons: IconPack::Tabler,
    fonts: [include_bytes!("assets/icons.ttf")])?;"#;
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
                .child(s.init_fonts.as_str()),
        )
    }
}
