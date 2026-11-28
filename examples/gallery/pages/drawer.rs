use gpui::{AnyElement, Context, Div, div, prelude::*, px};
use gpui_el::{DrawerDirection, el_button, el_drawer};

use crate::{Gallery, copy::Copy};

#[derive(Default)]
pub(crate) struct State {
    pub(crate) open: bool,
    pub(crate) direction: DrawerDirection,
    pub(crate) with_header: bool,
}

impl Gallery {
    pub(crate) fn drawers(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let direction =
            |id: &'static str, label: &str, value: DrawerDirection| {
                el_button().id(id).small().label(label).onclick(cx.listener(
                    move |this, _, _, cx| {
                        this.drawer.direction = value;
                        this.drawer.with_header = true;
                        this.drawer.open = true;
                        cx.notify();
                    },
                ))
            };
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "drawer-basic",
                &s.drawer_basic,
                || {
                    "el_drawer().open(open).direction(DrawerDirection::Right).title(\"Details\");"
                        .into()
                },
                div().flex().flex_wrap().gap(px(8.)).children([
                    direction(
                        "drawer-right",
                        s.drawer_right.as_str(),
                        DrawerDirection::Right,
                    ),
                    direction("drawer-left", s.drawer_left.as_str(), DrawerDirection::Left),
                    direction("drawer-top", s.drawer_top.as_str(), DrawerDirection::Top),
                    direction(
                        "drawer-bottom",
                        s.drawer_bottom.as_str(),
                        DrawerDirection::Bottom,
                    ),
                ]),
            ))
            .child(
                self.section(
                    cx,
                    "drawer-no-header",
                    &s.drawer_no_header,
                    || "el_drawer().with_header(false).size(360.);".into(),
                    el_button()
                        .id("drawer-no-header-open")
                        .label(s.drawer_open.as_str())
                        .onclick(cx.listener(|this, _, _, cx| {
                            this.drawer.with_header = false;
                            this.drawer.open = true;
                            cx.notify();
                        })),
                ),
            )
    }

    pub(crate) fn drawer_overlay(&self, s: &'static Copy, cx: &Context<Self>) -> AnyElement {
        let view = cx.entity();
        el_drawer()
            .id("drawer-demo")
            .open(self.drawer.open)
            .direction(self.drawer.direction)
            .with_header(self.drawer.with_header)
            .title(s.drawer_title.as_str())
            .child(s.drawer_body.as_str())
            .footer(
                el_button()
                    .id("drawer-close")
                    .label(s.drawer_close.as_str())
                    .onclick(cx.listener(|this, _, _, cx| {
                        this.drawer.open = false;
                        cx.notify();
                    })),
            )
            .onclose(move |_, cx| {
                view.update(cx, |this, cx| {
                    this.drawer.open = false;
                    cx.notify();
                });
            })
            .into_any_element()
    }
}
