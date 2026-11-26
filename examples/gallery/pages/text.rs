use gpui::{
    Context, Div, FontWeight, HighlightStyle, SharedString, StyledText, UnderlineStyle, div,
    prelude::*, px, rgb,
};
use gpui_el::{TextSize, TextType, el_text, tokens};

use crate::{Gallery, copy::Copy, demo::row};

impl Gallery {
    pub(crate) fn texts(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let section =
            |id, title, code: &str, controls| self.section(cx, id, title, || code.into(), controls);
        let types = [
            (TextType::Default, s.default.as_str()),
            (TextType::Primary, s.primary.as_str()),
            (TextType::Success, s.success.as_str()),
            (TextType::Info, s.info.as_str()),
            (TextType::Warning, s.warning.as_str()),
            (TextType::Danger, s.danger.as_str()),
        ];
        let basic =
            row().children(types.into_iter().enumerate().map(|(i, (kind, label))| {
                el_text().label(label).when(i > 0, |text| text.kind(kind))
            }));
        let sizes = row().children(
            [
                (TextSize::Large, s.large.as_str()),
                (TextSize::Default, s.default.as_str()),
                (TextSize::Small, s.small.as_str()),
            ]
            .map(|(size, label)| el_text().size(size).label(label)),
        );
        // A definite width is what turns the ellipsis flags into layout: the
        // row keeps the samples side by side, the box below clips them.
        let clamp_box = |text: gpui_el::Text| {
            div()
                .w(px(280.))
                .max_w_full()
                .border_1()
                .border_color(rgb(if self.dark { 0x414243 } else { 0xe4e7ed }))
                .rounded(px(tokens::radius::BASE))
                .p(px(12.))
                .child(text)
                .into_any_element()
        };
        // The upstream "mixed" demo styles runs inside one text; the native
        // equivalent is a StyledText passed through `content`.
        let mut mixed_text = String::new();
        let mut highlights = Vec::new();
        let segment =
            |text: &str, style: Option<HighlightStyle>, buf: &mut String, out: &mut Vec<_>| {
                let start = buf.len();
                buf.push_str(text);
                if let Some(style) = style {
                    out.push((start..buf.len(), style));
                }
            };
        let bold = || HighlightStyle {
            font_weight: Some(FontWeight::BOLD),
            ..Default::default()
        };
        let accent = || HighlightStyle {
            color: Some(rgb(0x409eff).into()),
            underline: Some(UnderlineStyle {
                thickness: px(1.),
                ..Default::default()
            }),
            ..Default::default()
        };
        segment(
            s.text_mixed_plain.as_str(),
            None,
            &mut mixed_text,
            &mut highlights,
        );
        segment(
            s.text_mixed_brand.as_str(),
            Some(bold()),
            &mut mixed_text,
            &mut highlights,
        );
        segment(
            s.text_mixed_middle.as_str(),
            None,
            &mut mixed_text,
            &mut highlights,
        );
        segment(
            s.text_mixed_accent.as_str(),
            Some(accent()),
            &mut mixed_text,
            &mut highlights,
        );
        segment(
            s.text_mixed_tail.as_str(),
            None,
            &mut mixed_text,
            &mut highlights,
        );
        let mixed = StyledText::new(SharedString::from(mixed_text)).with_highlights(highlights);
        div()
            .flex()
            .flex_col()
            .gap(px(24.))
            .child(section(
                "text-basic",
                &s.text_basic,
                r#"el_text().label("Default");
el_text().primary().label("Primary");
el_text().success().label("Success");
el_text().info().label("Info");
el_text().warning().label("Warning");
el_text().danger().label("Danger");"#,
                basic.into_any_element(),
            ))
            .child(section(
                "text-sizes",
                &s.text_sizes,
                r#"el_text().large().label("Large");
el_text().label("Default");
el_text().small().label("Small");"#,
                sizes.into_any_element(),
            ))
            .child(section(
                "text-truncated",
                &s.text_truncated,
                &format!(
                    r#"el_text()
    .truncated(true)
    .label("{}");"#,
                    s.text_long
                ),
                clamp_box(el_text().truncated(true).label(s.text_long.as_str())),
            ))
            .child(section(
                "text-clamp",
                &s.text_clamp,
                &format!(
                    r#"el_text()
    .line_clamp(2)
    .label("{}");"#,
                    s.text_long
                ),
                clamp_box(el_text().line_clamp(2).label(s.text_long.as_str())),
            ))
            .child(section(
                "text-decorations",
                &s.text_decorations,
                r#"el_text().bold(true).label("Bold");
el_text().italic(true).label("Italic");
el_text().underline(true).label("Underline");
el_text().strikethrough(true).label("Strikethrough");"#,
                row()
                    .child(el_text().bold(true).label(s.text_bold.as_str()))
                    .child(el_text().italic(true).label(s.text_italic.as_str()))
                    .child(el_text().underline(true).label(s.text_underline.as_str()))
                    .child(
                        el_text()
                            .strikethrough(true)
                            .label(s.text_strikethrough.as_str()),
                    )
                    .into_any_element(),
            ))
            .child(section(
                "text-mixed",
                &s.text_mixed,
                r#"// Append each segment while remembering its byte range, then:
let styled = StyledText::new(text).with_highlights(&[
    (brand, HighlightStyle { font_weight: Some(FontWeight::BOLD), ..Default::default() }),
    (accent, HighlightStyle {
        color: Some(rgb(0x409eff).into()),
        underline: Some(UnderlineStyle { thickness: px(1.), ..Default::default() }),
        ..Default::default()
    }),
]);

el_text().content(styled);"#,
                row().child(el_text().content(mixed)).into_any_element(),
            ))
    }
}
