//! Pins when GPUI decides that an element is hovered.
//!
//! GPUI recomputes the hover state inside the pointer event, before the frame
//! that reacts to it is laid out: the capture-phase listener in
//! `elements/div.rs` compares the hitbox against the previous paint and fires
//! the hover listeners immediately. So the highlight is never *logically* one
//! move late. What a user perceives is the rendering pipeline: the frame that
//! paints the highlight starts after the event, finishes after `layout` and
//! `prepaint`, and is presented on the next vsync.
//!
//! That is why frame cost, not hover bookkeeping, decides whether hover feels
//! native: at ~90 ms per frame (dev profile with unoptimized dependencies) the
//! highlight trails the pointer by several frames, while `Cargo.toml`
//! optimizing the dependency stack brings the same page into the 4-11 ms range.
//! Keep this test so a change that reintroduces a hover round trip (for example
//! tracking hover in view state instead of hitboxes) shows up here.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    Context, IntoElement, Modifiers, ParentElement, Render, Styled, Window, div, point, px,
};

struct HoverProbe {
    log: Rc<RefCell<Vec<bool>>>,
}

impl Render for HoverProbe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let log = self.log.clone();
        div().flex().w(px(200.)).h(px(200.)).child(
            div()
                .id("target")
                .w(px(100.))
                .h(px(100.))
                .on_hover(move |hovered, _, _| log.borrow_mut().push(*hovered)),
        )
    }
}

#[gpui::test]
fn hover_updates_inside_the_pointer_event(cx: &mut gpui::TestAppContext) {
    let log: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
    let (_, view) = cx.add_window_view({
        let log = log.clone();
        move |_, _| HoverProbe { log }
    });

    view.simulate_mouse_move(point(px(190.), px(190.)), None, Modifiers::default());
    view.update(|window, cx| window.draw(cx).clear(cx));
    log.borrow_mut().clear();

    // Entering the target reports hover during dispatch, before any layout runs.
    view.simulate_mouse_move(point(px(50.), px(50.)), None, Modifiers::default());
    assert_eq!(*log.borrow(), [true], "hover was not decided by the event");

    // The frame that paints the highlight does not re-decide it: nothing is
    // reported again while the pointer stays put.
    log.borrow_mut().clear();
    view.update(|window, cx| window.draw(cx).clear(cx));
    assert!(log.borrow().is_empty(), "hover was re-decided during paint");

    // Leaving is symmetric: the event decides it, the frame only paints.
    log.borrow_mut().clear();
    view.simulate_mouse_move(point(px(190.), px(190.)), None, Modifiers::default());
    assert_eq!(*log.borrow(), [false], "leaving was not decided by the event");
}
