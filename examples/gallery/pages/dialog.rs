use gpui::{AnyElement, Context, Div, div, prelude::*, px};
use gpui_el::{el_button, el_dialog};

use crate::{Gallery, copy::Copy};

#[derive(Default)]
pub(crate) struct State {
    pub(crate) open: bool,
    pub(crate) draggable: bool,
    pub(crate) fullscreen: bool,
    pub(crate) align_center: bool,
    pub(crate) modal: bool,
}

#[derive(Clone, Copy)]
enum Demo {
    Basic,
    Draggable,
    Fullscreen,
    Centered,
    Modeless,
}

impl Gallery {
    pub(crate) fn dialogs(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let open = |id: &'static str, label: &str, primary: bool, demo: Demo| {
            let mut button = el_button().id(id).label(label);
            if primary {
                button = button.primary();
            }
            button.onclick(cx.listener(move |this, _, _, cx| {
                this.dialog.open = true;
                this.dialog.draggable = matches!(demo, Demo::Draggable);
                this.dialog.fullscreen = matches!(demo, Demo::Fullscreen);
                this.dialog.align_center = matches!(demo, Demo::Centered);
                this.dialog.modal = !matches!(demo, Demo::Modeless);
                cx.notify();
            }))
        };
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                self.section(
                    cx,
                    "dialog-basic",
                    &s.dialog_basic,
                    || {
                        "el_dialog().open(open).title(\"Confirm\").child(\"Body\").footer(actions);"
                            .into()
                    },
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(12.))
                        .child(open(
                            "dialog-open",
                            s.dialog_open.as_str(),
                            true,
                            Demo::Basic,
                        ))
                        .child(open(
                            "dialog-open-draggable",
                            s.dialog_draggable.as_str(),
                            false,
                            Demo::Draggable,
                        )),
                ),
            )
            .child(
                self.section(
                    cx,
                    "dialog-options",
                    &s.dialog_options,
                    || "el_dialog().fullscreen(true).align_center(true).modal(false);".into(),
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(12.))
                        .child(open(
                            "dialog-open-fullscreen",
                            s.dialog_fullscreen.as_str(),
                            false,
                            Demo::Fullscreen,
                        ))
                        .child(open(
                            "dialog-open-centered",
                            s.dialog_center.as_str(),
                            false,
                            Demo::Centered,
                        ))
                        .child(open(
                            "dialog-open-modeless",
                            s.dialog_modeless.as_str(),
                            false,
                            Demo::Modeless,
                        )),
                ),
            )
    }

    pub(crate) fn dialog_overlay(&self, s: &'static Copy, cx: &Context<Self>) -> AnyElement {
        let view = cx.entity();
        el_dialog()
            .id("dialog-demo")
            .open(self.dialog.open)
            .title(s.dialog_title.as_str())
            .draggable(self.dialog.draggable)
            .fullscreen(self.dialog.fullscreen)
            .align_center(self.dialog.align_center)
            .modal(self.dialog.modal)
            .child(s.dialog_body.as_str())
            .footer(
                div().flex().gap(px(8.)).children([
                    el_button()
                        .id("dialog-cancel")
                        .label(s.dialog_cancel.as_str())
                        .onclick(cx.listener(|this, _, _, cx| {
                            this.dialog.open = false;
                            cx.notify();
                        })),
                    el_button()
                        .id("dialog-confirm")
                        .primary()
                        .label(s.dialog_confirm.as_str())
                        .onclick(cx.listener(|this, _, _, cx| {
                            this.dialog.open = false;
                            cx.notify();
                        })),
                ]),
            )
            .onclose(move |_, cx| {
                view.update(cx, |this, cx| {
                    this.dialog.open = false;
                    cx.notify();
                });
            })
            .into_any_element()
    }
}
