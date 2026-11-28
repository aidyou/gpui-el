//! Element Plus Message.
//!
//! Follows `message.md`, `components/message/src`, and Theme Chalk
//! `message.scss`: transient semantic feedback, placement/offset, plain mode,
//! optional close, grouping, and a 3000 ms default auto-close timer.

use std::{collections::BTreeMap, rc::Rc, sync::Arc, time::Duration};

use gpui::{
    App, ElementId, Entity, IntoElement, RenderOnce, SharedString, Window, deferred, div,
    prelude::*, px, rgb,
};

use crate::{FeedbackType, Locale, Theme, el_button, el_icon, theme::popup_priority};

/// Message placement relative to the window.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum MessagePlacement {
    #[default]
    Top,
    TopLeft,
    TopRight,
    Bottom,
    BottomLeft,
    BottomRight,
}

impl MessagePlacement {
    fn is_bottom(self) -> bool {
        matches!(self, Self::Bottom | Self::BottomLeft | Self::BottomRight)
    }

    fn alignment(self) -> (bool, bool, bool) {
        match self {
            Self::TopLeft | Self::BottomLeft => (false, true, false),
            Self::TopRight | Self::BottomRight => (true, false, false),
            _ => (false, false, true),
        }
    }
}

type CloseHandler = Rc<dyn Fn(&mut App)>;

/// One transient message configuration.
#[derive(Clone)]
pub struct MessageOptions {
    message: SharedString,
    kind: FeedbackType,
    plain: bool,
    show_close: bool,
    duration: Duration,
    offset: f32,
    placement: MessagePlacement,
    grouping: bool,
    repeat_num: u32,
    onclose: Option<CloseHandler>,
}

