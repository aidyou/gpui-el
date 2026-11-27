//! Element Plus InputNumber.
//!
//! Follows `input-number.md`, `components/input-number/src/input-number.vue`,
//! and Theme Chalk `input-number.scss`: a 150/180/120 px field with optional
//! side controls, numeric stepping, strict steps, precision, range clamping,
//! and a configurable clear value. The text field itself reuses [`Input`], so
//! caret movement, selection, clipboard, and IME composition stay native.
//!
//! Native differences: `formatter` receives an `f64` and `parser` returns the
//! text to parse; `name`, `inputmode`, `tabindex`, and the imperative
//! `focus`/`blur` exposes have no direct mapping yet. `validate-event` belongs
//! to the future Form component. An inverted `min`/`max` range collapses to
//! `min` instead of throwing, and precision is capped at f64's 15 useful
//! decimal places.

use std::{rc::Rc, sync::Arc, time::Duration};

use gpui::{
    AnyElement, App, ElementId, FocusHandle, IntoElement, KeyUpEvent, MouseButton, RenderOnce,
    SharedString, Task, TextAlign, Window, div, prelude::*, px, rgb,
};

use crate::{Icon, InputSize, Locale, Theme, el_icon, el_input};

const MIN_SAFE_INTEGER: f64 = -9_007_199_254_740_991.;
const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.;
const REPEAT_DELAY: Duration = Duration::from_millis(400);
const REPEAT_INTERVAL: Duration = Duration::from_millis(60);

/// Horizontal alignment of the numeric text.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum InputNumberAlign {
    Left,
    #[default]
    Center,
    Right,
}

impl InputNumberAlign {
    fn text_align(self) -> TextAlign {
        match self {
            Self::Left => TextAlign::Left,
            Self::Center => TextAlign::Center,
            Self::Right => TextAlign::Right,
        }
    }
}

/// The value committed when the field is cleared.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputNumberValueOnClear {
    Min,
    Max,
    Value(f64),
}

type NumberHandler = Rc<dyn Fn(Option<f64>, &mut Window, &mut App)>;
type ChangeHandler = Rc<dyn Fn(Option<f64>, Option<f64>, &mut Window, &mut App)>;
type FocusHandler = Rc<dyn Fn(&mut Window, &mut App)>;
type Formatter = Arc<dyn Fn(f64) -> String>;
type Parser = Arc<dyn Fn(&str) -> String>;

#[derive(Clone)]
struct NumberProps {
    value: Option<f64>,
    min: f64,
    max: f64,
    step: f64,
    step_strictly: bool,
    precision: Option<u32>,
    value_on_clear: Option<InputNumberValueOnClear>,
    readonly: bool,
    disabled: bool,
    disabled_scientific: bool,
    formatter: Option<Formatter>,
    parser: Option<Parser>,
}

impl Default for NumberProps {
    fn default() -> Self {
        Self {
            value: None,
            min: MIN_SAFE_INTEGER,
            max: MAX_SAFE_INTEGER,
            step: 1.,
            step_strictly: false,
            precision: None,
            value_on_clear: None,
            readonly: false,
            disabled: false,
            disabled_scientific: false,
            formatter: None,
            parser: None,
        }
    }
}

#[derive(Clone, Default)]
struct NumberHandlers {
    oninput: Option<NumberHandler>,
    onchange: Option<ChangeHandler>,
    onfocus: Option<FocusHandler>,
    onblur: Option<FocusHandler>,
}

struct InputNumberState {
    draft: Option<String>,
    focused: bool,
    last_value: Option<f64>,
    last_precision: Option<u32>,
    props: NumberProps,
    handlers: NumberHandlers,
    increase_focus: FocusHandle,
    decrease_focus: FocusHandle,
    repeat_task: Option<Task<()>>,
    repeating: bool,
}

impl InputNumberState {
    fn new(cx: &mut gpui::Context<Self>) -> Self {
        Self {
            draft: None,
            focused: false,
            last_value: None,
            last_precision: None,
            props: NumberProps::default(),
            handlers: NumberHandlers::default(),
            increase_focus: cx.focus_handle(),
            decrease_focus: cx.focus_handle(),
            repeat_task: None,
            repeating: false,
        }
    }

