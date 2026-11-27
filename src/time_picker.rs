//! Element Plus TimePicker.
//!
//! Follows `time-picker.md`, `time-picker.tsx`, and Theme Chalk
//! `date-picker/time-picker.scss`/`time-spinner.scss`: an Input trigger, scrolling
//! hour/minute/second columns, keyboard stepping, and confirm/cancel actions.
//!
//! Native differences: this implementation covers single-time picking only;
//! range mode, disabled-time callbacks, custom popper configuration, and
//! imperative exposes are not mapped.

use std::{rc::Rc, sync::Arc};

use gpui::{
    AnyElement, App, Bounds, ElementId, IntoElement, MouseButton, Pixels, RenderOnce, SharedString,
    Window, anchored, deferred, div, point, prelude::*, px, rgb,
};

use crate::{Icon, InputSize, Locale, Theme, el_button, el_icon, el_input};

/// Time display and binding format.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TimeFormat {
    /// Hour and minute, `HH:mm`.
    #[default]
    HourMinute,
    /// Hour, minute, and second, `HH:mm:ss`.
    HourMinuteSecond,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Time {
    hour: u32,
    minute: u32,
    second: u32,
}

impl Time {
    fn parse(value: &str, format: TimeFormat) -> Option<Self> {
        let parts: Vec<_> = value.trim().split(':').collect();
        let expected = if format == TimeFormat::HourMinuteSecond {
            3
        } else {
            2
        };
        if parts.len() != expected {
            return None;
        }
        let hour = parts.first()?.parse::<u32>().ok()?;
        let minute = parts.get(1)?.parse::<u32>().ok()?;
        let second = if expected == 3 {
            parts.get(2)?.parse::<u32>().ok()?
        } else {
            0
        };
        (hour < 24 && minute < 60 && second < 60).then_some(Self {
            hour,
            minute,
            second,
        })
    }

    fn format(self, format: TimeFormat) -> SharedString {
        if format == TimeFormat::HourMinuteSecond {
            format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second).into()
        } else {
            format!("{:02}:{:02}", self.hour, self.minute).into()
        }
    }

    fn step(mut self, unit: usize, direction: i32) -> Self {
        match unit {
            0 => self.hour = (self.hour as i32 + direction).rem_euclid(24) as u32,
            1 => self.minute = (self.minute as i32 + direction).rem_euclid(60) as u32,
            _ => self.second = (self.second as i32 + direction).rem_euclid(60) as u32,
        }
        self
    }
}

type ChangeHandler = Rc<dyn Fn(Option<SharedString>, &mut Window, &mut App)>;
type VisibleHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type FocusHandler = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Clone)]
struct TimePickerProps {
    value: Option<SharedString>,
    format: TimeFormat,
    disabled: bool,
    readonly: bool,
    editable: bool,
}

#[derive(Clone, Default)]
struct TimePickerHandlers {
    onchange: Option<ChangeHandler>,
    onvisiblechange: Option<VisibleHandler>,
    onclear: Option<FocusHandler>,
    onfocus: Option<FocusHandler>,
    onblur: Option<FocusHandler>,
}

struct TimePickerState {
    draft: Option<String>,
    open: bool,
    unit: usize,
    bounds: Option<Bounds<Pixels>>,
    popup_bounds: Option<Bounds<Pixels>>,
    props: TimePickerProps,
    handlers: TimePickerHandlers,
    last_value: Option<SharedString>,
}

impl Default for TimePickerState {
    fn default() -> Self {
        Self {
            draft: None,
            open: false,
            unit: 0,
            bounds: None,
            popup_bounds: None,
            props: TimePickerProps {
                value: None,
                format: TimeFormat::HourMinute,
                disabled: false,
                readonly: false,
                editable: true,
            },
            handlers: TimePickerHandlers::default(),
            last_value: None,
        }
    }
}

impl TimePickerState {
    fn sync_value(&mut self, value: Option<SharedString>) {
        if self.last_value == value {
            return;
        }
        self.last_value = value;
        if !self.open {
            self.draft = None;
        }
    }

