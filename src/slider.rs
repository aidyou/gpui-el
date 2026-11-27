//! Element Plus Slider.
//!
//! Follows `slider.md`, `components/slider/src`, and Theme Chalk `slider.scss`:
//! a 6 px runway with a primary-colored bar, one or two round handles, step
//! quantization, breakpoint stops, marks, a value tooltip on the handles, three
//! outer heights, read-only support, and a vertical orientation. Dragging,
//! runway clicks, and the keyboard (arrows, Page Up/Down, Home, End) all move
//! the value; `oninput` reports the change while a drag is in progress and
//! `onchange` reports the committed value, like upstream's `input` and `change`
//! events.
//!
//! Native differences: there is no DOM, so `show-input` (the embedded
//! `el-input-number`) is not reproduced, and `step` accepts only a number, not
//! the upstream `'mark'` string — snap to marks by passing a `step` that lands
//! on them. GPUI cannot scale an element on hover, so the handle's 1.2× hover
//! and drag enlargement is a 20 → 24 px size change instead, and the value
//! tooltip keeps the text it was built with rather than following a drag frame
//! by frame. The whole slider row (or vertical strip) is an interaction
//! target, and grabbing the handle preserves the pointer's offset from its
//! center so the 36 px hit area does not jump the value. A drag keeps tracking
//! the pointer after it leaves the slider through a window-level move listener,
//! which gpui-fast needs because it only reports an element exit at the window
//! edge.

use std::{cell::Cell as BoundsCell, rc::Rc, sync::Arc};

use gpui::{
    App, Bounds, CursorStyle, ElementId, FocusHandle, Hsla, IntoElement, KeyDownEvent, MouseButton,
    MouseMoveEvent, MouseUpEvent, Pixels, Point, RenderOnce, SharedString, Window, div, prelude::*,
    px, relative, rgb,
};

use crate::{Theme, tokens};

/// Outer height, from `$slider-height`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SliderSize {
    Large,
    #[default]
    Default,
    Small,
}

impl SliderSize {
    fn height(self) -> f32 {
        match self {
            SliderSize::Large => 40.,
            SliderSize::Default => 32.,
            SliderSize::Small => 24.,
        }
    }
}

/// The value reported by a slider: one number, or the low and high ends of a
/// range. Match on it the way the slider was built, with [`Slider::value`] or
/// [`Slider::range`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SliderValue {
    Single(f32),
    Range(f32, f32),
}

/// The handle a drag or a keyboard step moves.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Handle {
    Low,
    High,
}

impl Handle {
    fn index(self) -> usize {
        match self {
            Handle::Low => 0,
            Handle::High => 1,
        }
    }
}

/// An in-progress drag. `offset` is the pointer's distance from the handle
/// center in runway percent, so grabbing the edge of the 36 px handle does not
/// jump the value under the pointer.
#[derive(Clone, Copy, Debug)]
struct Drag {
    handle: Handle,
    offset: f32,
}

/// An in-progress drag, the last value it reported, and the measured runway,
/// all keyed per slider.
struct SliderState {
    drag: Option<Drag>,
    /// The last value sent through `oninput`, reported again by `onchange`
    /// when the drag ends.
    last: Option<f32>,
    /// Runway bounds recorded at paint, for mapping pointer positions to
    /// values.
    bounds: Rc<BoundsCell<Option<Bounds<Pixels>>>>,
    /// One focus handle per end; the low end is the single slider's handle.
    focus: [FocusHandle; 2],
}

impl SliderState {
    fn new(cx: &mut App) -> Self {
        Self {
            drag: None,
            last: None,
            bounds: Rc::new(BoundsCell::new(None)),
            focus: [cx.focus_handle(), cx.focus_handle()],
        }
    }
}

type TooltipFormatter = Arc<dyn Fn(f32) -> SharedString>;
type InputHandler = Arc<dyn Fn(SliderValue, &mut Window, &mut App)>;
type ChangeHandler = InputHandler;

/// One labeled breakpoint, from upstream's `marks` entries.
pub struct SliderMark {
    pub point: f32,
    pub label: SharedString,
}

/// An Element Plus slider.
///
/// Build a single slider with [`Self::value`] or a range with
/// [`Self::range`]; the reported value matches the mode through
/// [`SliderValue`].
#[derive(IntoElement)]
#[must_use]
pub struct Slider {
    id: Option<ElementId>,
    single: Option<f32>,
    range: Option<(f32, f32)>,
    min: f32,
    max: f32,
    step: f32,
    size: SliderSize,
    disabled: bool,
    vertical: bool,
    height: f32,
    show_stops: bool,
    show_tooltip: bool,
    active_color: Option<Hsla>,
    inactive_color: Option<Hsla>,
    format_tooltip: Option<TooltipFormatter>,
    marks: Vec<SliderMark>,
    oninput: Option<InputHandler>,
    onchange: Option<ChangeHandler>,
}