    fn sync_props(&mut self, value: Option<f64>, precision: Option<u32>) {
        if self.last_precision != precision {
            self.draft = None;
            self.last_precision = precision;
        }
        if self.last_value != value {
            let draft_matches = self
                .draft
                .as_deref()
                .and_then(|draft| parse_number(draft, self.props.parser.as_ref()))
                == value;
            if !self.focused || !draft_matches {
                self.draft = None;
            }
            self.last_value = value;
        }
        if !self.focused && self.draft.is_none() {
            self.last_value = value;
        }
    }

    fn set_focused(&mut self, focused: bool, window: &mut Window, cx: &mut App) {
        self.focused = focused;
        let handler = if focused {
            self.handlers.onfocus.clone()
        } else {
            self.handlers.onblur.clone()
        };
        if let Some(handler) = handler {
            handler(window, cx);
        }
    }

    fn accept_input(&mut self, reported: SharedString, window: &mut Window, cx: &mut App) {
        if self.props.disabled || self.props.readonly {
            return;
        }
        let mut raw = reported.to_string();
        if self.props.disabled_scientific {
            raw.retain(|ch| !matches!(ch, 'e' | 'E'));
        }
        let parsed = parse_number(&raw, self.props.parser.as_ref());
        self.draft = Some(raw);
        self.last_value = parsed;
        if let Some(handler) = self.handlers.oninput.clone() {
            handler(parsed, window, cx);
        }
    }

    fn commit(&mut self, reported: SharedString, window: &mut Window, cx: &mut App) {
        if self.props.disabled || self.props.readonly {
            return;
        }
        let raw = if self.draft.is_some() {
            reported.to_string()
        } else {
            format_value(self.props.value, &self.props)
        };
        let new_value = verify_value(&raw, &self.props);
        let old_value = self.last_value;
        self.draft = None;
        self.last_value = new_value;
        if new_value != old_value
            && let Some(handler) = self.handlers.onchange.clone()
        {
            handler(new_value, old_value, window, cx);
        }
    }

    fn current_value(&self) -> Option<f64> {
        self.draft
            .as_deref()
            .and_then(|draft| parse_number(draft, self.props.parser.as_ref()))
            .or(self.last_value)
    }

    fn step(&mut self, direction: i32, window: &mut Window, cx: &mut App) -> bool {
        let current = self.current_value();
        if !can_step(&self.props, current, direction) {
            return false;
        }
        let base = current.unwrap_or(0.);
        let Some(new_value) = verify_number(base + self.props.step * direction as f64, &self.props)
        else {
            return false;
        };
        let old_value = self.last_value;
        self.draft = None;
        self.last_value = Some(new_value);
        if let Some(handler) = self.handlers.oninput.clone() {
            handler(Some(new_value), window, cx);
        }
        if new_value != old_value.unwrap_or(f64::NAN)
            && let Some(handler) = self.handlers.onchange.clone()
        {
            handler(Some(new_value), old_value, window, cx);
        }
        can_step(&self.props, Some(new_value), direction)
    }

    fn start_repeat(
        &mut self,
        direction: i32,
        state_entity: gpui::Entity<Self>,
        window: &mut Window,
        cx: &mut App,
    ) -> bool {
        if !self.step(direction, window, cx) {
            return false;
        }
        self.repeat_task.take();
        let handle = window.window_handle();
        self.repeating = true;
        let task = cx.spawn(async move |cx| {
            cx.background_executor().timer(REPEAT_DELAY).await;
            loop {
                let Ok(keep_repeating) = handle.update(cx, |_, window, cx| {
                    state_entity.update(cx, |state, cx| {
                        if !state.repeating {
                            return false;
                        }
                        state.step(direction, window, cx)
                    })
                }) else {
                    break;
                };
                if !keep_repeating {
                    break;
                }
                cx.background_executor().timer(REPEAT_INTERVAL).await;
            }
        });
        self.repeat_task = Some(task);
        true
    }

