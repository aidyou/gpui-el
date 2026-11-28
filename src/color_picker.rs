//! Element Plus ColorPicker.
//!
//! Follows `color-picker.vue`, `color-picker.ts`, `color-picker.scss`, and
//! `color-picker-panel.scss`: a square swatch, saturation/value panel, hue and
//! alpha sliders, predefined colors, and clear/confirm actions. The panel uses
//! HSV directly, which is appropriate for a simple screen color picker.
//!
//! Native differences: custom popper configuration and imperative exposes are
//! not mapped; dragging is tracked while the pointer remains over the control.

use std::{rc::Rc, sync::Arc};

use gpui::{
    AnyElement, App, Bounds, ElementId, FocusHandle, IntoElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, Pixels, RenderOnce, SharedString, Subscription, Window, anchored, deferred,
    div, point, prelude::*, px, rgb,
};

use crate::{InputSize, Locale, Theme, el_button, el_icon, el_input};

/// Output format for committed ColorPicker values.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ColorFormat {
    #[default]
    Hex,
    Rgb,
    Rgba,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    const PRIMARY: Self = Self {
        r: 64. / 255.,
        g: 158. / 255.,
        b: 1.,
        a: 1.,
    };

    fn to_hsv(self) -> (f32, f32, f32) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let delta = max - min;
        let mut hue = if delta == 0. {
            0.
        } else if max == self.r {
            60. * (((self.g - self.b) / delta).rem_euclid(6.))
        } else if max == self.g {
            60. * (((self.b - self.r) / delta) + 2.)
        } else {
            60. * (((self.r - self.g) / delta) + 4.)
        };
        if hue < 0. {
            hue += 360.;
        }
        let saturation = if max == 0. { 0. } else { delta / max };
        (hue / 360., saturation, max)
    }

    fn from_hsv(hue: f32, saturation: f32, value: f32, alpha: f32) -> Self {
        let hue = hue.rem_euclid(1.) * 6.;
        let sector = hue.floor() as i32;
        let fraction = hue - sector as f32;
        let p = value * (1. - saturation);
        let q = value * (1. - fraction * saturation);
        let t = value * (1. - (1. - fraction) * saturation);
        let (r, g, b) = match sector.rem_euclid(6) {
            0 => (value, t, p),
            1 => (q, value, p),
            2 => (p, value, t),
            3 => (p, q, value),
            4 => (t, p, value),
            _ => (value, p, q),
        };
        Self { r, g, b, a: alpha }
    }

    fn parse(value: &str) -> Option<Self> {
        let value = value.trim();
        if let Some(hex) = value.strip_prefix('#') {
            let digits = hex.len();
            if !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
                return None;
            }
            let channel = |start: usize, end: usize| {
                u8::from_str_radix(hex.get(start..end)?, 16)
                    .ok()
                    .map(|v| v as f32 / 255.)
            };
            let nibble = |index: usize| {
                u8::from_str_radix(hex.get(index..index + 1)?, 16)
                    .ok()
                    .map(|value| value * 17)
            };
            return match digits {
                3 => Some(Self {
                    r: nibble(0)? as f32 / 255.,
                    g: nibble(1)? as f32 / 255.,
                    b: nibble(2)? as f32 / 255.,
                    a: 1.,
                }),
                4 => Some(Self {
                    r: nibble(0)? as f32 / 255.,
                    g: nibble(1)? as f32 / 255.,
                    b: nibble(2)? as f32 / 255.,
                    a: nibble(3)? as f32 / 255.,
                }),
                6 => Some(Self {
                    r: channel(0, 2)?,
                    g: channel(2, 4)?,
                    b: channel(4, 6)?,
                    a: 1.,
                }),
                8 => Some(Self {
                    r: channel(0, 2)?,
                    g: channel(2, 4)?,
                    b: channel(4, 6)?,
                    a: channel(6, 8)?,
                }),
                _ => None,
            };
        }
        let inner = value
            .strip_prefix("rgb(")
            .or_else(|| value.strip_prefix("rgba("))
            .and_then(|value| value.strip_suffix(')'))?;
        let mut parts = inner.split(',').map(str::trim);
        let channel = |part: Option<&str>| -> Option<f32> {
            let part = part?;
            let number = part.split('/').next()?.trim_end_matches('%');
            let value = number.parse::<f32>().ok()?;
            Some(
                if part.ends_with('%') {
                    value / 100.
                } else {
                    value / 255.
                }
                .clamp(0., 1.),
            )
        };
        Some(Self {
            r: channel(parts.next())?,
            g: channel(parts.next())?,
            b: channel(parts.next())?,
            a: parts
                .next()
                .and_then(|part| {
                    let part = part.split('/').nth(1).unwrap_or(part);
                    part.trim_end_matches('%').parse::<f32>().ok().map(|value| {
                        if part.ends_with('%') {
                            value / 100.
                        } else {
                            value
                        }
                    })
                })
                .unwrap_or(1.)
                .clamp(0., 1.),
        })
    }

    fn format(self, format: ColorFormat, show_alpha: bool) -> SharedString {
        let r = (self.r.clamp(0., 1.) * 255.).round() as u8;
        let g = (self.g.clamp(0., 1.) * 255.).round() as u8;
        let b = (self.b.clamp(0., 1.) * 255.).round() as u8;
        let a = (self.a.clamp(0., 1.) * 255.).round() as u8;
        match format {
            ColorFormat::Hex if show_alpha || self.a < 1. => {
                format!("#{r:02x}{g:02x}{b:02x}{a:02x}").into()
            }
            ColorFormat::Hex => format!("#{r:02x}{g:02x}{b:02x}").into(),
            ColorFormat::Rgb => format!("rgb({r}, {g}, {b})").into(),
            ColorFormat::Rgba => format!("rgba({r}, {g}, {b}, {:.3})", self.a).into(),
        }
    }

    fn hsla(self) -> gpui::Hsla {
        let r = (self.r.clamp(0., 1.) * 255.).round() as u32;
        let g = (self.g.clamp(0., 1.) * 255.).round() as u32;
        let b = (self.b.clamp(0., 1.) * 255.).round() as u32;
        gpui::Hsla::from(rgb((r << 16) | (g << 8) | b)).opacity(self.a.clamp(0., 1.))
    }
}

