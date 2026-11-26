//! Element Plus Splitter.
//!
//! Panel sizes live in the component state so a drag survives the parent
//! re-rendering, and the container is measured at paint time like
//! [`crate::Scrollbar`]. Theme Chalk `splitter.scss` supplies the 16 px drag
//! hit area over a 1 px divider line and the 12 px collapse arrows.

use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, Entity, IntoElement, KeyDownEvent, MouseButton, ParentElement,
    Pixels, Point, RenderOnce, SharedString, Window, div, prelude::*, px, rgb,
};

use crate::{Theme, el_icon, tokens};

/// Width of the divider hit area; the visible line is 1 px inside it.
const BAR: f32 = 16.;
/// How far the collapse arrows sit from the divider line.
const ARROW_GAP: f32 = 15.;
/// Keyboard step in logical pixels; Shift widens it.
const STEP: f32 = 10.;

/// Which side of a divider a collapse action targets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SplitterSide {
    Start,
    End,
}

/// Upstream `layout`: `horizontal` puts the panels side by side.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SplitterLayout {
    #[default]
    Horizontal,
    Vertical,
}

/// The sizes reported by the resize and collapse callbacks, in logical pixels.
pub type SplitterSizes = Rc<[f32]>;

/// One divider drag update.
#[derive(Clone, Debug, PartialEq)]
pub struct SplitterResizeEvent {
    /// The divider index, between panel `index` and `index + 1`.
    pub index: usize,
    /// The sizes of every panel after the update.
    pub sizes: SplitterSizes,
}

/// One collapse or expand through a divider arrow.
#[derive(Clone, Debug, PartialEq)]
pub struct SplitterCollapseEvent {
    /// The divider index, between panel `index` and `index + 1`.
    pub index: usize,
    /// Which side of the divider collapsed or expanded.
    pub side: SplitterSide,
    /// The sizes of every panel after the update.
    pub sizes: SplitterSizes,
}

type ResizeHandler = Box<dyn Fn(&SplitterResizeEvent, &mut Window, &mut App)>;
type CollapseHandler = Box<dyn Fn(&SplitterCollapseEvent, &mut Window, &mut App)>;

/// One resizable region of a [`Splitter`].
#[must_use]
pub struct SplitterPanel {
    size: Option<f32>,
    min: Option<f32>,
    max: Option<f32>,
    resizable: bool,
    collapsible: bool,
    children: Vec<AnyElement>,
}

impl Default for SplitterPanel {
    fn default() -> Self {
        Self {
            size: None,
            min: None,
            max: None,
            resizable: true,
            collapsible: false,
            children: Vec::new(),
        }
    }
}

impl SplitterPanel {
    /// Creates a panel that fills the leftover space and can be resized.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the initial size in logical pixels. Panels without a size share
    /// the leftover space equally.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size.is_finite().then_some(size);
        self
    }

    /// Sets the minimum size in logical pixels; dragging stops there.
    pub fn min(mut self, min: f32) -> Self {
        self.min = min.is_finite().then_some(min);
        self
    }

    /// Sets the maximum size in logical pixels; dragging stops there.
    pub fn max(mut self, max: f32) -> Self {
        self.max = max.is_finite().then_some(max);
        self
    }

    /// Lets this panel be resized by its dividers. `true` by default; a
    /// divider between two resizable panels is draggable.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Shows collapse arrows on this panel's divider and lets them shrink
    /// the panel to zero regardless of `min`.
    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }
}

impl ParentElement for SplitterPanel {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}

/// Creates a panel that fills the leftover space and can be resized.
pub fn el_splitter_panel() -> SplitterPanel {
    SplitterPanel::default()
}

#[derive(Default)]
struct SplitterState {
    /// Current panel sizes in logical pixels. `None` until the container has
    /// been measured, so the first frame falls back to equal flex panels.
    sizes: Option<Vec<f32>>,
    /// Cached size of each collapsed panel, for restoring it.
    collapsed: Vec<Option<f32>>,
    drag: Option<Drag>,
    /// Cumulative divider offset while a lazy drag is in progress.
    lazy_offset: Option<f32>,
    /// Container size along the main axis seen at paint time.
    seen: Option<f32>,
}

