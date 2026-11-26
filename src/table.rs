//! Element Plus Table, basic subset.
//!
//! Follows `table.md`, `components/table/src`, and Theme Chalk `table.scss`:
//! columns with fixed or distributed widths, header and cell alignment,
//! striped rows, borders, three sizes, a scrolling body with a fixed header,
//! status row backgrounds, row clicks, and an empty state. Text selection and
//! wheel containment are native additions: dragging inside text cells selects
//! and copies plain text, and the scrolling body never forwards wheel deltas to
//! an outer scroll container. Sorting, row selection, filtering, fixed columns,
//! tree data, lazy loading, and summary rows are not reproduced.

use std::{
    cell::{Cell as BoundsCell, RefCell},
    ops::Range,
    panic::Location,
    rc::Rc,
    sync::Arc,
};

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Div, ElementId, Entity, FocusHandle, Hsla, IntoElement,
    KeyDownEvent, MouseButton, Pixels, Point, RenderOnce, SharedString, StyledText, TextLayout,
    Window, div, prelude::*, px, rgb, rgba,
};

use crate::{Theme, tokens};

/// Cell padding and font size, from `$table-padding` and `$table-font-size`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TableSize {
    Large,
    #[default]
    Default,
    Small,
}

impl TableSize {
    fn vertical_padding(self) -> f32 {
        match self {
            TableSize::Large => 12.,
            TableSize::Default => 8.,
            TableSize::Small => 4.,
        }
    }
    fn font_size(self) -> f32 {
        match self {
            TableSize::Large | TableSize::Default => tokens::font_size::BASE,
            TableSize::Small => tokens::font_size::SMALL,
        }
    }
}

/// Horizontal alignment of cell content, from `align` and `header-align`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TableAlign {
    #[default]
    Left,
    Center,
    Right,
}

fn aligned(align: TableAlign, el: Div) -> Div {
    match align {
        TableAlign::Left => el,
        TableAlign::Center => el.text_center(),
        TableAlign::Right => el.text_right(),
    }
}

fn justified(align: TableAlign, el: Div) -> Div {
    match align {
        TableAlign::Left => el,
        TableAlign::Center => el.justify_center(),
        TableAlign::Right => el.justify_end(),
    }
}

/// A byte offset inside one text cell. Ordered row-major so selection
/// endpoints compare directly and ranges span cells in reading order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CellPos {
    row: usize,
    column: usize,
    offset: usize,
}

/// One rendered text cell: its text, measured layout, and painted bounds.
struct CellText {
    row: usize,
    column: usize,
    text: SharedString,
    layout: TextLayout,
    bounds: Rc<BoundsCell<Option<Bounds<Pixels>>>>,
}

/// Selected text, focus, and drag state for one table.
struct Selection {
    focus: FocusHandle,
    anchor: Option<CellPos>,
    head: Option<CellPos>,
    dragging: bool,
    /// Set once a drag produced a non-empty range. The row click is suppressed
    /// for that gesture so dragging text does not also activate the row.
    dragged: bool,
    cells: Rc<RefCell<Vec<CellText>>>,
}