/// Creates a slider, mirroring the other `el_*` constructors.
#[track_caller]
pub fn el_slider() -> Slider {
    Slider::new()
}

impl Default for Slider {
    fn default() -> Self {
        Self {
            id: None,
            single: Some(0.),
            range: None,
            min: 0.,
            max: 100.,
            step: 1.,
            size: SliderSize::Default,
            disabled: false,
            vertical: false,
            height: 200.,
            show_stops: false,
            show_tooltip: true,
            active_color: None,
            inactive_color: None,
            format_tooltip: None,
            marks: Vec::new(),
            oninput: None,
            onchange: None,
        }
    }
}

impl Slider {
    /// Creates a slider defaulting to 0.
    #[track_caller]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a stable element key; required when several sliders share a call
    /// site, since drag and focus state are keyed by it.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the value of a single-handle slider.
    pub fn value(mut self, value: f32) -> Self {
        self.single = Some(value);
        self.range = None;
        self
    }

    /// Switches to a range slider and sets its low and high ends. The handles
    /// may cross; the reported pair is always ordered low to high.
    pub fn range(mut self, low: f32, high: f32) -> Self {
        self.range = Some((low, high));
        self.single = None;
        self
    }

    /// Sets the minimum value; 0 by default.
    pub fn min(mut self, min: f32) -> Self {
        self.min = min;
        self
    }

    /// Sets the maximum value; 100 by default.
    pub fn max(mut self, max: f32) -> Self {
        self.max = max;
        self
    }

    /// Sets the step size, 1 by default. Values snap to steps, with the gap
    /// before the maximum rounding across to it, like upstream.
    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    pub fn size(mut self, size: SliderSize) -> Self {
        self.size = size;
        self
    }
    pub fn large(self) -> Self {
        self.size(SliderSize::Large)
    }
    pub fn small(self) -> Self {
        self.size(SliderSize::Small)
    }

    /// Makes the slider read-only, like upstream's `disabled`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Lays the runway out vertically. Its length comes from [`Self::height`].
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    /// Sets the vertical runway's length in logical pixels, 200 by default.
    /// Horizontal sliders fill their container's width.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Draws a stop at every step boundary, like `show-stops`.
    pub fn show_stops(mut self, show_stops: bool) -> Self {
        self.show_stops = show_stops;
        self
    }

    /// Shows the value in a tooltip on the handles; on by default.
    pub fn show_tooltip(mut self, show_tooltip: bool) -> Self {
        self.show_tooltip = show_tooltip;
        self
    }

    /// Sets the active bar and handle outline color; primary by default.
    /// Disabled sliders keep their neutral disabled colors.
    pub fn active_color(mut self, color: impl Into<Hsla>) -> Self {
        self.active_color = Some(color.into());
        self
    }

    /// Sets the inactive runway color; the theme border color by default.
    /// Disabled sliders keep their neutral disabled colors.
    pub fn inactive_color(mut self, color: impl Into<Hsla>) -> Self {
        self.inactive_color = Some(color.into());
        self
    }

    /// Formats the tooltip value; the raw number by default.
    pub fn format_tooltip(mut self, format: impl Fn(f32) -> SharedString + 'static) -> Self {
        self.format_tooltip = Some(Arc::new(format));
        self
    }