type ChangeHandler = Rc<dyn Fn(Option<SharedString>, &mut Window, &mut App)>;
type FocusHandler = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Clone)]
struct ColorPickerProps {
    value: Option<SharedString>,
    format: ColorFormat,
    show_alpha: bool,
    disabled: bool,
}

#[derive(Clone, Default)]
struct ColorPickerHandlers {
    onchange: Option<ChangeHandler>,
    onactivechange: Option<ChangeHandler>,
    onclear: Option<FocusHandler>,
    onfocus: Option<FocusHandler>,
    onblur: Option<FocusHandler>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum DragTarget {
    SaturationValue,
    Hue,
    Alpha,
}

struct ColorPickerState {
    color: Color,
    open: bool,
    focused: bool,
    drag: Option<DragTarget>,
    focus: FocusHandle,
    bounds: Option<Bounds<Pixels>>,
    popup_bounds: Option<Bounds<Pixels>>,
    sv_bounds: Option<Bounds<Pixels>>,
    hue_bounds: Option<Bounds<Pixels>>,
    alpha_bounds: Option<Bounds<Pixels>>,
    props: ColorPickerProps,
    handlers: ColorPickerHandlers,
    last_value: Option<SharedString>,
    _subscriptions: Vec<Subscription>,
}

impl ColorPickerState {
    fn new(window: &mut Window, cx: &mut gpui::Context<Self>) -> Self {
        let focus = cx.focus_handle();
        let state = cx.entity();
        let focus_in = window.on_focus_in(&focus, cx, {
            let state = state.clone();
            move |window, cx| {
                state.update(cx, |state, cx| {
                    state.focused = true;
                    let handler = state.handlers.onfocus.clone();
                    if let Some(handler) = handler {
                        handler(window, cx);
                    }
                });
            }
        });
        let focus_out = window.on_focus_out(&focus, cx, {
            let state = state.clone();
            move |_event, window, cx| {
                state.update(cx, |state, cx| {
                    state.focused = false;
                    let handler = state.handlers.onblur.clone();
                    if let Some(handler) = handler {
                        handler(window, cx);
                    }
                });
            }
        });
        Self {
            color: Color::PRIMARY,
            open: false,
            focused: false,
            drag: None,
            focus,
            bounds: None,
            popup_bounds: None,
            sv_bounds: None,
            hue_bounds: None,
            alpha_bounds: None,
            props: ColorPickerProps {
                value: None,
                format: ColorFormat::Hex,
                show_alpha: false,
                disabled: false,
            },
            handlers: ColorPickerHandlers::default(),
            last_value: None,
            _subscriptions: vec![focus_in, focus_out],
        }
    }

