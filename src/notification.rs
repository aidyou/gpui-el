//! Element Plus Notification.
//!
//! Follows `notification.md`, `components/notification/src`, and Theme Chalk
//! `notification.scss`: corner placement, semantic types, title/message,
//! click/close callbacks, a 4500 ms default timer, hover pause, and an optional
//! countdown progress bar.

use std::{
    collections::BTreeMap,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use gpui::{
    App, ElementId, Entity, IntoElement, RenderOnce, SharedString, Window, deferred, div,
    prelude::*, px, rgb,
};

use crate::{FeedbackType, Locale, Theme, el_button, el_icon, theme::popup_priority};

/// Notification corner placement.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum NotificationPosition {
    TopLeft,
    #[default]
    TopRight,
    BottomLeft,
    BottomRight,
}

impl NotificationPosition {
    fn is_bottom(self) -> bool {
        matches!(self, Self::BottomLeft | Self::BottomRight)
    }

    fn is_right(self) -> bool {
        matches!(self, Self::TopRight | Self::BottomRight)
    }
}

type Callback = Rc<dyn Fn(&mut App)>;

/// One notification configuration.
#[derive(Clone)]
pub struct NotificationOptions {
    title: Option<SharedString>,
    message: SharedString,
    kind: Option<FeedbackType>,
    duration: Duration,
    position: NotificationPosition,
    offset: f32,
    show_close: bool,
    progress: bool,
    pause_on_hover: bool,
    onclose: Option<Callback>,
    onclick: Option<Callback>,
}

impl NotificationOptions {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            title: None,
            message: message.into(),
            kind: None,
            duration: Duration::from_millis(4500),
            position: NotificationPosition::TopRight,
            offset: 0.,
            show_close: true,
            progress: false,
            pause_on_hover: true,
            onclose: None,
            onclick: None,
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn kind(mut self, kind: FeedbackType) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn position(mut self, position: NotificationPosition) -> Self {
        self.position = position;
        self
    }

    pub fn offset(mut self, offset: f32) -> Self {
        self.offset = offset.max(0.);
        self
    }

    pub fn show_close(mut self, show: bool) -> Self {
        self.show_close = show;
        self
    }

    pub fn progress(mut self, progress: bool) -> Self {
        self.progress = progress;
        self
    }

    pub fn pause_on_hover(mut self, pause: bool) -> Self {
        self.pause_on_hover = pause;
        self
    }

    pub fn onclose(mut self, handler: impl Fn(&mut App) + 'static) -> Self {
        self.onclose = Some(Rc::new(handler));
        self
    }

    pub fn onclick(mut self, handler: impl Fn(&mut App) + 'static) -> Self {
        self.onclick = Some(Rc::new(handler));
        self
    }
}

struct NotificationItem {
    id: u64,
    options: NotificationOptions,
    remaining: Duration,
    started_at: Option<Instant>,
    hovered: bool,
    close_task: Option<gpui::Task<()>>,
}

impl NotificationItem {
    fn remaining_now(&self) -> Duration {
        self.remaining.saturating_sub(
            self.started_at
                .map(|started| started.elapsed())
                .unwrap_or_default(),
        )
    }
}

/// Shared notification queue. Render one [`NotificationStack`] at the window root.
#[derive(Default)]
pub struct NotificationState {
    items: Vec<NotificationItem>,
    next_id: u64,
}

impl NotificationState {
    pub fn new(cx: &mut App) -> Entity<Self> {
        cx.new(|_| Self::default())
    }

