//! Pins the text style table headers and cells run under.
//!
//! GPUI resolves a font family per text run, so the header contract is
//! observable through `Window::text_style`. The family assertion matters
//! because a header only renders bold when the resolved family has a bold
//! face; the platform default can land on a single-weight fallback.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    Context, FontWeight, Hsla, IntoElement, Render, SharedString, TestAppContext, Window, canvas,
    div, prelude::*, px, rgb,
};
use gpui_el::{Column, Table, Theme};

#[derive(Clone, Debug)]
struct Style {
    weight: FontWeight,
    family: SharedString,
    color: Hsla,
}

impl Style {
    fn read(window: &Window) -> Self {
        let style = window.text_style();
        Self {
            weight: style.font_weight,
            family: style.font_family.clone(),
            color: style.color,
        }
    }
}

#[derive(Clone, Default)]
struct Seen(Rc<RefCell<Vec<Style>>>);

fn probe(seen: Seen) -> impl IntoElement {
    canvas(
        move |_, window, _| seen.0.borrow_mut().push(Style::read(window)),
        |_, _, _, _| {},
    )
}

struct View {
    seen: Seen,
}

impl Render for View {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let (header, cell) = (self.seen.clone(), self.seen.clone());
        let rows = ["a"];
        div().w(px(400.)).child(
            Table::new(rows.iter().copied())
                .id("header-style")
                .column(
                    Column::new("Header")
                        .header(move || probe(header.clone()).into_any_element())
                        .render(move |_: &&str, _| probe(cell.clone()).into_any_element()),
                ),
        )
    }
}

#[gpui::test]
fn headers_are_bold_and_cells_follow_the_theme_font(cx: &mut TestAppContext) {
    let family = SharedString::from("Test Sans");
    cx.update(|cx| {
        Theme::set(
            Theme {
                font_family: Some(family.clone()),
                ..Theme::light()
            },
            cx,
        )
    });
    let seen = Seen::default();
    let (_, view) = cx.add_window_view({
        let seen = seen.clone();
        move |_, _| View { seen }
    });
    view.update(|window, cx| window.draw(cx).clear(cx));

    let seen = seen.0.borrow();
    let header = seen.first().expect("the header content never rendered");
    let cell = seen.last().expect("the cell content never rendered");
    assert_eq!(header.weight, FontWeight::BOLD, "the header is not bold");
    assert_eq!(header.color, rgb(0x606266).into());
    assert_eq!(cell.weight, FontWeight::default(), "the cell is not regular");
    // The theme family reaches both paths, so a bold face can be selected.
    assert_eq!(header.family, "Test Sans");
    assert_eq!(cell.family, "Test Sans");
}