impl MessageOptions {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            kind: FeedbackType::Info,
            plain: false,
            show_close: false,
            duration: Duration::from_millis(3000),
            offset: 16.,
            placement: MessagePlacement::Top,
            grouping: false,
            repeat_num: 1,
            onclose: None,
        }
    }

    pub fn kind(mut self, kind: FeedbackType) -> Self {
        self.kind = kind;
        self
    }

    pub fn plain(mut self, plain: bool) -> Self {
        self.plain = plain;
        self
    }

    pub fn show_close(mut self, show: bool) -> Self {
        self.show_close = show;
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn offset(mut self, offset: f32) -> Self {
        self.offset = offset.max(0.);
        self
    }

    pub fn placement(mut self, placement: MessagePlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn grouping(mut self, grouping: bool) -> Self {
        self.grouping = grouping;
        self
    }

    pub fn repeat_num(mut self, repeat: u32) -> Self {
        self.repeat_num = repeat.max(1);
        self
    }

    pub fn onclose(mut self, handler: impl Fn(&mut App) + 'static) -> Self {
        self.onclose = Some(Rc::new(handler));
        self
    }
}

struct MessageItem {
    id: u64,
    options: MessageOptions,
    repeats: u32,
    close_task: Option<gpui::Task<()>>,
}

/// Shared message queue. Render one [`MessageStack`] at the window root.
#[derive(Default)]
pub struct MessageState {
    items: Vec<MessageItem>,
    next_id: u64,
}

impl MessageState {
    pub fn new(cx: &mut App) -> Entity<Self> {
        cx.new(|_| Self::default())
    }

    pub fn push(&mut self, options: MessageOptions, cx: &mut gpui::Context<Self>) -> u64 {
        let grouped = if options.grouping {
            self.items
                .iter_mut()
                .find(|item| {
                    item.options.message == options.message
                        && item.options.kind == options.kind
                        && item.options.placement == options.placement
                        && item.options.plain == options.plain
                })
                .map(|item| {
                    item.close_task.take();
                    item.options = options.clone();
                    item.repeats = item.repeats.saturating_add(options.repeat_num);
                    item.id
                })
        } else {
            None
        };
        if let Some(id) = grouped {
            self.schedule_close(id, options.duration, cx);
            cx.notify();
            return id;
        }
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.items.push(MessageItem {
            id,
            options: options.clone(),
            repeats: options.repeat_num,
            close_task: None,
        });
        self.schedule_close(id, options.duration, cx);
        cx.notify();
        id
    }

    fn schedule_close(&mut self, id: u64, duration: Duration, cx: &mut gpui::Context<Self>) {
        if duration.is_zero() {
            return;
        }
        let task = cx.spawn(async move |entity, cx| {
            cx.background_executor().timer(duration).await;
            let _ = entity.update(cx, |state, cx| state.close_internal(id, cx));
        });
        if let Some(item) = self.items.iter_mut().find(|item| item.id == id) {
            item.close_task = Some(task);
        }
    }

    fn close_internal(&mut self, id: u64, cx: &mut gpui::Context<Self>) {
        let Some(index) = self.items.iter().position(|item| item.id == id) else {
            return;
        };
        let item = self.items.remove(index);
        if let Some(handler) = item.options.onclose {
            handler(cx);
        }
        cx.notify();
    }

    pub fn close(&mut self, id: u64, cx: &mut gpui::Context<Self>) {
        if let Some(item) = self.items.iter_mut().find(|item| item.id == id) {
            item.close_task.take();
        }
        self.close_internal(id, cx);
    }

    pub fn close_all(&mut self, cx: &mut gpui::Context<Self>) {
        let ids: Vec<_> = self.items.iter().map(|item| item.id).collect();
        for id in ids {
            self.close(id, cx);
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Renders a [`MessageState`] as window-level transient feedback.
#[derive(IntoElement)]
#[must_use]
pub struct MessageStack {
    id: ElementId,
    state: Entity<MessageState>,
}

/// Creates a message stack for a shared queue.
#[track_caller]
pub fn el_message_stack(state: Entity<MessageState>) -> MessageStack {
    MessageStack::new(state)
}

impl MessageStack {
    pub fn new(state: Entity<MessageState>) -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            state,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for MessageStack {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let items = self.state.read(cx);
        if items.items.is_empty() {
            return div().into_any_element();
        }
        let theme = Theme::current(cx);
        let locale = Locale::current(cx);
        let viewport = window.viewport_size();
        let mut groups: BTreeMap<MessagePlacement, Vec<&MessageItem>> = BTreeMap::new();
        for item in &items.items {
            groups.entry(item.options.placement).or_default().push(item);
        }
        let mut root = div()
            .id(self.id.clone())
            .debug_selector(|| "message-stack".into())
            .absolute()
            .inset_0();
        for (placement, items) in groups {
            let (right, left, center) = placement.alignment();
            let offset = items.first().map(|item| item.options.offset).unwrap_or(16.);
            let mut stack = div()
                .absolute()
                .inset_0()
                .flex()
                .flex_col()
                .gap(px(12.))
                .when(right, |el| el.items_end().pr(px(offset)))
                .when(left, |el| el.items_start().pl(px(offset)))
                .when(center, |el| el.items_center())
                .when(placement.is_bottom(), |el| el.justify_end().pb(px(offset)))
                .when(!placement.is_bottom(), |el| {
                    el.justify_start().pt(px(offset))
                });
            for item in items {
                let options = &item.options;
                let color = options.kind.color();
                let background = if options.plain {
                    theme.overlay_background()
                } else {
                    options.kind.tint(&theme, 9)
                };
                let border = if options.plain {
                    theme.overlay_background()
                } else {
                    options.kind.tint(&theme, 8)
                };
                let mut message = div()
                    .id(ElementId::NamedChild(
                        Arc::new(self.id.clone()),
                        format!("message-{}", item.id).into(),
                    ))
                    .relative()
                    .max_w(viewport.width - px(32.))
                    .px(px(15.))
                    .py(px(11.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .rounded(px(4.))
                    .border_1()
                    .border_color(rgb(border))
                    .bg(rgb(background))
                    .text_color(rgb(color))
                    .when(options.plain, |el| el.shadow(theme.overlay_shadow()))
                    .aria_label(options.message.clone())
                    .child(el_icon(options.kind.icon()).size(16.))
                    .child(
                        div()
                            .text_size(px(14.))
                            .line_height(px(14.))
                            .child(options.message.clone()),
                    );
                if options.show_close {
                    message = message.child(
                        el_button()
                            .id(ElementId::NamedChild(
                                Arc::new(self.id.clone()),
                                format!("close-{}", item.id).into(),
                            ))
                            .text()
                            .small()
                            .icon(el_icon("close").size(16.))
                            .accessibility_label(locale.tag_close())
                            .onclick({
                                let state = self.state.clone();
                                let id = item.id;
                                move |_, _, cx| state.update(cx, |state, cx| state.close(id, cx))
                            }),
                    );
                }
                if item.repeats > 1 {
                    message = message.child(
                        div()
                            .absolute()
                            .top(px(-8.))
                            .right(px(-8.))
                            .px(px(6.))
                            .py(px(2.))
                            .rounded_full()
                            .bg(rgb(crate::theme::DANGER))
                            .text_color(rgb(0xffffff))
                            .text_size(px(12.))
                            .child(item.repeats.to_string()),
                    );
                }
                stack = stack.child(message);
            }
            root = root.child(stack);
        }
        deferred(root)
            .with_priority(popup_priority(30))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn auto_close_and_grouping_follow_upstream_defaults(cx: &mut gpui::TestAppContext) {
        let state = cx.update(MessageState::new);
        state.update(cx, |state, cx| {
            state.push(
                MessageOptions::new("Saved")
                    .duration(Duration::from_millis(50))
                    .grouping(true),
                cx,
            );
            state.push(
                MessageOptions::new("Saved")
                    .duration(Duration::from_millis(50))
                    .grouping(true),
                cx,
            );
        });
        assert_eq!(cx.update(|cx| state.read(cx).len()), 1);
        assert_eq!(cx.update(|cx| state.read(cx).items[0].repeats), 2);
        cx.executor().advance_clock(Duration::from_millis(60));
        cx.run_until_parked();
        assert!(cx.update(|cx| state.read(cx).is_empty()));
    }

    #[gpui::test]
    fn zero_duration_keeps_the_message_open(cx: &mut gpui::TestAppContext) {
        let state = cx.update(MessageState::new);
        state.update(cx, |state, cx| {
            state.push(
                MessageOptions::new("Persistent").duration(Duration::ZERO),
                cx,
            );
        });
        cx.executor().advance_clock(Duration::from_secs(10));
        cx.run_until_parked();
        assert_eq!(cx.update(|cx| state.read(cx).len()), 1);
    }
}
