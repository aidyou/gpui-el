//! Element Plus Scrollbar.
//!
//! GPUI scroll containers have no scrollbar chrome of their own, so this
//! component wraps the content in an overflow container and draws the Theme
//! Chalk thumbs itself: 6 px bars inset 2 px, 4 px radius, secondary-color
//! fills at 0.3 opacity (0.5 hovered).

use std::rc::Rc;

use gpui::{
    AnyElement, App, Axis, ElementId, Entity, IntoElement, MouseButton, ParentElement, Pixels,
    Point, RenderOnce, ScrollHandle, Size, Window, div, prelude::*, px, rgb,
};

use crate::Theme;

/// Bar metrics from Theme Chalk `scrollbar.scss`.
const BAR: f32 = 6.;
const INSET: f32 = 2.;
const RADIUS: f32 = 4.;
/// Upstream `min-size`, the shortest a thumb can get.
const DEFAULT_MIN_SIZE: f32 = 20.;

/// Scroll offsets reported while the viewport moves, in logical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollbarEvent {
    pub scroll_left: f32,
    pub scroll_top: f32,
}

/// The viewport edge a [`Scrollbar::onreached`] callback reports.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollbarDirection {
    Top,
    Bottom,
    Left,
    Right,
}

type ScrollHandler = Box<dyn Fn(&ScrollbarEvent, &mut Window, &mut App)>;
type EndReachedHandler = Box<dyn Fn(&ScrollbarDirection, &mut Window, &mut App)>;

#[derive(Clone, Copy)]
struct ThumbDrag {
    axis: Axis,
    pointer: Point<Pixels>,
    position: Point<f32>,
}

#[derive(Default)]
struct ScrollbarState {
    handle: ScrollHandle,
    hover: bool,
    drag: Option<ThumbDrag>,
    /// Viewport size and maximum offset seen at paint time. Comparing it
    /// detects the first frame, when the handle has no geometry yet, and
    /// later resizes, both of which need another render for the thumbs.
    seen: Option<(Size<Pixels>, Point<Pixels>)>,
    /// Scroll position reported by the last event delivery, for end-reached
    /// edges. GPUI translates content with a negative offset; this is the
    /// positive, DOM-like position.
    prev_position: Point<f32>,
}

/// The visible part of one scrollbar thumb.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Thumb {
    length: f32,
    position: f32,
}

/// Thumb geometry is pure layout math, so the tests can pin it directly.
fn thumb(viewport: f32, max: f32, offset: f32, min_size: f32) -> Option<Thumb> {
    if viewport <= 0. || max <= 0. {
        return None;
    }
    let track = (viewport - 2. * INSET).max(0.);
    let length = ((viewport * viewport) / (viewport + max)).clamp(min_size, track);
    let position = INSET + (offset / max) * (track - length);
    Some(Thumb { length, position })
}

/// The end an offset sits at, if any: the far end, the near end, or neither.
fn end_for(offset: f32, max: f32, distance: f32) -> Option<bool> {
    if max - offset <= distance {
        Some(true)
    } else if offset <= distance {
        Some(false)
    } else {
        None
    }
}

/// The end an offset crossed since the previous delivery, or `None` when the
/// viewport stayed on the same side of both edges.
fn newly_reached(prev: f32, now: f32, max: f32, distance: f32) -> Option<bool> {
    let was = end_for(prev, max, distance);
    let is = end_for(now, max, distance);
    (is != was).then_some(is).flatten()
}

/// The DOM-like scroll position: GPUI translates the content with a negative
/// offset, so flipping the sign gives a position that grows downward like
/// `scrollTop`.
fn scrolled_position(handle: &ScrollHandle) -> Point<f32> {
    let offset = handle.offset();
    Point::new(-offset.x.as_f32(), -offset.y.as_f32())
}

/// Delivers `onscroll` and `onreached` from one observed offset.
struct Sink {
    onscroll: Option<ScrollHandler>,
    onreached: Option<EndReachedHandler>,
    distance: f32,
}