impl Selection {
    fn new(cx: &mut App) -> Self {
        Self {
            focus: cx.focus_handle(),
            anchor: None,
            head: None,
            dragging: false,
            dragged: false,
            cells: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Selection endpoints in reading order.
    fn range(&self) -> Option<(CellPos, CellPos)> {
        let anchor = self.anchor?;
        let head = self.head?;
        Some(if anchor <= head {
            (anchor, head)
        } else {
            (head, anchor)
        })
    }

    /// Maps a window position back to the cell and byte offset under it. Cells
    /// without recorded paint bounds are skipped, so an event that arrives
    /// before the first paint cannot reach an unmeasured layout.
    fn position_at(&self, position: Point<Pixels>) -> Option<CellPos> {
        let cells = self.cells.borrow();
        for cell in cells.iter().rev() {
            let Some(bounds) = cell.bounds.get() else {
                continue;
            };
            if !bounds.contains(&position) {
                continue;
            }
            let offset = cell
                .layout
                .index_for_position(position)
                .unwrap_or_else(|edge| edge)
                .min(cell.text.len());
            return Some(CellPos {
                row: cell.row,
                column: cell.column,
                offset: cell.text.floor_char_boundary(offset),
            });
        }
        None
    }

    /// Selected text as tab-separated cells and newline-separated rows.
    fn selected_text(&self) -> Option<String> {
        let (start, end) = self.range()?;
        if start == end {
            return None;
        }
        let cells = self.cells.borrow();
        let mut picked: Vec<(usize, usize, String)> = Vec::new();
        for cell in cells.iter() {
            let Some(range) =
                local_range(Some((start, end)), cell.row, cell.column, cell.text.len())
            else {
                continue;
            };
            picked.push((
                cell.row,
                cell.column,
                slice_text(&cell.text, range).to_owned(),
            ));
        }
        picked.sort_by_key(|(row, column, _)| (*row, *column));
        let mut text = String::new();
        let mut row = None;
        for (cell_row, _, part) in picked {
            match row {
                Some(previous) if previous == cell_row => text.push('\t'),
                Some(_) => text.push('\n'),
                None => {}
            }
            text.push_str(&part);
            row = Some(cell_row);
        }
        (!text.is_empty()).then_some(text)
    }
}

/// Selected byte range inside one cell, or `None` when the cell is outside the
/// selection.
fn local_range(
    range: Option<(CellPos, CellPos)>,
    row: usize,
    column: usize,
    len: usize,
) -> Option<Range<usize>> {
    let (start, end) = range?;
    let (first, last) = ((start.row, start.column), (end.row, end.column));
    if (row, column) < first || (row, column) > last {
        return None;
    }
    let from = if (row, column) == first {
        start.offset
    } else {
        0
    };
    let to = if (row, column) == last {
        end.offset
    } else {
        len
    };
    let from = from.min(len);
    let to = to.min(len).max(from);
    (from < to).then_some(from..to)
}

/// Slices a cell at layout-reported byte offsets, widening to character
/// boundaries so a mid-glyph offset never panics.
fn slice_text(text: &str, range: Range<usize>) -> &str {
    let mut start = range.start.min(text.len());
    let mut end = range.end.min(text.len());
    while start > 0 && !text.is_char_boundary(start) {
        start -= 1;
    }
    while end < text.len() && !text.is_char_boundary(end) {
        end += 1;
    }
    if end < start {
        end = start;
    }
    &text[start..end]
}

/// Per-table wiring shared by the text cells (ids, selection, highlight).
#[derive(Clone)]
struct Cells {
    selection: Option<Entity<Selection>>,
    registry: Option<Rc<RefCell<Vec<CellText>>>>,
    range: Option<(CellPos, CellPos)>,
    highlight: Hsla,
}

type Text<T> = Box<dyn Fn(&T, usize) -> SharedString>;
type Cell<T> = Box<dyn Fn(&T, usize) -> AnyElement>;
type Header = Box<dyn Fn() -> AnyElement>;
type RowBackground<T> = Box<dyn Fn(&T, usize) -> Option<Hsla>>;
type RowClick<T> = Rc<dyn Fn(&T, usize, &ClickEvent, &mut Window, &mut App)>;

/// One table column, like `el-table-column`.
///
/// Columns with `width` keep that size; columns with `min_width` share the
/// remaining space proportionally, as upstream does; columns with neither grow
/// equally. Cell content comes from `text` or, for custom content, `render`.
pub struct Column<T: 'static> {
    label: SharedString,
    width: Option<f32>,
    min_width: Option<f32>,
    align: TableAlign,
    header_align: Option<TableAlign>,
    text: Option<Text<T>>,
    render: Option<Cell<T>>,
    header: Option<Header>,
}

impl<T: 'static> Column<T> {
    /// Creates a column with the given header label.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            width: None,
            min_width: None,
            align: TableAlign::Left,
            header_align: None,
            text: None,
            render: None,
            header: None,
        }
    }

    /// Fixes the column width in logical pixels.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets the minimum width; leftover space is distributed proportionally.
    pub fn min_width(mut self, min_width: f32) -> Self {
        self.min_width = Some(min_width);
        self
    }

    /// Aligns this column's cells. Defaults to left.
    pub fn align(mut self, align: TableAlign) -> Self {
        self.align = align;
        self
    }

    /// Aligns this column's header; cells fall back to `align`.
    pub fn header_align(mut self, align: TableAlign) -> Self {
        self.header_align = Some(align);
        self
    }

    /// Renders the cell text. The default when neither `text` nor `render`
    /// is set is an empty cell.
    pub fn text(mut self, text: impl Fn(&T, usize) -> SharedString + 'static) -> Self {
        self.text = Some(Box::new(text));
        self
    }

    /// Renders custom cell content, replacing `text`.
    pub fn render(mut self, render: impl Fn(&T, usize) -> AnyElement + 'static) -> Self {
        self.render = Some(Box::new(render));
        self
    }

    /// Replaces the header label with custom content.
    pub fn header(mut self, header: impl Fn() -> AnyElement + 'static) -> Self {
        self.header = Some(Box::new(header));
        self
    }
}

