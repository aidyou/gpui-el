use std::time::Duration;

use gpui::{AnyElement, Context, Div, div, prelude::*, px};
use gpui_el::{
    FeedbackType, NotificationOptions, NotificationPosition, el_button, el_notification_stack,
};

use crate::{Gallery, copy::Copy};

impl Gallery {
    pub(crate) fn notifications(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let Some(state) = self.notification.as_ref().cloned() else {
            return div();
        };
        let push = |id: &'static str, label: &str, options: NotificationOptions| {
            let state = state.clone();
            el_button().id(id).label(label).onclick(move |_, _, cx| {
                state.update(cx, |state, cx| state.push(options.clone(), cx));
            })
        };
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                self.section(
                    cx,
                    "notification-basic",
                    &s.notification_basic,
                    || {
                        "state.push(NotificationOptions::new(\"Saved\").title(\"Success\"), cx);"
                            .into()
                    },
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(8.))
                        .child(push(
                            "notification-info",
                            s.info.as_str(),
                            NotificationOptions::new(s.message_saved.as_str())
                                .title(s.notification_title.as_str()),
                        ))
                        .child(push(
                            "notification-success",
                            s.success.as_str(),
                            NotificationOptions::new(s.message_saved.as_str())
                                .title(s.notification_title.as_str())
                                .kind(FeedbackType::Success),
                        )),
                ),
            )
            .child(
                self.section(
                    cx,
                    "notification-options",
                    &s.notification_options,
                    || {
                        "NotificationOptions::new(\"Working\").progress(true).pause_on_hover(true);"
                            .into()
                    },
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(8.))
                        .child(push(
                            "notification-progress",
                            s.notification_progress.as_str(),
                            NotificationOptions::new(s.notification_working.as_str())
                                .kind(FeedbackType::Primary)
                                .progress(true),
                        ))
                        .child(push(
                            "notification-bottom",
                            s.notification_bottom.as_str(),
                            NotificationOptions::new(s.message_saved.as_str())
                                .position(NotificationPosition::BottomLeft),
                        ))
                        .child(push(
                            "notification-persistent",
                            s.message_persistent.as_str(),
                            NotificationOptions::new(s.message_persistent.as_str())
                                .duration(Duration::ZERO),
                        )),
                ),
            )
            .child(
                el_button()
                    .id("notification-clear")
                    .small()
                    .label(s.message_clear.as_str())
                    .onclick(move |_, _, cx| {
                        state.update(cx, |state, cx| state.close_all(cx));
                    }),
            )
    }

    pub(crate) fn notification_overlay(&self, _cx: &Context<Self>) -> AnyElement {
        let Some(state) = self.notification.as_ref().cloned() else {
            return div().into_any_element();
        };
        el_notification_stack(state)
            .id("notification-stack")
            .into_any_element()
    }
}