#[derive(Clone)]
struct Drag {
    index: usize,
    pointer: Point<Pixels>,
    sizes: Vec<f32>,
}

struct Handlers {
    onresize_start: Option<ResizeHandler>,
    onresize: Option<ResizeHandler>,
    onresize_end: Option<ResizeHandler>,
    oncollapse: Option<CollapseHandler>,
}

impl Handlers {
    fn resize(
        &self,
        handler: &Option<ResizeHandler>,
        index: usize,
        sizes: &[f32],
        window: &mut Window,
        cx: &mut App,
    ) {
        if let Some(handler) = handler {
            handler(
                &SplitterResizeEvent {
                    index,
                    sizes: Rc::from(sizes.to_vec()),
                },
                window,
                cx,
            );
        }
    }
}

/// Moves `offset` pixels between two adjacent panels, honoring their limits.
/// Returns the offset that could actually be applied.
fn drag_sizes(sizes: &mut [f32], index: usize, offset: f32, limits: &[(f32, f32)]) -> f32 {
    let (min_a, max_a) = limits[index];
    let (min_b, max_b) = limits[index + 1];
    let mut applied = offset;
    if sizes[index] + applied < min_a {
        applied = min_a - sizes[index];
    }
    if sizes[index + 1] - applied < min_b {
        applied = sizes[index + 1] - min_b;
    }
    if sizes[index] + applied > max_a {
        applied = max_a - sizes[index];
    }
    if sizes[index + 1] - applied > max_b {
        applied = sizes[index + 1] - max_b;
    }
    sizes[index] += applied;
    sizes[index + 1] -= applied;
    applied
}

/// Collapses one side of a divider, or restores it on a second activation,
/// like the upstream `onCollapse` handler.
fn collapse_sizes(
    sizes: &mut [f32],
    collapsed: &mut [Option<f32>],
    index: usize,
    side: SplitterSide,
) {
    let current = if side == SplitterSide::Start {
        index
    } else {
        index + 1
    };
    let target = if side == SplitterSide::Start {
        index + 1
    } else {
        index
    };
    let current_size = sizes[current];
    let target_size = sizes[target];
    if current_size != 0. && target_size != 0. {
        sizes[current] = 0.;
        sizes[target] += current_size;
        collapsed[index] = Some(current_size);
    } else {
        let total = current_size + target_size;
        let restored = collapsed[index].unwrap_or(total / 2.).clamp(0., total);
        sizes[target] = restored;
        sizes[current] = total - restored;
    }
}

/// Scales every size by the same ratio so the panels keep filling the
/// container, keeping collapsed panels at zero.
fn fit_container(sizes: &mut [f32], container: f32) {
    let sum: f32 = sizes.iter().copied().sum();
    if sum <= 0. || (sum - container).abs() < 0.5 {
        return;
    }
    let scale = container / sum;
    for size in sizes.iter_mut() {
        *size = (*size * scale).max(0.);
    }
}

/// First-size policy: explicit sizes first, the remainder split equally, and
/// everything clamped to the panel limits. Panels only shrink to fit the
/// container; a minimum that keeps the sum short leaves the rest empty.
fn initial_sizes(explicit: &[Option<f32>], container: f32, limits: &[(f32, f32)]) -> Vec<f32> {
    let reserved: f32 = explicit.iter().filter_map(|size| *size).sum();
    let automatic = explicit.iter().filter(|size| size.is_none()).count();
    let shared = if automatic > 0 {
        (container - reserved).max(0.) / automatic as f32
    } else {
        0.
    };
    let mut sizes: Vec<f32> = explicit
        .iter()
        .enumerate()
        .map(|(index, size)| size.unwrap_or(shared).clamp(limits[index].0, limits[index].1))
        .collect();
    let sum: f32 = sizes.iter().copied().sum();
    if sum > container {
        fit_container(&mut sizes, container);
    }
    sizes
}

/// Creates a Splitter with the default horizontal layout.
#[track_caller]
pub fn el_splitter() -> Splitter {
    Splitter::default().id(ElementId::CodeLocation(*std::panic::Location::caller()))
}