    fn sync_value(&mut self, value: Option<SharedString>) {
        if self.last_value == value {
            return;
        }
        self.last_value = value.clone();
        if !self.open {
            self.color = value
                .as_deref()
                .and_then(Color::parse)
                .unwrap_or(Color::PRIMARY);
        }
    }

    fn set_open(&mut self, open: bool, window: &mut Window, cx: &mut App) {
        if self.props.disabled || self.open == open {
            return;
        }
        self.open = open;
        self.drag = None;
        self.color = self
            .props
            .value
            .as_deref()
            .and_then(Color::parse)
            .unwrap_or(Color::PRIMARY);
        let _ = window;
        let _ = cx;
    }

    fn update_color(&mut self, color: Color, window: &mut Window, cx: &mut App) {
        if self.props.disabled {
            return;
        }
        self.color = color;
        if let Some(handler) = self.handlers.onactivechange.clone() {
            handler(
                Some(color.format(self.props.format, self.props.show_alpha)),
                window,
                cx,
            );
        }
    }

    fn update_sv(&mut self, position: gpui::Point<Pixels>) {
        let Some(bounds) = self.sv_bounds else {
            return;
        };
        let saturation = ((position.x - bounds.left()) / bounds.size.width).clamp(0., 1.);
        let value = ((bounds.bottom() - position.y) / bounds.size.height).clamp(0., 1.);
        let (hue, _, _) = self.color.to_hsv();
        self.color = Color::from_hsv(hue, saturation, value, self.color.a);
    }

    fn update_hue(&mut self, position: gpui::Point<Pixels>) {
        let Some(bounds) = self.hue_bounds else {
            return;
        };
        let hue = ((position.x - bounds.left()) / bounds.size.width).clamp(0., 1.);
        let (_, saturation, value) = self.color.to_hsv();
        self.color = Color::from_hsv(hue, saturation, value, self.color.a);
    }

    fn update_alpha(&mut self, position: gpui::Point<Pixels>) {
        let Some(bounds) = self.alpha_bounds else {
            return;
        };
        self.color.a = ((position.x - bounds.left()) / bounds.size.width).clamp(0., 1.);
    }

    fn drag_to(
        &mut self,
        target: DragTarget,
        position: gpui::Point<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) {
        match target {
            DragTarget::SaturationValue => self.update_sv(position),
            DragTarget::Hue => self.update_hue(position),
            DragTarget::Alpha => self.update_alpha(position),
        }
        self.update_color(self.color, window, cx);
    }

    fn confirm(&mut self, window: &mut Window, cx: &mut App) {
        if self.props.disabled {
            return;
        }
        if let Some(handler) = self.handlers.onchange.clone() {
            handler(
                Some(self.color.format(self.props.format, self.props.show_alpha)),
                window,
                cx,
            );
        }
        self.set_open(false, window, cx);
    }

    fn clear(&mut self, window: &mut Window, cx: &mut App) {
        if self.props.disabled {
            return;
        }
        if let Some(handler) = self.handlers.onchange.clone() {
            handler(None, window, cx);
        }
        if let Some(handler) = self.handlers.onclear.clone() {
            handler(window, cx);
        }
        self.set_open(false, window, cx);
    }