/// An Element Plus table over typed rows.
///
/// Native differences: there is no DOM, so `span-method`, `row-class-name`,
/// and CSS class props become the `row_background` color hook; sorting,
/// selection, filtering, fixed columns, tree data, lazy loading, and summary
/// rows are not reproduced. `show-overflow-tooltip` is unnecessary because
/// text cells clip with an ellipsis by default.
#[derive(IntoElement)]
pub struct Table<T: 'static> {
    rows: Vec<T>,
    columns: Vec<Column<T>>,
    size: TableSize,
    stripe: bool,
    border: bool,
    show_header: bool,
    height: Option<f32>,
    max_height: Option<f32>,
    empty: Option<AnyElement>,
    row_background: Option<RowBackground<T>>,
    onrowclick: Option<RowClick<T>>,
    key: ElementId,
    selectable: bool,
}

impl<T: 'static> Table<T> {
    /// Creates a table with the given rows.
    ///
    /// The construction site doubles as the table's element key, so several
    /// tables created at the same source location should call [`Self::id`] to
    /// keep their hover and selection state apart.
    #[track_caller]
    pub fn new(rows: impl IntoIterator<Item = T>) -> Self {
        Self {
            rows: rows.into_iter().collect(),
            columns: Vec::new(),
            size: TableSize::Default,
            stripe: false,
            border: false,
            show_header: true,
            height: None,
            max_height: None,
            empty: None,
            row_background: None,
            onrowclick: None,
            key: ElementId::CodeLocation(*Location::caller()),
            selectable: true,
        }
    }

    /// Sets a stable element key; required when several tables are built at
    /// one call site or need distinct hover and selection state.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.key = id.into();
        self
    }

    /// Allows dragging to select cell text and `Ctrl+C`/`Cmd+C` to copy it.
    /// Enabled by default; custom `render` cells are not selectable.
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    pub fn column(mut self, column: Column<T>) -> Self {
        self.columns.push(column);
        self
    }

    pub fn size(mut self, size: TableSize) -> Self {
        self.size = size;
        self
    }

    /// Alternates row backgrounds, like `stripe`.
    pub fn stripe(mut self, stripe: bool) -> Self {
        self.stripe = stripe;
        self
    }

    /// Draws vertical cell borders, like `border`.
    pub fn border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }

    pub fn show_header(mut self, show_header: bool) -> Self {
        self.show_header = show_header;
        self
    }

    /// Fixes the table height and keeps the header visible while the body
    /// scrolls, like `height`.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Caps the body height so the header stays visible while it scrolls,
    /// like `max-height`.
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = Some(max_height);
        self
    }

    /// Sets the text shown when there are no rows.
    pub fn empty_text(mut self, text: impl Into<SharedString>) -> Self {
        self.empty = Some(
            div()
                .py(px(20.))
                .text_center()
                .child(text.into())
                .into_any_element(),
        );
        self
    }

    /// Replaces the empty state with custom content.
    pub fn empty(mut self, empty: impl IntoElement) -> Self {
        self.empty = Some(empty.into_any_element());
        self
    }

    /// Paints a status background for chosen rows, like `row-class-name`.
    pub fn row_background(
        mut self,
        row_background: impl Fn(&T, usize) -> Option<Hsla> + 'static,
    ) -> Self {
        self.row_background = Some(Box::new(row_background));
        self
    }

    /// Handles a row click.
    pub fn onrowclick(
        mut self,
        handler: impl Fn(&T, usize, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onrowclick = Some(Rc::new(handler));
        self
    }
}

