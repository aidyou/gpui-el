//! Element Plus DatePicker.
//!
//! Follows `date-picker.md`, `date-picker.tsx`, and Theme Chalk
//! `date-picker/date-picker.scss`/`picker-panel.scss`: an Input trigger, month
//! navigation, a seven-column day grid, keyboard movement, and clearable dates.
//!
//! Native differences: this implementation covers single `YYYY-MM-DD` dates
//! only; ranges, week/month/year/quarter modes, disabled-date callbacks,
//! shortcuts, custom popper configuration, and imperative exposes are not mapped.

use std::{rc::Rc, sync::Arc};

use chrono::{Datelike, Local, NaiveDate, Weekday};
use gpui::{
    AnyElement, App, Bounds, ElementId, IntoElement, Pixels, RenderOnce, SharedString, Window,
    anchored, deferred, div, point, prelude::*, px, rgb,
};

use crate::{Icon, InputSize, Locale, Theme, el_icon, el_input};

type ChangeHandler = Rc<dyn Fn(Option<SharedString>, &mut Window, &mut App)>;
type VisibleHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type FocusHandler = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Clone)]
struct DatePickerProps {
    value: Option<SharedString>,
    disabled: bool,
    readonly: bool,
    editable: bool,
}

#[derive(Clone, Default)]
struct DatePickerHandlers {
    onchange: Option<ChangeHandler>,
    onvisiblechange: Option<VisibleHandler>,
    onclear: Option<FocusHandler>,
    onfocus: Option<FocusHandler>,
    onblur: Option<FocusHandler>,
}

struct DatePickerState {
    draft: Option<String>,
    open: bool,
    view_date: NaiveDate,
    bounds: Option<Bounds<Pixels>>,
    popup_bounds: Option<Bounds<Pixels>>,
    props: DatePickerProps,
    handlers: DatePickerHandlers,
    last_value: Option<SharedString>,
}

impl Default for DatePickerState {
    fn default() -> Self {
        Self {
            draft: None,
            open: false,
            view_date: Local::now().date_naive(),
            bounds: None,
            popup_bounds: None,
            props: DatePickerProps {
                value: None,
                disabled: false,
                readonly: false,
                editable: true,
            },
            handlers: DatePickerHandlers::default(),
            last_value: None,
        }
    }
}

impl DatePickerState {
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
            self.view_date = self
                .props
                .value
                .as_deref()
                .and_then(parse_date)
                .unwrap_or_else(|| Local::now().date_naive());
        } else {
            self.draft = None;
        }
        if let Some(handler) = self.handlers.onvisiblechange.clone() {
            handler(open, window, cx);
        }
    }

    fn selected_date(&self) -> Option<NaiveDate> {
        self.draft
            .as_deref()
            .or(self.props.value.as_deref())
            .and_then(parse_date)
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
        let normalized = value.as_deref().and_then(parse_date).map(format_date);
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

    fn select_date(&mut self, date: NaiveDate, window: &mut Window, cx: &mut App) {
        if self.props.disabled || self.props.readonly {
            return;
        }
        self.view_date = date;
        self.draft = Some(format_date(date).to_string());
        if let Some(handler) = self.handlers.onchange.clone() {
            handler(Some(format_date(date)), window, cx);
        }
        self.set_open(false, window, cx);
    }

    fn shift_month(&mut self, delta: i32) {
        self.view_date = shift_month(self.view_date, delta);
    }

    fn move_selection(&mut self, days: i64, window: &mut Window, cx: &mut App) {
        if self.props.disabled || self.props.readonly {
            return;
        }
        let date = self
            .selected_date()
            .unwrap_or_else(|| Local::now().date_naive());
        if let Some(next) = date.checked_add_signed(chrono::Duration::days(days)) {
            self.view_date = next;
            self.draft = Some(format_date(next).to_string());
            if let Some(handler) = self.handlers.onchange.clone() {
                handler(Some(format_date(next)), window, cx);
            }
        }
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
            "left" => {
                self.move_selection(-1, window, cx);
                true
            }
            "right" => {
                self.move_selection(1, window, cx);
                true
            }
            "up" => {
                self.move_selection(-7, window, cx);
                true
            }
            "down" => {
                self.move_selection(7, window, cx);
                true
            }
            "page-up" => {
                self.shift_month(-1);
                true
            }
            "page-down" => {
                self.shift_month(1);
                true
            }
            "enter" => {
                if let Some(date) = self.selected_date() {
                    self.select_date(date, window, cx);
                } else {
                    self.set_open(true, window, cx);
                }
                true
            }
            "escape" => {
                self.set_open(false, window, cx);
                true
            }
            _ => false,
        }
    }
}