    /// Sets the labeled breakpoints, like `marks`. Points outside
    /// `[min, max]` are dropped.
    pub fn marks(mut self, marks: impl IntoIterator<Item = SliderMark>) -> Self {
        self.marks = marks
            .into_iter()
            .filter(|mark| mark.point >= self.min && mark.point <= self.max)
            .collect();
        self.marks.sort_by(|a, b| {
            a.point
                .partial_cmp(&b.point)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self
    }

    /// Reports the value while a drag or a keyboard step is in progress, like
    /// upstream's `input` event.
    #[track_caller]
    pub fn oninput(
        mut self,
        handler: impl Fn(SliderValue, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.oninput = Some(Arc::new(handler));
        self.ensure_id();
        self
    }

    /// Reports the committed value when a drag ends, a runway click lands, or
    /// a key step finishes, like upstream's `change` event.
    #[track_caller]
    pub fn onchange(
        mut self,
        handler: impl Fn(SliderValue, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onchange = Some(Arc::new(handler));
        self.ensure_id();
        self
    }

    #[track_caller]
    fn ensure_id(&mut self) {
        if self.id.is_none() {
            self.id = Some(ElementId::CodeLocation(*std::panic::Location::caller()));
        }
    }
}

/// The value a handle change reports: the moved handle takes the new value,
/// the other end keeps its own, and a range pair stays ordered low to high,
/// like upstream's `minValue`/`maxValue` emission.
fn report_value(range: Option<(f32, f32)>, handle: Handle, value: f32) -> SliderValue {
    let Some((low, high)) = range else {
        return SliderValue::Single(value);
    };
    let (low, high) = match handle {
        Handle::Low => (value, high),
        Handle::High => (low, value),
    };
    SliderValue::Range(low.min(high), low.max(high))
}

/// The current value of one handle.
fn handle_value(range: Option<(f32, f32)>, single: Option<f32>, handle: Handle) -> Option<f32> {
    match handle {
        Handle::Low => single.or_else(|| range.map(|(low, _)| low)),
        Handle::High => range.map(|(_, high)| high).or(single),
    }
}

/// Where a point sits on the runway, clamped to 0..=100.
fn percent_at(position: Point<Pixels>, bounds: &Bounds<Pixels>, vertical: bool) -> f32 {
    if vertical {
        ((bounds.bottom() - position.y).as_f32() / bounds.size.height.as_f32() * 100.)
            .clamp(0., 100.)
    } else {
        ((position.x - bounds.left()).as_f32() / bounds.size.width.as_f32() * 100.).clamp(0., 100.)
    }
}

/// Fractional digits of the shortest representation, for rounding to the
/// precision of `min`, `max`, and `step`, like upstream.
fn decimals(value: f32) -> u32 {
    format!("{value}")
        .split('.')
        .nth(1)
        .map(|digits| digits.len() as u32)
        .unwrap_or(0)
}

/// Snaps a runway percent to a step value, like upstream's `setPosition`:
/// steps below the last full one round to the nearest step, the gap between
/// the last full step and the maximum rounds across it, and the result keeps
/// the precision of the endpoints.
fn quantize(percent: f32, min: f32, max: f32, step: f32) -> f32 {
    let span = max - min;
    if span <= 0. {
        return min;
    }
    if step <= 0. || step >= span {
        return min + percent / 100. * span;
    }
    let full_steps = (span / step).floor();
    let full_range = full_steps * step / span * 100.;
    let threshold = full_range + (100. - full_range) / 2.;
    let value = if percent < full_range {
        let between = full_range / full_steps;
        min + (percent / between).round() * step
    } else if percent < threshold {
        min + full_steps * step
    } else {
        max
    };
    let precision = decimals(min).max(decimals(max)).max(decimals(step));
    let factor = 10f32.powi(precision as i32);
    (value * factor).round() / factor
}

/// The handle a click at this percent should move: the nearest end of the
/// current bar, like upstream's `getButtonRefByPercent`.
fn handle_at(
    percent: f32,
    min: f32,
    max: f32,
    single: Option<f32>,
    range: Option<(f32, f32)>,
) -> Handle {
    let target = min + percent / 100. * (max - min);
    let Some((low, high)) = range.or_else(|| single.map(|value| (value, value))) else {
        return Handle::Low;
    };
    if (target - low).abs() <= (high - target).abs() {
        Handle::Low
    } else {
        Handle::High
    }
}

/// The value tooltip on a handle, styled like the other hints.
struct SliderHint(SharedString, bool);

impl gpui::Render for SliderHint {
    fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
        div()
            .px(px(10.))
            .py(px(5.))
            .rounded(px(tokens::radius::BASE))
            .text_size(px(12.))
            .bg(rgb(if self.1 { 0xe5eaf3 } else { 0x303133 }))
            .text_color(rgb(if self.1 { 0x141414 } else { 0xffffff }))
            .child(self.0.clone())
    }
}

/// The slider's geometry, from Theme Chalk `$slider-*`.
mod geometry {
    /// `$slider-height`, the runway's thickness.
    pub const RUNWAY: f32 = 6.;
    /// `$slider-border-radius`.
    pub const RADIUS: f32 = 3.;
    /// `$slider-button-size`.
    pub const HANDLE: f32 = 20.;
    /// The upstream 1.2× hover and drag scale, as a size.
    pub const HANDLE_ACTIVE: f32 = 24.;
    /// `$slider-button-wrapper-size`.
    pub const HANDLE_AREA: f32 = 36.;
    /// `$slider-button-wrapper-offset`.
    pub const HANDLE_OFFSET: f32 = 15.;
}

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let single = self.single;
        let range = self.range;
        let min = self.min;
        let max = self.max;
        let disabled = self.disabled;
        let vertical = self.vertical;
        let Slider {
            id,
            step,
            size,
            height,
            show_stops,
            show_tooltip,
            active_color,
            inactive_color,
            format_tooltip,
            marks,
            oninput,
            onchange,
            ..
        } = self;
        let theme = Theme::current(cx);
        let Some(id) = id else {
            return div().into_any_element();
        };
        let state = window.use_keyed_state(id.clone(), cx, |_, cx| SliderState::new(cx));
        let drag = state.read(cx).drag;
        let active_handle = drag.map(|drag| drag.handle);
        let runway_bg: Hsla = if disabled {
            rgb(theme.border(true)).into()
        } else {
            inactive_color.unwrap_or_else(|| rgb(theme.border(true)).into())
        };
        let bar_color: Hsla = if disabled {
            rgb(theme.disabled_text()).into()
        } else {
            active_color.unwrap_or_else(|| rgb(crate::theme::PRIMARY).into())
        };
        let stop = rgb(theme.background());
        let info = rgb(theme.secondary_text());
        let span = max - min;
        let low = range.map(|(low, _)| low).unwrap_or(min);
        let high = range
            .map(|(_, high)| high)
            .unwrap_or_else(|| single.unwrap_or(min));
        let low_pct = if span > 0. {
            (low - min) / span * 100.
        } else {
            0.
        };
        let high_pct = if span > 0. {
            (high - min) / span * 100.
        } else {
            0.
        };

        // The runway measures itself at paint and arms window-level move and
        // up listeners, so a drag keeps tracking the pointer after it leaves
        // the slider. gpui-fast only reports an element exit at the window
        // edge, so without these a drag would stall at the runway's edge.
        let drag_state = state.clone();
        let move_oninput = oninput.clone();
        let release_state = state.clone();
        let release_cfg = (range, min, single);
        let release_change = onchange.clone();
        let bounds_cell = state.read(cx).bounds.clone();
        let runway = div()
            .id(ElementId::NamedChild(Arc::new(id.clone()), "runway".into()))
            .relative()
            .when(vertical, |el| {
                el.w(px(geometry::RUNWAY)).h_full().mx(px(16.))
            })
            .when(!vertical, |el| el.h(px(geometry::RUNWAY)).flex_1())
            .rounded(px(geometry::RADIUS))
            .bg(runway_bg)
            .when(!disabled, |el| el.cursor_pointer())
            .child(
                // The bar spans the selection: the minimum to the value for a
                // single slider, the low end to the high end for a range.
                div()
                    .absolute()
                    .rounded(px(geometry::RADIUS))
                    .bg(bar_color)
                    .when(vertical, |el| {
                        el.left_0()
                            .right_0()
                            .bottom(relative(low_pct / 100.))
                            .h(relative((high_pct - low_pct) / 100.))
                    })
                    .when(!vertical, |el| {
                        el.top_0()
                            .bottom_0()
                            .left(relative(low_pct / 100.))
                            .w(relative((high_pct - low_pct) / 100.))
                    }),
            )
            .children({
                // Stops at every step boundary.
                let stop_count = if show_stops && step > 0. && span > 0. {
                    (span / step).ceil()
                } else {
                    0.
                };
                let step_width = if span > 0. { 100. * step / span } else { 0. };
                (1..stop_count as usize).map(move |index| {
                    let percent = index as f32 * step_width;
                    div()
                        .absolute()
                        .rounded(px(geometry::RUNWAY / 2.))
                        .bg(stop)
                        .when(vertical, |el| {
                            el.left_0()
                                .w(px(geometry::RUNWAY))
                                .h(px(geometry::RUNWAY))
                                .bottom(relative(percent / 100.))
                                .mb(px(-geometry::RUNWAY / 2.))
                        })
                        .when(!vertical, |el| {
                            el.top_0()
                                .w(px(geometry::RUNWAY))
                                .h(px(geometry::RUNWAY))
                                .left(relative(percent / 100.))
                                .ml(px(-geometry::RUNWAY / 2.))
                        })
                })
            })
            .children(marks.iter().map(|mark| {
                let percent = if span > 0. {
                    (mark.point - min) / span * 100.
                } else {
                    0.
                };
                div()
                    .absolute()
                    .rounded(px(geometry::RUNWAY / 2.))
                    .bg(stop)
                    .when(vertical, |el| {
                        el.left_0()
                            .w(px(geometry::RUNWAY))
                            .h(px(geometry::RUNWAY))
                            .bottom(relative(percent / 100.))
                            .mb(px(-geometry::RUNWAY / 2.))
                    })
                    .when(!vertical, |el| {
                        el.top_0()
                            .w(px(geometry::RUNWAY))
                            .h(px(geometry::RUNWAY))
                            .left(relative(percent / 100.))
                            .ml(px(-geometry::RUNWAY / 2.))
                    })
                    .child(
                        div()
                            .absolute()
                            .text_size(px(tokens::font_size::BASE))
                            .text_color(info)
                            .when(vertical, |el| {
                                el.left(px(15.))
                                    .bottom(relative(percent / 100.))
                                    .mb(px(-10.))
                                    .h(px(20.))
                                    .flex()
                                    .items_center()
                            })
                            .when(!vertical, |el| {
                                el.top(px(15.))
                                    .left(relative(percent / 100.))
                                    .ml(px(-24.))
                                    .w(px(48.))
                                    .text_center()
                            })
                            .child(mark.label.clone()),
                    )
            }))
            .children({
                // The handles: one for a single slider, two for a range.
                let handles: Vec<Handle> = match range {
                    Some(_) => vec![Handle::Low, Handle::High],
                    None => vec![Handle::Low],
                };
                handles.into_iter().map(|handle| {
                    let value = handle_value(range, single, handle).unwrap_or(min);
                    let percent = if span > 0. {
                        (value - min) / span * 100.
                    } else {
                        0.
                    };
                    let active = active_handle == Some(handle);
                    let hint: SharedString = match &format_tooltip {
                        Some(format) => format(value),
                        None => SharedString::from(format!("{value}")),
                    };
                    let dark = theme.dark;
                    let key_state = state.clone();
                    let key_oninput = oninput.clone();
                    let key_onchange = onchange.clone();
                    div()
                        .id(ElementId::NamedChild(
                            Arc::new(id.clone()),
                            format!("handle-{}", handle.index()).into(),
                        ))
                        .absolute()
                        .w(px(geometry::HANDLE_AREA))
                        .h(px(geometry::HANDLE_AREA))
                        .when(vertical, |el| {
                            el.left(px(-geometry::HANDLE_OFFSET))
                                .bottom(relative(percent / 100.))
                                .mb(px(-geometry::HANDLE_AREA / 2.))
                        })
                        .when(!vertical, |el| {
                            el.top(px(-geometry::HANDLE_OFFSET))
                                .left(relative(percent / 100.))
                                .ml(px(-geometry::HANDLE_AREA / 2.))
                        })
                        .flex()
                        .items_center()
                        .justify_center()
                        .track_focus({
                            let focus = &state.read(cx).focus;
                            match handle {
                                Handle::Low => &focus[0],
                                Handle::High => &focus[1],
                            }
                        })
                        .tab_stop(true)
                        .when(!disabled, |el| el.cursor(CursorStyle::OpenHand))
                        .on_mouse_down(MouseButton::Left, {
                            let state = state.clone();
                            move |event, window, cx| {
                                if disabled {
                                    return;
                                }
                                let Some(bounds) = state.read(cx).bounds.get() else {
                                    return;
                                };
                                // Preserve the grab point: the 36 px handle is
                                // much larger than the 6 px runway, and its
                                // outer area must drag without jumping.
                                let offset =
                                    percent_at(event.position, &bounds, vertical) - percent;
                                state.update(cx, |state, cx| {
                                    state.drag = Some(Drag { handle, offset });
                                    state.last = Some(value);
                                    let focus = match handle {
                                        Handle::Low => &state.focus[0],
                                        Handle::High => &state.focus[1],
                                    };
                                    focus.focus(window, cx);
                                    cx.notify();
                                });
                                cx.stop_propagation();
                            }
                        })
                        .when(show_tooltip && !disabled, |el| {
                            el.tooltip(move |_, cx| {
                                cx.new(|_| SliderHint(hint.clone(), dark)).into()
                            })
                        })
                        .on_key_down(move |event: &KeyDownEvent, window, cx| {
                            if disabled {
                                return;
                            }
                            let next = match event.keystroke.key.as_str() {
                                "left" | "down" => Some(value - step),
                                "right" | "up" => Some(value + step),
                                "pagedown" => Some(value - step * 4.),
                                "pageup" => Some(value + step * 4.),
                                "home" => Some(min),
                                "end" => Some(max),
                                _ => None,
                            };
                            let Some(next) = next.map(|next| next.clamp(min, max)) else {
                                return;
                            };
                            key_state.update(cx, |state, cx| {
                                state.drag = Some(Drag { handle, offset: 0. });
                                state.last = Some(next);
                                cx.notify();
                            });
                            if let Some(onchange) = &key_onchange {
                                onchange(report_value(range, handle, next), window, cx);
                            }
                            if let Some(oninput) = &key_oninput {
                                oninput(report_value(range, handle, next), window, cx);
                            }
                            cx.stop_propagation();
                        })
                        .child(
                            div()
                                .rounded(px(geometry::HANDLE / 2.))
                                .border_2()
                                .border_color(bar_color)
                                .bg(stop)
                                .w(px(geometry::HANDLE))
                                .h(px(geometry::HANDLE))
                                // GPUI cannot scale on hover; the upstream
                                // 1.2× hover and drag enlargement is a size
                                // change instead.
                                .when(!disabled, |el| {
                                    el.hover(|style| {
                                        style
                                            .w(px(geometry::HANDLE_ACTIVE))
                                            .h(px(geometry::HANDLE_ACTIVE))
                                    })
                                    .when(active, |el| {
                                        el.w(px(geometry::HANDLE_ACTIVE))
                                            .h(px(geometry::HANDLE_ACTIVE))
                                    })
                                }),
                        )
                })
            })
            .child(
                // Measures the runway and arms the window-level drag listeners.
                gpui::canvas(
                    |_, _, _| {},
                    move |bounds, _, window, _| {
                        bounds_cell.set(Some(bounds));
                        let state = drag_state.clone();
                        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                            if phase != gpui::DispatchPhase::Bubble || disabled {
                                return;
                            }
                            let Some(drag) = state.read(cx).drag else {
                                return;
                            };
                            let Some(bounds) = state.read(cx).bounds.get() else {
                                return;
                            };
                            let value = quantize(
                                percent_at(event.position, &bounds, vertical) - drag.offset,
                                min,
                                max,
                                step,
                            );
                            state.update(cx, |state, cx| {
                                state.last = Some(value);
                                cx.notify();
                            });
                            if let Some(oninput) = &move_oninput {
                                oninput(report_value(range, drag.handle, value), window, cx);
                            }
                        });
                        let state = release_state.clone();
                        let (range, min, single) = release_cfg;
                        window.on_mouse_event(move |_: &MouseUpEvent, phase, window, cx| {
                            if phase != gpui::DispatchPhase::Bubble || disabled {
                                return;
                            }
                            let Some(drag) = state.read(cx).drag else {
                                return;
                            };
                            // The drag's last reported value, or the
                            // handle's current value when the press never
                            // moved.
                            let value = state.read(cx).last.unwrap_or_else(|| {
                                handle_value(range, single, drag.handle).unwrap_or(min)
                            });
                            state.update(cx, |state, cx| {
                                state.drag = None;
                                cx.notify();
                            });
                            if let Some(onchange) = &release_change {
                                onchange(report_value(range, drag.handle, value), window, cx);
                            }
                        });
                    },
                )
                .absolute()
                .inset_0(),
            );

