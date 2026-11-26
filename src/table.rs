//! Element Plus Table, basic subset.
//!
//! Follows `table.md`, `components/table/src`, and Theme Chalk `table.scss`:
//! columns with fixed or distributed widths, header and cell alignment,
//! striped rows, borders, three sizes, a scrolling body with a fixed header,
//! status row backgrounds, row clicks, and an empty state. Sorting, selection,
//! filtering, fixed columns, tree data, lazy loading, and summary rows are not
//! reproduced.

use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, Div, Hsla, IntoElement, RenderOnce, SharedString, Window, div,
    prelude::*, px, rgb,
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
            TableSize::Small => tokens::font_size::EXTRA_SMALL,
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
}

impl<T: 'static> Table<T> {
    /// Creates a table with the given rows.
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
        }
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
        Some(header) => justified(align, inner.flex().items_center().child(header())),
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
        .py(px(size.vertical_padding()))
        .child(inner)
}

fn body_cell<T: 'static>(
    column: &Column<T>,
    row: &T,
    index: usize,
    border: bool,
    size: TableSize,
    theme: &Theme,
) -> Div {
    let line = rgb(theme.border_lighter());
    let inner = div()
        .flex_1()
        .min_w_0()
        .px(px(tokens::spacing::DEFAULT))
        .line_height(px(23.));
    let inner = match (&column.render, &column.text) {
        (Some(render), _) => justified(
            column.align,
            inner.flex().items_center().child(render(row, index)),
        ),
        (None, Some(text)) => aligned(column.align, inner.child(text(row, index))),
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
        .py(px(size.vertical_padding()))
        .child(inner)
}

impl<T: 'static> RenderOnce for Table<T> {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
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
        } = self;
        let theme = Theme::current(cx);
        let border = rgb(theme.border_lighter());
        let hover = rgb(theme.fill(crate::theme::Fill::Light));
        let stripe = rgb(theme.fill(crate::theme::Fill::Lighter));
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
                let row_base = div()
                    .flex()
                    .w_full()
                    .flex_none()
                    .border_b_1()
                    .border_color(border)
                    .when_some(status, |el, color| el.bg(color))
                    .when(is_striped && status.is_none(), |el| el.bg(stripe))
                    .children(
                        columns
                            .iter()
                            .map(|c| body_cell(c, row, index, bordered, size, &theme)),
                    );
                let row_el: AnyElement = match &onrowclick {
                    Some(handler) => {
                        let handler = handler.clone();
                        let rows = rows.clone();
                        let row_index = index;
                        row_base
                            .id(("table-row", index as u32))
                            .cursor_pointer()
                            .hover(move |style| style.bg(hover))
                            .on_click(move |event, window, cx| {
                                handler(&rows[row_index], row_index, event, window, cx);
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
        // The header sits outside the scrolling body, so it stays fixed.
        let body: AnyElement = if height.is_some() {
            body.id("table-body")
                .flex_1()
                .min_h(px(0.))
                .overflow_y_scroll()
                .into_any_element()
        } else if let Some(max_height) = max_height {
            body.id("table-body")
                .max_h(px(max_height))
                .overflow_y_scroll()
                .into_any_element()
        } else {
            body.into_any_element()
        };
        div()
            .w_full()
            .flex()
            .flex_col()
            .text_size(px(size.font_size()))
            .text_color(rgb(theme.regular_text()))
            .bg(rgb(theme.background()))
            .when(bordered, |el| el.border_1().border_color(border))
            .when_some(height, |el, height| el.h(px(height)))
            .when_some(header, |el, header| el.child(header))
            .child(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_map_to_upstream_padding_and_font() {
        assert_eq!(TableSize::Large.vertical_padding(), 12.);
        assert_eq!(TableSize::Default.vertical_padding(), 8.);
        assert_eq!(TableSize::Small.vertical_padding(), 4.);
        assert_eq!(TableSize::Small.font_size(), 12.);
        assert_eq!(TableSize::Default.font_size(), 14.);
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
}
