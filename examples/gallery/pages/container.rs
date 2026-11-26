use crate::{Gallery, copy::Copy};
use gpui::{Context, Div, div, prelude::*, px, rgb};
use gpui_el::{el_aside, el_container, el_footer, el_header, el_main};

impl Gallery {
    pub(crate) fn containers(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let (bar_color, aside, main) = if self.dark {
            (0x213d5b, 0x1d3043, 0x18222c)
        } else {
            (0xc6e2ff, 0xd9ecff, 0xecf5ff)
        };
        let center = |text| {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(text)
        };
        let bar = || {
            el_header()
                .background(rgb(bar_color))
                .child(center(s.layout_header.as_str()))
        };
        let side = |id| {
            el_aside()
                .id(id)
                .width(200.)
                .background(rgb(aside))
                .child(center(s.layout_aside.as_str()))
        };
        let body = || {
            el_main()
                .padding(0.)
                .child(center(s.layout_main.as_str()).bg(rgb(main)))
        };
        let foot = || {
            el_footer()
                .background(rgb(bar_color))
                .child(center(s.layout_footer.as_str()))
        };
        let layouts = [
            (
                "Header / Main",
                210.,
                el_container().child(bar()).child(body()),
            ),
            (
                "Header / Main / Footer",
                270.,
                el_container().child(bar()).child(body()).child(foot()),
            ),
            (
                "Aside / Main / Aside",
                150.,
                el_container()
                    .child(side("left"))
                    .child(body())
                    .child(side("right")),
            ),
            (
                "Header / (Aside + Main)",
                210.,
                el_container()
                    .child(bar())
                    .child(el_container().child(side("left")).child(body())),
            ),
            (
                "Header / (Aside + (Main / Footer))",
                270.,
                el_container().child(bar()).child(
                    el_container()
                        .child(side("left"))
                        .child(el_container().child(body()).child(foot())),
                ),
            ),
            (
                "Aside + (Header / Main)",
                210.,
                el_container()
                    .child(side("left"))
                    .child(el_container().child(bar()).child(body())),
            ),
            (
                "Aside + (Header / Main / Footer)",
                270.,
                el_container()
                    .child(side("left"))
                    .child(el_container().child(bar()).child(body()).child(foot())),
            ),
            (
                "Header / (Aside + Main) / Footer",
                270.,
                el_container()
                    .child(bar())
                    .child(el_container().child(side("left")).child(body()))
                    .child(foot()),
            ),
        ];
        self.section(
            cx,
            "container",
            &s.container_title,
            || {
                r#"let header = || el_header().child("Header");
let main = || el_main().child("Main");
let aside = |id| el_aside().id(id).width(200.).child("Aside");
let footer = || el_footer().child("Footer");

el_container().child(header()).child(main());
el_container().child(header()).child(main()).child(footer());
el_container().child(aside("left")).child(main()).child(aside("right"));
el_container().child(header())
    .child(el_container().child(aside("left")).child(main()));
el_container().child(header()).child(
    el_container().child(aside("left"))
        .child(el_container().child(main()).child(footer())),
);
el_container().child(aside("left"))
    .child(el_container().child(header()).child(main()));
el_container().child(aside("left"))
    .child(el_container().child(header()).child(main()).child(footer()));
el_container().child(header())
    .child(el_container().child(aside("left")).child(main()))
    .child(footer());"#
                    .into()
            },
            div().flex().flex_col().gap(px(24.)).children(layouts.map(
                |(label, height, layout)| {
                    div().flex().flex_col().gap(px(8.)).child(label).child(
                        div()
                            .id(label)
                            .overflow_x_scroll()
                            .child(div().flex().min_w(px(480.)).h(px(height)).child(layout)),
                    )
                },
            )),
        )
    }
}