fn parse_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").ok()
}

fn format_date(date: NaiveDate) -> SharedString {
    date.format("%Y-%m-%d").to_string().into()
}

fn shift_month(date: NaiveDate, delta: i32) -> NaiveDate {
    let month = date.year() * 12 + date.month() as i32 - 1 + delta;
    let year = month.div_euclid(12);
    let month = month.rem_euclid(12) + 1;
    NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap_or(date)
}

fn calendar_cells(view: NaiveDate) -> Vec<Option<NaiveDate>> {
    let Some(first) = view.with_day(1) else {
        return Vec::new();
    };
    let offset = match first.weekday() {
        Weekday::Sun => 0,
        Weekday::Mon => 1,
        Weekday::Tue => 2,
        Weekday::Wed => 3,
        Weekday::Thu => 4,
        Weekday::Fri => 5,
        Weekday::Sat => 6,
    };
    let mut cells = vec![None; 42];
    for day in 1..=view.num_days_in_month() as u32 {
        if let Some(date) = first.with_day(day)
            && let Some(slot) = cells.get_mut(offset + day as usize - 1)
        {
            *slot = Some(date);
        }
    }
    cells
}

/// An Element Plus single-date picker.
#[derive(IntoElement)]
#[must_use]
pub struct DatePicker {
    id: ElementId,
    value: Option<SharedString>,
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

/// Creates a date picker.
#[track_caller]
pub fn el_date_picker() -> DatePicker {
    DatePicker::new()
}

impl Default for DatePicker {
    fn default() -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            value: None,
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

impl DatePicker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn value(mut self, value: Option<SharedString>) -> Self {
        self.value = value;
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

fn measure(state: gpui::Entity<DatePickerState>, popup: bool) -> AnyElement {
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

impl RenderOnce for DatePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let DatePicker {
            id,
            value,
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
        let props = DatePickerProps {
            value: value.clone(),
            disabled,
            readonly,
            editable,
        };
        let handlers = DatePickerHandlers {
            onchange,
            onvisiblechange,
            onclear,
            onfocus,
            onblur,
        };
        let state = window.use_keyed_state(id.clone(), cx, |_, _| DatePickerState::default());
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
        let prefix = prefix_icon.unwrap_or_else(|| el_icon("calendar"));
        let mut input = el_input()
            .id(ElementId::NamedChild(Arc::new(id.clone()), "input".into()))
            .value(display)
            .placeholder(placeholder.unwrap_or_else(|| locale.date_picker_placeholder().into()))
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
                                gpui::size(px(280.), px(340.)),
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
            let below = viewport.height - target.bottom() >= px(344.) || target.top() < px(344.);
            let view_date = state.read(cx).view_date;
            let selected = state.read(cx).selected_date();
            let today = Local::now().date_naive();
            let mut panel = div()
                .id(ElementId::NamedChild(Arc::new(id.clone()), "panel".into()))
                .w(px(280.))
                .flex_col()
                .p(px(8.))
                .rounded(px(4.))
                .border_1()
                .border_color(rgb(theme.border_lighter()))
                .bg(rgb(theme.overlay_background()))
                .shadow(theme.overlay_shadow())
                .occlude();
            let mut header = div()
                .flex()
                .items_center()
                .justify_between()
                .h(px(36.))
                .px(px(4.));
            header = header.child(
                div()
                    .id(ElementId::NamedChild(Arc::new(id.clone()), "prev".into()))
                    .w(px(32.))
                    .h(px(32.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(4.))
                    .cursor_pointer()
                    .text_color(rgb(theme.regular_text()))
                    .hover(|style| style.bg(rgb(theme.fill(crate::theme::Fill::Light))))
                    .child(el_icon("arrow-left").size(14.))
                    .on_click({
                        let state = state.clone();
                        move |_, window, cx| {
                            state.update(cx, |state, _| state.shift_month(-1));
                            window.refresh();
                        }
                    }),
            );
            header = header.child(
                div()
                    .text_size(px(14.))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(format!(
                        "{:04} / {:02}",
                        view_date.year(),
                        view_date.month()
                    )),
            );
            header = header.child(
                div()
                    .id(ElementId::NamedChild(Arc::new(id.clone()), "next".into()))
                    .w(px(32.))
                    .h(px(32.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(4.))
                    .cursor_pointer()
                    .text_color(rgb(theme.regular_text()))
                    .hover(|style| style.bg(rgb(theme.fill(crate::theme::Fill::Light))))
                    .child(el_icon("arrow-right").size(14.))
                    .on_click({
                        let state = state.clone();
                        move |_, window, cx| {
                            state.update(cx, |state, _| state.shift_month(1));
                            window.refresh();
                        }
                    }),
            );
            panel = panel.child(header);
            let mut weekdays = div().flex().w(px(264.));
            for label in locale.date_picker_weekdays() {
                weekdays = weekdays.child(
                    div()
                        .w(px(264. / 7.))
                        .h(px(28.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(12.))
                        .text_color(rgb(theme.secondary_text()))
                        .child(label.clone()),
                );
            }
            panel = panel.child(weekdays);
            let mut grid = div().flex().flex_wrap().w(px(264.));
            for (index, date) in calendar_cells(view_date).into_iter().enumerate() {
                let Some(date) = date else {
                    grid = grid.child(div().w(px(264. / 7.)).h(px(40.)));
                    continue;
                };
                let is_selected = selected == Some(date);
                let is_today = date == today;
                grid = grid.child(
                    div()
                        .id(ElementId::NamedChild(
                            Arc::new(id.clone()),
                            format!("day-{index}").into(),
                        ))
                        .w(px(264. / 7.))
                        .h(px(40.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(13.))
                        .rounded(px(4.))
                        .cursor_pointer()
                        .text_color(if is_selected {
                            rgb(0xffffff)
                        } else {
                            rgb(theme.regular_text())
                        })
                        .when(is_selected, |el| el.bg(rgb(crate::theme::PRIMARY)))
                        .when(!is_selected, |el| {
                            el.hover(|style| style.bg(rgb(theme.fill(crate::theme::Fill::Light))))
                        })
                        .when(is_today && !is_selected, |el| {
                            el.border_1().border_color(rgb(crate::theme::PRIMARY))
                        })
                        .child(date.day().to_string())
                        .on_click({
                            let state = state.clone();
                            move |_, window, cx| {
                                cx.stop_propagation();
                                state.update(cx, |state, cx| state.select_date(date, window, cx));
                                window.refresh();
                            }
                        }),
                );
            }
            panel = panel.child(grid).child(measure(state.clone(), true));
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
    fn parses_and_formats_iso_dates() {
        let date = parse_date("2026-11-28").unwrap();
        assert_eq!(format_date(date).as_ref(), "2026-11-28");
        assert!(parse_date("2026-02-30").is_none());
        assert!(parse_date("not-a-date").is_none());
    }

    #[test]
    fn calendar_has_six_weeks_and_month_navigation_wraps() {
        let cells = calendar_cells(NaiveDate::from_ymd_opt(2026, 11, 1).unwrap());
        assert_eq!(cells.len(), 42);
        assert_eq!(cells.iter().flatten().count(), 30);
        assert_eq!(
            shift_month(NaiveDate::from_ymd_opt(2026, 12, 15).unwrap(), 1),
            NaiveDate::from_ymd_opt(2027, 1, 1).unwrap()
        );
        assert_eq!(
            shift_month(NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(), -1),
            NaiveDate::from_ymd_opt(2025, 12, 1).unwrap()
        );
    }

    #[gpui::test]
    fn selecting_a_day_reports_and_closes(cx: &mut gpui::TestAppContext) {
        struct Harness {
            state: gpui::Entity<DatePickerState>,
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
                let state = cx.new(|_| DatePickerState::default());
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
            state.update(cx, |state, cx| {
                state.select_date(NaiveDate::from_ymd_opt(2026, 11, 28).unwrap(), window, cx)
            });
            assert!(!state.read(cx).open);
        });
        assert_eq!(*changes.borrow(), [Some(SharedString::from("2026-11-28"))]);
    }
}
