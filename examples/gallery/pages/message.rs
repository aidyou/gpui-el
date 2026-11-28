use std::time::Duration;

use gpui::{AnyElement, Context, Div, div, prelude::*, px};
use gpui_el::{FeedbackType, MessageOptions, MessagePlacement, el_button, el_message_stack};

use crate::{Gallery, copy::Copy};

impl Gallery {
    pub(crate) fn messages(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let Some(state) = self.message.as_ref().cloned() else {
            return div();
        };
        let push = |id: &'static str, label: &str, options: MessageOptions| {
            let state = state.clone();
            el_button().id(id).label(label).onclick(move |_, _, cx| {
                state.update(cx, |state, cx| state.push(options.clone(), cx));
            })
        };
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "message-basic",
                &s.message_basic,
                || "let id = state.push(MessageOptions::new(\"Saved\"), cx);".into(),
                div().flex().flex_wrap().gap(px(8.))
                    .child(push("message-info", s.info.as_str(), MessageOptions::new(s.message_saved.as_str())))
                    .child(push("message-success", s.success.as_str(), MessageOptions::new(s.message_saved.as_str()).kind(FeedbackType::Success)))
                    .child(push("message-warning", s.warning.as_str(), MessageOptions::new(s.message_warning.as_str()).kind(FeedbackType::Warning))),
            ))
            .child(self.section(
                cx,
                "message-options",
                &s.message_options,
                || "MessageOptions::new(\"Persistent\").duration(Duration::ZERO).show_close(true);".into(),
                div().flex().flex_wrap().gap(px(8.))
                    .child(push("message-persistent", s.message_persistent.as_str(), MessageOptions::new(s.message_persistent.as_str()).duration(Duration::ZERO).show_close(true)))
                    .child(push("message-plain", s.message_plain.as_str(), MessageOptions::new(s.message_plain.as_str()).plain(true).show_close(true)))
                    .child(push("message-group-one", s.message_group.as_str(), MessageOptions::new(s.message_group.as_str()).grouping(true).show_close(true)))
                    .child(push("message-group-two", s.message_group.as_str(), MessageOptions::new(s.message_group.as_str()).grouping(true).show_close(true)))
                    .child(push("message-bottom", s.message_bottom.as_str(), MessageOptions::new(s.message_bottom.as_str()).placement(MessagePlacement::BottomRight))),
            ))
            .child(
                el_button()
                    .id("message-clear")
                    .small()
                    .label(s.message_clear.as_str())
                    .onclick(move |_, _, cx| {
                        state.update(cx, |state, cx| state.close_all(cx));
                    }),
            )
    }

    pub(crate) fn message_overlay(&self, _cx: &Context<Self>) -> AnyElement {
        let Some(state) = self.message.as_ref().cloned() else {
            return div().into_any_element();
        };
        el_message_stack(state)
            .id("message-stack")
            .into_any_element()
    }
}
