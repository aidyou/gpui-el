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
        let block = |tone, height| {
            div()
                .h(px(height))
                .rounded(px(tokens::radius::BASE))
                .bg(rgb(tone))
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
                    el_col().span(span).child(block(tone, 36.))
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
                    .col(el_col().span(6).child(block(light, 36.)))
                    .col(el_col().span(6).offset(6).child(block(light, 36.))),
            )
            .child(
                el_row()
                    .gutter(20.)
                    .cols((0..2).map(|_| el_col().span(6).offset(6).child(block(light, 36.)))),
            )
            .child(
                el_row()
                    .gutter(20.)
                    .col(el_col().span(12).offset(6).child(block(light, 36.))),
            )
            .child(
                el_row()
                    .gutter(20.)
                    .col(el_col().span(12).push(12).child(block(light, 36.)))
                    .col(el_col().span(12).pull(12).child(block(alternate, 36.))),
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
                                    .col(el_col().span(8).child(block(light, 24.)))
                                    .col(el_col().span(8).child(block(alternate, 56.)))
                                    .col(el_col().span(8).child(block(light, 36.))),
                            ),
                        )
                    }),
                ),
            );
        let responsive = el_row().gutter(10.).cols((0..4).map(|i| {
            if i == 0 || i == 3 {
                el_col().xs(8).sm(6).md(4).lg(3).xl(1)
            } else {
                el_col().xs(4).sm(6).md(8).lg(9).xl(11)
            }
            .child(block(if i % 2 == 0 { light } else { alternate }, 36.))
        }));
        let section = |id, title, controls| {
            self.section(cx, id, title, || include_str!("layout.rs").into(), controls)
        };
        stack()
            .child(self.section(
                cx,
                "grid-basic",
                &s.grid_basic,
                || include_str!("layout.rs").into(),
                basic,
            ))
            .child(self.section(
                cx,
                "grid-gutter",
                &s.grid_gutter,
                || include_str!("layout.rs").into(),
                gutter,
            ))
            .child(section("grid-mixed", &s.grid_mixed, mixed))
            .child(section("grid-offset", &s.grid_offset, offset))
            .child(section("grid-alignment", &s.grid_alignment, alignment))
            .child(section(
                "grid-responsive",
                &s.grid_responsive,
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(s.grid_resize.as_str())
                    .child(responsive),
            ))
    }
}
