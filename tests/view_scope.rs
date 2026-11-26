//! Pins how much work a GPUI frame rebuilds.
//!
//! GPUI 0.3.14 re-renders the whole view tree on every draw: it walks the root
//! view down through `ChildView` elements and calls `Render::render` on every
//! view, and `Window::mark_view_dirty` additionally marks *ancestors* of a
//! notified entity, so a notification from a deeply nested view still dirties
//! the root. Splitting a page into child views therefore does not shrink the
//! per-frame work of this crate's gallery; only `Entity::cached` (which needs a
//! definite size and freezes the subtree) skips a rebuild.
//!
//! Keep this test as the reason the gallery stays a single view. If a GPUI
//! upgrade makes child views lazy, this test fails and the trade-off can be
//! revisited.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px,
};

struct Leaf {
    renders: Rc<Cell<usize>>,
}

impl Render for Leaf {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.renders.set(self.renders.get() + 1);
        div().w(px(20.)).h(px(20.))
    }
}

struct Parent {
    a: Entity<Leaf>,
    b: Entity<Leaf>,
    renders: Rc<Cell<usize>>,
}

impl Render for Parent {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.renders.set(self.renders.get() + 1);
        div().child(self.a.clone()).child(self.b.clone())
    }
}

#[gpui::test]
fn notify_scope_does_not_shrink_a_frame(cx: &mut gpui::TestAppContext) {
    let parent_renders = Rc::new(Cell::new(0));
    let a_renders = Rc::new(Cell::new(0));
    let b_renders = Rc::new(Cell::new(0));
    let handles: Rc<RefCell<Option<Entity<Leaf>>>> = Rc::new(RefCell::new(None));
    let (parent_count, a_count, b_count) = (
        parent_renders.clone(),
        a_renders.clone(),
        b_renders.clone(),
    );
    let handles_for_window = handles.clone();
    let (_, view) = cx.add_window_view(move |_, cx| {
        let a = cx.new(|_| Leaf { renders: a_count });
        let b = cx.new(|_| Leaf { renders: b_count });
        *handles_for_window.borrow_mut() = Some(a.clone());
        Parent {
            a,
            b,
            renders: parent_count,
        }
    });

    view.update(|window, cx| window.draw(cx).clear(cx));
    let parent_before = parent_renders.get();
    let a_before = a_renders.get();
    let b_before = b_renders.get();
    assert!(parent_before > 0 && a_before > 0 && b_before > 0);

    // A draw with no state change still rebuilds every view.
    view.update(|window, cx| window.draw(cx).clear(cx));
    let idle_step = parent_renders.get() - parent_before;
    assert!(idle_step > 0, "an idle draw rebuilt nothing");
    assert_eq!(a_renders.get() - a_before, idle_step);
    assert_eq!(b_renders.get() - b_before, idle_step);

    // Notifying one leaf rebuilds everything as well: the leaf, its sibling and
    // the parent view that owns both.
    let a = handles.borrow().clone().unwrap();
    view.update(|_, cx| a.update(cx, |_, cx| cx.notify()));
    view.update(|window, cx| window.draw(cx).clear(cx));
    let a_after = a_renders.get();
    let b_after = b_renders.get();
    let parent_after = parent_renders.get();
    assert!(a_after - a_before > idle_step);
    assert_eq!(b_after - b_before, a_after - a_before);
    assert_eq!(parent_after - parent_before, a_after - a_before);
}
