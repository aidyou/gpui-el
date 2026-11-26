use gpui::{Context, Div, Point, div, prelude::*, px, rgb};
use gpui::{ScrollHandle};
use gpui_el::{ScrollbarDirection, ScrollbarEvent, el_button, el_scrollbar, tokens};

use crate::{Gallery, copy::Copy, demo::row};

/// Infinite-scroll demo state: the items on the left and the last reported
/// offsets, so the page can show what the events delivered.
pub(crate) struct State {
    pub(crate) handle: ScrollHandle,
    pub(crate) items: Vec<u32>,
    pub(crate) scroll_left: f32,
    pub(crate) scroll_top: f32,
    pub(crate) reached: Option<ScrollbarDirection>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            handle: ScrollHandle::new(),
            items: (1..=20).collect(),
            scroll_left: 0.,
            scroll_top: 0.,
            reached: None,
        }
    }
}

impl State {
    fn more(&mut self) {
        let start = self.items.len() as u32 + 1;
        self.items.extend(start..start + 5);
    }
}

const FIXED_HEIGHT: f32 = 240.;
const MAX_HEIGHT: f32 = 200.;

impl Gallery {
    pub(crate) fn scrollbars(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let section =
            |id, title, code: &str, controls| self.section(cx, id, title, || code.into(), controls);
        let border = rgb(if self.dark { 0x414243 } else { 0xe4e7ed });
        let stripe = rgb(if self.dark { 0x262727 } else { 0xf5f7fa });
        // A row of labeled stripes stands in for any content wider than the box.
        // GPUI has no inline-block content wrapper, so horizontally
        // scrolling content must state its width; a block child alone stays
        // at the viewport width and only paints outside.
        let wide = |labels: &[&'static str]| {
            div()
                .w(px(labels.len() as f32 * 172. - 12.))
                .flex()
                .gap(px(12.))
                .children(labels.iter().enumerate().map(|(i, label)| {
                div()
                    .flex_none()
                    .w(px(160.))
                    .h(px(120.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(tokens::radius::BASE))
                    .bg(if i % 2 == 0 { stripe } else { border })
                    .child(*label)
            }))
        };
        let list = |items: &[u32]| {
            div().flex().flex_col().children(
                items.iter().map(|item| {
                    div()
                        .h(px(36.))
                        .flex()
                        .items_center()
                        .px(px(12.))
                        .child(format!("{item}. {}", s.scrollbar_row))
                }),
            )
        };
        let report = div()
            .text_size(px(tokens::font_size::SMALL))
            .child(s.scrollbar_report.replace(
                "%{top}",
                &format!("{:.0}", self.scrollbar.scroll_top),
            ));

        let handle = self.scrollbar.handle.clone();
        let manual = row()
            .child(
                el_button()
                    .id("scroll-to-top")
                    .label(s.scrollbar_to_top.as_str())
                    .onclick({
                        let handle = handle.clone();
                        move |_, window, _| {
                            handle.set_offset(Point::default());
                            window.refresh();
                        }
                    }),
            )
            .child(
                el_button()
                    .id("scroll-to-bottom")
                    .label(s.scrollbar_to_bottom.as_str())
                    .onclick({
                        let handle = handle.clone();
                        move |_, window, _| {
                            handle.set_offset(gpui::point(px(0.), px(-10_000.)));
                            window.refresh();
                        }
                    }),
            );

        div()
            .flex()
            .flex_col()
            .gap(px(24.))
            .child(section(
                "scrollbar-basic",
                &s.scrollbar_basic,
                r#"el_scrollbar()
    .id("basic")
    .height(240.)
    .onscroll(cx.listener(|this, event: &ScrollbarEvent, _, cx| {
        this.scrollbar.scroll_top = event.scroll_top;
        cx.notify();
    }))
    .child(list);"#,
                div()
                    .child(
                        el_scrollbar()
                            .id("basic")
                            .height(FIXED_HEIGHT)
                            .onscroll(cx.listener(|this, event: &ScrollbarEvent, _, cx| {
                                this.scrollbar.scroll_top = event.scroll_top;
                                this.scrollbar.scroll_left = event.scroll_left;
                                cx.notify();
                            }))
                            .child(list(&self.scrollbar.items)),
                    )
                    .child(report)
                    .into_any_element(),
            ))
            .child(section(
                "scrollbar-horizontal",
                &s.scrollbar_horizontal,
                r#"el_scrollbar()
    .id("horizontal")
    .height(160.)
    .child(row_of_fixed_width_boxes);"#,
                el_scrollbar()
                    .id("horizontal")
                    .height(160.)
                    .child(wide(&[s.scrollbar_row.as_str(); 6]))
                    .into_any_element(),
            ))
            .child(section(
                "scrollbar-max-height",
                &s.scrollbar_max_height,
                r#"el_scrollbar()
    .id("max-height")
    .max_height(200.)
    .child(list);"#,
                el_scrollbar()
                    .id("max-height")
                    .max_height(MAX_HEIGHT)
                    .child(list(&self.scrollbar.items))
                    .into_any_element(),
            ))
            .child(section(
                "scrollbar-manual",
                &s.scrollbar_manual,
                r#"let handle = ScrollHandle::new();

el_scrollbar()
    .id("manual")
    .height(200.)
    .handle(handle.clone())
    .child(list);

// In a button handler:
handle.set_offset(Point::default());           // to top
handle.set_offset(point(px(0.), px(-10_000.))); // to bottom, clamped
window.refresh();"#,
                div()
                    .child(
                        el_scrollbar()
                            .id("manual")
                            .height(200.)
                            .handle(handle.clone())
                            .child(list(&self.scrollbar.items)),
                    )
                    .child(manual)
                    .into_any_element(),
            ))
            .child(section(
                "scrollbar-infinite",
                &s.scrollbar_infinite,
                r#"el_scrollbar()
    .id("infinite")
    .height(200.)
    .on_end_reached(0., cx.listener(|this, direction: &ScrollbarDirection, _, cx| {
        if direction == ScrollbarDirection::Bottom {
            this.scrollbar.more();
            cx.notify();
        }
    }))
    .child(list);"#,
                div()
                    .child(
                        el_scrollbar()
                            .id("infinite")
                            .height(200.)
                            .on_end_reached(0., cx.listener(|this, direction: &ScrollbarDirection, _, cx| {
                                this.scrollbar.reached = Some(*direction);
                                this.scrollbar.more();
                                cx.notify();
                            }))
                            .child(list(&self.scrollbar.items)),
                    )
                    .child(
                        div().text_size(px(tokens::font_size::SMALL)).child(
                            s.scrollbar_reached
                                .replace("%{count}", &self.scrollbar.items.len().to_string())
                                .replace(
                                    "%{direction}",
                                    match self.scrollbar.reached {
                                        Some(ScrollbarDirection::Bottom) => &s.scrollbar_bottom,
                                        Some(ScrollbarDirection::Top) => &s.scrollbar_top,
                                        Some(ScrollbarDirection::Left) => &s.scrollbar_left,
                                        Some(ScrollbarDirection::Right) => &s.scrollbar_right,
                                        None => &s.scrollbar_none,
                                    },
                                ),
                        ),
                    )
                    .into_any_element(),
            ))
    }
}
