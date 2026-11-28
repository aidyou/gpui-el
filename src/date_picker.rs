//! Element Plus DatePicker.
//!
//! Follows `date-picker.md`, `date-picker.tsx`, the date-picker-panel tables,
//! and Theme Chalk `date-picker.scss`/`date-table.scss`: an Input trigger,
//! linked calendar panels, date/month/year/quarter selection, range selection,
//! keyboard movement, disabled dates, and clearable values.
//!
//! Native differences: values are strings rather than JavaScript `Date`
//! objects. Popper options, DOM slots, Vue exposes, and web-only formatting
//! extensions are not mapped. Date-time panels are not provided yet; use the
//! standalone [`crate::TimePicker`] when time input is needed.

use std::{rc::Rc, sync::Arc};

use chrono::{Datelike, Local, NaiveDate, Weekday};
use gpui::{
    AnyElement, App, Bounds, Div, ElementId, IntoElement, Pixels, RenderOnce, SharedString,
    Stateful, Window, anchored, deferred, div, point, prelude::*, px, rgb,
};

use crate::{Icon, InputSize, Locale, Theme, el_button, el_icon, el_input};

type ChangeHandler = Rc<dyn Fn(Option<SharedString>, &mut Window, &mut App)>;
type RangeChangeHandler =
    Rc<dyn Fn(Option<SharedString>, Option<SharedString>, &mut Window, &mut App)>;
type MultipleChangeHandler = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut App)>;
type VisibleHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type FocusHandler = Rc<dyn Fn(&mut Window, &mut App)>;
type DisabledDate = Rc<dyn Fn(NaiveDate) -> bool>;

/// Element Plus picker measurement.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum DatePickerType {
    #[default]
    Date,
    DateRange,
    Month,
    MonthRange,
    Year,
    YearRange,
    Quarter,
    QuarterRange,
    Week,
    Dates,
    Months,
    Years,
    Quarters,
}

impl DatePickerType {
    fn is_range(self) -> bool {
        matches!(
            self,
            Self::DateRange | Self::MonthRange | Self::YearRange | Self::QuarterRange
        )
    }

    fn is_multiple(self) -> bool {
        matches!(
            self,
            Self::Dates | Self::Months | Self::Years | Self::Quarters
        )
    }

    fn unit(self) -> DateUnit {
        match self {
            Self::Month | Self::MonthRange | Self::Months => DateUnit::Month,
            Self::Year | Self::YearRange | Self::Years => DateUnit::Year,
            Self::Quarter | Self::QuarterRange | Self::Quarters => DateUnit::Quarter,
            _ => DateUnit::Day,
        }
    }

    fn initial_view(self) -> PanelView {
        match self.unit() {
            DateUnit::Day => PanelView::Date,
            DateUnit::Month => PanelView::Month,
            DateUnit::Year => PanelView::Year,
            DateUnit::Quarter => PanelView::Quarter,
        }
    }