/// Divide an area into resizable panels.
///
/// Native differences: sizes are plain logical pixels instead of the upstream
/// mix of pixels and percentages, and `lazy` moves only the divider line
/// during a drag by keeping the offset in the component state. The collapse
/// arrows are rendered at low opacity rather than only on hover, and the
/// dividers are keyboard-focusable, which the upstream `div` is not.
#[derive(Default, IntoElement)]
#[must_use]
pub struct Splitter {
    id: Option<ElementId>,
    layout: SplitterLayout,
    lazy: bool,
    panels: Vec<SplitterPanel>,
    onresize_start: Option<ResizeHandler>,
    onresize: Option<ResizeHandler>,
    onresize_end: Option<ResizeHandler>,
    oncollapse: Option<CollapseHandler>,
}

impl Splitter {
    /// Creates a splitter with a stable GPUI element ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self::default().id(id)
    }

    /// Enables state tracking for this ID.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the split direction; horizontal by default.
    pub fn layout(mut self, layout: SplitterLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Lays the panels out top to bottom.
    pub fn vertical(self) -> Self {
        self.layout(SplitterLayout::Vertical)
    }

    /// Lays the panels out side by side.
    pub fn horizontal(self) -> Self {
        self.layout(SplitterLayout::Horizontal)
    }

    /// Applies the new sizes only when the drag ends, moving just the
    /// divider line while the pointer is down.
    pub fn lazy(mut self, lazy: bool) -> Self {
        self.lazy = lazy;
        self
    }

    /// Adds a panel.
    pub fn panel(mut self, panel: SplitterPanel) -> Self {
        self.panels.push(panel);
        self
    }

    /// Adds several panels.
    pub fn panels(mut self, panels: impl IntoIterator<Item = SplitterPanel>) -> Self {
        self.panels.extend(panels);
        self
    }

    /// Fires when a divider drag starts, with the divider index and sizes.
    #[track_caller]
    pub fn onresize_start(
        mut self,
        handler: impl Fn(&SplitterResizeEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onresize_start = Some(Box::new(handler));
        self
    }

    /// Fires while a divider moves, with the divider index and sizes.
    #[track_caller]
    pub fn onresize(
        mut self,
        handler: impl Fn(&SplitterResizeEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onresize = Some(Box::new(handler));
        self
    }

    /// Fires when a divider drag ends, with the divider index and sizes.
    #[track_caller]
    pub fn onresize_end(
        mut self,
        handler: impl Fn(&SplitterResizeEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onresize_end = Some(Box::new(handler));
        self
    }

    /// Fires when a panel collapses or expands through a divider arrow.
    #[track_caller]
    pub fn oncollapse(
        mut self,
        handler: impl Fn(&SplitterCollapseEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.oncollapse = Some(Box::new(handler));
        self
    }
}

struct Divider {
    id: ElementId,
    group: SharedString,
    index: usize,
    horizontal: bool,
    resizable: bool,
    start_collapsible: bool,
    end_collapsible: bool,
    lazy_offset: Option<f32>,
    line_color: gpui::Hsla,
    active_color: gpui::Hsla,
    arrow_color: gpui::Hsla,
    state: Entity<SplitterState>,
    handlers: Rc<Handlers>,
    limits: Vec<(f32, f32)>,
    owner: gpui::EntityId,
}

fn divider(info: Divider) -> impl IntoElement {
    // Keep the lazy line inside the hit area.
    let line_offset = info.lazy_offset.unwrap_or(0.).clamp(-BAR / 2., BAR / 2.);
    let line = div()
        .absolute()
        .bg(info.line_color)
        .when(info.horizontal, |el| {
            el.left(px(line_offset - 0.5)).top_0().bottom_0().w(px(1.))
        })
        .when(!info.horizontal, |el| {
            el.top(px(line_offset - 0.5)).left_0().right_0().h(px(1.))
        })
        .group_hover(info.group.clone(), move |style| {
            style.bg(info.active_color)
        });
    let arrow = |side: SplitterSide, icon: &'static str| {
        let state = info.state.clone();
        let handlers = info.handlers.clone();
        let index = info.index;
        let owner = info.owner;
        let suffix = if side == SplitterSide::Start { "s" } else { "e" };
        div()
            .id((info.id.clone(), format!("arrow-{suffix}")))
            .absolute()
            .flex()
            .items_center()
            .justify_center()
            .w(px(12.))
            .h(px(12.))
            .rounded(px(tokens::radius::SMALL))
            .text_color(info.arrow_color)
            .opacity(0.35)
            .hover(move |style| style.opacity(1.))
            .group_hover(info.group.clone(), move |style| style.opacity(1.))
            .when(info.horizontal, |el| {
                if side == SplitterSide::Start {
                    el.left(px(-ARROW_GAP - 12.)).top_0().bottom_0()
                } else {
                    el.right(px(-ARROW_GAP - 12.)).top_0().bottom_0()
                }
            })
            .when(!info.horizontal, |el| {
                if side == SplitterSide::Start {
                    el.top(px(-ARROW_GAP - 12.)).left_0().right_0()
                } else {
                    el.bottom(px(-ARROW_GAP - 12.)).left_0().right_0()
                }
            })
            .child(el_icon(icon).size(12.))
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                let mut sizes = state.read(cx).sizes.clone().unwrap_or_default();
                let mut collapsed = state.read(cx).collapsed.clone();
                if sizes.len() > index + 1 {
                    collapse_sizes(&mut sizes, &mut collapsed, index, side);
                    if let Some(handler) = &handlers.oncollapse {
                        handler(
                            &SplitterCollapseEvent {
                                index,
                                side,
                                sizes: Rc::from(sizes.clone()),
                            },
                            window,
                            cx,
                        );
                    }
                    state.update(cx, |state, _| {
                        state.sizes = Some(sizes);
                        state.collapsed = collapsed;
                    });
                }
                cx.notify(owner);
                window.refresh();
                cx.stop_propagation();
            })
    };
    let down_state = info.state.clone();
    let key_state = info.state.clone();
    let key_limits = info.limits.clone();
    let key_handlers = info.handlers.clone();
    let key_owner = info.owner;
    let key_index = info.index;
    let key_horizontal = info.horizontal;
    // The anchor takes no layout space, like upstream's zero-width bar wrap,
    // so the panels keep filling the container and the hit area overlaps
    // both edges of the split.
    div()
        .group(info.group.clone())
        .flex_none()
        .relative()
        .when(info.horizontal, |el| el.w(px(0.)))
        .when(!info.horizontal, |el| el.h(px(0.)))
        .when(
            info.start_collapsible,
            |el| {
                el.child(arrow(
                    SplitterSide::Start,
                    if info.horizontal { "arrow-left" } else { "arrow-up" },
                ))
            },
        )
        .when(
            info.end_collapsible,
            |el| {
                el.child(arrow(
                    SplitterSide::End,
                    if info.horizontal { "arrow-right" } else { "arrow-down" },
                ))
            },
        )
        .child(line)
        .when(info.resizable, |el| {
            el.child(
                div()
                    .id((info.id.clone(), format!("bar{}", info.index)))
                    .absolute()
                    .cursor(if info.horizontal {
                        gpui::CursorStyle::ResizeLeftRight
                    } else {
                        gpui::CursorStyle::ResizeUpDown
                    })
                    .when(info.horizontal, |el| {
                        el.left(px(-BAR / 2.)).top_0().bottom_0().w(px(BAR))
                    })
                    .when(!info.horizontal, |el| {
                        el.top(px(-BAR / 2.)).left_0().right_0().h(px(BAR))
                    })
                    .focusable()
                    .tab_stop(true)
                    .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                        let sizes = down_state.read(cx).sizes.clone().unwrap_or_default();
                        down_state.update(cx, |state, _| {
                            state.drag = Some(Drag {
                                index: key_index,
                                pointer: event.position,
                                sizes,
                            });
                        });
                        cx.stop_propagation();
                    })
                    .on_key_down(move |event: &KeyDownEvent, window, cx| {
                        let forward = match (key_horizontal, event.keystroke.key.as_str()) {
                            (true, "right") | (false, "down") => true,
                            (true, "left") | (false, "up") => false,
                            _ => return,
                        };
                        let step = if event.keystroke.modifiers.shift { 40. } else { STEP };
                        let offset = if forward { step } else { -step };
                        let mut sizes = key_state.read(cx).sizes.clone().unwrap_or_default();
                        if sizes.len() <= key_index + 1 {
                            return;
                        }
                        drag_sizes(&mut sizes, key_index, offset, &key_limits);
                        if let Some(handler) = &key_handlers.onresize {
                            handler(
                                &SplitterResizeEvent {
                                    index: key_index,
                                    sizes: Rc::from(sizes.clone()),
                                },
                                window,
                                cx,
                            );
                        }
                        key_state.update(cx, |state, _| state.sizes = Some(sizes));
                        cx.notify(key_owner);
                        window.refresh();
                    }),
            )
        })
}