    fn stop_repeat(&mut self) {
        self.repeating = false;
        self.repeat_task.take();
    }
}

/// An Element Plus numeric input field.
#[derive(IntoElement)]
#[must_use]
pub struct InputNumber {
    id: ElementId,
    value: Option<f64>,
    min: f64,
    max: f64,
    step: f64,
    step_strictly: bool,
    precision: Option<u32>,
    value_on_clear: Option<InputNumberValueOnClear>,
    size: InputSize,
    align: InputNumberAlign,
    controls: bool,
    controls_right: bool,
    disabled: bool,
    readonly: bool,
    disabled_scientific: bool,
    placeholder: SharedString,
    accessibility_label: Option<SharedString>,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
    decrease_icon: Option<Icon>,
    increase_icon: Option<Icon>,
    formatter: Option<Formatter>,
    parser: Option<Parser>,
    oninput: Option<NumberHandler>,
    onchange: Option<ChangeHandler>,
    onfocus: Option<FocusHandler>,
    onblur: Option<FocusHandler>,
}

/// Creates a numeric input field.
#[track_caller]
pub fn el_input_number() -> InputNumber {
    InputNumber::new()
}

impl Default for InputNumber {
    fn default() -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            value: None,
            min: MIN_SAFE_INTEGER,
            max: MAX_SAFE_INTEGER,
            step: 1.,
            step_strictly: false,
            precision: None,
            value_on_clear: None,
            size: InputSize::Default,
            align: InputNumberAlign::Center,
            controls: true,
            controls_right: false,
            disabled: false,
            readonly: false,
            disabled_scientific: false,
            placeholder: "".into(),
            accessibility_label: None,
            prefix: None,
            suffix: None,
            decrease_icon: None,
            increase_icon: None,
            formatter: None,
            parser: None,
            oninput: None,
            onchange: None,
            onfocus: None,
            onblur: None,
        }
    }
}

impl InputNumber {
    /// Creates an empty numeric field.
    #[track_caller]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// Sets the controlled value. Non-finite values are treated as empty.
    pub fn value(mut self, value: Option<f64>) -> Self {
        self.value = value.filter(|value| value.is_finite());
        self
    }

    pub fn min(mut self, min: f64) -> Self {
        if min.is_finite() {
            self.min = min;
        }
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        if max.is_finite() {
            self.max = max;
        }
        self
    }

    /// Sets the increment. Non-positive or non-finite values restore 1.
    pub fn step(mut self, step: f64) -> Self {
        self.step = if step.is_finite() && step > 0. {
            step
        } else {
            1.
        };
        self
    }

    pub fn step_strictly(mut self, strict: bool) -> Self {
        self.step_strictly = strict;
        self
    }

    /// Sets the decimal precision, capped at 15 useful f64 digits.
    pub fn precision(mut self, precision: u32) -> Self {
        self.precision = Some(precision.min(15));
        self
    }