    fn default_format(self) -> &'static str {
        match self {
            Self::Month | Self::MonthRange | Self::Months => "%Y-%m",
            Self::Year | Self::YearRange | Self::Years => "%Y",
            Self::Quarter | Self::QuarterRange | Self::Quarters => "%Y-Q%q",
            Self::Week => "%Y-W%V",
            _ => "%Y-%m-%d",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DateUnit {
    Day,
    Month,
    Year,
    Quarter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PanelView {
    Date,
    Month,
    Year,
    Quarter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PanelSide {
    Left,
    Right,
}

impl PanelSide {
    fn name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum BoundValue {
    Single(Option<SharedString>),
    Range(Option<SharedString>, Option<SharedString>),
    Multiple(Vec<SharedString>),
}

#[derive(Clone)]
struct DatePickerProps {
    value: Option<SharedString>,
    range: (Option<SharedString>, Option<SharedString>),
    multiple: Vec<SharedString>,
    picker_type: DatePickerType,
    disabled: bool,
    readonly: bool,
    editable: bool,
    format: Option<String>,
    value_format: Option<String>,
    default_value: Option<SharedString>,
    start_placeholder: Option<SharedString>,
    end_placeholder: Option<SharedString>,
    range_separator: SharedString,
    unlink_panels: bool,
    single_panel: bool,
    disabled_date: Option<DisabledDate>,
    show_now: bool,
    show_confirm: bool,
    show_week_number: bool,
}

impl Default for DatePickerProps {
    fn default() -> Self {
        Self {
            value: None,
            range: (None, None),
            multiple: Vec::new(),
            picker_type: DatePickerType::Date,
            disabled: false,
            readonly: false,
            editable: true,
            format: None,
            value_format: None,
            default_value: None,
            start_placeholder: None,
            end_placeholder: None,
            range_separator: "-".into(),
            unlink_panels: false,
            single_panel: false,
            disabled_date: None,
            show_now: true,
            show_confirm: false,
            show_week_number: false,
        }
    }
}

impl DatePickerProps {
    fn bound_value(&self) -> BoundValue {
        if self.picker_type.is_range() {
            BoundValue::Range(self.range.0.clone(), self.range.1.clone())
        } else if self.picker_type.is_multiple() {
            BoundValue::Multiple(self.multiple.clone())
        } else {
            BoundValue::Single(self.value.clone())
        }
    }

    fn disabled_on(&self, date: NaiveDate) -> bool {
        self.disabled_date
            .as_ref()
            .is_some_and(|disabled| disabled(date))
    }

    fn unit_disabled(&self, date: NaiveDate, unit: DateUnit) -> bool {
        if self.disabled_date.is_none() {
            return false;
        }
        let mut current = start_of_unit(date, unit);
        let end = match unit {
            DateUnit::Day => current,
            DateUnit::Month => shift_month(current, 1),
            DateUnit::Year => shift_year(current, 1),
            DateUnit::Quarter => shift_month(current, 3),
        };
        let mut all_disabled = true;
        while current < end {
            if !self.disabled_on(current) {
                all_disabled = false;
                break;
            }
            let Some(next) = current.checked_add_signed(chrono::Duration::days(1)) else {
                break;
            };
            current = next;
        }
        all_disabled
    }
}

#[derive(Clone, Default)]
struct DatePickerHandlers {
    onchange: Option<ChangeHandler>,
    onrangechange: Option<RangeChangeHandler>,
    onmultiplechange: Option<MultipleChangeHandler>,
    onvisiblechange: Option<VisibleHandler>,
    onclear: Option<FocusHandler>,
    onfocus: Option<FocusHandler>,
    onblur: Option<FocusHandler>,
}

struct DatePickerState {
    draft: Option<String>,
    open: bool,
    left_date: NaiveDate,
    right_date: NaiveDate,
    left_view: PanelView,
    right_view: PanelView,
    range_start: Option<NaiveDate>,
    range_end: Option<NaiveDate>,
    range_hover: Option<NaiveDate>,
    selecting_end: bool,
    bounds: Option<Bounds<Pixels>>,
    popup_bounds: Option<Bounds<Pixels>>,
    props: DatePickerProps,
    handlers: DatePickerHandlers,
    last_value: BoundValue,
}

impl Default for DatePickerState {
    fn default() -> Self {
        let today = Local::now().date_naive();
        Self {
            draft: None,
            open: false,
            left_date: today,
            right_date: shift_month(today, 1),
            left_view: PanelView::Date,
            right_view: PanelView::Date,
            range_start: None,
            range_end: None,
            range_hover: None,
            selecting_end: false,
            bounds: None,
            popup_bounds: None,
            props: DatePickerProps::default(),
            handlers: DatePickerHandlers::default(),
            last_value: BoundValue::Single(None),
        }
    }
}

impl DatePickerState {
    fn sync_value(&mut self, value: BoundValue) {
        if self.last_value == value {
            return;
        }
        self.last_value = value;
        if !self.open {
            self.draft = None;
            self.sync_range_from_props();
        }
    }

    fn sync_range_from_props(&mut self) {
        self.range_start = self
            .props
            .range
            .0
            .as_deref()
            .and_then(|value| parse_bound_date_with(&self.props, value));
        self.range_end = self
            .props
            .range
            .1
            .as_deref()
            .and_then(|value| parse_bound_date_with(&self.props, value));
        self.range_hover = None;
        self.selecting_end = false;
    }

    fn set_open(&mut self, open: bool, window: &mut Window, cx: &mut App) {
        if self.props.disabled || self.props.readonly || self.open == open {
            return;
        }
        self.open = open;
        if open {
            let initial = self
                .props
                .default_value
                .as_deref()
                .and_then(|value| parse_bound_date_with(&self.props, value))
                .or_else(|| match self.props.bound_value() {
                    BoundValue::Single(value) => value
                        .as_deref()
                        .and_then(|value| parse_bound_date_with(&self.props, value)),
                    BoundValue::Range(start, _) => start
                        .as_deref()
                        .and_then(|value| parse_bound_date_with(&self.props, value)),
                    BoundValue::Multiple(values) => values
                        .first()
                        .and_then(|value| parse_bound_date_with(&self.props, value)),
                })
                .unwrap_or_else(|| Local::now().date_naive());
            self.left_date = initial;
            self.right_date = shift_by_unit(initial, 1, self.props.picker_type.unit());
            self.left_view = self.props.picker_type.initial_view();
            self.right_view = self.left_view;
            self.sync_range_from_props();
            self.draft = Some(self.display_value(Locale::current(cx)));
        } else {
            self.draft = None;
            self.range_hover = None;
            self.selecting_end = false;
        }
        if let Some(handler) = self.handlers.onvisiblechange.clone() {
            handler(open, window, cx);
        }
    }

    fn display_value(&self, locale: Locale) -> String {
        match self.props.bound_value() {
            BoundValue::Single(value) => value
                .as_deref()
                .and_then(|value| parse_bound_date_with(&self.props, value))
                .map(|date| self.format_display(date, locale).to_string())
                .unwrap_or_default(),
            BoundValue::Range(start, end) => {
                let start = start
                    .as_deref()
                    .and_then(|value| parse_bound_date_with(&self.props, value))
                    .map(|date| self.format_display(date, locale).to_string())
                    .unwrap_or_default();
                let end = end
                    .as_deref()
                    .and_then(|value| parse_bound_date_with(&self.props, value))
                    .map(|date| self.format_display(date, locale).to_string())
                    .unwrap_or_default();
                if start.is_empty() && end.is_empty() {
                    String::new()
                } else {
                    format!("{start}{}{end}", self.props.range_separator)
                }
            }
            BoundValue::Multiple(values) => values
                .iter()
                .filter_map(|value| parse_bound_date_with(&self.props, value))
                .map(|date| self.format_display(date, locale).to_string())
                .collect::<Vec<_>>()
                .join(", "),
        }
    }

    fn placeholder(&self, locale: Locale) -> SharedString {
        if self.props.picker_type.is_range() {
            let start = self
                .props
                .start_placeholder
                .clone()
                .unwrap_or_else(|| locale.date_picker_start_placeholder().into());
            let end = self
                .props
                .end_placeholder
                .clone()
                .unwrap_or_else(|| locale.date_picker_end_placeholder().into());
            format!("{start}{}{end}", self.props.range_separator).into()
        } else {
            self.props
                .start_placeholder
                .clone()
                .unwrap_or_else(|| locale.date_picker_placeholder().into())
        }
    }

    fn format_display(&self, date: NaiveDate, locale: Locale) -> SharedString {
        format_date_pattern(
            date,
            self.props
                .format
                .as_deref()
                .unwrap_or(self.props.picker_type.default_format()),
            locale,
        )
        .into()
    }

    fn format_bound(&self, date: NaiveDate) -> SharedString {
        format_date_pattern(
            date,
            self.props
                .value_format
                .as_deref()
                .unwrap_or(self.props.picker_type.default_format()),
            Locale::English,
        )
        .into()
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
        let Some(value) = value else {
            self.clear(window, cx);
            return;
        };
        if self.props.picker_type.is_range() {
            if let Some((start, end)) = self.parse_range_input(&value) {
                self.emit_range(start, end, window, cx);
            }
        } else if self.props.picker_type.is_multiple() {
            let values = value
                .split(',')
                .filter_map(|value| parse_bound_date_with(&self.props, value.trim()))
                .map(|date| self.format_bound(date))
                .collect();
            self.emit_multiple(values, window, cx);
        } else if value.is_empty() {
            self.emit_single(None, window, cx);
        } else if let Some(date) = parse_bound_date_with(&self.props, &value) {
            self.emit_single(Some(self.format_bound(date)), window, cx);
        }
        self.draft = None;
    }

    fn parse_range_input(
        &self,
        value: &str,
    ) -> Option<(Option<SharedString>, Option<SharedString>)> {
        let separator = self.props.range_separator.as_ref();
        if separator.is_empty() {
            return None;
        }
        for (index, _) in value.match_indices(separator) {
            let start = &value[..index];
            let end = &value[index + separator.len()..];
            let start = parse_bound_date_with(&self.props, start.trim());
            let end = parse_bound_date_with(&self.props, end.trim());
            if start.is_some() || end.is_some() {
                return Some((
                    start.map(|date| self.format_bound(date)),
                    end.map(|date| self.format_bound(date)),
                ));
            }
        }
        None
    }

    fn emit_single(&mut self, value: Option<SharedString>, window: &mut Window, cx: &mut App) {
        self.props.value = value.clone();
        self.last_value = BoundValue::Single(value.clone());
        if let Some(handler) = self.handlers.onchange.clone() {
            handler(value, window, cx);
        }
    }

    fn emit_range(
        &mut self,
        start: Option<SharedString>,
        end: Option<SharedString>,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.props.range = (start.clone(), end.clone());
        self.last_value = BoundValue::Range(start.clone(), end.clone());
        if let Some(handler) = self.handlers.onrangechange.clone() {
            handler(start, end, window, cx);
        }
    }

    fn emit_multiple(&mut self, values: Vec<SharedString>, window: &mut Window, cx: &mut App) {
        self.props.multiple = values.clone();
        self.last_value = BoundValue::Multiple(values.clone());
        if let Some(handler) = self.handlers.onmultiplechange.clone() {
            handler(values, window, cx);
        }
    }

    fn select_date(&mut self, date: NaiveDate, window: &mut Window, cx: &mut App) {
        if self.props.disabled || self.props.readonly || self.props.disabled_on(date) {
            return;
        }
        let date = start_of_unit(date, self.props.picker_type.unit());
        if self.props.picker_type.is_range() {
            if !self.selecting_end {
                self.range_start = Some(date);
                self.range_end = None;
                self.range_hover = None;
                self.selecting_end = true;
            } else {
                let start = self.range_start.unwrap_or(date);
                let (start, end) = if date < start {
                    (date, start)
                } else {
                    (start, date)
                };
                self.range_start = Some(start);
                self.range_end = Some(end);
                self.selecting_end = false;
                self.emit_range(
                    Some(self.format_bound(start)),
                    Some(self.format_bound(end)),
                    window,
                    cx,
                );
                self.set_open(false, window, cx);
            }
        } else if self.props.picker_type.is_multiple() {
            let value = self.format_bound(date);
            let mut values = self.props.multiple.clone();
            if let Some(index) = values.iter().position(|candidate| candidate == &value) {
                values.remove(index);
            } else {
                values.push(value);
            }
            self.emit_multiple(values, window, cx);
        } else {
            self.emit_single(Some(self.format_bound(date)), window, cx);
            self.set_open(false, window, cx);
        }
    }

    fn select_header_date(
        &mut self,
        side: PanelSide,
        date: NaiveDate,
        next_view: PanelView,
        window: &mut Window,
        cx: &mut App,
    ) {
        if self.props.disabled || self.props.readonly {
            return;
        }
        match side {
            PanelSide::Left => {
                self.left_date = date;
                self.left_view = next_view;
                if !self.props.unlink_panels && self.props.picker_type.is_range() {
                    self.right_date = shift_by_unit(date, 1, self.props.picker_type.unit());
                    self.right_view = next_view;
                }
            }
            PanelSide::Right => {
                self.right_date = date;
                self.right_view = next_view;
                if !self.props.unlink_panels && self.props.picker_type.is_range() {
                    self.left_date = shift_by_unit(date, -1, self.props.picker_type.unit());
                    self.left_view = next_view;
                }
            }
        }
        if self.props.picker_type == DatePickerType::Date && !self.props.picker_type.is_range() {
            self.emit_single(Some(self.format_bound(date)), window, cx);
        }
    }

    fn shift_panel(&mut self, side: PanelSide, months: i32, years: i32) {
        if self.props.disabled || self.props.readonly {
            return;
        }
        let apply = |date: NaiveDate| {
            let date = if years == 0 {
                date
            } else {
                shift_year(date, years)
            };
            if months == 0 {
                date
            } else {
                shift_month(date, months)
            }
        };
        match side {
            PanelSide::Left => {
                self.left_date = apply(self.left_date);
                if !self.props.unlink_panels && self.props.picker_type.is_range() {
                    self.right_date = apply(self.right_date);
                }
            }
            PanelSide::Right => {
                self.right_date = apply(self.right_date);
                if !self.props.unlink_panels && self.props.picker_type.is_range() {
                    self.left_date = apply(self.left_date);
                }
            }
        }
    }

    fn shift_active(&mut self, side: PanelSide, delta: i32) {
        let view = match side {
            PanelSide::Left => self.left_view,
            PanelSide::Right => self.right_view,
        };
        let date = match side {
            PanelSide::Left => self.left_date,
            PanelSide::Right => self.right_date,
        };
        let next = match view {
            PanelView::Date => date.checked_add_signed(chrono::Duration::days(delta as i64)),
            PanelView::Month => Some(shift_month(date, delta)),
            PanelView::Year => Some(shift_year(date, delta)),
            PanelView::Quarter => Some(shift_month(date, delta * 3)),
        };
        if let Some(next) = next {
            match side {
                PanelSide::Left => {
                    self.left_date = next;
                    if !self.props.unlink_panels && self.props.picker_type.is_range() {
                        self.right_date = shift_by_unit(next, 1, self.props.picker_type.unit());
                    }
                }
                PanelSide::Right => {
                    self.right_date = next;
                    if !self.props.unlink_panels && self.props.picker_type.is_range() {
                        self.left_date = shift_by_unit(next, -1, self.props.picker_type.unit());
                    }
                }
            }
        }
    }

    fn move_selection(&mut self, days: i64, window: &mut Window, cx: &mut App) {
        if self.props.disabled || self.props.readonly {
            return;
        }
        let current = self
            .range_start
            .or_else(|| {
                self.props
                    .value
                    .as_deref()
                    .and_then(|value| parse_bound_date_with(&self.props, value))
            })
            .unwrap_or(self.left_date);
        if let Some(next) = current.checked_add_signed(chrono::Duration::days(days)) {
            self.left_date = next;
            self.right_date = shift_by_unit(next, 1, self.props.picker_type.unit());
            self.draft = Some(self.format_bound(next).to_string());
            if !self.props.picker_type.is_range() && !self.props.picker_type.is_multiple() {
                self.emit_single(Some(self.format_bound(next)), window, cx);
            }
        }
    }

    fn confirm(&mut self, window: &mut Window, cx: &mut App) {
        if self.props.disabled || self.props.readonly {
            return;
        }
        if self.props.picker_type.is_range() {
            if let (Some(start), Some(end)) = (self.range_start, self.range_end) {
                let (start, end) = if start <= end {
                    (start, end)
                } else {
                    (end, start)
                };
                self.emit_range(
                    Some(self.format_bound(start)),
                    Some(self.format_bound(end)),
                    window,
                    cx,
                );
                self.set_open(false, window, cx);
            }
        } else if self.props.picker_type.is_multiple() {
            self.emit_multiple(self.props.multiple.clone(), window, cx);
            self.set_open(false, window, cx);
        } else if let Some(date) = self.selected_date() {
            self.select_date(date, window, cx);
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut App) {
        if self.props.picker_type.is_range() {
            self.emit_range(None, None, window, cx);
        } else if self.props.picker_type.is_multiple() {
            self.emit_multiple(Vec::new(), window, cx);
        } else {
            self.emit_single(None, window, cx);
        }
        self.draft = None;
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
                if self.left_view == PanelView::Date {
                    self.move_selection(-1, window, cx);
                } else {
                    self.shift_active(PanelSide::Left, -1);
                }
                true
            }
            "right" => {
                if self.left_view == PanelView::Date {
                    self.move_selection(1, window, cx);
                } else {
                    self.shift_active(PanelSide::Left, 1);
                }
                true
            }
            "up" => {
                if self.left_view == PanelView::Date {
                    self.move_selection(-7, window, cx);
                } else {
                    self.shift_active(PanelSide::Left, -4);
                }
                true
            }
            "down" => {
                if self.left_view == PanelView::Date {
                    self.move_selection(7, window, cx);
                } else {
                    self.shift_active(PanelSide::Left, 4);
                }
                true
            }
            "page-up" => {
                self.shift_panel(PanelSide::Left, -1, 0);
                true
            }
            "page-down" => {
                self.shift_panel(PanelSide::Left, 1, 0);
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

    fn selected_date(&self) -> Option<NaiveDate> {
        self.draft
            .as_deref()
            .and_then(|value| parse_bound_date_with(&self.props, value))
            .or_else(|| {
                self.props
                    .value
                    .as_deref()
                    .and_then(|value| parse_bound_date_with(&self.props, value))
            })
            .or(self.range_start)
    }
}

/// An Element Plus date picker.
#[derive(IntoElement)]
#[must_use]
pub struct DatePicker {
    id: ElementId,
    value: Option<SharedString>,
    range: (Option<SharedString>, Option<SharedString>),
    multiple: Vec<SharedString>,
    picker_type: DatePickerType,
    size: InputSize,
    disabled: bool,
    readonly: bool,
    editable: bool,
    clearable: bool,
    placeholder: Option<SharedString>,
    start_placeholder: Option<SharedString>,
    end_placeholder: Option<SharedString>,
    range_separator: SharedString,
    accessibility_label: Option<SharedString>,
    prefix_icon: Option<Icon>,
    format: Option<String>,
    value_format: Option<String>,
    default_value: Option<SharedString>,
    unlink_panels: bool,
    single_panel: bool,
    disabled_date: Option<DisabledDate>,
    show_now: bool,
    show_confirm: bool,
    show_week_number: bool,
    onchange: Option<ChangeHandler>,
    onrangechange: Option<RangeChangeHandler>,
    onmultiplechange: Option<MultipleChangeHandler>,
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
            range: (None, None),
            multiple: Vec::new(),
            picker_type: DatePickerType::Date,
            size: InputSize::Default,
            disabled: false,
            readonly: false,
            editable: true,
            clearable: true,
            placeholder: None,
            start_placeholder: None,
            end_placeholder: None,
            range_separator: "-".into(),
            accessibility_label: None,
            prefix_icon: None,
            format: None,
            value_format: None,
            default_value: None,
            unlink_panels: false,
            single_panel: false,
            disabled_date: None,
            show_now: true,
            show_confirm: false,
            show_week_number: false,
            onchange: None,
            onrangechange: None,
            onmultiplechange: None,
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

    pub fn value_range(mut self, start: Option<SharedString>, end: Option<SharedString>) -> Self {
        self.range = (start, end);
        self.picker_type = DatePickerType::DateRange;
        self
    }

    pub fn values(mut self, values: impl IntoIterator<Item = SharedString>) -> Self {
        self.multiple = values.into_iter().collect();
        self
    }

    pub fn picker_type(mut self, picker_type: DatePickerType) -> Self {
        self.picker_type = picker_type;
        self
    }

    pub fn range(mut self, range: bool) -> Self {
        if range {
            self.picker_type = DatePickerType::DateRange;
        }
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

    pub fn start_placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.start_placeholder = Some(placeholder.into());
        self
    }

    pub fn end_placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.end_placeholder = Some(placeholder.into());
        self
    }

    pub fn range_separator(mut self, separator: impl Into<SharedString>) -> Self {
        self.range_separator = separator.into();
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

    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    pub fn value_format(mut self, format: impl Into<String>) -> Self {
        self.value_format = Some(format.into());
        self
    }

    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    pub fn unlink_panels(mut self, unlink: bool) -> Self {
        self.unlink_panels = unlink;
        self
    }

    pub fn single_panel(mut self, single: bool) -> Self {
        self.single_panel = single;
        self
    }

    pub fn disabled_date(mut self, disabled: impl Fn(NaiveDate) -> bool + 'static) -> Self {
        self.disabled_date = Some(Rc::new(disabled));
        self
    }

    pub fn show_now(mut self, show: bool) -> Self {
        self.show_now = show;
        self
    }

    pub fn show_confirm(mut self, show: bool) -> Self {
        self.show_confirm = show;
        self
    }

    pub fn show_week_number(mut self, show: bool) -> Self {
        self.show_week_number = show;
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
    pub fn onrangechange(
        mut self,
        handler: impl Fn(Option<SharedString>, Option<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onrangechange = Some(Rc::new(handler));
        self
    }

    #[track_caller]
    pub fn onmultiplechange(
        mut self,
        handler: impl Fn(Vec<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onmultiplechange = Some(Rc::new(handler));
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

fn measure(
    state: gpui::Entity<DatePickerState>,
    popup: bool,
    control_height: Option<f32>,
) -> AnyElement {
    let canvas = gpui::canvas(
        move |bounds: Bounds<Pixels>, _, cx: &mut App| {
            let bounds = if let Some(height) = control_height {
                gpui::Bounds::new(bounds.origin, gpui::size(bounds.size.width, px(height)))
            } else {
                bounds
            };
            state.update(cx, |state, _| {
                if popup {
                    state.popup_bounds = Some(bounds);
                } else {
                    state.bounds = Some(bounds);
                }
            });
        },
        |_, _, _, _| {},
    );
    if popup {
        canvas.absolute().inset_0().into_any_element()
    } else {
        canvas.w_full().h(px(0.)).into_any_element()
    }
}

impl RenderOnce for DatePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let DatePicker {
            id,
            value,
            range,
            multiple,
            picker_type,
            size,
            disabled,
            readonly,
            editable,
            clearable,
            placeholder,
            start_placeholder,
            end_placeholder,
            range_separator,
            accessibility_label,
            prefix_icon,
            format,
            value_format,
            default_value,
            unlink_panels,
            single_panel,
            disabled_date,
            show_now,
            show_confirm,
            show_week_number,
            onchange,
            onrangechange,
            onmultiplechange,
            onvisiblechange,
            onclear,
            onfocus,
            onblur,
        } = self;
        let props = DatePickerProps {
            value: value.clone(),
            range: range.clone(),
            multiple: multiple.clone(),
            picker_type,
            disabled,
            readonly,
            editable,
            format,
            value_format,
            default_value,
            start_placeholder,
            end_placeholder,
            range_separator,
            unlink_panels,
            single_panel,
            disabled_date,
            show_now,
            show_confirm,
            show_week_number,
        };
        let handlers = DatePickerHandlers {
            onchange,
            onrangechange,
            onmultiplechange,
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
        state.update(cx, |state, _| state.sync_value(props.bound_value()));
        let locale = Locale::current(cx);
        let open = state.read(cx).open && !disabled && !readonly;
        let display = state
            .read(cx)
            .draft
            .clone()
            .unwrap_or_else(|| state.read(cx).display_value(locale));
        let theme = Theme::current(cx);
        let placeholder = placeholder.unwrap_or_else(|| state.read(cx).placeholder(locale));
        let prefix = prefix_icon.unwrap_or_else(|| el_icon("calendar"));
        let mut input = el_input()
            .id(ElementId::NamedChild(Arc::new(id.clone()), "input".into()))
            .value(display)
            .placeholder(placeholder)
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
            .child(measure(state.clone(), false, Some(size.height())))
            .child(input)
            .on_mouse_down_out({
                let state = state.clone();
                move |event, window, cx| {
                    let inside_popup = state
                        .read(cx)
                        .popup_bounds
                        .is_some_and(|bounds| bounds.contains(&event.position))
                        || state
                            .read(cx)
                            .bounds
                            .is_some_and(|bounds| bounds.contains(&event.position));
                    if !inside_popup {
                        state.update(cx, |state, cx| state.set_open(false, window, cx));
                        window.refresh();
                    }
                }
            });
        if open && let Some(target) = state.read(cx).bounds {
            let mut panel = div()
                .id(ElementId::NamedChild(Arc::new(id.clone()), "panel".into()))
                .debug_selector(|| "date-picker-panel".into())
                .flex()
                .rounded(px(4.))
                .border_1()
                .border_color(rgb(theme.border_lighter()))
                .bg(rgb(theme.overlay_background()))
                .shadow(theme.overlay_shadow())
                .occlude()
                .child(render_panel(
                    state.clone(),
                    PanelSide::Left,
                    id.clone(),
                    props.clone(),
                    theme.clone(),
                    cx,
                ));
            if props.picker_type.is_range() && !props.single_panel {
                panel = panel.child(
                    render_panel(
                        state.clone(),
                        PanelSide::Right,
                        id.clone(),
                        props.clone(),
                        theme.clone(),
                        cx,
                    )
                    .border_l_1()
                    .border_color(rgb(theme.border_lighter())),
                );
            }
            if props.show_now && !props.picker_type.is_range() && !props.picker_type.is_multiple() {
                panel = panel.child(
                    div()
                        .flex()
                        .justify_end()
                        .px(px(12.))
                        .py(px(4.))
                        .border_t_1()
                        .border_color(rgb(theme.border_lighter()))
                        .child(
                            el_button()
                                .id(ElementId::NamedChild(Arc::new(id.clone()), "now".into()))
                                .small()
                                .text()
                                .label(locale.date_picker_now())
                                .onclick({
                                    let state = state.clone();
                                    move |_, window, cx| {
                                        state.update(cx, |state, cx| {
                                            state.select_date(Local::now().date_naive(), window, cx)
                                        });
                                        window.refresh();
                                    }
                                }),
                        ),
                );
            }
            if props.show_confirm {
                panel = panel.child(
                    div()
                        .flex()
                        .justify_end()
                        .px(px(12.))
                        .py(px(4.))
                        .border_t_1()
                        .border_color(rgb(theme.border_lighter()))
                        .child(
                            el_button()
                                .id(ElementId::NamedChild(
                                    Arc::new(id.clone()),
                                    "confirm".into(),
                                ))
                                .small()
                                .primary()
                                .label(locale.date_picker_confirm())
                                .onclick({
                                    let state = state.clone();
                                    move |_, window, cx| {
                                        state.update(cx, |state, cx| state.confirm(window, cx));
                                        window.refresh();
                                    }
                                }),
                        ),
                );
            }
            panel = panel.child(measure(state.clone(), true, None));
            root = root.child(
                deferred(
                    anchored()
                        .attach(target, gpui::Anchor::BottomLeft)
                        .offset(point(px(0.), px(4.)))
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

fn render_panel(
    state: gpui::Entity<DatePickerState>,
    side: PanelSide,
    id: ElementId,
    props: DatePickerProps,
    theme: Theme,
    cx: &mut App,
) -> Div {
    let locale = Locale::current(cx);
    let (date, view) = state.read(cx).panel(side);
    let today = Local::now().date_naive();
    let selected = state.read(cx).selected_date();
    let range = state.read(cx).range_dates();
    let mut panel = div().w(px(280.)).flex_col().p(px(12.));
    let child = |name: &str| ElementId::NamedChild(Arc::new(id.clone()), name.into());
    let prev_year = child(&format!("{}-prev-year", side.name()));
    let prev_month = child(&format!("{}-prev-month", side.name()));
    let next_month = child(&format!("{}-next-month", side.name()));
    let next_year = child(&format!("{}-next-year", side.name()));
    let year_label = child(&format!("{}-year-label", side.name()));
    let month_label = child(&format!("{}-month-label", side.name()));
    let mut header = div()
        .flex()
        .items_center()
        .justify_between()
        .h(px(32.))
        .mb(px(8.));
    header = header.child(
        div()
            .flex()
            .items_center()
            .gap(px(2.))
            .child(nav_button(prev_year, "d-arrow-left", theme.clone(), {
                let state = state.clone();
                move |window, cx| {
                    state.update(cx, |state, _| {
                        state.shift_panel(
                            side,
                            0,
                            if state.panel_view(side) == PanelView::Year {
                                -10
                            } else {
                                -1
                            },
                        )
                    });
                    window.refresh();
                }
            }))
            .when(view == PanelView::Date, |el| {
                el.child(nav_button(prev_month, "arrow-left", theme.clone(), {
                    let state = state.clone();
                    move |window, cx| {
                        state.update(cx, |state, _| state.shift_panel(side, -1, 0));
                        window.refresh();
                    }
                }))
            }),
    );
    let year_text = if view == PanelView::Year {
        let start = date.year().div_euclid(10) * 10;
        format!("{start} - {}", start + 9)
    } else {
        date.year().to_string()
    };
    header = header.child(
        div()
            .id(year_label)
            .debug_selector(move || format!("date-picker-year-label-{}", side.name()))
            .px(px(5.))
            .py(px(4.))
            .rounded(px(4.))
            .cursor_pointer()
            .text_size(px(16.))
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(rgb(theme.regular_text()))
            .hover(|style| style.text_color(rgb(crate::theme::PRIMARY)))
            .child(year_text)
            .on_click({
                let state = state.clone();
                move |_, window, cx| {
                    state.update(cx, |state, _| state.set_panel_view(side, PanelView::Year));
                    window.refresh();
                }
            }),
    );
    if view == PanelView::Date {
        header = header.child(
            div()
                .id(month_label)
                .debug_selector(move || format!("date-picker-month-label-{}", side.name()))
                .px(px(5.))
                .py(px(4.))
                .rounded(px(4.))
                .cursor_pointer()
                .text_size(px(16.))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(rgb(theme.regular_text()))
                .hover(|style| style.text_color(rgb(crate::theme::PRIMARY)))
                .child(
                    locale
                        .date_picker_months()
                        .get(date.month() as usize - 1)
                        .cloned()
                        .unwrap_or_else(|| date.month().to_string()),
                )
                .on_click({
                    let state = state.clone();
                    move |_, window, cx| {
                        state.update(cx, |state, _| state.set_panel_view(side, PanelView::Month));
                        window.refresh();
                    }
                }),
        );
    }
    header = header.child(
        div()
            .flex()
            .items_center()
            .gap(px(2.))
            .when(view == PanelView::Date, |el| {
                el.child(nav_button(next_month, "arrow-right", theme.clone(), {
                    let state = state.clone();
                    move |window, cx| {
                        state.update(cx, |state, _| state.shift_panel(side, 1, 0));
                        window.refresh();
                    }
                }))
            })
            .child(nav_button(next_year, "d-arrow-right", theme.clone(), {
                let state = state.clone();
                move |window, cx| {
                    state.update(cx, |state, _| {
                        state.shift_panel(
                            side,
                            0,
                            if state.panel_view(side) == PanelView::Year {
                                10
                            } else {
                                1
                            },
                        )
                    });
                    window.refresh();
                }
            })),
    );
    panel = panel.child(header);
    match view {
        PanelView::Date => {
            let day_width = if props.show_week_number {
                (256. - 24.) / 7.
            } else {
                256. / 7.
            };
            let mut weekdays = div().flex().w_full();
            if props.show_week_number {
                weekdays = weekdays.child(div().w(px(24.)).h(px(28.)));
            }
            for label in locale.date_picker_weekdays() {
                weekdays = weekdays.child(
                    div()
                        .w(px(day_width))
                        .h(px(28.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(12.))
                        .text_color(rgb(theme.secondary_text()))
                        .border_b_1()
                        .border_color(rgb(theme.border_lighter()))
                        .child(label.clone()),
                );
            }
            panel = panel.child(weekdays);
            let multiple = state.read(cx).multiple_dates();
            let mut grid = div().flex_col().w_full();
            for (row_index, row) in calendar_cells(date).chunks(7).enumerate() {
                let mut row_view = div().flex().w_full();
                if props.show_week_number {
                    row_view = row_view.child(
                        div()
                            .w(px(24.))
                            .h(px(30.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(11.))
                            .text_color(rgb(theme.secondary_text()))
                            .child(
                                row.first()
                                    .map(|date| date.format("%V").to_string())
                                    .unwrap_or_default(),
                            ),
                    );
                }
                for (column_index, cell_date) in row.iter().copied().enumerate() {
                    let index = row_index * 7 + column_index;
                    let in_month =
                        cell_date.month() == date.month() && cell_date.year() == date.year();
                    let is_start = range.0 == Some(cell_date);
                    let is_end = range.1 == Some(cell_date);
                    let hover_end = if state.read(cx).selecting_end {
                        state.read(cx).range_hover
                    } else {
                        None
                    };
                    let preview_end = hover_end.or(range.1);
                    let in_range = range.0.is_some_and(|start| {
                        preview_end.is_some_and(|end| {
                            let (start, end) = if start <= end {
                                (start, end)
                            } else {
                                (end, start)
                            };
                            cell_date >= start && cell_date <= end
                        })
                    });
                    let selected = selected == Some(cell_date)
                        || is_start
                        || is_end
                        || (props.picker_type.is_multiple() && multiple.contains(&cell_date));
                    let disabled = props.disabled_on(cell_date);
                    let cell_id = child(&format!("{}-day-{index}", side.name()));
                    let cell = picker_cell(
                        cell_id,
                        cell_date.day().to_string(),
                        day_width,
                        30.,
                        selected,
                        in_range,
                        is_start,
                        is_end,
                        cell_date == today,
                        disabled,
                        !in_month,
                        theme.clone(),
                        {
                            let state = state.clone();
                            move |window, cx| {
                                state.update(cx, |state, cx| {
                                    state.select_date(cell_date, window, cx)
                                });
                                window.refresh();
                            }
                        },
                    )
                    .debug_selector(move || format!("date-picker-{}-day-{index}", side.name()));
                    row_view = row_view.child(cell.on_hover({
                        let state = state.clone();
                        move |_, window, cx| {
                            let selecting = state.read(cx).selecting_end;
                            let changed = state.read(cx).range_hover != Some(cell_date);
                            if selecting && changed {
                                state.update(cx, |state, _| state.range_hover = Some(cell_date));
                                window.refresh();
                            }
                        }
                    }));
                }
                grid = grid.child(row_view);
            }
            panel = panel.child(grid);
        }
        PanelView::Month => {
            panel = panel.child(render_unit_grid(
                state.clone(),
                side,
                id.clone(),
                props.clone(),
                theme.clone(),
                locale,
                today,
                PanelView::Month,
                cx,
            ));
        }
        PanelView::Year => {
            panel = panel.child(render_unit_grid(
                state,
                side,
                id,
                props,
                theme,
                locale,
                today,
                PanelView::Year,
                cx,
            ));
        }
        PanelView::Quarter => {
            panel = panel.child(render_unit_grid(
                state,
                side,
                id,
                props,
                theme,
                locale,
                today,
                PanelView::Quarter,
                cx,
            ));
        }
    }
    panel
}

#[allow(clippy::too_many_arguments)]
fn render_unit_grid(
    state: gpui::Entity<DatePickerState>,
    side: PanelSide,
    id: ElementId,
    props: DatePickerProps,
    theme: Theme,
    locale: Locale,
    today: NaiveDate,
    view: PanelView,
    cx: &App,
) -> Div {
    let date = state.read(cx).panel_date(side);
    let selected = state.read(cx).selected_date();
    let multiple = state.read(cx).multiple_dates();
    let range = state.read(cx).range_dates();
    let count = match view {
        PanelView::Month => 12,
        PanelView::Year => 10,
        PanelView::Quarter => 4,
        PanelView::Date => return div(),
    };
    let columns = 4;
    let start_year = date.year().div_euclid(10) * 10;
    let mut grid = div().flex().flex_wrap().w_full();
    for index in 0..count {
        let (cell_date, label, unit) = match view {
            PanelView::Month => {
                let month = index as u32 + 1;
                let cell_date = NaiveDate::from_ymd_opt(date.year(), month, 1).unwrap_or(date);
                let label = locale
                    .date_picker_months()
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| month.to_string());
                (cell_date, label, DateUnit::Month)
            }
            PanelView::Year => {
                let year = start_year + index as i32;
                let cell_date = NaiveDate::from_ymd_opt(year, 1, 1).unwrap_or(date);
                (cell_date, year.to_string(), DateUnit::Year)
            }
            PanelView::Quarter => {
                let month = index as u32 * 3 + 1;
                let cell_date = NaiveDate::from_ymd_opt(date.year(), month, 1).unwrap_or(date);
                (cell_date, format!("Q{}", index + 1), DateUnit::Quarter)
            }
            PanelView::Date => continue,
        };
        let is_selected = selected == Some(cell_date)
            || multiple.contains(&cell_date)
            || range.0 == Some(cell_date)
            || range.1 == Some(cell_date);
        let in_range = range.0.is_some_and(|start| {
            range.1.is_some_and(|end| {
                let (start, end) = if start <= end {
                    (start, end)
                } else {
                    (end, start)
                };
                cell_date >= start_of_unit(start, unit) && cell_date <= start_of_unit(end, unit)
            })
        });
        let cell_id = ElementId::NamedChild(
            Arc::new(id.clone()),
            format!("{}-{view:?}-{index}", side.name()).into(),
        );
        let next_view = match (props.picker_type, view) {
            (DatePickerType::Date, PanelView::Year) => PanelView::Month,
            (DatePickerType::Date, PanelView::Month) => PanelView::Date,
            (DatePickerType::DateRange, PanelView::Year) => PanelView::Month,
            (DatePickerType::DateRange, PanelView::Month) => PanelView::Date,
            _ => view,
        };
        let cell = picker_cell(
            cell_id,
            label,
            256. / columns as f32,
            48.,
            is_selected,
            in_range,
            range.0 == Some(cell_date),
            range.1 == Some(cell_date),
            start_of_unit(today, unit) == cell_date,
            props.unit_disabled(cell_date, unit),
            false,
            theme.clone(),
            {
                let state = state.clone();
                move |window, cx| {
                    if props.picker_type == DatePickerType::Date
                        || props.picker_type == DatePickerType::DateRange
                    {
                        state.update(cx, |state, cx| {
                            state.select_header_date(side, cell_date, next_view, window, cx)
                        });
                    } else {
                        state.update(cx, |state, cx| state.select_date(cell_date, window, cx));
                    }
                    window.refresh();
                }
            },
        )
        .debug_selector(move || format!("date-picker-{}-{view:?}-{index}", side.name()));
        grid = grid.child(cell);
    }
    grid
}

fn nav_button(
    id: ElementId,
    icon: &'static str,
    theme: Theme,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    div()
        .id(id)
        .w(px(28.))
        .h(px(28.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(4.))
        .cursor_pointer()
        .text_color(rgb(theme.regular_text()))
        .hover(|style| style.bg(rgb(theme.fill(crate::theme::Fill::Light))))
        .child(el_icon(icon).size(14.))
        .on_click(move |_, window, cx| {
            cx.stop_propagation();
            on_click(window, cx);
        })
}

#[allow(clippy::too_many_arguments)]
fn picker_cell(
    id: ElementId,
    label: impl IntoElement,
    width: f32,
    height: f32,
    selected: bool,
    in_range: bool,
    start: bool,
    end: bool,
    today: bool,
    disabled: bool,
    muted: bool,
    theme: Theme,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let text_size = if height <= 30. { 24. } else { 36. };
    let mut cell = div()
        .id(id)
        .w(px(width))
        .h(px(height))
        .flex()
        .items_center()
        .justify_center()
        .when(in_range, |el| {
            el.bg(rgb(theme.tint(crate::theme::PRIMARY, 8)))
        })
        .when(start, |el| el.rounded_tl(px(18.)).rounded_bl(px(18.)))
        .when(end, |el| el.rounded_tr(px(18.)).rounded_br(px(18.)))
        .when(disabled, |el| el.cursor_not_allowed())
        .when(!disabled, |el| el.cursor_pointer())
        .aria_selected(selected)
        .child(
            div()
                .w(px(text_size))
                .h(px(text_size))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(text_size / 2.))
                .text_size(px(12.))
                .text_color(if disabled {
                    rgb(theme.disabled_text())
                } else if selected {
                    rgb(0xffffff)
                } else if muted {
                    rgb(theme.disabled_text())
                } else if today {
                    rgb(crate::theme::PRIMARY)
                } else {
                    rgb(theme.regular_text())
                })
                .when(today && !selected, |el| {
                    el.font_weight(gpui::FontWeight::BOLD)
                })
                .when(selected, |el| el.bg(rgb(crate::theme::PRIMARY)))
                .when(disabled, |el| {
                    el.bg(rgb(theme.fill(crate::theme::Fill::Light)))
                })
                .when(!disabled && !selected, |el| {
                    el.hover(|style| style.text_color(rgb(crate::theme::PRIMARY)))
                })
                .child(label),
        );
    if !disabled {
        cell = cell.on_click(move |_, window, cx| {
            cx.stop_propagation();
            on_click(window, cx);
        });
    }
    cell
}

impl DatePickerState {
    fn panel(&self, side: PanelSide) -> (NaiveDate, PanelView) {
        match side {
            PanelSide::Left => (self.left_date, self.left_view),
            PanelSide::Right => (self.right_date, self.right_view),
        }
    }

    fn panel_date(&self, side: PanelSide) -> NaiveDate {
        self.panel(side).0
    }

    fn panel_view(&self, side: PanelSide) -> PanelView {
        self.panel(side).1
    }

    fn set_panel_view(&mut self, side: PanelSide, view: PanelView) {
        match side {
            PanelSide::Left => self.left_view = view,
            PanelSide::Right => self.right_view = view,
        }
    }

    fn range_dates(&self) -> (Option<NaiveDate>, Option<NaiveDate>) {
        let end = if self.selecting_end {
            self.range_hover.or(self.range_end)
        } else {
            self.range_end
        };
        (self.range_start, end)
    }

    fn multiple_dates(&self) -> Vec<NaiveDate> {
        self.props
            .multiple
            .iter()
            .filter_map(|value| parse_bound_date_with(&self.props, value))
            .collect()
    }
}

#[cfg(test)]
fn parse_bound_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").ok()
}

fn parse_bound_date_with(props: &DatePickerProps, value: &str) -> Option<NaiveDate> {
    let value = value.trim();
    if let Some(pattern) = props.value_format.as_deref() {
        return NaiveDate::parse_from_str(value, &chrono_pattern(pattern)).ok();
    }
    match props.picker_type {
        DatePickerType::Quarter | DatePickerType::QuarterRange | DatePickerType::Quarters => {
            let (year, quarter) = value.split_once("-Q")?;
            let quarter = quarter.parse::<u32>().ok()?;
            if !(1..=4).contains(&quarter) {
                return None;
            }
            let year = year.parse::<i32>().ok()?;
            NaiveDate::from_ymd_opt(year, (quarter - 1) * 3 + 1, 1)
        }
        _ => NaiveDate::parse_from_str(value, props.picker_type.default_format()).ok(),
    }
}

fn chrono_pattern(pattern: &str) -> String {
    pattern
        .replace("MMM", "%b")
        .replace("YYYY", "%Y")
        .replace("YY", "%y")
        .replace("MM", "%m")
        .replace("DD", "%d")
        .replace("WW", "%V")
        .replace("HH", "%H")
        .replace("mm", "%M")
        .replace("ss", "%S")
}

fn format_date_pattern(date: NaiveDate, pattern: &str, locale: Locale) -> String {
    if pattern == "%Y-%m-%d" {
        return date.format("%Y-%m-%d").to_string();
    }
    if pattern == "%Y-%m" {
        return date.format("%Y-%m").to_string();
    }
    if pattern == "%Y" {
        return date.format("%Y").to_string();
    }
    if pattern == "%Y-Q%q" {
        return format!("{:04}-Q{}", date.year(), (date.month() - 1) / 3 + 1);
    }
    if pattern == "%Y-W%V" {
        return date.format("%Y-W%V").to_string();
    }
    let mut output = String::new();
    let mut rest = pattern;
    while !rest.is_empty() {
        if rest.starts_with("YYYY") {
            output.push_str(&format!("{:04}", date.year()));
            rest = &rest[4..];
        } else if rest.starts_with("YY") {
            output.push_str(&format!("{:02}", date.year().rem_euclid(100)));
            rest = &rest[2..];
        } else if rest.starts_with("MMM") {
            output.push_str(
                locale
                    .date_picker_months()
                    .get(date.month() as usize - 1)
                    .map(String::as_str)
                    .unwrap_or(""),
            );
            rest = &rest[3..];
        } else if rest.starts_with("MM") {
            output.push_str(&format!("{:02}", date.month()));
            rest = &rest[2..];
        } else if rest.starts_with("DD") {
            output.push_str(&format!("{:02}", date.day()));
            rest = &rest[2..];
        } else if rest.starts_with("Q") {
            output.push_str(&((date.month() - 1) / 3 + 1).to_string());
            rest = &rest[1..];
        } else if rest.starts_with("WW") {
            output.push_str(&date.format("%V").to_string());
            rest = &rest[2..];
        } else if rest.starts_with("HH") || rest.starts_with("mm") || rest.starts_with("ss") {
            output.push_str("00");
            rest = &rest[2..];
        } else if let Some(character) = rest.chars().next() {
            output.push(character);
            rest = &rest[character.len_utf8()..];
        }
    }
    output
}

fn shift_year(date: NaiveDate, delta: i32) -> NaiveDate {
    let Some(year) = date.year().checked_add(delta) else {
        return date;
    };
    NaiveDate::from_ymd_opt(
        year,
        date.month(),
        date.day().min(days_in_month(year, date.month())),
    )
    .unwrap_or(date)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let Some(first) = NaiveDate::from_ymd_opt(year, month, 1) else {
        return 28;
    };
    let next = if month == 12 {
        year.checked_add(1)
            .and_then(|year| NaiveDate::from_ymd_opt(year, 1, 1))
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    next.and_then(|next| next.pred_opt())
        .map(|last| last.day())
        .unwrap_or_else(|| first.day())
}

fn shift_month(date: NaiveDate, delta: i32) -> NaiveDate {
    let month = date.year() as i64 * 12 + date.month() as i64 - 1 + delta as i64;
    let year = month.div_euclid(12) as i32;
    let month = month.rem_euclid(12) as u32 + 1;
    let day = date.day().min(days_in_month(year, month));
    NaiveDate::from_ymd_opt(year, month, day).unwrap_or(date)
}

fn shift_by_unit(date: NaiveDate, delta: i32, unit: DateUnit) -> NaiveDate {
    match unit {
        DateUnit::Day | DateUnit::Month => shift_month(date, delta),
        DateUnit::Year => shift_year(date, delta),
        DateUnit::Quarter => shift_month(date, delta * 3),
    }
}

fn start_of_unit(date: NaiveDate, unit: DateUnit) -> NaiveDate {
    match unit {
        DateUnit::Day => date,
        DateUnit::Month => NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap_or(date),
        DateUnit::Year => NaiveDate::from_ymd_opt(date.year(), 1, 1).unwrap_or(date),
        DateUnit::Quarter => {
            let month = (date.month() - 1) / 3 * 3 + 1;
            NaiveDate::from_ymd_opt(date.year(), month, 1).unwrap_or(date)
        }
    }
}

fn calendar_cells(view: NaiveDate) -> Vec<NaiveDate> {
    let Some(first) = NaiveDate::from_ymd_opt(view.year(), view.month(), 1) else {
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
    let Some(start) = first.checked_sub_signed(chrono::Duration::days(offset as i64)) else {
        return Vec::new();
    };
    (0..42)
        .filter_map(|index| start.checked_add_signed(chrono::Duration::days(index)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_formats_iso_dates() {
        let date = parse_bound_date("2026-11-28").unwrap();
        assert_eq!(
            format_date_pattern(date, "%Y-%m-%d", Locale::English),
            "2026-11-28"
        );
        assert!(parse_bound_date("2026-02-30").is_none());
        assert!(parse_bound_date("not-a-date").is_none());
    }

    #[test]
    fn calendar_has_six_weeks_and_month_navigation_wraps() {
        let cells = calendar_cells(NaiveDate::from_ymd_opt(2026, 11, 1).unwrap());
        assert_eq!(cells.len(), 42);
        assert_eq!(
            cells
                .iter()
                .filter(|date| date.month() == 11 && date.year() == 2026)
                .count(),
            30
        );
        assert_eq!(
            shift_month(NaiveDate::from_ymd_opt(2026, 12, 15).unwrap(), 1),
            NaiveDate::from_ymd_opt(2027, 1, 15).unwrap()
        );
        assert_eq!(
            shift_month(NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(), -1),
            NaiveDate::from_ymd_opt(2025, 12, 15).unwrap()
        );
    }

    #[test]
    fn formats_picker_measurements() {
        let date = NaiveDate::from_ymd_opt(2026, 11, 28).unwrap();
        assert_eq!(
            format_date_pattern(
                date,
                DatePickerType::Month.default_format(),
                Locale::English
            ),
            "2026-11"
        );
        assert_eq!(
            format_date_pattern(date, DatePickerType::Year.default_format(), Locale::English),
            "2026"
        );
        assert_eq!(
            format_date_pattern(
                date,
                DatePickerType::Quarter.default_format(),
                Locale::English
            ),
            "2026-Q4"
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

    #[gpui::test]
    fn range_selection_swaps_dates_and_reports_once(cx: &mut gpui::TestAppContext) {
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
                    state.props.picker_type = DatePickerType::DateRange;
                    state.handlers.onrangechange = Some(Rc::new({
                        let changes = changes.clone();
                        move |start, end, _, _| changes.borrow_mut().push((start, end))
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
            assert!(state.read(cx).open);
            state.update(cx, |state, cx| {
                state.select_date(NaiveDate::from_ymd_opt(2026, 11, 3).unwrap(), window, cx)
            });
            assert!(!state.read(cx).open);
        });
        assert_eq!(
            *changes.borrow(),
            [(
                Some(SharedString::from("2026-11-03")),
                Some(SharedString::from("2026-11-28"))
            )]
        );
    }
}