    fn set_open(&mut self, open: bool, window: &mut Window, cx: &mut App) {
        if self.props.disabled || self.props.readonly || self.open == open {
            return;
        }
        self.open = open;
        if open {
            self.draft = self.props.value.as_ref().map(|value| value.to_string());
        } else {
            self.draft = None;
        }
        if let Some(handler) = self.handlers.onvisiblechange.clone() {
            handler(open, window, cx);
        }
    }

    fn current_time(&self) -> Time {
        self.draft
            .as_deref()
            .or(self.props.value.as_deref())
            .and_then(|value| Time::parse(value, self.props.format))
            .unwrap_or_default()
    }

    fn accept_input(&mut self, value: SharedString) {
        if self.props.disabled || self.props.readonly || !self.props.editable {
            return;
        }
        self.draft = Some(value.to_string());
    }

    fn commit(&mut self, value: Option<SharedString>, window: &mut Window, cx: &mut App) {
        if self.props.disabled || self.props.readonly {
            return;
        }
        let normalized = value
            .as_deref()
            .and_then(|value| Time::parse(value, self.props.format))
            .map(|time| time.format(self.props.format));
        let new_value = if value.as_deref().is_some_and(|value| value.is_empty()) {
            None
        } else {
            normalized.or(value)
        };
        self.draft = None;
        if let Some(handler) = self.handlers.onchange.clone() {
            handler(new_value, window, cx);
        }
    }

    fn select_component(&mut self, unit: usize, value: u32, window: &mut Window, cx: &mut App) {
        if self.props.disabled || self.props.readonly {
            return;
        }
        let mut time = self.current_time();
        match unit {
            0 => time.hour = value.min(23),
            1 => time.minute = value.min(59),
            _ => time.second = value.min(59),
        }
        self.unit = unit;
        self.draft = Some(time.format(self.props.format).to_string());
        if let Some(handler) = self.handlers.onchange.clone() {
            handler(self.draft.clone().map(SharedString::from), window, cx);
        }
    }

    fn step(&mut self, direction: i32, window: &mut Window, cx: &mut App) {
        if self.props.disabled || self.props.readonly {
            return;
        }
        let time = self.current_time().step(self.unit, direction);
        self.draft = Some(time.format(self.props.format).to_string());
        if let Some(handler) = self.handlers.onchange.clone() {
            handler(self.draft.clone().map(SharedString::from), window, cx);
        }
    }

    fn confirm(&mut self, window: &mut Window, cx: &mut App) {
        let value = self.draft.clone().map(SharedString::from);
        self.commit(value, window, cx);
        self.set_open(false, window, cx);
    }

    fn cancel(&mut self, window: &mut Window, cx: &mut App) {
        self.draft = None;
        self.set_open(false, window, cx);
    }

    fn clear(&mut self, window: &mut Window, cx: &mut App) {
        self.commit(None, window, cx);
        if let Some(handler) = self.handlers.onclear.clone() {
            handler(window, cx);
        }
        self.set_open(false, window, cx);
    }

    fn key(&mut self, key: &str, window: &mut Window, cx: &mut App) -> bool {
        if self.props.disabled || self.props.readonly {
            return false;
        }
        match key {
            "up" => {
                self.step(1, window, cx);
                true
            }
            "down" => {
                self.step(-1, window, cx);
                true
            }
            "left" => {
                self.unit = self.unit.saturating_sub(1);
                true
            }
            "right" => {
                self.unit =
                    (self.unit + 1).min(if self.props.format == TimeFormat::HourMinuteSecond {
                        2
                    } else {
                        1
                    });
                true
            }
            "enter" => {
                self.confirm(window, cx);
                true
            }
            "escape" => {
                self.cancel(window, cx);
                true
            }
            _ => false,
        }
    }
}