        div()
            .when(vertical, |el| {
                el.flex()
                    .flex_col()
                    .items_center()
                    .h(px(height))
                    .w(px(geometry::RUNWAY + 32.))
            })
            .when(!vertical, |el| {
                el.flex().items_center().h(px(size.height())).w_full()
            })
            .when(!disabled, |el| el.cursor_pointer())
            .on_mouse_down(MouseButton::Left, {
                let state = state.clone();
                let oninput = oninput.clone();
                move |event, window, cx| {
                    if disabled {
                        return;
                    }
                    let Some(bounds) = state.read(cx).bounds.get() else {
                        return;
                    };
                    // The whole row/strip is an interaction target, not just
                    // the 6 px visual runway. This keeps horizontal sliders
                    // easy to grab and gives the vertical slider a usable
                    // width while preserving the upstream geometry.
                    let percent = percent_at(event.position, &bounds, vertical);
                    let handle = handle_at(percent, min, max, single, range);
                    let value = quantize(percent, min, max, step);
                    state.update(cx, |state, cx| {
                        state.drag = Some(Drag { handle, offset: 0. });
                        state.last = Some(value);
                        let focus = match handle {
                            Handle::Low => &state.focus[0],
                            Handle::High => &state.focus[1],
                        };
                        focus.focus(window, cx);
                        cx.notify();
                    });
                    if let Some(oninput) = &oninput {
                        oninput(report_value(range, handle, value), window, cx);
                    }
                    cx.stop_propagation();
                }
            })
            .child(runway)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_map_to_upstream_heights() {
        assert_eq!(SliderSize::Large.height(), 40.);
        assert_eq!(SliderSize::Default.height(), 32.);
        assert_eq!(SliderSize::Small.height(), 24.);
    }