impl RenderOnce for Splitter {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self
            .id
            .unwrap_or_else(|| ElementId::CodeLocation(*std::panic::Location::caller()));
        let state = window.use_keyed_state(id.clone(), cx, |_, _| SplitterState::default());
        let owner = window.current_view();
        let theme = Theme::current(cx);
        let line_color: gpui::Hsla = rgb(theme.border(true)).into();
        let active_color: gpui::Hsla = rgb(crate::theme::PRIMARY).into();
        let arrow_color: gpui::Hsla = rgb(theme.secondary_text()).into();
        let horizontal = self.layout == SplitterLayout::Horizontal;
        let count = self.panels.len();
        let limits: Vec<(f32, f32)> = self
            .panels
            .iter()
            .map(|panel| (panel.min.unwrap_or(0.), panel.max.unwrap_or(f32::INFINITY)))
            .collect();
        let resizable: Vec<bool> = self.panels.iter().map(|panel| panel.resizable).collect();
        let collapsible: Vec<bool> = self.panels.iter().map(|panel| panel.collapsible).collect();
        let explicit: Vec<Option<f32>> = self.panels.iter().map(|panel| panel.size).collect();
        let group: SharedString = format!("{id}-bar").into();
        let handlers = Rc::new(Handlers {
            onresize_start: self.onresize_start,
            onresize: self.onresize,
            onresize_end: self.onresize_end,
            oncollapse: self.oncollapse,
        });
        let lazy = self.lazy;

