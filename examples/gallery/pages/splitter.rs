use gpui::{Context, Div, div, prelude::*, px, rgb};
use gpui_el::{SplitterLayout, SplitterResizeEvent, el_splitter, el_splitter_panel, tokens};

use crate::{Gallery, copy::Copy, demo::row};

/// Reported sizes from the size demo, so the page can show what the events
/// delivered.
#[derive(Default)]
pub(crate) struct State {
    pub(crate) sizes: Vec<f32>,
}

const HEIGHT: f32 = 240.;

impl Gallery {
    pub(crate) fn splitters(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let section =
            |id, title, code: &str, controls| self.section(cx, id, title, || code.into(), controls);
        // The splitter fills its container, so the demos give it a box.
        let content = |label: &'static str, color: u32| {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(rgb(color))
                .child(label)
        };
        let panel =
            |label: &'static str, color: u32| el_splitter_panel().child(content(label, color));
        let sized =
            |splitter: gpui_el::Splitter| div().h(px(HEIGHT)).child(splitter).into_any_element();
        let report = row().child(div().text_size(px(tokens::font_size::SMALL)).child(
            s.splitter_report.replace(
                "%{sizes}",
                &if self.splitter.sizes.is_empty() {
                    s.splitter_none.to_string()
                } else {
                    self.splitter
                        .sizes
                        .iter()
                        .map(|size| format!("{size:.0}"))
                        .collect::<Vec<_>>()
                        .join(" · ")
                },
            ),
        ));

        div()
            .flex()
            .flex_col()
            .gap(px(24.))
            .child(section(
                "splitter-basic",
                &s.splitter_basic,
                r#"el_splitter()
    .id("basic")
    .panel(el_splitter_panel().child("Left"))
    .panel(el_splitter_panel().child("Right"));"#,
                sized(
                    el_splitter()
                        .id("basic")
                        .panel(panel(s.splitter_left.as_str(), 0x409eff))
                        .panel(panel(s.splitter_right.as_str(), 0x67c23a)),
                ),
            ))
            .child(section(
                "splitter-vertical",
                &s.splitter_vertical,
                r#"el_splitter()
    .id("vertical")
    .vertical()
    .panel(el_splitter_panel().child("Top"))
    .panel(el_splitter_panel().child("Bottom"));"#,
                sized(
                    el_splitter()
                        .id("vertical")
                        .layout(SplitterLayout::Vertical)
                        .panel(panel(s.splitter_top.as_str(), 0xe6a23c))
                        .panel(panel(s.splitter_bottom.as_str(), 0xf56c6c)),
                ),
            ))
            .child(section(
                "splitter-collapsible",
                &s.splitter_collapsible,
                r#"el_splitter()
    .id("collapsible")
    .panel(el_splitter_panel().min(120.).collapsible(true).child("Left"))
    .panel(el_splitter_panel().min(120.).collapsible(true).child("Right"));"#,
                sized(
                    el_splitter()
                        .id("collapsible")
                        .panel(
                            el_splitter_panel()
                                .min(120.)
                                .collapsible(true)
                                .child(content(s.splitter_left.as_str(), 0x409eff)),
                        )
                        .panel(
                            el_splitter_panel()
                                .min(120.)
                                .collapsible(true)
                                .child(content(s.splitter_right.as_str(), 0x67c23a)),
                        ),
                ),
            ))
            .child(section(
                "splitter-disabled",
                &s.splitter_disabled,
                r#"el_splitter()
    .id("disabled")
    .panel(el_splitter_panel().child("Left"))
    .panel(el_splitter_panel().resizable(false).child("Right"));"#,
                sized(
                    el_splitter()
                        .id("disabled")
                        .panel(panel(s.splitter_left.as_str(), 0x409eff))
                        .panel(
                            el_splitter_panel()
                                .resizable(false)
                                .child(content(s.splitter_frozen.as_str(), 0x67c23a)),
                        ),
                ),
            ))
            .child(section(
                "splitter-size",
                &s.splitter_size,
                r#"el_splitter()
    .id("sizes")
    .panel(el_splitter_panel().size(200.).child("Left"))
    .panel(el_splitter_panel().child("Right"))
    .onresize(move |_, sizes, _, _| {
        // sizes: SplitterSizes (Rc<[f32]>) in logical pixels
    });"#,
                div()
                    .child(sized(
                        el_splitter()
                            .id("sizes")
                            .panel(
                                el_splitter_panel()
                                    .size(200.)
                                    .child(content(s.splitter_left.as_str(), 0x409eff)),
                            )
                            .panel(panel(s.splitter_right.as_str(), 0x67c23a))
                            .onresize(cx.listener(|this, event: &SplitterResizeEvent, _, cx| {
                                this.splitter.sizes = event.sizes.to_vec();
                                cx.notify();
                            })),
                    ))
                    .child(report)
                    .into_any_element(),
            ))
            .child(section(
                "splitter-lazy",
                &s.splitter_lazy,
                r#"el_splitter()
    .id("lazy")
    .lazy(true)
    .panel(el_splitter_panel().child("Left"))
    .panel(el_splitter_panel().child("Right"));"#,
                sized(
                    el_splitter()
                        .id("lazy")
                        .lazy(true)
                        .panel(panel(s.splitter_left.as_str(), 0x409eff))
                        .panel(panel(s.splitter_right.as_str(), 0x67c23a)),
                ),
            ))
            .child(
                div()
                    .text_size(px(tokens::font_size::SMALL))
                    .child(s.splitter_keyboard.as_str()),
            )
    }
}