    #[test]
    fn color_settings_do_not_reset_other_dimensions() {
        let widget = el_slider()
            .value(42.)
            .small()
            .active_color(rgb(0x13ce66))
            .inactive_color(rgb(0xff4949));
        assert_eq!(widget.single, Some(42.));
        assert_eq!(widget.size, SliderSize::Small);
        assert_eq!(widget.active_color, Some(rgb(0x13ce66).into()));
        assert_eq!(widget.inactive_color, Some(rgb(0xff4949).into()));
        let overridden = el_slider()
            .active_color(rgb(0x13ce66))
            .inactive_color(rgb(0xff4949))
            .active_color(rgb(0x626aef))
            .inactive_color(rgb(0x909399));
        assert_eq!(overridden.active_color, Some(rgb(0x626aef).into()));
        assert_eq!(overridden.inactive_color, Some(rgb(0x909399).into()));
    }

    #[test]
    fn quantize_snaps_to_steps_and_the_maximum() {
        // Steps of 10 over 0..100: 14 rounds down to 10, 16 up to 20.
        assert_eq!(quantize(14., 0., 100., 10.), 10.);
        assert_eq!(quantize(16., 0., 100., 10.), 20.);
        // The last partial step rounds across to the maximum.
        assert_eq!(quantize(96., 0., 100., 30.), 100.);
        assert_eq!(quantize(89., 0., 100., 30.), 90.);
        // Precision follows the endpoints.
        assert_eq!(quantize(14., 0., 1., 0.25), 0.25);
        assert_eq!(quantize(60., 0., 1., 0.25), 0.5);
        // Degenerate ranges and steps fall back to the endpoints.
        assert_eq!(quantize(50., 5., 5., 1.), 5.);
        assert_eq!(quantize(50., 0., 100., 0.), 50.);
    }

