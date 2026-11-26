use gpui::{Context, Div, div, prelude::*, px, rgb};
use gpui_el::{RowAlign, RowJustify, el_col, el_row, tokens};

use crate::{Gallery, copy::Copy};

impl Gallery {
    pub(crate) fn grids(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        // Element Plus docs/examples/layout/index.scss: demo colors, not component defaults.
        let (panel, colors) = if self.dark {
            (0x18191a, [0x46494d, 0x667180, 0x242526])
        } else {
            (0xf9fafc, [0xd3dce6, 0xe5e9f2, 0x99a9bf])
        };
        let block = |tone, height, label: &'static str| {
            div()
                .h(px(height))
                .rounded(px(tokens::radius::BASE))
                .bg(rgb(tone))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(tokens::font_size::EXTRA_SMALL))
                .child(label)
        };
        let [light, alternate, dark] = colors;
        let columns = |spans: &[u8], gutter| {
            el_row()
                .gutter(gutter)
                .cols(spans.iter().enumerate().map(|(i, &span)| {
                    let tone = if spans.len() == 1 {
                        dark
                    } else if i % 2 == 0 {
                        light
                    } else {
                        alternate
                    };
                    el_col().span(span).child(block(tone, 36., "Content"))
                }))
        };
        let stack = || div().flex().flex_col().gap(px(20.));
        let basic = stack().children(
            [&[24][..], &[12, 12], &[8, 8, 8], &[6; 4], &[4; 6]].map(|spans| columns(spans, 0.)),
        );
        let gutter = columns(&[6; 4], 20.);
        let mixed = stack()
            .children([&[16, 8][..], &[8, 8, 4, 4], &[4, 16, 4]].map(|spans| columns(spans, 20.)));
        let offset = stack()
            .child(
                el_row()
                    .gutter(20.)
                    .col(el_col().span(6).child(block(light, 36., "Content")))
                    .col(
                        el_col()
                            .span(6)
                            .offset(6)
                            .child(block(light, 36., "Content")),
                    ),
            )
            .child(el_row().gutter(20.).cols((0..2).map(|_| {
                el_col()
                    .span(6)
                    .offset(6)
                    .child(block(light, 36., "Content"))
            })))
            .child(
                el_row().gutter(20.).col(
                    el_col()
                        .span(12)
                        .offset(6)
                        .child(block(light, 36., "Content")),
                ),
            )
            .child(
                el_row()
                    .gutter(20.)
                    .col(
                        el_col()
                            .span(12)
                            .push(12)
                            .child(block(light, 36., "Content")),
                    )
                    .col(
                        el_col()
                            .span(12)
                            .pull(12)
                            .child(block(alternate, 36., "Content")),
                    ),
            );
        let alignment = stack()
            .children(
                [
                    ("start", RowJustify::Start),
                    ("center", RowJustify::Center),
                    ("end", RowJustify::End),
                    ("space-between", RowJustify::SpaceBetween),
                    ("space-around", RowJustify::SpaceAround),
                    ("space-evenly", RowJustify::SpaceEvenly),
                ]
                .map(|(name, justify)| {
                    div().flex().flex_col().gap(px(4.)).child(name).child(
                        div()
                            .py(px(10.))
                            .bg(rgb(panel))
                            .child(columns(&[6; 3], 0.).justify(justify)),
                    )
                }),
            )
            .child(
                stack().children(
                    [
                        ("top", RowAlign::Top),
                        ("middle", RowAlign::Middle),
                        ("bottom", RowAlign::Bottom),
                    ]
                    .map(|(name, align)| {
                        div().flex().flex_col().gap(px(4.)).child(name).child(
                            div().py(px(10.)).bg(rgb(panel)).child(
                                el_row()
                                    .gutter(20.)
                                    .align(align)
                                    .col(el_col().span(8).child(block(light, 24., "Short")))
                                    .col(el_col().span(8).child(block(alternate, 56., "Tall")))
                                    .col(el_col().span(8).child(block(light, 36., "Medium"))),
                            ),
                        )
                    }),
                ),
            );
        let responsive = el_row().gutter(10.).cols((0..4).map(|i| {
            let narrow = i == 0 || i == 3;
            if narrow {
                el_col().xs(8).sm(6).md(4).lg(3).xl(1)
            } else {
                el_col().xs(4).sm(6).md(8).lg(9).xl(11)
            }
            .child(block(
                if i % 2 == 0 { light } else { alternate },
                36.,
                if narrow { "Narrow" } else { "Wide" },
            ))
        }));
        let section =
            |id, title, code: &str, controls| self.section(cx, id, title, || code.into(), controls);
        stack()
            .child(section(
                "grid-basic",
                &s.grid_basic,
                r#"let col = |span| el_col().span(span).child("Content");
el_row().col(col(24));
el_row().cols([col(12), col(12)]);
el_row().cols((0..3).map(|_| col(8)));
el_row().cols((0..4).map(|_| col(6)));
el_row().cols((0..6).map(|_| col(4)));"#,
                basic,
            ))
            .child(section(
                "grid-gutter",
                &s.grid_gutter,
                r#"el_row().gutter(20.).cols((0..4).map(|_| {
    el_col().span(6).child("Content")
}));"#,
                div().child(gutter),
            ))
            .child(section(
                "grid-mixed",
                &s.grid_mixed,
                r#"let col = |span| el_col().span(span).child("Content");
el_row().gutter(20.).cols([col(16), col(8)]);
el_row().gutter(20.).cols([col(8), col(8), col(4), col(4)]);
el_row().gutter(20.).cols([col(4), col(16), col(4)]);"#,
                mixed,
            ))
            .child(section(
                "grid-offset",
                &s.grid_offset,
                r#"let col = |span| el_col().span(span).child("Content");
el_row().gutter(20.).cols([col(6), col(6).offset(6)]);
el_row().gutter(20.).cols([col(6).offset(6), col(6).offset(6)]);
el_row().gutter(20.).col(col(12).offset(6));
el_row().gutter(20.).cols([col(12).push(12), col(12).pull(12)]);"#,
                offset,
            ))
            .child(section(
                "grid-alignment",
                &s.grid_alignment,
                r#"let justified = |justify| el_row().justify(justify)
    .cols((0..3).map(|_| el_col().span(6).child("Content")));
justified(RowJustify::Start);
justified(RowJustify::Center);
justified(RowJustify::End);
justified(RowJustify::SpaceBetween);
justified(RowJustify::SpaceAround);
justified(RowJustify::SpaceEvenly);

let aligned = |align| el_row().gutter(20.).align(align).cols([
    el_col().span(8).child(div().h(px(24.)).child("Short")),
    el_col().span(8).child(div().h(px(56.)).child("Tall")),
    el_col().span(8).child(div().h(px(36.)).child("Medium")),
]);
aligned(RowAlign::Top);
aligned(RowAlign::Middle);
aligned(RowAlign::Bottom);"#,
                alignment,
            ))
            .child(section(
                "grid-responsive",
                &s.grid_responsive,
                r#"let narrow = || el_col().xs(8).sm(6).md(4).lg(3).xl(1).child("Narrow");
let wide = || el_col().xs(4).sm(6).md(8).lg(9).xl(11).child("Wide");
el_row().gutter(10.).cols([narrow(), wide(), wide(), narrow()]);"#,
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(s.grid_resize.as_str())
                    .child(responsive),
            ))
    }
}