        let sizes = state.read(cx).sizes.clone();
        let lazy_offset = state.read(cx).lazy_offset;
        let mut root = div()
            .id(id.clone())
            .flex()
            .when(!horizontal, |el| el.flex_col())
            .size_full()
            .min_w_0()
            .min_h_0();

        for (index, panel) in self.panels.into_iter().enumerate() {
            let sized = sizes.as_ref().map(|sizes| sizes[index]);
            root = root.child(
                div()
                    .flex_none()
                    .min_w_0()
                    .min_h_0()
                    .overflow_hidden()
                    .when_some(sized, |el, size| {
                        if horizontal {
                            el.w(px(size))
                        } else {
                            el.h(px(size))
                        }
                    })
                    .when(sized.is_none(), |el| el.flex_1())
                    .children(panel.children),
            );
            if index + 1 < count {
                root = root.child(divider(Divider {
                    id: id.clone(),
                    group: group.clone(),
                    index,
                    horizontal,
                    resizable: resizable[index] && resizable[index + 1],
                    start_collapsible: collapsible[index],
                    end_collapsible: collapsible[index + 1],
                    lazy_offset,
                    line_color,
                    active_color,
                    arrow_color,
                    state: state.clone(),
                    handlers: handlers.clone(),
                    limits: limits.clone(),
                    owner,
                }));
            }
        }

