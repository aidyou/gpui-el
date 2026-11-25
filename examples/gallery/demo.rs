use gpui::{ClipboardItem, Context, Div, IntoElement, SharedString, div, prelude::*, px, rgb};
use gpui_el::{Locale, el_button, el_icon, tokens};

pub(crate) use crate::row::row;
use crate::{Gallery, copy::strings};

// Keep the snippet tied to the executing page, including its imports and state.
pub(crate) fn source(page: &str) -> String {
    format!("{page}\n\n// gallery/row.rs\n{}", include_str!("row.rs"))
}

impl Gallery {
    pub(crate) fn section(
        &self,
        cx: &Context<Self>,
        id: &'static str,
        title: &str,
        example: impl FnOnce() -> String,
        controls: impl IntoElement,
    ) -> Div {
        let s = strings(Locale::current(cx));
        let snippet: Option<SharedString> = self.expanded.contains(id).then(|| example().into());
        let expanded = snippet.is_some();
        let border = rgb(if self.dark { 0x414243 } else { 0xe4e7ed });
        let fill = rgb(if self.dark { 0x262727 } else { 0xf5f7fa });
        div()
            .flex()
            .flex_col()
            .gap(px(tokens::spacing::DEFAULT))
            .child(
                div()
                    .text_size(px(18.))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(title.to_owned()),
            )
            .child(
                div()
                    .border_1()
                    .border_color(border)
                    .rounded(px(tokens::radius::BASE))
                    .child(div().p(px(20.)).child(controls))
                    .when_some(snippet.clone(), |el, snippet| {
                        el.child(
                            div()
                                .border_t_1()
                                .border_color(border)
                                .bg(fill)
                                .p(px(12.))
                                .font_family("monospace")
                                .text_size(px(13.))
                                .child(snippet),
                        )
                    })
                    .child(
                        div().border_t_1().border_color(border).p(px(6.)).child(
                            row()
                                .child(
                                    el_button()
                                        .id((id, 0_u32))
                                        .small()
                                        .text()
                                        .label(if expanded {
                                            s.hide_code.as_str()
                                        } else {
                                            s.show_code.as_str()
                                        })
                                        .icon(el_icon(if expanded {
                                            "arrow-up"
                                        } else {
                                            "arrow-down"
                                        }))
                                        .onclick(cx.listener(move |this, _, _, cx| {
                                            if !this.expanded.insert(id) {
                                                this.expanded.remove(id);
                                            }
                                            cx.notify();
                                        })),
                                )
                                .when_some(snippet, |el, snippet| {
                                    el.child(
                                        el_button()
                                            .id((id, 1_u32))
                                            .small()
                                            .text()
                                            .label(s.copy.as_str())
                                            .icon(el_icon("document-copy"))
                                            .onclick(move |_, _, cx| {
                                                cx.write_to_clipboard(ClipboardItem::new_string(
                                                    snippet.to_string(),
                                                ))
                                            }),
                                    )
                                }),
                        ),
                    ),
            )
    }
}