/// An Element Plus single-time picker.
#[derive(IntoElement)]
#[must_use]
pub struct TimePicker {
    id: ElementId,
    #[cfg(test)]
    state: Option<gpui::Entity<TimePickerState>>,
    value: Option<SharedString>,
    format: TimeFormat,
    size: InputSize,
    disabled: bool,
    readonly: bool,
    editable: bool,
    clearable: bool,
    placeholder: Option<SharedString>,
    accessibility_label: Option<SharedString>,
    prefix_icon: Option<Icon>,
    onchange: Option<ChangeHandler>,
    onvisiblechange: Option<VisibleHandler>,
    onclear: Option<FocusHandler>,
    onfocus: Option<FocusHandler>,
    onblur: Option<FocusHandler>,
}

/// Creates a time picker.
#[track_caller]
pub fn el_time_picker() -> TimePicker {
    TimePicker::new()
}

impl Default for TimePicker {
    fn default() -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            #[cfg(test)]
            state: None,
            value: None,
            format: TimeFormat::HourMinute,
            size: InputSize::Default,
            disabled: false,
            readonly: false,
            editable: true,
            clearable: true,
            placeholder: None,
            accessibility_label: None,
            prefix_icon: None,
            onchange: None,
            onvisiblechange: None,
            onclear: None,
            onfocus: None,
            onblur: None,
        }
    }
}