fn header_cell<T: 'static>(
    column: &Column<T>,
    border: bool,
    size: TableSize,
    theme: &Theme,
) -> Div {
    let line = rgb(theme.border_lighter());
    let align = column.header_align.unwrap_or(column.align);
    let inner = div()
        .flex_1()
        .min_w_0()
        .px(px(tokens::spacing::DEFAULT))
        .line_height(px(23.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(theme.secondary_text()));
    let inner = match &column.header {
        Some(header) => justified(
            align,
            inner
                .flex()
                .items_center()
                // Constrain custom content so it wraps inside the cell.
                .child(div().min_w_0().child(header())),
        ),
        None => aligned(align, inner.child(column.label.clone())),
    };
    div()
        .flex()
        .items_center()
        .min_w_0()
        .when_some(column.width, |el, width| el.w(px(width)).flex_none())
        .when_some(column.min_width, |el, min| el.flex_1().min_w(px(min)))
        .when(column.width.is_none() && column.min_width.is_none(), |el| {
            el.flex_1()
        })
        .when(border, |el| el.border_r_1().border_color(line))
        .overflow_hidden()
        .py(px(size.vertical_padding()))
        .child(inner)
}

/// Shared geometry and colors for one table's cells.
struct CellStyle<'a> {
    border: bool,
    size: TableSize,
    theme: &'a Theme,
}

fn body_cell<T: 'static>(
    column: &Column<T>,
    row: &T,
    index: usize,
    column_index: usize,
    style: &CellStyle<'_>,
    cells: &Cells,
) -> Div {
    let CellStyle {
        border,
        size,
        theme,
    } = *style;
    let line = rgb(theme.border_lighter());
    let inner = div()
        .flex_1()
        .min_w_0()
        .px(px(tokens::spacing::DEFAULT))
        .line_height(px(23.));
    let inner = match (&column.render, &column.text) {
        (Some(render), _) => justified(
            column.align,
            inner
                .flex()
                .items_center()
                // Constrain custom content so it wraps inside the cell instead
                // of drawing over the next column.
                .child(div().min_w_0().child(render(row, index))),
        ),
        (None, Some(text)) => {
            let text = text(row, index);
            let highlights = local_range(cells.range, index, column_index, text.len())
                .map(|range| {
                    vec![(
                        range,
                        gpui::HighlightStyle {
                            background_color: Some(cells.highlight),
                            ..gpui::HighlightStyle::default()
                        },
                    )]
                })
                .unwrap_or_default();
            let styled = StyledText::new(text.clone()).with_highlights(highlights);
            let bounds = Rc::new(BoundsCell::new(None));
            if let Some(registry) = cells.registry.clone() {
                registry.borrow_mut().push(CellText {
                    row: index,
                    column: column_index,
                    text,
                    layout: styled.layout().clone(),
                    bounds: bounds.clone(),
                });
            }
            let selectable = cells.selection.is_some();
            aligned(
                column.align,
                inner
                    .child(styled)
                    .when(selectable, |el| el.relative())
                    .when(selectable, |el| {
                        // The canvas only records painted bounds; the text
                        // itself keeps painting and handling selection.
                        el.child(
                            gpui::canvas(
                                move |painted, _, _| bounds.set(Some(painted)),
                                |_, _, _, _| {},
                            )
                            .absolute()
                            .inset_0(),
                        )
                    }),
            )
        }
        (None, None) => inner,
    };
    div()
        .flex()
        .items_center()
        .min_w_0()
        .when_some(column.width, |el, width| el.w(px(width)).flex_none())
        .when_some(column.min_width, |el, min| el.flex_1().min_w(px(min)))
        .when(column.width.is_none() && column.min_width.is_none(), |el| {
            el.flex_1()
        })
        .when(border, |el| el.border_r_1().border_color(line))
        .overflow_hidden()
        .py(px(size.vertical_padding()))
        .child(inner)
}