    #[test]
    fn handle_at_picks_the_nearest_end() {
        assert_eq!(
            handle_at(10., 0., 100., Some(30.), None),
            Handle::Low,
            "a single slider always moves its one handle"
        );
        assert_eq!(
            handle_at(10., 0., 100., None, Some((20., 60.))),
            Handle::Low
        );
        assert_eq!(
            handle_at(80., 0., 100., None, Some((20., 60.))),
            Handle::High
        );
        assert_eq!(
            handle_at(40., 0., 100., None, Some((20., 60.))),
            Handle::Low,
            "a tie goes to the low end"
        );
    }

    #[test]
    fn reports_keep_the_range_ordered() {
        assert_eq!(
            report_value(Some((20., 60.)), Handle::Low, 45.),
            SliderValue::Range(45., 60.)
        );
        // Handles may cross: dragging the low end past the high end swaps the
        // reported pair.
        assert_eq!(
            report_value(Some((20., 60.)), Handle::Low, 80.),
            SliderValue::Range(60., 80.)
        );
        assert_eq!(
            report_value(None, Handle::Low, 42.),
            SliderValue::Single(42.)
        );
    }

    #[test]
    fn marks_drop_out_of_range_points_and_sort() {
        let slider = el_slider().min(0.).max(100.).marks([
            SliderMark {
                point: 75.,
                label: "75".into(),
            },
            SliderMark {
                point: -10.,
                label: "negative".into(),
            },
            SliderMark {
                point: 25.,
                label: "25".into(),
            },
            SliderMark {
                point: 150.,
                label: "too far".into(),
            },
        ]);
        assert_eq!(
            slider
                .marks
                .iter()
                .map(|mark| mark.point)
                .collect::<Vec<_>>(),
            [25., 75.]
        );
    }