impl TimePicker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    #[cfg(test)]
    fn state(mut self, state: gpui::Entity<TimePickerState>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn value(mut self, value: Option<SharedString>) -> Self {
        self.value = value;
        self
    }

    pub fn format(mut self, format: TimeFormat) -> Self {
        self.format = format;
        self
    }

    pub fn size(mut self, size: InputSize) -> Self {
        self.size = size;
        self
    }

    pub fn large(self) -> Self {
        self.size(InputSize::Large)
    }

    pub fn small(self) -> Self {
        self.size(InputSize::Small)
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    pub fn clearable(mut self, clearable: bool) -> Self {
        self.clearable = clearable;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    pub fn prefix_icon(mut self, icon: Icon) -> Self {
        self.prefix_icon = Some(icon);
        self
    }

    #[track_caller]
    pub fn onchange(
        mut self,
        handler: impl Fn(Option<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onchange = Some(Rc::new(handler));
        self
    }

    #[track_caller]
    pub fn onvisiblechange(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onvisiblechange = Some(Rc::new(handler));
        self
    }

    #[track_caller]
    pub fn onclear(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.onclear = Some(Rc::new(handler));
        self
    }

    pub fn onfocus(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.onfocus = Some(Rc::new(handler));
        self
    }

    pub fn onblur(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.onblur = Some(Rc::new(handler));
        self
    }
}

fn measure(state: gpui::Entity<TimePickerState>, popup: bool) -> AnyElement {
    gpui::canvas(
        move |bounds: Bounds<Pixels>, _, cx: &mut App| {
            state.update(cx, |state, _| {
                if popup {
                    state.popup_bounds = Some(bounds);
                } else {
                    state.bounds = Some(bounds);
                }
            });
        },
        |_, _, _, _| {},
    )
    .absolute()
    .inset_0()
    .into_any_element()
}

impl RenderOnce for TimePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let TimePicker {
            id,
            #[cfg(test)]
                state: provided_state,
            value,
            format,
            size,
            disabled,
            readonly,
            editable,
            clearable,
            placeholder,
            accessibility_label,
            prefix_icon,
            onchange,
            onvisiblechange,
            onclear,
            onfocus,
            onblur,
        } = self;
        let props = TimePickerProps {
            value: value.clone(),
            format,
            disabled,
            readonly,
            editable,
        };
        let handlers = TimePickerHandlers {
            onchange,
            onvisiblechange,
            onclear,
            onfocus,
            onblur,
        };
        #[cfg(test)]
        let state = provided_state.unwrap_or_else(|| {
            window.use_keyed_state(id.clone(), cx, |_, _| TimePickerState::default())
        });
        #[cfg(not(test))]
        let state = window.use_keyed_state(id.clone(), cx, |_, _| TimePickerState::default());
        state.update(cx, |state, _| {
            state.props = props.clone();
            state.handlers = handlers;
        });
        state.update(cx, |state, _| state.sync_value(value.clone()));
        let open = state.read(cx).open && !disabled && !readonly;
        let display = state
            .read(cx)
            .draft
            .clone()
            .or_else(|| value.as_ref().map(SharedString::to_string))
            .unwrap_or_default();
        let locale = Locale::current(cx);
        let theme = Theme::current(cx);
        let prefix = prefix_icon.unwrap_or_else(|| el_icon("clock"));
        let mut input = el_input()
            .id(ElementId::NamedChild(Arc::new(id.clone()), "input".into()))
            .value(display)
            .placeholder(placeholder.unwrap_or_else(|| locale.time_picker_placeholder().into()))
            .size(size)
            .radius_small()
            .disabled(disabled)
            .readonly(readonly || !editable)
            .clearable(clearable)
            .prefix(prefix.size(size.font_size()))
            .suffix(
                div()
                    .flex()
                    .items_center()
                    .text_color(rgb(theme.disabled_text()))
                    .child(
                        el_icon("arrow-down")
                            .size(size.font_size())
                            .rotate(if open { std::f32::consts::PI } else { 0. }),
                    ),
            )
            .oninput({
                let state = state.clone();
                move |reported, _, cx| state.update(cx, |state, _| state.accept_input(reported))
            })
            .on_mouse_down_filter({
                let state = state.clone();
                move |_, window, cx| {
                    if !disabled && !readonly {
                        let open = !state.read(cx).open;
                        state.update(cx, |state, cx| state.set_open(open, window, cx));
                        window.refresh();
                    }
                    false
                }
            })
            .onchange({
                let state = state.clone();
                move |reported, window, cx| {
                    state.update(cx, |state, cx| state.commit(Some(reported), window, cx));
                }
            })
            .onfocus({
                let state = state.clone();
                move |window, cx| {
                    state.update(cx, |state, cx| {
                        if let Some(handler) = state.handlers.onfocus.clone() {
                            handler(window, cx);
                        }
                    });
                    window.refresh();
                }
            })
            .onblur({
                let state = state.clone();
                move |window, cx| {
                    state.update(cx, |state, cx| {
                        if let Some(handler) = state.handlers.onblur.clone() {
                            handler(window, cx);
                        }
                    });
                }
            })
            .onclear({
                let state = state.clone();
                move |_, window, cx| {
                    state.update(cx, |state, cx| state.clear(window, cx));
                    window.refresh();
                }
            })
            .on_key_down_filter({
                let state = state.clone();
                move |event, window, cx| {
                    let handled =
                        state.update(cx, |state, cx| state.key(&event.keystroke.key, window, cx));
                    if handled {
                        window.refresh();
                    }
                    handled
                }
            });
        if let Some(label) = accessibility_label {
            input = input.aria_label(label);
        }
        let mut root = div()
            .id(id.clone())
            .relative()
            .w_full()
            .child(input)
            .child(measure(state.clone(), false))
            .on_mouse_down_out({
                let state = state.clone();
                move |event, window, cx| {
                    let inside_popup = state
                        .read(cx)
                        .popup_bounds
                        .is_some_and(|bounds| bounds.contains(&event.position))
                        || state.read(cx).bounds.is_some_and(|target| {
                            gpui::Bounds::new(
                                point(target.left(), target.bottom() + px(4.)),
                                gpui::size(target.size.width, px(240.)),
                            )
                            .contains(&event.position)
                        });
                    if !inside_popup {
                        state.update(cx, |state, cx| state.set_open(false, window, cx));
                        window.refresh();
                    }
                }
            });
        if open && let Some(target) = state.read(cx).bounds {
            let viewport = window.viewport_size();
            let below = viewport.height - target.bottom() >= px(244.) || target.top() < px(244.);
            let time = state.read(cx).current_time();
            let unit = state.read(cx).unit;
            let columns = [
                ("hour", 24, time.hour),
                ("minute", 60, time.minute),
                ("second", 60, time.second),
            ];
            let mut panel = div()
                .id(ElementId::NamedChild(Arc::new(id.clone()), "panel".into()))
                .w(px(180.))
                .flex_col()
                .rounded(px(2.))
                .border_1()
                .border_color(rgb(theme.border_lighter()))
                .bg(rgb(theme.overlay_background()))
                .shadow(theme.overlay_shadow())
                .occlude();
            let mut column_lists = div().flex();
            for (index, (name, count, selected)) in columns.into_iter().enumerate() {
                if index == 2 && format == TimeFormat::HourMinute {
                    continue;
                }
                let mut list = div()
                    .id(ElementId::NamedChild(
                        Arc::new(id.clone()),
                        format!("{name}-list").into(),
                    ))
                    .flex_1()
                    .h(px(192.))
                    .overflow_y_scroll()
                    .on_scroll(|_, _, cx| cx.stop_propagation());
                for value in 0..count {
                    let active = value == selected;
                    list = list.child(
                        div()
                            .id(ElementId::NamedChild(
                                Arc::new(id.clone()),
                                format!("{name}-{value}").into(),
                            ))
                            .h(px(32.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(12.))
                            .text_color(if active {
                                rgb(crate::theme::PRIMARY)
                            } else {
                                rgb(theme.regular_text())
                            })
                            .when(active, |el| el.font_weight(gpui::FontWeight::MEDIUM))
                            .when(unit == index, |el| {
                                el.bg(rgb(theme.fill(crate::theme::Fill::Light)))
                            })
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, {
                                let state = state.clone();
                                move |_, window, cx| {
                                    cx.stop_propagation();
                                    state.update(cx, |state, cx| {
                                        state.select_component(index, value, window, cx)
                                    });
                                    window.refresh();
                                }
                            }),
                    );
                }
                column_lists = column_lists.child(list);
            }
            panel = panel.child(column_lists);
            let mut actions = div()
                .flex()
                .items_center()
                .justify_end()
                .gap(px(4.))
                .p(px(4.))
                .border_t_1()
                .border_color(rgb(theme.border_lighter()));
            actions = actions.child(
                el_button()
                    .id(ElementId::NamedChild(Arc::new(id.clone()), "cancel".into()))
                    .small()
                    .text()
                    .label(locale.time_picker_cancel())
                    .onclick({
                        let state = state.clone();
                        move |_, window, cx| {
                            state.update(cx, |state, cx| state.cancel(window, cx));
                            window.refresh();
                        }
                    }),
            );
            actions = actions.child(
                el_button()
                    .id(ElementId::NamedChild(
                        Arc::new(id.clone()),
                        "confirm".into(),
                    ))
                    .small()
                    .primary()
                    .label(locale.time_picker_confirm())
                    .onclick({
                        let state = state.clone();
                        move |_, window, cx| {
                            state.update(cx, |state, cx| state.confirm(window, cx));
                            window.refresh();
                        }
                    }),
            );
            panel = panel.child(actions).child(measure(state.clone(), true));
            root = root.child(
                deferred(
                    anchored()
                        .attach(
                            target,
                            if below {
                                gpui::Anchor::BottomLeft
                            } else {
                                gpui::Anchor::TopLeft
                            },
                        )
                        .offset(point(px(0.), if below { px(4.) } else { px(-4.) }))
                        .snap_to_window_with_margin(px(4.))
                        .child(panel),
                )
                .with_priority(crate::theme::popup_priority(0)),
            );
        } else {
            state.update(cx, |state, _| state.popup_bounds = None);
        }
        root.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_formats_supported_time_formats() {
        let time = Time::parse("09:08", TimeFormat::HourMinute).unwrap();
        assert_eq!(time.format(TimeFormat::HourMinute).as_ref(), "09:08");
        assert_eq!(
            Time::parse("23:59:59", TimeFormat::HourMinuteSecond)
                .unwrap()
                .format(TimeFormat::HourMinuteSecond)
                .as_ref(),
            "23:59:59"
        );
        assert!(Time::parse("24:00", TimeFormat::HourMinute).is_none());
        assert!(Time::parse("09:60", TimeFormat::HourMinute).is_none());
    }

    #[test]
    fn keyboard_steps_wrap_each_unit() {
        let time = Time {
            hour: 23,
            minute: 0,
            second: 59,
        };
        assert_eq!(time.step(0, 1).hour, 0);
        assert_eq!(time.step(1, -1).minute, 59);
        assert_eq!(time.step(2, 1).second, 0);
    }

    #[gpui::test]
    fn rendered_panel_selects_hours(cx: &mut gpui::TestAppContext) {
        use std::cell::RefCell;
        struct Harness {
            changes: Rc<RefCell<Vec<Option<SharedString>>>>,
            visible: Rc<RefCell<Vec<bool>>>,
            state: gpui::Entity<TimePickerState>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let changes = self.changes.clone();
                let visible = self.visible.clone();
                div().p(px(20.)).w(px(240.)).child(
                    el_time_picker()
                        .id("rendered-time-picker")
                        .state(self.state.clone())
                        .value(Some("09:30".into()))
                        .onchange(move |value, _, _| changes.borrow_mut().push(value))
                        .onvisiblechange(move |open, _, _| visible.borrow_mut().push(open)),
                )
            }
        }
        let changes = Rc::new(RefCell::new(Vec::new()));
        let visible = Rc::new(RefCell::new(Vec::new()));
        let (harness, view) = cx.add_window_view({
            let changes = changes.clone();
            let visible = visible.clone();
            move |_, cx| Harness {
                changes,
                visible,
                state: cx.new(|_| TimePickerState::default()),
            }
        });
        let state = view.update(|_, cx| harness.read(cx).state.clone());
        view.simulate_click(gpui::point(px(100.), px(36.)), gpui::Modifiers::default());
        assert_eq!(visible.borrow().as_slice(), [true]);
        view.update(|window, cx| window.draw(cx).clear(cx));
        let point = view.update(|_, cx| {
            cx.read_entity(&state, |state, _| {
                state
                    .popup_bounds
                    .map(|bounds| gpui::point(bounds.left() + px(40.), bounds.top() + px(80.)))
            })
        });
        let Some(point) = point else {
            panic!("time panel bounds were not measured");
        };
        view.simulate_click(point, gpui::Modifiers::default());
        assert_eq!(
            changes.borrow().as_slice(),
            [Some(SharedString::from("02:30"))],
            "point={point:?}, popup={:?}, open={}",
            view.update(|_, cx| cx.read_entity(&state, |state, _| state.popup_bounds)),
            view.update(|_, cx| cx.read_entity(&state, |state, _| state.open)),
        );
    }

    #[gpui::test]
    fn selecting_a_column_commits_and_closes(cx: &mut gpui::TestAppContext) {
        struct Harness {
            state: gpui::Entity<TimePickerState>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                div()
            }
        }
        let changes = Rc::new(std::cell::RefCell::new(Vec::new()));
        let (harness, view) = cx.add_window_view({
            let changes = changes.clone();
            move |_, cx| {
                let state = cx.new(|_| TimePickerState::default());
                state.update(cx, |state, _| {
                    state.handlers.onchange = Some(Rc::new({
                        let changes = changes.clone();
                        move |value, _, _| changes.borrow_mut().push(value)
                    }));
                });
                Harness { state }
            }
        });
        view.update(|window, cx| {
            let state = harness.read(cx).state.clone();
            state.update(cx, |state, cx| state.set_open(true, window, cx));
            state.update(cx, |state, cx| state.select_component(0, 9, window, cx));
            state.update(cx, |state, cx| state.select_component(1, 30, window, cx));
            assert!(state.read(cx).open);
            state.update(cx, |state, cx| state.confirm(window, cx));
            assert!(!state.read(cx).open);
        });
        assert_eq!(
            *changes.borrow(),
            [
                Some(SharedString::from("09:00")),
                Some(SharedString::from("09:30")),
                Some(SharedString::from("09:30")),
            ]
        );
    }
}