        // The container is only measurable at paint time; the first paint
        // initializes the sizes and later resizes rescale them, both through
        // another render of the owning view.
        let sync_state = state.clone();
        let sync_explicit = explicit.clone();
        let sync_limits = limits.clone();
        root = root.child(
            gpui::canvas(
                move |_, _, _| {},
                move |bounds, _, window, cx| {
                    let container = if horizontal {
                        bounds.size.width.as_f32()
                    } else {
                        bounds.size.height.as_f32()
                    };
                    if sync_state.read(cx).seen == Some(container) {
                        return;
                    }
                    sync_state.update(cx, |state, _| {
                        state.seen = Some(container);
                        if state.sizes.is_none() {
                            state.sizes =
                                Some(initial_sizes(&sync_explicit, container, &sync_limits));
                            state.collapsed = vec![None; sync_explicit.len().saturating_sub(1)];
                        } else if let Some(sizes) = state.sizes.as_mut() {
                            fit_container(sizes, container);
                        }
                    });
                    let owner = window.current_view();
                    cx.notify(owner);
                    window.refresh();
                },
            )
            .absolute()
            .inset_0(),
        );

        let move_state = state.clone();
        let move_handlers = handlers.clone();
        let move_limits = limits.clone();
        let up_state = state.clone();
        let up_handlers = handlers.clone();
        let up_limits = limits.clone();
        root = root
            .on_mouse_move(move |event, window, cx| {
                let drag = move_state.read(cx).drag.clone();
                let Some(drag) = drag else {
                    return;
                };
                let offset = if horizontal {
                    event.position.x.as_f32() - drag.pointer.x.as_f32()
                } else {
                    event.position.y.as_f32() - drag.pointer.y.as_f32()
                };
                let mut sizes = drag.sizes.clone();
                let applied = drag_sizes(&mut sizes, drag.index, offset, &move_limits);
                if lazy {
                    move_state.update(cx, |state, _| state.lazy_offset = Some(applied));
                } else {
                    move_state.update(cx, |state, _| state.sizes = Some(sizes.clone()));
                }
                move_handlers.resize(&move_handlers.onresize, drag.index, &sizes, window, cx);
                cx.notify(owner);
                window.refresh();
            })
            .on_mouse_up(MouseButton::Left, move |_, window, cx| {
                let drag = up_state.read(cx).drag.clone();
                let Some(drag) = drag else {
                    return;
                };
                // A lazy drag kept the panels in place; apply the offset now.
                let mut sizes = if up_state.read(cx).lazy_offset.is_some() {
                    drag.sizes.clone()
                } else {
                    up_state.read(cx).sizes.clone().unwrap_or_default()
                };
                if let Some(offset) = up_state.read(cx).lazy_offset {
                    drag_sizes(&mut sizes, drag.index, offset, &up_limits);
                }
                up_state.update(cx, |state, _| {
                    state.sizes = Some(sizes.clone());
                    state.drag = None;
                    state.lazy_offset = None;
                });
                up_handlers.resize(&up_handlers.onresize_end, drag.index, &sizes, window, cx);
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
    fn drag_moves_size_between_neighbors_and_stops_at_limits() {
        let mut sizes = [300., 300.];
        let limits = [(100., 500.), (100., 500.)];
        assert_eq!(drag_sizes(&mut sizes, 0, 50., &limits), 50.);
        assert_eq!(sizes, [350., 250.]);
        // The left panel cannot exceed its maximum.
        assert_eq!(drag_sizes(&mut sizes, 0, 200., &limits), 150.);
        assert_eq!(sizes, [500., 100.]);
        // The right panel's minimum stops the divider.
        assert_eq!(drag_sizes(&mut sizes, 0, 10., &limits), 0.);
        assert_eq!(sizes, [500., 100.]);
        // Dragging back the other way.
        assert_eq!(drag_sizes(&mut sizes, 0, -400., &limits), -400.);
        assert_eq!(sizes, [100., 500.]);
    }

    #[test]
    fn collapse_swaps_size_with_the_neighbor_and_restores_it() {
        let mut sizes = [300., 300.];
        let mut collapsed = [None];
        collapse_sizes(&mut sizes, &mut collapsed, 0, SplitterSide::Start);
        assert_eq!(sizes, [0., 600.]);
        assert_eq!(collapsed, [Some(300.)]);
        // A second activation restores the cached size.
        collapse_sizes(&mut sizes, &mut collapsed, 0, SplitterSide::Start);
        assert_eq!(sizes, [300., 300.]);
        // Collapsing the end side moves size the other way.
        collapse_sizes(&mut sizes, &mut collapsed, 0, SplitterSide::End);
        assert_eq!(sizes, [600., 0.]);
    }

    #[test]
    fn initial_sizes_split_the_leftover_space() {
        let limits = [(0., f32::INFINITY); 3];
        let sizes = initial_sizes(&[Some(200.), None, None], 800., &limits);
        assert_eq!(sizes, [200., 300., 300.]);
        // A minimum keeps its panel short; the leftover stays empty.
        let limits = [(0., 100.), (0., f32::INFINITY)];
        let sizes = initial_sizes(&[None, None], 800., &limits);
        assert_eq!(sizes, [100., 400.]);
        // Explicit sizes that overflow the container shrink together.
        let limits = [(0., f32::INFINITY); 2];
        let sizes = initial_sizes(&[Some(300.), Some(300.)], 400., &limits);
        assert_eq!(sizes, [200., 200.]);
    }

    #[test]
    fn fit_container_scales_proportions() {
        let mut sizes = [200., 600.];
        fit_container(&mut sizes, 400.);
        assert_eq!(sizes, [100., 300.]);
        // Already filling: unchanged.
        let mut sizes = [200., 600.];
        fit_container(&mut sizes, 800.);
        assert_eq!(sizes, [200., 600.]);
    }

    #[gpui::test]
    fn dragging_and_collapsing_a_splitter(cx: &mut gpui::TestAppContext) {
        struct Harness {
            sizes: Rc<RefCell<Vec<Vec<f32>>>>,
            collapses: Rc<RefCell<Vec<(usize, SplitterSide)>>>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let sizes = self.sizes.clone();
                let collapses = self.collapses.clone();
                el_splitter()
                    .id("splitter")
                    .onresize(move |event, _, _| sizes.borrow_mut().push(event.sizes.to_vec()))
                    .oncollapse(move |event, _, _| {
                        collapses.borrow_mut().push((event.index, event.side))
                    })
                    .panel(
                        el_splitter_panel()
                            .collapsible(true)
                            .child(div().bg(rgb(0x409eff)).size_full()),
                    )
                    .panel(
                        el_splitter_panel()
                            .collapsible(true)
                            .child(div().bg(rgb(0x67c23a)).size_full()),
                    )
            }
        }
        let sizes: Rc<RefCell<Vec<Vec<f32>>>> = Rc::new(RefCell::new(Vec::new()));
        let collapses: Rc<RefCell<Vec<(usize, SplitterSide)>>> = Rc::new(RefCell::new(Vec::new()));
        let (_, view) = cx.add_window_view({
            let sizes = sizes.clone();
            let collapses = collapses.clone();
            move |_, _| Harness { sizes, collapses }
        });
        for _ in 0..2 {
            view.update(|window, cx| window.draw(cx).clear(cx));
            view.run_until_parked();
        }
        // Two equal panels fill the window; the divider sits at the middle.
        let width = view.update(|window, _| window.viewport_size().width).as_f32();
        let half = width / 2.;
        view.simulate_mouse_down(
            gpui::point(px(half), px(10.)),
            MouseButton::Left,
            Default::default(),
        );
        view.simulate_mouse_move(
            gpui::point(px(half + 100.), px(10.)),
            None,
            Default::default(),
        );
        view.run_until_parked();
        view.simulate_mouse_up(
            gpui::point(px(half + 100.), px(10.)),
            MouseButton::Left,
            Default::default(),
        );
        view.run_until_parked();
        view.update(|window, cx| window.draw(cx).clear(cx));
        let events = sizes.borrow();
        assert!(!events.is_empty(), "dragging should report sizes");
        assert_eq!(events[events.len() - 1], [half + 100., half - 100.]);

        // The start collapse arrow sits just left of the divider, which the
        // drag moved 100 px to the right.
        view.simulate_mouse_down(
            gpui::point(px(half + 100. - ARROW_GAP - 6.), px(10.)),
            MouseButton::Left,
            Default::default(),
        );
        view.run_until_parked();
        view.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(*collapses.borrow(), vec![(0, SplitterSide::Start)]);
    }
}
