use gpui::{Context, Div, div, prelude::*, px, rgba};
use gpui_el::{Column, el_table};

use crate::{Gallery, copy::Copy};

type Row = (&'static str, &'static str, &'static str);

const ROWS: &[Row] = &[
    ("2016-05-03", "Tom", "No. 189, Grove St, Los Angeles"),
    ("2016-05-02", "Tom", "No. 189, Grove St, Los Angeles"),
    ("2016-05-04", "Linda", "No. 189, Grove St, Los Angeles"),
    ("2016-05-01", "Tom", "No. 189, Grove St, Los Angeles"),
    ("2016-05-08", "Linda", "No. 189, Grove St, Los Angeles"),
    ("2016-05-06", "Tom", "No. 189, Grove St, Los Angeles"),
    ("2016-05-07", "Linda", "No. 189, Grove St, Los Angeles"),
];

/// The shared snippet prologue: the row type and the data every demo below
/// feeds into `el_table`. Shown once, then referenced by the later demos.
const ROWS_SNIPPET: &str = r#"// A table renders any row type; columns read the fields out of it.
type Row = (&'static str, &'static str, &'static str);

let rows: Vec<Row> = vec![
    ("2016-05-03", "Tom", "No. 189, Grove St, Los Angeles"),
    ("2016-05-02", "Tom", "No. 189, Grove St, Los Angeles"),
    ("2016-05-04", "Linda", "No. 189, Grove St, Los Angeles"),
    ("2016-05-01", "Tom", "No. 189, Grove St, Los Angeles"),
    ("2016-05-08", "Linda", "No. 189, Grove St, Los Angeles"),
    ("2016-05-06", "Tom", "No. 189, Grove St, Los Angeles"),
    ("2016-05-07", "Linda", "No. 189, Grove St, Los Angeles"),
];

fn columns(rows: Vec<Row>) -> Table<Row> {
    el_table(rows)
        .column(Column::new("Date").width(120.).text(|row, _| row.0.into()))
        .column(Column::new("Name").width(100.).text(|row, _| row.1.into()))
        .column(Column::new("Address").text(|row, _| row.2.into()))
}"#;

impl Gallery {
    pub(crate) fn tables(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let section =
            |id, title, code: &str, controls| self.section(cx, id, title, || code.into(), controls);
        let columns = |key: &'static str, size_rows: &[Row]| {
            el_table(size_rows.iter().copied())
                .id(key)
                .column(
                    Column::new(s.table_date.as_str())
                        .width(120.)
                        .text(|row: &Row, _| row.0.into()),
                )
                .column(
                    Column::new(s.table_name.as_str())
                        .width(100.)
                        .text(|row: &Row, _| row.1.into()),
                )
                .column(Column::new(s.table_address.as_str()).text(|row: &Row, _| row.2.into()))
        };
        let basic = columns("table-basic", ROWS);
        let striped = columns("table-stripe", ROWS).stripe(true).border(true);
        let fixed = columns("table-fixed", ROWS).height(220.);
        let status = columns("table-status", ROWS).row_background(|_, index| match index {
            1 => Some(rgba(0xf56c6c1a).into()),
            2 => Some(rgba(0xe6a23c1a).into()),
            3 => Some(rgba(0x67c23a1a).into()),
            _ => None,
        });
        let empty = columns("table-empty", &[]).empty_text(s.table_empty.as_str());
        let copyable = columns("table-copy", ROWS).copyable(true);
        div()
            .flex()
            .flex_col()
            .gap(px(24.))
            .child(section(
                "table-basic",
                &s.table_basic,
                &format!("{ROWS_SNIPPET}\n\ncolumns(rows).id(\"table-basic\");"),
                basic.into_any_element(),
            ))
            .child(section(
                "table-stripe",
                &s.table_stripe,
                r#"// `rows` and `columns` come from the first example.
columns(rows).id("table-stripe").stripe(true).border(true);"#,
                striped.into_any_element(),
            ))
            .child(section(
                "table-fixed",
                &s.table_fixed,
                r#"// A fixed height keeps the header visible while the body scrolls; the
// body also consumes wheel deltas so the page never scrolls with it.
columns(rows).id("table-fixed").height(220.);"#,
                fixed.into_any_element(),
            ))
            .child(section(
                "table-status",
                &s.table_status,
                r#"// row_background replaces the upstream row-class-name status classes.
el_table(rows).id("table-status")
    .row_background(|_, index| match index {
        1 => Some(rgba(0xf56c6c1a).into()),
        2 => Some(rgba(0xe6a23c1a).into()),
        3 => Some(rgba(0x67c23a1a).into()),
        _ => None,
    })
    .column(Column::new("Date").width(120.).text(|row, _| row.0.into()))
    .column(Column::new("Name").width(100.).text(|row, _| row.1.into()))
    .column(Column::new("Address").text(|row, _| row.2.into()));"#,
                status.into_any_element(),
            ))
            .child(section(
                "table-empty",
                &s.table_empty_section,
                r#"// The same columns over no rows at all.
columns(Vec::new()).id("table-empty").empty_text("No Data");"#,
                empty.into_any_element(),
            ))
            .child(section(
                "table-copy",
                &s.table_copy_section,
                r#"// While the pointer is over the table, a control sits in the top-right
// corner. Hovering it labels the action, and clicking it puts the table on
// the clipboard as Markdown, confirming next to the icon for three seconds.
columns(rows).id("table-copy").copyable(true);"#,
                copyable.into_any_element(),
            ))
    }
}