    fn key(&mut self, key: &str, window: &mut Window, cx: &mut App) -> bool {
        if self.props.disabled {
            return false;
        }
        match key {
            "enter" | "space" => {
                self.set_open(!self.open, window, cx);
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

/// An Element Plus color picker.
#[derive(IntoElement)]
#[must_use]
pub struct ColorPicker {
    id: ElementId,
    value: Option<SharedString>,
    format: ColorFormat,
    show_alpha: bool,
    size: InputSize,
    disabled: bool,
    clearable: bool,
    predefine: Vec<SharedString>,
    accessibility_label: Option<SharedString>,
    onchange: Option<ChangeHandler>,
    onactivechange: Option<ChangeHandler>,
    onclear: Option<FocusHandler>,
    onfocus: Option<FocusHandler>,
    onblur: Option<FocusHandler>,
}

/// Creates a color picker.
#[track_caller]
pub fn el_color_picker() -> ColorPicker {
    ColorPicker::new()
}

impl Default for ColorPicker {
    fn default() -> Self {
        Self {
            id: ElementId::CodeLocation(*std::panic::Location::caller()),
            value: None,
            format: ColorFormat::Hex,
            show_alpha: false,
            size: InputSize::Default,
            disabled: false,
            clearable: true,
            predefine: Vec::new(),
            accessibility_label: None,
            onchange: None,
            onactivechange: None,
            onclear: None,
            onfocus: None,
            onblur: None,
        }
    }
}

impl ColorPicker {
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

    pub fn format(mut self, format: ColorFormat) -> Self {
        self.format = format;
        self
    }

    pub fn show_alpha(mut self, show: bool) -> Self {
        self.show_alpha = show;
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

    pub fn clearable(mut self, clearable: bool) -> Self {
        self.clearable = clearable;
        self
    }

    pub fn predefine(mut self, colors: impl IntoIterator<Item = SharedString>) -> Self {
        self.predefine = colors.into_iter().collect();
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
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
    pub fn onactivechange(
        mut self,
        handler: impl Fn(Option<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onactivechange = Some(Rc::new(handler));
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

fn checkerboard() -> AnyElement {
    gpui::canvas(
        |_: Bounds<Pixels>, _: &mut Window, _: &mut App| {},
        move |bounds: Bounds<Pixels>, _: (), window: &mut Window, _: &mut App| {
            let cell = 6.;
            let columns = (bounds.size.width.as_f32() / cell).ceil() as i32;
            let rows = (bounds.size.height.as_f32() / cell).ceil() as i32;
            for row in 0..rows {
                for column in 0..columns {
                    let color = if (row + column) % 2 == 0 {
                        rgb(0xcccccc)
                    } else {
                        rgb(0x333333)
                    };
                    let origin = point(
                        bounds.left() + px(column as f32 * cell),
                        bounds.top() + px(row as f32 * cell),
                    );
                    let size = gpui::size(
                        px(cell.min((bounds.right() - origin.x).as_f32())),
                        px(cell.min((bounds.bottom() - origin.y).as_f32())),
                    );
                    window.paint_quad(gpui::fill(gpui::Bounds::new(origin, size), color));
                }
            }
        },
    )
    .absolute()
    .inset_0()
    .into_any_element()
}

fn measure<F>(state: gpui::Entity<ColorPickerState>, set: F) -> AnyElement
where
    F: Fn(&mut ColorPickerState, Bounds<Pixels>) + 'static,
{
    gpui::canvas(
        move |bounds: Bounds<Pixels>, _, cx: &mut App| {
            state.update(cx, |state, _| set(state, bounds));
        },
        |_, _, _, _| {},
    )
    .absolute()
    .inset_0()
    .into_any_element()
}

fn measure_control<F>(state: gpui::Entity<ColorPickerState>, height: f32, set: F) -> AnyElement
where
    F: Fn(&mut ColorPickerState, Bounds<Pixels>) + 'static,
{
    gpui::canvas(
        move |bounds: Bounds<Pixels>, _, cx: &mut App| {
            let bounds =
                gpui::Bounds::new(bounds.origin, gpui::size(bounds.size.width, px(height)));
            state.update(cx, |state, _| set(state, bounds));
        },
        |_, _, _, _| {},
    )
    .w_full()
    .h(px(0.))
    .into_any_element()
}

fn paint_hue_gradient(bounds: Bounds<Pixels>, window: &mut Window) {
    let segments = f32::from(bounds.size.width).max(1.).ceil() as usize;
    for index in 0..segments {
        let left = bounds.left() + px(index as f32);
        let right = bounds.left() + px((index + 1) as f32);
        let hue = index as f32 / segments as f32;
        window.paint_quad(gpui::fill(
            gpui::Bounds::new(
                gpui::point(left, bounds.top()),
                gpui::size(right - left, bounds.size.height),
            ),
            Color::from_hsv(hue, 1., 1., 1.).hsla(),
        ));
    }
}

impl RenderOnce for ColorPicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let ColorPicker {
            id,
            value,
            format,
            show_alpha,
            size,
            disabled,
            clearable,
            predefine,
            accessibility_label,
            onchange,
            onactivechange,
            onclear,
            onfocus,
            onblur,
        } = self;
        let props = ColorPickerProps {
            value: value.clone(),
            format,
            show_alpha,
            disabled,
        };
        let handlers = ColorPickerHandlers {
            onchange,
            onactivechange,
            onclear,
            onfocus,
            onblur,
        };
        let state = window.use_keyed_state(id.clone(), cx, |window, cx| {
            ColorPickerState::new(window, cx)
        });
        state.update(cx, |state, _| {
            state.props = props.clone();
            state.handlers = handlers;
        });
        state.update(cx, |state, _| state.sync_value(value.clone()));
        let open = state.read(cx).open && !disabled;
        let color = state.read(cx).color;
        let theme = Theme::current(cx);
        let locale = Locale::current(cx);
        let trigger_size = size.height();
        let trigger_color = if value.is_some() || open {
            color.hsla()
        } else {
            gpui::transparent_black()
        };
        let mut trigger = div()
            .id(id.clone())
            .relative()
            .w(px(trigger_size))
            .h(px(trigger_size))
            .p(px(4.))
            .border_1()
            .border_color(rgb(if disabled {
                theme.border(true)
            } else if state.read(cx).focused {
                crate::theme::PRIMARY
            } else {
                theme.border(false)
            }))
            .rounded(px(4.))
            .bg(rgb(if disabled {
                theme.fill(crate::theme::Fill::Light)
            } else {
                theme.background()
            }))
            .track_focus(&state.read(cx).focus)
            .tab_stop(!disabled)
            .aria_label(
                accessibility_label.unwrap_or_else(|| locale.color_picker_default_label().into()),
            )
            .when(disabled, |el| el.cursor_not_allowed())
            .when(!disabled, |el| el.cursor_pointer())
            .on_click({
                let state = state.clone();
                move |_, window, cx| {
                    cx.stop_propagation();
                    let open = !state.read(cx).open;
                    state.update(cx, |state, cx| state.set_open(open, window, cx));
                    window.refresh();
                }
            })
            .on_key_down({
                let state = state.clone();
                move |event, window, cx| {
                    if state.update(cx, |state, cx| state.key(&event.keystroke.key, window, cx)) {
                        cx.stop_propagation();
                        window.refresh();
                    }
                }
            });
        if value.is_some() || open {
            trigger = trigger.child(
                div()
                    .relative()
                    .size_full()
                    .rounded(px(2.))
                    .border_1()
                    .border_color(rgb(theme.secondary_text()))
                    .when(show_alpha, |el| el.child(checkerboard()))
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .rounded(px(2.))
                            .bg(trigger_color)
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(el_icon("arrow-down").size(12.).color(rgb(0xffffff))),
                    ),
            );
        } else {
            trigger = trigger.child(
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(rgb(theme.secondary_text()))
                    .child(el_icon("close").size(12.)),
            );
        }
        let measure_trigger = state.clone();
        let mut root = div()
            .relative()
            .child(measure_control(
                measure_trigger,
                trigger_size,
                |state, bounds| state.bounds = Some(bounds),
            ))
            .child(trigger)
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
                                gpui::size(px(304.), px(360.)),
                            )
                            .contains(&event.position)
                        });
                    if !inside_popup {
                        state.update(cx, |state, cx| {
                            state.drag = None;
                            state.set_open(false, window, cx)
                        });
                        window.refresh();
                    }
                }
            });
        if open && let Some(target) = state.read(cx).bounds {
            let viewport = window.viewport_size();
            let below = viewport.height - target.bottom() >= px(364.) || target.top() < px(364.);
            let (hue, saturation, value) = color.to_hsv();
            let pure = Color::from_hsv(hue, 1., 1., 1.).hsla();
            let mut panel = div()
                .id(ElementId::NamedChild(Arc::new(id.clone()), "panel".into()))
                .debug_selector(|| "color-picker-panel".into())
                .w(px(304.))
                .p(px(12.))
                .flex()
                .flex_col()
                .rounded(px(4.))
                .border_1()
                .border_color(rgb(theme.border_lighter()))
                .bg(rgb(theme.overlay_background()))
                .shadow(theme.overlay_shadow())
                .occlude();
            let sv = div()
                .id(ElementId::NamedChild(Arc::new(id.clone()), "sv".into()))
                .relative()
                .w(px(280.))
                .h(px(180.))
                .bg(pure)
                .child(div().absolute().inset_0().bg(gpui::linear_gradient(
                    90.,
                    gpui::linear_color_stop(gpui::rgb(0xffffff), 0.),
                    gpui::linear_color_stop(gpui::Hsla::white().opacity(0.), 1.),
                )))
                .child(div().absolute().inset_0().bg(gpui::linear_gradient(
                    0.,
                    gpui::linear_color_stop(gpui::Hsla::black().opacity(0.), 0.),
                    gpui::linear_color_stop(gpui::rgb(0x000000), 1.),
                )))
                .child(
                    div()
                        .absolute()
                        .left(px(saturation * 280. - 2.))
                        .top(px((1. - value) * 180. - 2.))
                        .w(px(4.))
                        .h(px(4.))
                        .rounded_full()
                        .bg(gpui::rgb(0xffffff))
                        .border_1()
                        .border_color(gpui::rgb(0x333333)),
                )
                .child(measure(state.clone(), |state, bounds| {
                    state.sv_bounds = Some(bounds)
                }))
                .on_mouse_down(MouseButton::Left, {
                    let state = state.clone();
                    move |event: &MouseDownEvent, window, cx| {
                        cx.stop_propagation();
                        state.update(cx, |state, cx| {
                            state.drag = Some(DragTarget::SaturationValue);
                            state.drag_to(DragTarget::SaturationValue, event.position, window, cx);
                        });
                        window.refresh();
                    }
                })
                .on_mouse_move({
                    let state = state.clone();
                    move |event: &MouseMoveEvent, window, cx| {
                        if event.pressed_button != Some(MouseButton::Left)
                            || state.read(cx).drag != Some(DragTarget::SaturationValue)
                        {
                            return;
                        }
                        state.update(cx, |state, cx| {
                            state.drag_to(DragTarget::SaturationValue, event.position, window, cx)
                        });
                        window.refresh();
                    }
                })
                .on_mouse_up(MouseButton::Left, {
                    let state = state.clone();
                    move |_, _, cx| state.update(cx, |state, _| state.drag = None)
                });
            panel = panel.child(sv);
            let mut hue_bar = div()
                .id(ElementId::NamedChild(Arc::new(id.clone()), "hue".into()))
                .relative()
                .w(px(280.))
                .h(px(12.))
                .mt(px(8.))
                .rounded(px(2.))
                .overflow_hidden()
                .child(
                    gpui::canvas(
                        |_, _, _| {},
                        move |bounds, _, window, _| paint_hue_gradient(bounds, window),
                    )
                    .absolute()
                    .inset_0(),
                );
            hue_bar = hue_bar
                .child(
                    div()
                        .absolute()
                        .left(px(hue * 280. - 2.))
                        .top(px(0.))
                        .w(px(4.))
                        .h(px(12.))
                        .rounded(px(1.))
                        .bg(gpui::rgb(0xffffff))
                        .border_1()
                        .border_color(rgb(theme.border_lighter())),
                )
                .child(measure(state.clone(), |state, bounds| {
                    state.hue_bounds = Some(bounds)
                }))
                .on_mouse_down(MouseButton::Left, {
                    let state = state.clone();
                    move |event: &MouseDownEvent, window, cx| {
                        cx.stop_propagation();
                        state.update(cx, |state, cx| {
                            state.drag = Some(DragTarget::Hue);
                            state.drag_to(DragTarget::Hue, event.position, window, cx);
                        });
                        window.refresh();
                    }
                })
                .on_mouse_move({
                    let state = state.clone();
                    move |event: &MouseMoveEvent, window, cx| {
                        if event.pressed_button != Some(MouseButton::Left)
                            || state.read(cx).drag != Some(DragTarget::Hue)
                        {
                            return;
                        }
                        state.update(cx, |state, cx| {
                            state.drag_to(DragTarget::Hue, event.position, window, cx)
                        });
                        window.refresh();
                    }
                })
                .on_mouse_up(MouseButton::Left, {
                    let state = state.clone();
                    move |_, _, cx| state.update(cx, |state, _| state.drag = None)
                });
            panel = panel.child(hue_bar);
            if show_alpha {
                let opaque = Color::from_hsv(hue, saturation, value, 1.).hsla();
                let alpha = div()
                    .id(ElementId::NamedChild(Arc::new(id.clone()), "alpha".into()))
                    .relative()
                    .w(px(280.))
                    .h(px(12.))
                    .mt(px(8.))
                    .child(checkerboard())
                    .child(div().absolute().inset_0().bg(gpui::linear_gradient(
                        90.,
                        gpui::linear_color_stop(gpui::Hsla::black().opacity(0.), 0.),
                        gpui::linear_color_stop(opaque, 1.),
                    )))
                    .child(
                        div()
                            .absolute()
                            .left(px(color.a * 280. - 2.))
                            .top(px(0.))
                            .w(px(4.))
                            .h(px(12.))
                            .rounded(px(1.))
                            .bg(gpui::rgb(0xffffff))
                            .border_1()
                            .border_color(rgb(theme.border_lighter())),
                    )
                    .child(measure(state.clone(), |state, bounds| {
                        state.alpha_bounds = Some(bounds)
                    }))
                    .on_mouse_down(MouseButton::Left, {
                        let state = state.clone();
                        move |event: &MouseDownEvent, window, cx| {
                            cx.stop_propagation();
                            state.update(cx, |state, cx| {
                                state.drag = Some(DragTarget::Alpha);
                                state.drag_to(DragTarget::Alpha, event.position, window, cx);
                            });
                            window.refresh();
                        }
                    })
                    .on_mouse_move({
                        let state = state.clone();
                        move |event: &MouseMoveEvent, window, cx| {
                            if event.pressed_button != Some(MouseButton::Left)
                                || state.read(cx).drag != Some(DragTarget::Alpha)
                            {
                                return;
                            }
                            state.update(cx, |state, cx| {
                                state.drag_to(DragTarget::Alpha, event.position, window, cx)
                            });
                            window.refresh();
                        }
                    })
                    .on_mouse_up(MouseButton::Left, {
                        let state = state.clone();
                        move |_, _, cx| state.update(cx, |state, _| state.drag = None)
                    });
                panel = panel.child(alpha);
            }
            if !predefine.is_empty() {
                let mut colors = div().flex().flex_wrap().gap(px(8.)).mt(px(8.));
                for (index, preset) in predefine.iter().enumerate() {
                    let Some(preset_color) = Color::parse(preset) else {
                        continue;
                    };
                    let selected = (preset_color.r - color.r).abs() < 0.005
                        && (preset_color.g - color.g).abs() < 0.005
                        && (preset_color.b - color.b).abs() < 0.005
                        && (!show_alpha || (preset_color.a - color.a).abs() < 0.005);
                    colors = colors.child(
                        div()
                            .id(ElementId::NamedChild(
                                Arc::new(id.clone()),
                                format!("preset-{index}").into(),
                            ))
                            .w(px(20.))
                            .h(px(20.))
                            .rounded(px(4.))
                            .bg(preset_color.hsla())
                            .when(show_alpha && preset_color.a < 1., |el| {
                                el.child(checkerboard())
                            })
                            .when(selected, |el| {
                                el.shadow(vec![gpui::BoxShadow {
                                    color: rgb(crate::theme::PRIMARY).into(),
                                    offset: point(px(0.), px(0.)),
                                    blur_radius: px(0.),
                                    spread_radius: px(2.),
                                    inset: false,
                                }])
                            })
                            .cursor_pointer()
                            .on_click({
                                let state = state.clone();
                                move |_, window, cx| {
                                    cx.stop_propagation();
                                    state.update(cx, |state, cx| {
                                        state.update_color(preset_color, window, cx)
                                    });
                                    window.refresh();
                                }
                            }),
                    );
                }
                panel = panel.child(colors);
            }
            let current = color.format(format, show_alpha);
            let mut actions = div()
                .flex()
                .items_center()
                .justify_end()
                .gap(px(8.))
                .mt(px(12.));
            if clearable {
                actions = actions.child(
                    el_button()
                        .id(ElementId::NamedChild(Arc::new(id.clone()), "clear".into()))
                        .small()
                        .text()
                        .label(locale.color_picker_clear())
                        .onclick({
                            let state = state.clone();
                            move |_, window, cx| {
                                state.update(cx, |state, cx| state.clear(window, cx));
                                window.refresh();
                            }
                        }),
                );
            }
            actions = actions.child(
                div().child(
                    el_button()
                        .id(ElementId::NamedChild(
                            Arc::new(id.clone()),
                            "confirm".into(),
                        ))
                        .small()
                        .primary()
                        .label(locale.color_picker_confirm())
                        .onclick({
                            let state = state.clone();
                            move |_, window, cx| {
                                state.update(cx, |state, cx| state.confirm(window, cx));
                                window.refresh();
                            }
                        }),
                ),
            );
            panel = panel.child(
                div().mt(px(12.)).child(
                    el_input()
                        .id(ElementId::NamedChild(
                            Arc::new(id.clone()),
                            "text-input".into(),
                        ))
                        .value(current)
                        .small()
                        .aria_label(locale.color_picker_default_label())
                        .onchange({
                            let state = state.clone();
                            move |reported, window, cx| {
                                if let Some(parsed) = Color::parse(reported.as_ref()) {
                                    state.update(cx, |state, cx| {
                                        state.update_color(parsed, window, cx)
                                    });
                                    window.refresh();
                                }
                            }
                        }),
                ),
            );
            panel = panel.child(actions);
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
    fn parses_and_formats_hex_and_rgb() {
        let color = Color::parse("#409eff").unwrap();
        assert_eq!(color.format(ColorFormat::Hex, false).as_ref(), "#409eff");
        assert_eq!(
            color.format(ColorFormat::Rgb, false).as_ref(),
            "rgb(64, 158, 255)"
        );
        assert_eq!(Color::parse("#f008").unwrap().a, 136. / 255.);
        assert_eq!(Color::parse("rgba(64, 158, 255, 0.5)").unwrap().a, 0.5);
        assert!(Color::parse("not-a-color").is_none());
    }

    #[test]
    fn hsv_round_trips_primary_and_black() {
        for color in [
            Color::PRIMARY,
            Color {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 1.,
            },
        ] {
            let (h, s, v) = color.to_hsv();
            let round_trip = Color::from_hsv(h, s, v, color.a);
            assert!((round_trip.r - color.r).abs() < 1e-6);
            assert!((round_trip.g - color.g).abs() < 1e-6);
            assert!((round_trip.b - color.b).abs() < 1e-6);
        }
    }

    #[gpui::test]
    fn rendered_presets_report_active_changes(cx: &mut gpui::TestAppContext) {
        use std::cell::RefCell;
        struct Harness {
            active: Rc<RefCell<Vec<Option<SharedString>>>>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let active = self.active.clone();
                div().p(px(20.)).child(
                    el_color_picker()
                        .id("rendered-color-picker")
                        .value(Some("#409eff".into()))
                        .predefine([
                            SharedString::from("#ff0000"),
                            SharedString::from("#00ff00"),
                            SharedString::from("#0000ff"),
                        ])
                        .onactivechange(move |value, _, _| active.borrow_mut().push(value)),
                )
            }
        }
        let active = Rc::new(RefCell::new(Vec::new()));
        let (_, view) = cx.add_window_view({
            let active = active.clone();
            move |_, _| Harness { active }
        });
        view.simulate_click(gpui::point(px(36.), px(36.)), gpui::Modifiers::default());
        view.update(|window, cx| window.draw(cx).clear(cx));
        view.simulate_click(gpui::point(px(42.), px(286.)), gpui::Modifiers::default());
        assert_eq!(
            active.borrow().as_slice(),
            [Some(SharedString::from("#ff0000"))]
        );
    }

    #[gpui::test]
    fn confirm_and_clear_report_controlled_values(cx: &mut gpui::TestAppContext) {
        struct Harness {
            state: gpui::Entity<ColorPickerState>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                div()
            }
        }
        let changes = Rc::new(std::cell::RefCell::new(Vec::new()));
        let (harness, view) = cx.add_window_view({
            let changes = changes.clone();
            move |window, cx| {
                let state = cx.new(|cx| ColorPickerState::new(window, cx));
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
            state.update(cx, |state, cx| {
                state.color = Color::parse("#ff0000").unwrap();
                state.confirm(window, cx);
            });
            state.update(cx, |state, cx| state.clear(window, cx));
        });
        assert_eq!(
            *changes.borrow(),
            [Some(SharedString::from("#ff0000")), None]
        );
    }
}