    #[gpui::test]
    fn dragging_and_clicking_the_runway_report_values(cx: &mut gpui::TestAppContext) {
        use std::cell::Cell as StdCell;
        struct Harness {
            value: Rc<StdCell<f32>>,
            committed: Rc<StdCell<f32>>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let value = self.value.clone();
                let committed = self.committed.clone();
                div().id("slider-harness").w(px(400.)).child(
                    el_slider()
                        .id("drag-slider")
                        .value(value.get())
                        .oninput(move |reported, _, _| {
                            let SliderValue::Single(reported) = reported else {
                                return;
                            };
                            value.set(reported);
                        })
                        .onchange(move |reported, _, _| {
                            let SliderValue::Single(reported) = reported else {
                                return;
                            };
                            committed.set(reported);
                        }),
                )
            }
        }
        let value = Rc::new(StdCell::new(0.));
        let committed = Rc::new(StdCell::new(0.));
        let (_, view) = cx.add_window_view({
            let value = value.clone();
            let committed = committed.clone();
            move |_, _| Harness { value, committed }
        });
        let draw = |view: &mut gpui::VisualTestContext| {
            view.update(|window, cx| window.draw(cx).clear(cx));
        };
        draw(view);
        // The whole 32 px row is interactive, not only the 6 px runway. This
        // press is above the runway and outside the handle wrapper.
        view.simulate_mouse_down(
            gpui::point(px(20.), px(2.)),
            MouseButton::Left,
            gpui::Modifiers::default(),
        );
        assert_eq!(value.get(), 5.);
        view.simulate_mouse_up(
            gpui::point(px(20.), px(2.)),
            MouseButton::Left,
            gpui::Modifiers::default(),
        );
        // The runway fills the 400 px harness, so 60 % of it is x 240. A
        // click there jumps, and releasing commits.
        view.simulate_mouse_down(
            gpui::point(px(240.), px(16.)),
            MouseButton::Left,
            gpui::Modifiers::default(),
        );
        assert_eq!(value.get(), 60.);
        view.simulate_mouse_up(
            gpui::point(px(240.), px(16.)),
            MouseButton::Left,
            gpui::Modifiers::default(),
        );
        assert_eq!(committed.get(), 60.);
        // A drag past the runway's edge still tracks: down at 50 %, move beyond
        // the slider to the right, then up.
        view.simulate_mouse_down(
            gpui::point(px(200.), px(16.)),
            MouseButton::Left,
            gpui::Modifiers::default(),
        );
        view.simulate_mouse_move(
            gpui::point(px(500.), px(16.)),
            Some(MouseButton::Left),
            gpui::Modifiers::default(),
        );
        draw(view);
        view.simulate_mouse_up(
            gpui::point(px(500.), px(16.)),
            MouseButton::Left,
            gpui::Modifiers::default(),
        );
        assert_eq!(value.get(), 100., "the drag clamps at the maximum");
        assert_eq!(committed.get(), 100.);
    }

    #[gpui::test]
    fn handle_wrapper_drags_without_jumping_in_both_orientations(cx: &mut gpui::TestAppContext) {
        use std::cell::Cell as StdCell;
        struct Harness {
            horizontal: Rc<StdCell<f32>>,
            vertical: Rc<StdCell<f32>>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let horizontal = self.horizontal.clone();
                let vertical = self.vertical.clone();
                div()
                    .flex()
                    .flex_col()
                    .child(
                        // The horizontal handle is centered at x 120. Its wrapper
                        // reaches x 102..138 and y -2..34, so x 110/y 4 is firmly
                        // inside the handle but outside the 6 px runway.
                        div().w(px(400.)).child(
                            el_slider()
                                .id("horizontal-handle")
                                .value(horizontal.get())
                                .oninput(move |reported, _, _| {
                                    let SliderValue::Single(reported) = reported else {
                                        return;
                                    };
                                    horizontal.set(reported);
                                }),
                        ),
                    )
                    .child(
                        // The vertical runway is x 16..22 and starts at y 32. Its
                        // 40 % handle wrapper reaches x 1..37 and y 134..170, so
                        // x 30/y 144 is also outside the runway.
                        div().w(px(400.)).h(px(200.)).child(
                            el_slider()
                                .id("vertical-handle")
                                .value(vertical.get())
                                .vertical(true)
                                .height(200.)
                                .oninput(move |reported, _, _| {
                                    let SliderValue::Single(reported) = reported else {
                                        return;
                                    };
                                    vertical.set(reported);
                                }),
                        ),
                    )
            }
        }
        let horizontal = Rc::new(StdCell::new(30.));
        let vertical = Rc::new(StdCell::new(40.));
        let (_, view) = cx.add_window_view({
            let horizontal = horizontal.clone();
            let vertical = vertical.clone();
            move |_, _| Harness {
                horizontal,
                vertical,
            }
        });
        let draw = |view: &mut gpui::VisualTestContext| {
            view.update(|window, cx| window.draw(cx).clear(cx));
        };
        draw(view);

        view.simulate_mouse_down(
            gpui::point(px(110.), px(4.)),
            MouseButton::Left,
            gpui::Modifiers::default(),
        );
        assert_eq!(horizontal.get(), 30., "grabbing the handle must not jump");
        view.simulate_mouse_move(
            gpui::point(px(210.), px(4.)),
            Some(MouseButton::Left),
            gpui::Modifiers::default(),
        );
        draw(view);
        view.simulate_mouse_up(
            gpui::point(px(210.), px(4.)),
            MouseButton::Left,
            gpui::Modifiers::default(),
        );
        assert_eq!(horizontal.get(), 55., "the grab offset is preserved");

        // The vertical slider starts below the 32 px horizontal row.
        view.simulate_mouse_down(
            gpui::point(px(30.), px(144.)),
            MouseButton::Left,
            gpui::Modifiers::default(),
        );
        assert_eq!(vertical.get(), 40., "vertical handle grabs must not jump");
        view.simulate_mouse_move(
            gpui::point(px(30.), px(94.)),
            Some(MouseButton::Left),
            gpui::Modifiers::default(),
        );
        draw(view);
        view.simulate_mouse_up(
            gpui::point(px(30.), px(94.)),
            MouseButton::Left,
            gpui::Modifiers::default(),
        );
        assert_eq!(
            vertical.get(),
            65.,
            "vertical dragging uses the same offset"
        );
    }

    #[gpui::test]
    fn keyboard_steps_report_the_value(cx: &mut gpui::TestAppContext) {
        use std::cell::Cell as StdCell;
        struct Harness(Rc<StdCell<f32>>);
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let value = self.0.clone();
                div().id("key-harness").w(px(400.)).child(
                    el_slider()
                        .id("key-slider")
                        .value(value.get())
                        .step(10.)
                        .oninput(move |reported, _, _| {
                            let SliderValue::Single(reported) = reported else {
                                return;
                            };
                            value.set(reported);
                        }),
                )
            }
        }
        let value = Rc::new(StdCell::new(20.));
        let (_, view) = cx.add_window_view({
            let value = value.clone();
            move |_, _| Harness(value)
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        // The handle sits at 20 % of the 400 px harness; press it to focus,
        // then step with the keyboard.
        view.simulate_mouse_down(
            gpui::point(px(80.), px(16.)),
            MouseButton::Left,
            gpui::Modifiers::default(),
        );
        view.simulate_mouse_up(
            gpui::point(px(80.), px(16.)),
            MouseButton::Left,
            gpui::Modifiers::default(),
        );
        let keystroke = view.update(|_, cx| {
            gpui::Keystroke::parse_with_os("right", cx.platform_info().os).unwrap()
        });
        view.simulate_event(gpui::KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        view.simulate_event(gpui::KeyUpEvent { keystroke });
        assert_eq!(value.get(), 30., "a step right advances by the step size");
    }
}