    pub fn value_on_clear(mut self, value: InputNumberValueOnClear) -> Self {
        self.value_on_clear = match value {
            InputNumberValueOnClear::Value(value) if value.is_finite() => {
                Some(InputNumberValueOnClear::Value(value))
            }
            InputNumberValueOnClear::Value(_) => None,
            value => Some(value),
        };
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

    pub fn align(mut self, align: InputNumberAlign) -> Self {
        self.align = align;
        self
    }

    pub fn controls(mut self, controls: bool) -> Self {
        self.controls = controls;
        self
    }

    /// Stacks the increase/decrease controls at the right edge.
    pub fn controls_right(mut self, right: bool) -> Self {
        self.controls_right = right;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    pub fn disabled_scientific(mut self, disabled: bool) -> Self {
        self.disabled_scientific = disabled;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    pub fn prefix(mut self, prefix: impl IntoElement) -> Self {
        self.prefix = Some(prefix.into_any_element());
        self
    }

    pub fn suffix(mut self, suffix: impl IntoElement) -> Self {
        self.suffix = Some(suffix.into_any_element());
        self
    }

    pub fn decrease_icon(mut self, icon: Icon) -> Self {
        self.decrease_icon = Some(icon);
        self
    }

    pub fn increase_icon(mut self, icon: Icon) -> Self {
        self.increase_icon = Some(icon);
        self
    }

    /// Formats a committed value for display. Draft text stays unformatted.
    pub fn formatter(mut self, formatter: impl Fn(f64) -> String + 'static) -> Self {
        self.formatter = Some(Arc::new(formatter));
        self
    }

    /// Extracts numeric text from the raw edit before parsing.
    pub fn parser(mut self, parser: impl Fn(&str) -> String + 'static) -> Self {
        self.parser = Some(Arc::new(parser));
        self
    }

    /// Reports the parsed value while typing, stepping, or clearing.
    #[track_caller]
    pub fn oninput(
        mut self,
        handler: impl Fn(Option<f64>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.oninput = Some(Rc::new(handler));
        self
    }

    /// Reports a committed value and its previous value on blur or stepping.
    #[track_caller]
    pub fn onchange(
        mut self,
        handler: impl Fn(Option<f64>, Option<f64>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onchange = Some(Rc::new(handler));
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

fn sanitize_value(value: Option<f64>) -> Option<f64> {
    value.filter(|value| value.is_finite())
}

fn decimal_places(value: f64) -> u32 {
    let text = format!("{value}");
    let (mantissa, exponent) = text
        .split_once('e')
        .or_else(|| text.split_once('E'))
        .unwrap_or((text.as_str(), "0"));
    let exponent: i32 = exponent.parse().unwrap_or(0);
    let places = mantissa
        .split_once('.')
        .map(|(_, fraction)| fraction.len() as i32)
        .unwrap_or(0);
    (places - exponent).max(0) as u32
}

fn round_to(value: f64, precision: u32) -> f64 {
    let precision = precision.min(15) as usize;
    format!("{value:.precision$}").parse().unwrap_or(value)
}

fn parse_number(raw: &str, parser: Option<&Parser>) -> Option<f64> {
    let parsed = parser
        .map(|parser| parser(raw))
        .unwrap_or_else(|| raw.to_owned());
    let text = parsed.trim_start();
    let bytes = text.as_bytes();
    let mut index = 0;
    if bytes.get(index) == Some(&b'-') || bytes.get(index) == Some(&b'+') {
        index += 1;
    }
    let mut digits = 0;
    while bytes.get(index).is_some_and(u8::is_ascii_digit) {
        digits += 1;
        index += 1;
    }
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            digits += 1;
            index += 1;
        }
    }
    if digits == 0 {
        return None;
    }
    if bytes.get(index) == Some(&b'e') || bytes.get(index) == Some(&b'E') {
        let exponent_start = index;
        index += 1;
        if bytes.get(index) == Some(&b'-') || bytes.get(index) == Some(&b'+') {
            index += 1;
        }
        let exponent_digits = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if index == exponent_digits {
            index = exponent_start;
        }
    }
    let parsed = text[..index].parse::<f64>().ok();
    parsed.filter(|value| value.is_finite())
}

fn verify_number(mut value: f64, props: &NumberProps) -> Option<f64> {
    if !value.is_finite() {
        return None;
    }
    if props.step_strictly {
        let precision = props
            .precision
            .unwrap_or_else(|| decimal_places(props.step));
        value = round_to((value / props.step).round() * props.step, precision);
    }
    if let Some(precision) = props.precision {
        value = round_to(value, precision);
    }
    Some(value.clamp(props.min, props.max.max(props.min)))
}

fn verify_value(raw: &str, props: &NumberProps) -> Option<f64> {
    if raw.trim().is_empty() {
        return match props.value_on_clear {
            Some(InputNumberValueOnClear::Min) => Some(props.min),
            Some(InputNumberValueOnClear::Max) => Some(props.max.max(props.min)),
            Some(InputNumberValueOnClear::Value(value)) if value.is_finite() => {
                verify_number(value, props)
            }
            _ => None,
        };
    }
    parse_number(raw, props.parser.as_ref()).and_then(|value| verify_number(value, props))
}

fn format_value(value: Option<f64>, props: &NumberProps) -> String {
    let Some(value) = value else {
        return String::new();
    };
    if let Some(formatter) = &props.formatter {
        return formatter(value);
    }
    if let Some(precision) = props.precision {
        return format!("{value:.precision$}", precision = precision as usize);
    }
    let text = format!("{value}");
    if text == "-0" { "0".into() } else { text }
}

fn can_step(props: &NumberProps, current: Option<f64>, direction: i32) -> bool {
    if props.disabled || props.readonly {
        return false;
    }
    let max = props.max.max(props.min);
    match current {
        Some(value) if direction > 0 => value < max,
        Some(value) if direction < 0 => value > props.min,
        _ => true,
    }
}

fn number_width(size: InputSize) -> f32 {
    match size {
        InputSize::Large => 180.,
        InputSize::Default => 150.,
        InputSize::Small => 120.,
    }
}

fn control_padding(size: InputSize, controls_right: bool, controls: bool) -> (f32, f32) {
    if !controls {
        return (15., 15.);
    }
    let control = size.height() + if size == InputSize::Default { 10. } else { 7. };
    if controls_right {
        (15., control)
    } else {
        (control, control)
    }
}

impl RenderOnce for InputNumber {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let InputNumber {
            id,
            value,
            min,
            max,
            step,
            step_strictly,
            precision,
            value_on_clear,
            size,
            align,
            controls,
            controls_right,
            disabled,
            readonly,
            disabled_scientific,
            placeholder,
            accessibility_label,
            prefix,
            suffix,
            decrease_icon,
            increase_icon,
            formatter,
            parser,
            oninput,
            onchange,
            onfocus,
            onblur,
        } = self;
        let value = sanitize_value(value);
        let props = NumberProps {
            value,
            min,
            max,
            step,
            step_strictly,
            precision,
            value_on_clear,
            readonly,
            disabled,
            disabled_scientific,
            formatter,
            parser,
        };
        let handlers = NumberHandlers {
            oninput,
            onchange,
            onfocus,
            onblur,
        };
        let state = window.use_keyed_state(id.clone(), cx, |_, cx| InputNumberState::new(cx));
        state.update(cx, |state, _| {
            state.props = props.clone();
            state.handlers = handlers;
        });
        state.update(cx, |state, _| state.sync_props(value, precision));
        let display = state
            .read(cx)
            .draft
            .clone()
            .unwrap_or_else(|| format_value(value, &props));
        let current = state.read(cx).current_value();
        let increase_disabled = disabled || readonly || !can_step(&props, current, 1);
        let decrease_disabled = disabled || readonly || !can_step(&props, current, -1);
        let increase_focused = state.read(cx).increase_focus.is_focused(window);
        let decrease_focused = state.read(cx).decrease_focus.is_focused(window);
        let theme = Theme::current(cx);
        let locale = Locale::current(cx);
        let height = size.height();
        let width = number_width(size);
        let (padding_left, padding_right) = control_padding(size, controls_right, controls);
        let input_id = ElementId::NamedChild(Arc::new(id.clone()), "input".into());
        let mut input = el_input()
            .id(input_id)
            .value(display)
            .placeholder(placeholder)
            .size(size)
            .radius_small()
            .disabled(disabled)
            .readonly(readonly)
            .horizontal_padding(padding_left, padding_right)
            .text_alignment(align.text_align())
            .oninput({
                let state = state.clone();
                move |reported, window, cx| {
                    state.update(cx, |state, cx| state.accept_input(reported, window, cx));
                }
            })
            .onchange({
                let state = state.clone();
                move |reported, window, cx| {
                    state.update(cx, |state, cx| state.commit(reported, window, cx));
                }
            })
            .onfocus({
                let state = state.clone();
                move |window, cx| {
                    state.update(cx, |state, cx| state.set_focused(true, window, cx));
                }
            })
            .onblur({
                let state = state.clone();
                move |window, cx| {
                    state.update(cx, |state, cx| state.set_focused(false, window, cx));
                }
            })
            .on_key_down_filter({
                let state = state.clone();
                move |event, window, cx| {
                    let direction = match event.keystroke.key.as_str() {
                        "up" => 1,
                        "down" => -1,
                        _ => return false,
                    };
                    state.update(cx, |state, cx| state.step(direction, window, cx));
                    true
                }
            });
        if let Some(label) = accessibility_label {
            input = input.aria_label(label);
        }
        if let Some(prefix) = prefix {
            input = input.prefix(prefix);
        }
        if let Some(suffix) = suffix {
            input = input.suffix(suffix);
        }

        let control_color = if disabled {
            rgb(theme.border(true))
        } else {
            rgb(theme.regular_text())
        };
        let control_bg = rgb(theme.fill(crate::theme::Fill::Light));
        let control_border = rgb(if disabled {
            theme.border(true)
        } else {
            theme.border(false)
        });
        let primary = rgb(crate::theme::PRIMARY);
        let icon_size = match size {
            InputSize::Small => 10.8,
            InputSize::Default => 13.,
            InputSize::Large => 14.,
        };
        let control = |direction: i32,
                       focus: FocusHandle,
                       focused: bool,
                       icon: Icon,
                       label: SharedString,
                       control_id: ElementId,
                       control_disabled: bool| {
            let state = state.clone();
            let mut control = div()
                .id(control_id)
                .absolute()
                .flex()
                .items_center()
                .justify_center()
                .w(px(height))
                .bg(control_bg)
                .text_color(control_color)
                .track_focus(&focus)
                .tab_stop(!control_disabled)
                .aria_label(label)
                .child(icon.size(icon_size))
                .when(control_disabled, |el| el.cursor_not_allowed())
                .when(!control_disabled, |el| {
                    el.cursor_pointer().hover(|style| style.text_color(primary))
                })
                .when(focused && !control_disabled, |el| el.text_color(primary));
            if controls_right {
                control = control.right(px(1.)).h(px((height - 2.) / 2.));
                if direction > 0 {
                    control = control
                        .top(px(1.))
                        .border_b_1()
                        .border_color(control_border)
                        .rounded_tr(px(4.));
                } else {
                    control = control
                        .bottom(px(1.))
                        .border_l_1()
                        .border_color(control_border)
                        .rounded_br(px(4.));
                }
            } else {
                control = control.top(px(1.)).bottom(px(1.)).h(px(height - 2.));
                if direction > 0 {
                    control = control
                        .right(px(1.))
                        .border_l_1()
                        .border_color(control_border)
                        .rounded_tr(px(4.))
                        .rounded_br(px(4.));
                } else {
                    control = control
                        .left(px(1.))
                        .border_r_1()
                        .border_color(control_border)
                        .rounded_tl(px(4.))
                        .rounded_bl(px(4.));
                }
            }
            control
                .on_mouse_down(MouseButton::Left, {
                    let state = state.clone();
                    let focus = focus.clone();
                    move |_, window, cx| {
                        if control_disabled {
                            return;
                        }
                        cx.stop_propagation();
                        focus.focus(window, cx);
                        let entity = state.clone();
                        state.update(cx, |state, cx| {
                            state.start_repeat(direction, entity, window, cx);
                        });
                    }
                })
                .on_mouse_up(MouseButton::Left, {
                    let state = state.clone();
                    move |_, _, cx| {
                        cx.stop_propagation();
                        state.update(cx, |state, _| state.stop_repeat());
                    }
                })
                .on_key_up({
                    let state = state.clone();
                    move |event: &KeyUpEvent, window, cx| {
                        if !matches!(event.keystroke.key.as_str(), "enter" | "space") {
                            return;
                        }
                        cx.stop_propagation();
                        state.update(cx, |state, cx| state.step(direction, window, cx));
                    }
                })
        };

        let mut controls_vec = Vec::new();
        if controls {
            let decrease_icon = decrease_icon.unwrap_or_else(|| {
                el_icon(if controls_right {
                    "arrow-down"
                } else {
                    "minus"
                })
            });
            let increase_icon = increase_icon
                .unwrap_or_else(|| el_icon(if controls_right { "arrow-up" } else { "plus" }));
            controls_vec.push(
                control(
                    -1,
                    state.read(cx).decrease_focus.clone(),
                    decrease_focused,
                    decrease_icon,
                    locale.input_number_decrease().into(),
                    ElementId::NamedChild(Arc::new(id.clone()), "decrease".into()),
                    decrease_disabled,
                )
                .into_any_element(),
            );
            controls_vec.push(
                control(
                    1,
                    state.read(cx).increase_focus.clone(),
                    increase_focused,
                    increase_icon,
                    locale.input_number_increase().into(),
                    ElementId::NamedChild(Arc::new(id.clone()), "increase".into()),
                    increase_disabled,
                )
                .into_any_element(),
            );
        }

        div()
            .id(id)
            .relative()
            .w(px(width))
            .h(px(height))
            .child(input)
            .children(controls_vec)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn props() -> NumberProps {
        NumberProps::default()
    }

    #[test]
    fn parses_numeric_prefixes_and_custom_parsers() {
        assert_eq!(parse_number("12.5px", None), Some(12.5));
        assert_eq!(parse_number(" -2e3ignored", None), Some(-2000.));
        assert_eq!(parse_number("1e", None), Some(1.));
        assert_eq!(parse_number("nope", None), None);
        let parser: Parser = Arc::new(|raw| raw.replace(" USD", ""));
        assert_eq!(parse_number("42 USD", Some(&parser)), Some(42.));
    }

    #[test]
    fn verifies_ranges_precision_and_strict_steps() {
        let mut config = props();
        config.min = 0.;
        config.max = 10.;
        config.precision = Some(2);
        assert_eq!(verify_value("12.345", &config), Some(10.));
        assert_eq!(verify_value("-1", &config), Some(0.));
        assert_eq!(verify_value("3.12345", &config), Some(3.12));

        config.step = 0.5;
        config.step_strictly = true;
        assert_eq!(verify_value("3.2", &config), Some(3.));
        config.step = 0.1;
        assert_eq!(verify_number(0.1 + 0.2, &config), Some(0.3));
    }

    #[test]
    fn clear_values_and_inverted_ranges_have_explicit_fallbacks() {
        let mut config = props();
        config.min = 5.;
        config.max = 1.;
        assert_eq!(verify_value("", &config), None);
        config.value_on_clear = Some(InputNumberValueOnClear::Min);
        assert_eq!(verify_value("", &config), Some(5.));
        config.value_on_clear = Some(InputNumberValueOnClear::Max);
        assert_eq!(verify_value("", &config), Some(5.));
        config.value_on_clear = Some(InputNumberValueOnClear::Value(3.));
        assert_eq!(verify_value("", &config), Some(5.));
    }

    #[test]
    fn formats_fixed_precision_and_custom_values() {
        let mut config = props();
        config.precision = Some(2);
        assert_eq!(format_value(Some(3.), &config), "3.00");
        config.formatter = Some(Arc::new(|value| format!("${value}")));
        assert_eq!(format_value(Some(3.), &config), "$3");
        assert_eq!(format_value(None, &config), "");
    }

    #[test]
    fn invalid_steps_and_non_finite_values_fall_back() {
        let number = el_input_number().step(0.).value(Some(f64::NAN));
        assert_eq!(number.step, 1.);
        assert_eq!(number.value, None);
        assert_eq!(sanitize_value(Some(f64::INFINITY)), None);
    }

    #[gpui::test]
    fn drafts_emit_input_and_commit_a_verified_change(cx: &mut gpui::TestAppContext) {
        struct Harness {
            state: gpui::Entity<InputNumberState>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                div()
            }
        }
        type Event = (&'static str, Option<f64>, Option<f64>);
        type Events = Rc<std::cell::RefCell<Vec<Event>>>;
        let events: Events = Rc::new(std::cell::RefCell::new(Vec::new()));
        let (harness, view) = cx.add_window_view({
            let events = events.clone();
            move |_, cx| {
                let state = cx.new(InputNumberState::new);
                state.update(cx, |state, _| {
                    state.props.value = Some(5.);
                    state.props.min = 0.;
                    state.props.max = 10.;
                    state.last_value = Some(5.);
                    state.handlers.oninput = Some(Rc::new({
                        let events = events.clone();
                        move |value, _, _| events.borrow_mut().push(("input", value, None))
                    }));
                    state.handlers.onchange = Some(Rc::new({
                        let events = events.clone();
                        move |new, old, _, _| events.borrow_mut().push(("change", new, old))
                    }));
                });
                Harness { state }
            }
        });
        view.update(|window, cx| {
            let state = harness.read(cx).state.clone();
            state.update(cx, |state, cx| {
                state.accept_input("12.5".into(), window, cx)
            });
            assert_eq!(state.read(cx).draft.as_deref(), Some("12.5"));
            state.update(cx, |state, cx| state.commit("12.5".into(), window, cx));
            assert_eq!(state.read(cx).draft, None);
            assert_eq!(state.read(cx).last_value, Some(10.));
        });
        assert_eq!(
            *events.borrow(),
            vec![
                ("input", Some(12.5), None),
                ("change", Some(10.), Some(12.5)),
            ]
        );
    }

    #[gpui::test]
    fn stepping_stops_at_bounds_and_respects_readonly(cx: &mut gpui::TestAppContext) {
        struct Harness {
            state: gpui::Entity<InputNumberState>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                div()
            }
        }
        let (harness, view) = cx.add_window_view(|_, cx| {
            let state = cx.new(InputNumberState::new);
            state.update(cx, |state, _| {
                state.props.value = Some(9.);
                state.props.min = 0.;
                state.props.max = 10.;
                state.props.step = 2.;
                state.last_value = Some(9.);
            });
            Harness { state }
        });
        view.update(|window, cx| {
            let state = harness.read(cx).state.clone();
            state.update(cx, |state, cx| state.step(1, window, cx));
            assert_eq!(state.read(cx).last_value, Some(10.));
            assert!(!state.update(cx, |state, cx| state.step(1, window, cx)));
            state.update(cx, |state, _| state.props.readonly = true);
            assert!(!state.update(cx, |state, cx| state.step(-1, window, cx)));
        });
    }

    #[gpui::test]
    fn rendered_controls_and_arrow_keys_move_the_controlled_value(cx: &mut gpui::TestAppContext) {
        use std::cell::RefCell;
        struct Harness {
            value: Rc<RefCell<Option<f64>>>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let value = self.value.clone();
                let current = value.borrow().to_owned();
                div().p(px(20.)).child(
                    el_input_number()
                        .id("rendered-number")
                        .value(current)
                        .min(0.)
                        .max(10.)
                        .oninput(move |reported, _, _| *value.borrow_mut() = reported),
                )
            }
        }
        let value = Rc::new(RefCell::new(Some(5.)));
        let (_, view) = cx.add_window_view({
            let value = value.clone();
            move |_, _| Harness { value }
        });
        view.simulate_click(gpui::point(px(154.), px(36.)), gpui::Modifiers::default());
        assert_eq!(*value.borrow(), Some(6.));
        view.simulate_click(gpui::point(px(95.), px(36.)), gpui::Modifiers::default());
        press_key(view, "up");
        assert_eq!(*value.borrow(), Some(7.));
        press_key(view, "down");
        assert_eq!(*value.borrow(), Some(6.));
    }

    fn press_key(view: &mut gpui::VisualTestContext, key: &str) {
        let keystroke = view
            .update(|_, cx| gpui::Keystroke::parse_with_os(key, cx.platform_info().os).unwrap());
        view.simulate_event(gpui::KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        view.simulate_event(gpui::KeyUpEvent { keystroke });
    }
}