impl Sink {
    fn deliver(
        &self,
        state: &Entity<ScrollbarState>,
        handle: &ScrollHandle,
        window: &mut Window,
        cx: &mut App,
    ) {
        let position = scrolled_position(handle);
        let max = handle.max_offset();
        if let Some(onscroll) = &self.onscroll {
            onscroll(
                &ScrollbarEvent {
                    scroll_left: position.x,
                    scroll_top: position.y,
                },
                window,
                cx,
            );
        }
        if self.onreached.is_some() {
            let prev = state.read(cx).prev_position;
            let mut directions = Vec::new();
            if let Some(reached) = newly_reached(prev.y, position.y, max.y.as_f32(), self.distance)
            {
                directions.push(if reached {
                    ScrollbarDirection::Bottom
                } else {
                    ScrollbarDirection::Top
                });
            }
            if let Some(reached) = newly_reached(prev.x, position.x, max.x.as_f32(), self.distance)
            {
                directions.push(if reached {
                    ScrollbarDirection::Right
                } else {
                    ScrollbarDirection::Left
                });
            }
            for direction in directions {
                if let Some(handler) = &self.onreached {
                    handler(&direction, window, cx);
                }
            }
        }
        state.update(cx, |state, _| state.prev_position = position);
    }
}

/// Creates a Scrollbar with the default Element Plus metrics.
///
/// A call-site ID is provided by default. Set an explicit [`Scrollbar::id`]
/// when several scrollbars are created at the same call site, for example in
/// a loop, so the tracked state stays with its own viewport.
#[track_caller]
pub fn el_scrollbar() -> Scrollbar {
    Scrollbar::default().id(ElementId::CodeLocation(*std::panic::Location::caller()))
}

/// A scroll viewport with Element Plus styled scrollbars.
///
/// Content taller than the viewport scrolls on its own; content wider than it
/// must state a width (for example `div().w(px(1200.))`) because GPUI has no
/// inline-block wrapper that would size a block child to its contents.
///
/// Pass a [`ScrollHandle`] to read or set the offset imperatively, which is
/// the native replacement for the upstream `setScrollTop`, `setScrollLeft`
/// and `scrollTo` exposes. Native differences: the gradient `mask` property,
/// the `wrap`/`view` class and style hooks, and the `noresize` flag are not
/// reproduced because layout owns the geometry here. Thumbs appear on hover
/// or with `always`, without the upstream 340 ms / 120 ms fade, and the wheel
/// is stopped at the viewport like every other scroll container in this
/// crate.
#[derive(Default, IntoElement)]
#[must_use]
pub struct Scrollbar {
    id: Option<ElementId>,
    height: Option<f32>,
    max_height: Option<f32>,
    always: bool,
    native: bool,
    min_size: f32,
    handle: Option<ScrollHandle>,
    children: Vec<AnyElement>,
    onscroll: Option<ScrollHandler>,
    onreached: Option<EndReachedHandler>,
    distance: f32,
}