    pub fn push(&mut self, options: NotificationOptions, cx: &mut gpui::Context<Self>) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.items.push(NotificationItem {
            id,
            remaining: options.duration,
            options,
            started_at: None,
            hovered: false,
            close_task: None,
        });
        let duration = self
            .items
            .last()
            .map(|item| item.remaining)
            .unwrap_or_default();
        self.schedule_close(id, duration, cx);
        cx.notify();
        id
    }

    fn schedule_close(&mut self, id: u64, duration: Duration, cx: &mut gpui::Context<Self>) {
        let Some(item) = self.items.iter_mut().find(|item| item.id == id) else {
            return;
        };
        item.close_task.take();
        if duration.is_zero() {
            item.started_at = None;
            return;
        }
        item.remaining = duration;
        item.started_at = Some(Instant::now());
        let task = cx.spawn(async move |entity, cx| {
            cx.background_executor().timer(duration).await;
            let _ = entity.update(cx, |state, cx| state.close_internal(id, cx));
        });
        item.close_task = Some(task);
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

    pub fn set_hovered(&mut self, id: u64, hovered: bool, cx: &mut gpui::Context<Self>) {
        let Some(item) = self.items.iter_mut().find(|item| item.id == id) else {
            return;
        };
        if item.hovered == hovered {
            return;
        }
        item.hovered = hovered;
        if hovered && item.options.pause_on_hover {
            item.remaining = item.remaining_now();
            item.started_at = None;
            item.close_task.take();
        } else if !hovered && item.options.pause_on_hover {
            let remaining = item.remaining;
            self.schedule_close(id, remaining, cx);
        }
        cx.notify();
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Renders a [`NotificationState`] as window-level corner notifications.
#[derive(IntoElement)]
#[must_use]
pub struct NotificationStack {
    id: ElementId,
    state: Entity<NotificationState>,
}

/// Creates a notification stack for a shared queue.
#[track_caller]
pub fn el_notification_stack(state: Entity<NotificationState>) -> NotificationStack {
    NotificationStack::new(state)
}

impl NotificationStack {
    pub fn new(state: Entity<NotificationState>) -> Self {
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

impl RenderOnce for NotificationStack {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let items = self.state.read(cx);
        if items.items.is_empty() {
            return div().into_any_element();
        }
        let theme = Theme::current(cx);
        let locale = Locale::current(cx);
        let viewport = window.viewport_size();
        let mut groups: BTreeMap<NotificationPosition, Vec<&NotificationItem>> = BTreeMap::new();
        for item in &items.items {
            groups.entry(item.options.position).or_default().push(item);
        }
        if items.items.iter().any(|item| {
            item.options.progress
                && !item.options.duration.is_zero()
                && item.started_at.is_some()
                && !item.hovered
        }) {
            window.request_animation_frame();
        }
        let mut root = div()
            .id(self.id.clone())
            .debug_selector(|| "notification-stack".into())
            .absolute()
            .inset_0();
        for (position, items) in groups {
            let offset = items.first().map(|item| item.options.offset).unwrap_or(0.);
            let mut stack = div()
                .absolute()
                .inset_0()
                .flex()
                .flex_col()
                .gap(px(16.))
                .when(position.is_right(), |el| el.items_end().pr(px(offset)))
                .when(!position.is_right(), |el| el.items_start().pl(px(offset)))
                .when(position.is_bottom(), |el| el.justify_end().pb(px(offset)))
                .when(!position.is_bottom(), |el| {
                    el.justify_start().pt(px(offset))
                });
            for item in items {
                let options = &item.options;
                let color = options
                    .kind
                    .map(|kind| kind.color())
                    .unwrap_or(theme.secondary_text());
                let mut notification = div()
                    .id(ElementId::NamedChild(
                        Arc::new(self.id.clone()),
                        format!("notification-{}", item.id).into(),
                    ))
                    .relative()
                    .w(px(330.))
                    .max_w(viewport.width - px(32.))
                    .px(px(13.))
                    .pt(px(14.))
                    .pb(px(14.))
                    .pr(px(26.))
                    .flex()
                    .gap(px(13.))
                    .rounded(px(8.))
                    .border_1()
                    .border_color(rgb(theme.border_lighter()))
                    .bg(rgb(theme.overlay_background()))
                    .shadow(theme.overlay_shadow())
                    .text_color(rgb(theme.primary_text()))
                    .on_hover({
                        let state = self.state.clone();
                        let id = item.id;
                        move |hovered, _, cx| {
                            state.update(cx, |state, cx| state.set_hovered(id, *hovered, cx))
                        }
                    })
                    .on_click({
                        let handler = options.onclick.clone();
                        move |_, _, cx| {
                            if let Some(handler) = handler.clone() {
                                handler(cx);
                            }
                            cx.stop_propagation();
                        }
                    });
                if let Some(kind) = options.kind {
                    notification =
                        notification.child(el_icon(kind.icon()).size(24.).color(rgb(color)));
                }
                let mut group = div().flex_1().min_w_0().mr(px(8.));
                if let Some(title) = options.title.clone().filter(|title| !title.is_empty()) {
                    group = group.child(
                        div()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_size(px(16.))
                            .line_height(px(24.))
                            .child(title),
                    );
                }
                if !options.message.is_empty() {
                    group = group.child(
                        div()
                            .mt(if options.title.is_some() {
                                px(6.)
                            } else {
                                px(0.)
                            })
                            .text_size(px(14.))
                            .line_height(px(24.))
                            .text_color(rgb(theme.regular_text()))
                            .child(options.message.clone()),
                    );
                }
                notification = notification.child(group);
                if options.show_close {
                    notification = notification.child(
                        div().absolute().top(px(14.)).right(px(11.)).child(
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
                                    move |_, _, cx| {
                                        state.update(cx, |state, cx| state.close(id, cx))
                                    }
                                }),
                        ),
                    );
                }
                if options.progress && !options.duration.is_zero() {
                    let percentage = (item.remaining_now().as_secs_f32()
                        / options.duration.as_secs_f32())
                    .clamp(0., 1.);
                    notification = notification.child(
                        div()
                            .absolute()
                            .bottom_0()
                            .left_0()
                            .h(px(3.))
                            .w(px(330. * percentage))
                            .bg(rgb(color)),
                    );
                }
                stack = stack.child(notification);
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
    fn auto_close_zero_duration_and_hover_pause(cx: &mut gpui::TestAppContext) {
        let state = cx.update(NotificationState::new);
        let id = state.update(cx, |state, cx| {
            state.push(
                NotificationOptions::new("Saved")
                    .duration(Duration::from_millis(100))
                    .pause_on_hover(true),
                cx,
            )
        });
        state.update(cx, |state, cx| state.set_hovered(id, true, cx));
        cx.executor().advance_clock(Duration::from_millis(200));
        cx.run_until_parked();
        assert_eq!(cx.update(|cx| state.read(cx).len()), 1);
        state.update(cx, |state, cx| state.set_hovered(id, false, cx));
        cx.executor().advance_clock(Duration::from_millis(120));
        cx.run_until_parked();
        assert!(cx.update(|cx| state.read(cx).is_empty()));

        state.update(cx, |state, cx| {
            state.push(
                NotificationOptions::new("Persistent").duration(Duration::ZERO),
                cx,
            );
        });
        cx.executor().advance_clock(Duration::from_secs(10));
        cx.run_until_parked();
        assert_eq!(cx.update(|cx| state.read(cx).len()), 1);
    }
}