impl<T: 'static> RenderOnce for Table<T> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Table {
            rows,
            columns,
            size,
            stripe: striped,
            border: bordered,
            show_header,
            height,
            max_height,
            empty,
            row_background,
            onrowclick,
            key,
            selectable,
        } = self;
        let theme = Theme::current(cx);
        let border = rgb(theme.border_lighter());
        // One step stronger than the web value so pointer feedback reads on
        // a white surface, close to KDE's row highlight.
        let hover = rgb(theme.fill(crate::theme::Fill::Base));
        let stripe = rgb(theme.fill(crate::theme::Fill::Lighter));
        let highlight: Hsla = rgba(if theme.dark { 0x409eff66 } else { 0x409eff40 }).into();
        let selection = selectable.then(|| {
            window.use_keyed_state(
                ElementId::NamedChild(Arc::new(key.clone()), "selection".into()),
                cx,
                |_, cx| Selection::new(cx),
            )
        });
        let (range, registry) = match selection.as_ref() {
            Some(selection) => {
                let state = selection.read(cx);
                let registry = state.cells.clone();
                // Layouts are rebuilt every frame; stale entries would map
                // pointer positions to cells that no longer exist.
                registry.borrow_mut().clear();
                (state.range(), Some(registry))
            }
            None => (None, None),
        };
        let cell_style = CellStyle {
            border: bordered,
            size,
            theme: &theme,
        };
        let cells = Cells {
            selection: selection.clone(),
            registry,
            range,
            highlight,
        };
        let header = show_header.then(|| {
            div()
                .flex()
                .flex_none()
                .border_b_1()
                .border_color(border)
                .children(
                    columns
                        .iter()
                        .map(|c| header_cell(c, bordered, size, &theme)),
                )
        });
        // Row data is shared with click handlers, which must be 'static.
        let rows = Rc::new(rows);
        let count = rows.len();
        let mut body = div().flex().flex_col().w_full();
        if count == 0 {
            body = body.child(empty.unwrap_or_else(|| {
                div()
                    .py(px(20.))
                    .text_center()
                    .text_color(rgb(theme.secondary_text()))
                    .child("No Data")
                    .into_any_element()
            }));
        } else {
            for (index, row) in rows.iter().enumerate() {
                let status = row_background.as_ref().and_then(|paint| paint(row, index));
                let is_striped = striped && index % 2 == 1;
                // GPUI applies hover styles through hitboxes, which need an id.
                let row_base = div()
                    .id(ElementId::NamedChild(
                        Arc::new(key.clone()),
                        format!("row-{index}").into(),
                    ))
                    .flex()
                    .w_full()
                    .flex_none()
                    .border_b_1()
                    .border_color(border)
                    .when_some(status, |el, color| el.bg(color))
                    .when(is_striped && status.is_none(), |el| el.bg(stripe))
                    .children(columns.iter().enumerate().map(|(column_index, column)| {
                        body_cell(column, row, index, column_index, &cell_style, &cells)
                    }));
                let row_el: AnyElement = match &onrowclick {
                    Some(handler) => {
                        let handler = handler.clone();
                        let rows = rows.clone();
                        let row_index = index;
                        let selection = selection.clone();
                        row_base
                            .cursor_pointer()
                            .hover(move |style| style.bg(hover))
                            .on_click(move |event, window, cx| {
                                // A drag that selected text owns the gesture, so
                                // the row does not also activate.
                                if selection
                                    .as_ref()
                                    .is_some_and(|selection| selection.read(cx).dragged)
                                {
                                    return;
                                }
                                handler(&rows[row_index], row_index, event, window, cx);
                                cx.stop_propagation();
                            })
                            .into_any_element()
                    }
                    None => row_base
                        .hover(move |style| style.bg(hover))
                        .into_any_element(),
                };
                body = body.child(row_el);
            }
        }
        // The header sits outside the scrolling body, so it stays fixed. The
        // body consumes wheel deltas after scrolling itself, so an outer page
        // or window never scrolls with the table.
        let body_id = ElementId::NamedChild(Arc::new(key.clone()), "body".into());
        let body: AnyElement = if height.is_some() {
            body.id(body_id)
                .flex_1()
                .min_h(px(0.))
                .overflow_y_scroll()
                .on_scroll(|_, _, cx| cx.stop_propagation())
                .into_any_element()
        } else if let Some(max_height) = max_height {
            body.id(body_id)
                .max_h(px(max_height))
                .overflow_y_scroll()
                .on_scroll(|_, _, cx| cx.stop_propagation())
                .into_any_element()
        } else {
            body.into_any_element()
        };
        let mut table = div()
            .id(ElementId::NamedChild(
                Arc::new(key.clone()),
                "surface".into(),
            ))
            .w_full()
            .flex()
            .flex_col()
            .text_size(px(size.font_size()))
            .text_color(rgb(theme.regular_text()))
            .bg(rgb(theme.background()))
            .when(bordered, |el| el.border_1().border_color(border))
            .when_some(height, |el, height| el.h(px(height)));
        if let Some(selection) = selection {
            let focus = selection.read(cx).focus.clone();
            table = table
                .track_focus(&focus)
                .cursor_text()
                .on_mouse_down(MouseButton::Left, {
                    let selection = selection.clone();
                    move |event, window, cx| {
                        let Some(position) = selection.read(cx).position_at(event.position) else {
                            return;
                        };
                        selection.update(cx, |state, cx| {
                            state.focus.focus(window, cx);
                            state.anchor = Some(position);
                            state.head = Some(position);
                            state.dragging = true;
                            state.dragged = false;
                        });
                        cx.stop_propagation();
                        window.refresh();
                    }
                })
                .on_mouse_move({
                    let selection = selection.clone();
                    move |event, window, cx| {
                        if !selection.read(cx).dragging {
                            return;
                        }
                        let Some(position) = selection.read(cx).position_at(event.position) else {
                            return;
                        };
                        let changed = selection.update(cx, |state, _| {
                            if state.head == Some(position) {
                                return false;
                            }
                            state.head = Some(position);
                            state.dragged = state.range().is_some_and(|(start, end)| start != end);
                            true
                        });
                        cx.stop_propagation();
                        if changed {
                            window.refresh();
                        }
                    }
                })
                .on_mouse_up(MouseButton::Left, {
                    let selection = selection.clone();
                    move |_, _, cx| {
                        selection.update(cx, |state, _| state.dragging = false);
                        cx.stop_propagation();
                    }
                })
                .on_mouse_up_out(MouseButton::Left, {
                    let selection = selection.clone();
                    move |_, _, cx| {
                        selection.update(cx, |state, _| state.dragging = false);
                    }
                })
                .on_key_down({
                    let selection = selection.clone();
                    move |event: &KeyDownEvent, _, cx| {
                        let keystroke = &event.keystroke;
                        if keystroke.key == "c" && keystroke.modifiers.secondary(cx) {
                            if let Some(text) = selection.read(cx).selected_text() {
                                cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
                            }
                            cx.stop_propagation();
                        } else if keystroke.key == "escape" {
                            selection.update(cx, |state, _| {
                                state.anchor = None;
                                state.head = None;
                                state.dragging = false;
                                state.dragged = false;
                            });
                            cx.stop_propagation();
                        }
                    }
                });
        }
        table
            .when_some(header, |el, header| el.child(header))
            .child(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_ranges_span_cells_and_slice_on_char_boundaries() {
        let start = CellPos {
            row: 0,
            column: 0,
            offset: 2,
        };
        let end = CellPos {
            row: 1,
            column: 1,
            offset: 1,
        };
        let range = Some((start, end));
        assert_eq!(local_range(range, 0, 0, 5), Some(2..5));
        assert_eq!(local_range(range, 0, 1, 4), Some(0..4));
        assert_eq!(local_range(range, 0, 2, 4), Some(0..4));
        assert_eq!(local_range(range, 1, 0, 3), Some(0..3));
        assert_eq!(local_range(range, 1, 1, 6), Some(0..1));
        assert_eq!(local_range(range, 1, 2, 6), None);
        assert_eq!(local_range(range, 0, 0, 2), None, "empty start cell range");
        // Wide glyphs: offsets inside a character widen to the boundary.
        assert_eq!(slice_text("确认一下", 0..1), "确");
        assert_eq!(slice_text("确认一下", 1..4), "确认");
        assert_eq!(slice_text("确认一下", 4..13), "认一下");
    }

    #[test]
    fn sizes_map_to_upstream_padding_and_font() {
        assert_eq!(TableSize::Large.vertical_padding(), 12.);
        assert_eq!(TableSize::Default.vertical_padding(), 8.);
        assert_eq!(TableSize::Small.vertical_padding(), 4.);
        assert_eq!(TableSize::Small.font_size(), 12.);
        assert_eq!(TableSize::Default.font_size(), 13.);
    }

    #[gpui::test]
    fn renders_header_and_rows_with_stripe_and_border(cx: &mut gpui::TestAppContext) {
        struct Harness;
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                Table::new([("2016-05-03", "Tom"), ("2016-05-02", "Tom")])
                    .border(true)
                    .stripe(true)
                    .column(
                        Column::new("Date")
                            .width(120.)
                            .text(|row: &(&str, &str), _| row.0.into()),
                    )
                    .column(
                        Column::new("Name")
                            .min_width(80.)
                            .text(|row: &(&str, &str), _| row.1.into()),
                    )
            }
        }
        let (_, view) = cx.add_window_view(|_, _| Harness);
        view.update(|window, cx| window.draw(cx).clear(cx));
    }

    #[gpui::test]
    fn row_clicks_report_their_index(cx: &mut gpui::TestAppContext) {
        use std::cell::Cell;
        struct Harness(Rc<Cell<usize>>);
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let clicks = self.0.clone();
                Table::new(["first", "second"])
                    .onrowclick(move |_, index, _, _, _| clicks.set(index))
                    .column(Column::new("Name").text(|row: &&str, _| (*row).into()))
            }
        }
        let clicks = Rc::new(Cell::new(0));
        let (_, view) = cx.add_window_view({
            let clicks = clicks.clone();
            move |_, _| Harness(clicks)
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        // Header and rows are 40 px tall: click the second row.
        view.simulate_click(gpui::point(px(10.), px(100.)), gpui::Modifiers::default());
        assert_eq!(clicks.get(), 1, "the second row reports index 1");
    }

    #[gpui::test]
    fn dragging_text_suppresses_the_row_click(cx: &mut gpui::TestAppContext) {
        use std::cell::Cell as StdCell;
        struct Harness(Rc<StdCell<usize>>);
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let clicks = self.0.clone();
                Table::new(["alpha", "beta"])
                    .id("drag-table")
                    .onrowclick(move |_, _, _, _, _| clicks.set(clicks.get() + 1))
                    .column(
                        Column::new("Name")
                            .width(120.)
                            .text(|row: &&str, _| (*row).into()),
                    )
                    .column(Column::new("Note").width(120.).text(|_, _| "text".into()))
            }
        }
        let clicks = Rc::new(StdCell::new(0));
        let (_, view) = cx.add_window_view({
            let clicks = clicks.clone();
            move |_, _| Harness(clicks)
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        // Drag from the first cell into the second: a selection is made and the
        // row must not activate.
        view.simulate_mouse_down(
            gpui::point(px(20.), px(60.)),
            MouseButton::Left,
            gpui::Modifiers::default(),
        );
        view.simulate_mouse_move(
            gpui::point(px(200.), px(60.)),
            Some(MouseButton::Left),
            gpui::Modifiers::default(),
        );
        view.simulate_mouse_up(
            gpui::point(px(200.), px(60.)),
            MouseButton::Left,
            gpui::Modifiers::default(),
        );
        assert_eq!(clicks.get(), 0, "selecting text does not click the row");
        // A plain click still reports the row.
        view.simulate_click(gpui::point(px(20.), px(60.)), gpui::Modifiers::default());
        assert_eq!(clicks.get(), 1);
    }

    #[gpui::test]
    fn table_scroll_does_not_reach_an_outer_container(cx: &mut gpui::TestAppContext) {
        use std::cell::Cell as StdCell;
        struct Harness(Rc<StdCell<usize>>);
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let outer = self.0.clone();
                div()
                    .id("outer-scroll")
                    .w(px(400.))
                    .h(px(400.))
                    .overflow_scroll()
                    .on_scroll(move |_, _, _| outer.set(outer.get() + 1))
                    .child(
                        div().w(px(300.)).child(
                            Table::new(["a", "b", "c", "d", "e", "f"])
                                .id("wheel-table")
                                .height(120.)
                                .column(Column::new("Name").text(|row: &&str, _| (*row).into())),
                        ),
                    )
            }
        }
        let outer = Rc::new(StdCell::new(0));
        let (_, view) = cx.add_window_view({
            let outer = outer.clone();
            move |_, _| Harness(outer)
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        view.simulate_event(gpui::ScrollEvent {
            position: gpui::point(px(20.), px(60.)),
            delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.), px(-20.))),
            modifiers: gpui::Modifiers::default(),
            touch_phase: gpui::TouchPhase::Moved,
            source: gpui::ScrollSource::Pointer,
        });
        assert_eq!(outer.get(), 0, "the table body owns the wheel gesture");
        view.simulate_event(gpui::ScrollEvent {
            position: gpui::point(px(20.), px(380.)),
            delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.), px(-20.))),
            modifiers: gpui::Modifiers::default(),
            touch_phase: gpui::TouchPhase::Moved,
            source: gpui::ScrollSource::Pointer,
        });
        assert_eq!(
            outer.get(),
            1,
            "outside the table the outer container scrolls"
        );
    }
}