impl Scrollbar {
    /// Creates a scrollbar with a stable GPUI element ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self::default().id(id)
    }

    /// Enables pointer tracking for this ID.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Fixes the viewport height in logical pixels.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height.is_finite().then_some(height);
        self
    }

    /// Scrolls once the content grows past this height in logical pixels.
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = max_height.is_finite().then_some(max_height);
        self
    }

    /// Keeps the thumbs visible instead of showing them on hover.
    pub fn always(mut self, always: bool) -> Self {
        self.always = always;
        self
    }

    /// Skips the custom thumbs and leaves the plain overflow container.
    pub fn native(mut self, native: bool) -> Self {
        self.native = native;
        self
    }

    /// Sets the minimum thumb length in logical pixels, like `min-size`.
    pub fn min_size(mut self, min_size: f32) -> Self {
        self.min_size = min_size;
        self
    }

    /// Tracks this viewport with the given handle, which can then read the
    /// offset and maximum, and set the offset for manual scrolling.
    pub fn handle(mut self, handle: ScrollHandle) -> Self {
        self.handle = Some(handle);
        self
    }

    /// Reports the viewport offset on every scroll, in logical pixels.
    #[track_caller]
    pub fn onscroll(
        mut self,
        handler: impl Fn(&ScrollbarEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onscroll = Some(Box::new(handler));
        self
    }

    /// Reports the viewport edge that was newly reached while scrolling.
    /// `distance` widens the edge in logical pixels, like the upstream
    /// `distance` attribute.
    #[track_caller]
    pub fn onreached(
        mut self,
        distance: f32,
        handler: impl Fn(&ScrollbarDirection, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.distance = distance;
        self.onreached = Some(Box::new(handler));
        self
    }
}

impl ParentElement for Scrollbar {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}

impl ThumbDrag {
    /// The position component this drag is adjusting.
    fn position_component(&self) -> f32 {
        match self.axis {
            Axis::Vertical => self.position.y,
            Axis::Horizontal => self.position.x,
        }
    }
}

impl RenderOnce for Scrollbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self
            .id
            .unwrap_or_else(|| ElementId::CodeLocation(*std::panic::Location::caller()));
        let state = window.use_keyed_state(id.clone(), cx, |_, _| ScrollbarState::default());
        if let Some(handle) = self.handle.clone() {
            state.update(cx, |state, _| state.handle = handle);
        }
        let owner = window.current_view();
        let handle = state.read(cx).handle.clone();
        let viewport = handle.bounds().size;
        let max = handle.max_offset();
        let position = scrolled_position(&handle);
        let interactive = state.read(cx).hover || state.read(cx).drag.is_some();
        let show_thumbs = !self.native && (self.always || interactive);
        let theme = Theme::current(cx);
        let thumb_color = rgb(theme.secondary_text());
        let min_size = if self.min_size > 0. {
            self.min_size
        } else {
            DEFAULT_MIN_SIZE
        };
        let sink = Rc::new(Sink {
            onscroll: self.onscroll,
            onreached: self.onreached,
            distance: self.distance,
        });

        let mut root = div()
            .id(id.clone())
            .relative()
            // Upstream defaults to `height: 100%` of the parent. With only a
            // maximum height the box stays content-sized so the clamp applies
            // to the scroll container below instead of a stretched root.
            .when_some(self.height, |el, height| el.h(px(height)))
            .when(self.height.is_none() && self.max_height.is_none(), |el| {
                el.h_full()
            })
            .when_some(self.max_height, |el, max| el.max_h(px(max)))
            .min_w_0()
            .min_h_0()
            .child(
                div()
                    .id((id.clone(), "view"))
                    .w_full()
                    .when_some(self.height, |el, height| el.h(px(height)))
                    .when(self.height.is_none() && self.max_height.is_none(), |el| {
                        el.h_full()
                    })
                    .when_some(self.max_height, |el, max| el.max_h(px(max)))
                    .overflow_x_scroll()
                    .overflow_y_scroll()
                    .track_scroll(&handle)
                    .children(self.children)
                    .on_scroll({
                        let state = state.clone();
                        let handle = handle.clone();
                        let sink = sink.clone();
                        move |_, window, cx| {
                            sink.deliver(&state, &handle, window, cx);
                            // The viewport owns the wheel, like the other
                            // scroll containers in this crate.
                            cx.stop_propagation();
                        }
                    }),
            );

        if show_thumbs {
            let make_thumb = |axis: Axis, geometry: Thumb| {
                let state = state.clone();
                let start_offset = position;
                div()
                    .id((id.clone(), if axis == Axis::Vertical { "v" } else { "h" }))
                    .absolute()
                    .bg(thumb_color.opacity(0.3))
                    .rounded(px(RADIUS))
                    .cursor_pointer()
                    .hover(move |style| style.bg(thumb_color.opacity(0.5)))
                    .when(axis == Axis::Vertical, |el| {
                        el.top(px(geometry.position))
                            .right(px(INSET))
                            .w(px(BAR))
                            .h(px(geometry.length))
                    })
                    .when(axis == Axis::Horizontal, |el| {
                        el.left(px(geometry.position))
                            .bottom(px(INSET))
                            .w(px(geometry.length))
                            .h(px(BAR))
                    })
                    .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                        state.update(cx, |state, _| {
                            state.drag = Some(ThumbDrag {
                                axis,
                                pointer: event.position,
                                position: start_offset,
                            });
                        });
                        cx.stop_propagation();
                    })
            };
            if let Some(geometry) = thumb(
                viewport.height.as_f32(),
                max.y.as_f32(),
                position.y,
                min_size,
            ) {
                root = root.child(make_thumb(Axis::Vertical, geometry));
            }
            if let Some(geometry) = thumb(
                viewport.width.as_f32(),
                max.x.as_f32(),
                position.x,
                min_size,
            ) {
                root = root.child(make_thumb(Axis::Horizontal, geometry));
            }
        }

        // The handle only knows its geometry after the first paint; a resize
        // changes it again. Syncing here asks the owning view for another
        // render so the thumbs follow.
        let sync_state = state.clone();
        let sync_handle = handle.clone();
        root = root.child(
            gpui::canvas(
                move |_, _, _| {},
                move |_, _, window, cx| {
                    let viewport = sync_handle.bounds().size;
                    let max = sync_handle.max_offset();
                    if sync_state.read(cx).seen != Some((viewport, max)) {
                        sync_state.update(cx, |state, _| state.seen = Some((viewport, max)));
                        let owner = window.current_view();
                        cx.notify(owner);
                        window.refresh();
                    }
                },
            )
            .absolute()
            .inset_0(),
        );

        let up_state = state.clone();
        let hover_state = state.clone();
        root = root
            .on_mouse_move(move |event, window, cx| {
                let drag = state.read(cx).drag;
                let Some(drag) = drag else {
                    return;
                };
                let max = handle.max_offset();
                let viewport = handle.bounds().size;
                let (viewport, max, delta, travel) = match drag.axis {
                    Axis::Vertical => {
                        let track = (viewport.height.as_f32() - 2. * INSET).max(0.);
                        let travel = thumb(viewport.height.as_f32(), max.y.as_f32(), 0., min_size)
                            .map(|geometry| track - geometry.length)
                            .unwrap_or(0.);
                        (
                            viewport.height.as_f32(),
                            max.y.as_f32(),
                            event.position.y.as_f32() - drag.pointer.y.as_f32(),
                            travel,
                        )
                    }
                    Axis::Horizontal => {
                        let track = (viewport.width.as_f32() - 2. * INSET).max(0.);
                        let travel = thumb(viewport.width.as_f32(), max.x.as_f32(), 0., min_size)
                            .map(|geometry| track - geometry.length)
                            .unwrap_or(0.);
                        (
                            viewport.width.as_f32(),
                            max.x.as_f32(),
                            event.position.x.as_f32() - drag.pointer.x.as_f32(),
                            travel,
                        )
                    }
                };
                if max <= 0. || viewport <= 0. {
                    return;
                }
                // One pixel of thumb movement is worth `max / travel` pixels of
                // content, so the thumb stays under the pointer.
                let ratio = if travel > 0. {
                    max / travel
                } else {
                    viewport + max
                };
                let scrolled = drag.position_component() + delta * ratio;
                let clamped = scrolled.clamp(0., max);
                let mut offset = handle.offset();
                match drag.axis {
                    Axis::Vertical => offset.y = px(-clamped),
                    Axis::Horizontal => offset.x = px(-clamped),
                }
                handle.set_offset(offset);
                sink.deliver(&state, &handle, window, cx);
                window.refresh();
            })
            .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                up_state.update(cx, |state, _| state.drag = None);
            })
            .on_hover(move |hovered, window, cx| {
                hover_state.update(cx, |state, _| state.hover = *hovered);
                cx.notify(owner);
                window.refresh();
            });

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn thumb_scales_with_the_overflow_and_clamps_to_the_track() {
        // A 200 px viewport over 600 px of content shows a third of it.
        let geometry = thumb(200., 400., 0., DEFAULT_MIN_SIZE).unwrap();
        assert_eq!(geometry.length, 200. * 200. / 600.);
        assert_eq!(geometry.position, INSET);
        // At the far end the thumb sits at the bottom of the track.
        let geometry = thumb(200., 400., 400., DEFAULT_MIN_SIZE).unwrap();
        assert_eq!(geometry.position, 200. - INSET - geometry.length);
        // A tiny thumb grows to the minimum size.
        let geometry = thumb(100., 900., 0., DEFAULT_MIN_SIZE).unwrap();
        assert_eq!(geometry.length, DEFAULT_MIN_SIZE);
        // A thumb longer than the track shrinks to it.
        let geometry = thumb(200., 2., 0., DEFAULT_MIN_SIZE).unwrap();
        assert_eq!(geometry.length, 200. - 2. * INSET);
    }

    #[test]
    fn thumb_is_hidden_without_overflow() {
        assert!(thumb(200., 0., 0., DEFAULT_MIN_SIZE).is_none());
        assert!(thumb(0., 100., 0., DEFAULT_MIN_SIZE).is_none());
    }

    #[test]
    fn end_detection_reports_newly_reached_edges_only() {
        assert_eq!(newly_reached(0., 0., 400., 0.), None);
        assert_eq!(newly_reached(0., 100., 400., 0.), None);
        assert_eq!(newly_reached(100., 400., 400., 0.), Some(true));
        assert_eq!(newly_reached(400., 400., 400., 0.), None);
        assert_eq!(newly_reached(400., 0., 400., 0.), Some(false));
        // A positive distance widens the edge.
        assert_eq!(newly_reached(100., 390., 400., 20.), Some(true));
        assert_eq!(newly_reached(100., 370., 400., 0.), None);
    }

    #[gpui::test]
    fn scrolling_reports_offsets_and_reaching_the_end(cx: &mut gpui::TestAppContext) {
        struct Harness {
            handle: ScrollHandle,
            scrolled: Rc<RefCell<Vec<ScrollbarEvent>>>,
            ends: Rc<RefCell<Vec<ScrollbarDirection>>>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let handle = self.handle.clone();
                let scrolled = self.scrolled.clone();
                let ends = self.ends.clone();
                el_scrollbar()
                    .id("scroller")
                    .height(200.)
                    .handle(handle)
                    .onscroll(move |event, _, _| scrolled.borrow_mut().push(*event))
                    .onreached(0., move |direction, _, _| {
                        ends.borrow_mut().push(*direction)
                    })
                    .child(
                        div()
                            .w(px(400.))
                            .h(px(600.))
                            .bg(rgb(0xff0000))
                            .child("content"),
                    )
            }
        }
        let handle = ScrollHandle::new();
        let scrolled: Rc<RefCell<Vec<ScrollbarEvent>>> = Rc::new(RefCell::new(Vec::new()));
        let ends: Rc<RefCell<Vec<ScrollbarDirection>>> = Rc::new(RefCell::new(Vec::new()));
        let (_, view) = cx.add_window_view({
            let handle = handle.clone();
            let scrolled = scrolled.clone();
            let ends = ends.clone();
            move |_, _| Harness {
                handle,
                scrolled,
                ends,
            }
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        view.run_until_parked();
        view.update(|window, cx| window.draw(cx).clear(cx));
        let max = handle.max_offset();
        assert!(max.y > px(0.), "the viewport should overflow vertically");
        assert_eq!(max.x, px(0.), "the content is narrower than the viewport");

        // Scroll to the bottom in one wheel event.
        view.simulate_event(gpui::ScrollEvent {
            position: gpui::point(px(10.), px(10.)),
            delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.), -max.y)),
            modifiers: Default::default(),
            touch_phase: Default::default(),
            source: Default::default(),
        });
        view.run_until_parked();
        view.update(|window, cx| window.draw(cx).clear(cx));
        let events = scrolled.borrow();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].scroll_top, max.y.as_f32());
        assert_eq!(*ends.borrow(), vec![ScrollbarDirection::Bottom]);
    }

    #[gpui::test]
    fn max_height_limits_the_scroll_box(cx: &mut gpui::TestAppContext) {
        // Mirrors the gallery: a scrollable main region, a centered column,
        // and a padded section box around the scrollbar. A root that only
        // clamps itself would let the content grow past the maximum here.
        struct Harness {
            handle: ScrollHandle,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                div().id("main").size_full().overflow_y_scroll().child(
                    div()
                        .w_full()
                        .max_w(px(760.))
                        .mx_auto()
                        .flex()
                        .flex_col()
                        .gap(px(28.))
                        .child(
                            div().border_1().child(
                                div().p(px(24.)).child(
                                    el_scrollbar()
                                        .id("scroller")
                                        .max_height(200.)
                                        .handle(self.handle.clone())
                                        .child(div().h(px(600.)).child("content")),
                                ),
                            ),
                        )
                        .child(div().h(px(40.)).child("next section")),
                )
            }
        }
        let handle = ScrollHandle::new();
        let (_, view) = cx.add_window_view({
            let handle = handle.clone();
            move |_, _| Harness { handle }
        });
        for _ in 0..2 {
            view.update(|window, cx| window.draw(cx).clear(cx));
            view.run_until_parked();
        }
        // The scroll box stops at the maximum height instead of growing with
        // its content and painting over the next section.
        assert_eq!(handle.bounds().size.height, px(200.));
        assert_eq!(handle.max_offset().y, px(400.));
    }

    #[gpui::test]
    fn short_content_keeps_the_box_content_sized(cx: &mut gpui::TestAppContext) {
        // The gallery max-height demo starts with five 36 px rows: the box
        // must stay at 180 px and only grow into the 200 px limit later.
        struct Harness {
            handle: ScrollHandle,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                el_scrollbar()
                    .id("scroller")
                    .max_height(200.)
                    .handle(self.handle.clone())
                    .child(div().h(px(180.)).child("five rows"))
            }
        }
        let handle = ScrollHandle::new();
        let (_, view) = cx.add_window_view({
            let handle = handle.clone();
            move |_, _| Harness { handle }
        });
        for _ in 0..2 {
            view.update(|window, cx| window.draw(cx).clear(cx));
            view.run_until_parked();
        }
        assert_eq!(handle.bounds().size.height, px(180.));
        assert_eq!(handle.max_offset().y, px(0.));
    }

    #[gpui::test]
    fn wide_content_scrolls_horizontally(cx: &mut gpui::TestAppContext) {
        struct Harness {
            handle: ScrollHandle,
            scrolled: Rc<RefCell<Vec<ScrollbarEvent>>>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let scrolled = self.scrolled.clone();
                el_scrollbar()
                    .id("scroller")
                    .height(160.)
                    .handle(self.handle.clone())
                    .onscroll(move |event, _, _| scrolled.borrow_mut().push(*event))
                    .child(div().w(px(3000.)).h(px(120.)).child("wide"))
            }
        }
        let handle = ScrollHandle::new();
        let scrolled: Rc<RefCell<Vec<ScrollbarEvent>>> = Rc::new(RefCell::new(Vec::new()));
        let (_, view) = cx.add_window_view({
            let handle = handle.clone();
            let scrolled = scrolled.clone();
            move |_, _| Harness { handle, scrolled }
        });
        for _ in 0..2 {
            view.update(|window, cx| window.draw(cx).clear(cx));
            view.run_until_parked();
        }
        let max = handle.max_offset();
        assert!(max.x > px(0.), "wide content should overflow horizontally");
        assert_eq!(
            max.y,
            px(0.),
            "short content should not overflow vertically"
        );
        // A horizontal wheel delta scrolls the viewport and reports it.
        view.simulate_event(gpui::ScrollEvent {
            position: gpui::point(px(10.), px(10.)),
            delta: gpui::ScrollDelta::Pixels(gpui::point(-max.x, px(0.))),
            modifiers: Default::default(),
            touch_phase: Default::default(),
            source: Default::default(),
        });
        view.run_until_parked();
        view.update(|window, cx| window.draw(cx).clear(cx));
        let events = scrolled.borrow();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].scroll_left, max.x.as_f32());
        assert_eq!(events[0].scroll_top, 0.);
    }

    #[gpui::test]
    fn hovering_shows_the_thumb(cx: &mut gpui::TestAppContext) {
        struct Harness {
            handle: ScrollHandle,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                el_scrollbar()
                    .id("scroller")
                    .height(200.)
                    .handle(self.handle.clone())
                    .child(div().h(px(600.)).child("content"))
            }
        }
        let handle = ScrollHandle::new();
        let (_, view) = cx.add_window_view({
            let handle = handle.clone();
            move |_, _| Harness { handle }
        });
        for _ in 0..2 {
            view.update(|window, cx| window.draw(cx).clear(cx));
            view.run_until_parked();
        }
        // Hovering the viewport renders the thumb, which can then be dragged.
        view.simulate_mouse_move(gpui::point(px(20.), px(20.)), None, Default::default());
        for _ in 0..2 {
            view.update(|window, cx| window.draw(cx).clear(cx));
            view.run_until_parked();
        }
        let viewport = handle.bounds().size;
        let max = handle.max_offset();
        let geometry = thumb(
            viewport.height.as_f32(),
            max.y.as_f32(),
            0.,
            DEFAULT_MIN_SIZE,
        )
        .unwrap();
        let thumb_center = gpui::point(
            px(viewport.width.as_f32() - INSET - BAR / 2.),
            px(geometry.position + geometry.length / 2.),
        );
        view.simulate_mouse_down(thumb_center, gpui::MouseButton::Left, Default::default());
        view.simulate_mouse_move(
            thumb_center + gpui::point(px(0.), px(30.)),
            None,
            Default::default(),
        );
        view.run_until_parked();
        view.update(|window, cx| window.draw(cx).clear(cx));
        assert!(
            scrolled_position(&handle).y > 0.,
            "the hovered thumb should be draggable"
        );
    }

    #[gpui::test]
    fn dragging_the_thumb_moves_the_viewport(cx: &mut gpui::TestAppContext) {
        struct Harness {
            handle: ScrollHandle,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                el_scrollbar()
                    .id("scroller")
                    .height(200.)
                    .always(true)
                    .handle(self.handle.clone())
                    .child(div().w(px(400.)).h(px(600.)).child("content"))
            }
        }
        let handle = ScrollHandle::new();
        let (_, view) = cx.add_window_view({
            let handle = handle.clone();
            move |_, _| Harness { handle }
        });
        // Two draws let the sync canvas publish the geometry and render the
        // thumb from it.
        for _ in 0..2 {
            view.update(|window, cx| window.draw(cx).clear(cx));
            view.run_until_parked();
        }
        let viewport = handle.bounds().size;
        let max = handle.max_offset();
        let geometry = thumb(
            viewport.height.as_f32(),
            max.y.as_f32(),
            0.,
            DEFAULT_MIN_SIZE,
        )
        .unwrap();
        let thumb_center = gpui::point(
            px(viewport.width.as_f32() - INSET - BAR / 2.),
            px(geometry.position + geometry.length / 2.),
        );
        view.simulate_mouse_down(thumb_center, gpui::MouseButton::Left, Default::default());
        view.simulate_mouse_move(
            thumb_center + gpui::point(px(0.), px(30.)),
            None,
            Default::default(),
        );
        view.run_until_parked();
        view.update(|window, cx| window.draw(cx).clear(cx));
        let position = scrolled_position(&handle);
        assert!(position.y > 0., "dragging the thumb should scroll down");
        // The thumb follows the pointer: 30 px of travel is worth
        // 30 * max / travel px of content.
        let track = viewport.height.as_f32() - 2. * INSET;
        let expected = 30. * max.y.as_f32() / (track - geometry.length);
        assert!(
            (position.y - expected).abs() < 0.5,
            "{} vs {expected}",
            position.y
        );
    }
}
