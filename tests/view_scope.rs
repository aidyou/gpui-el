//! Pins how much work a GPUI frame rebuilds.
//!
//! gpui-fast keeps the rendered tree between frames: an idle draw rebuilds
//! nothing, and notifying a view rebuilds that view only, not its siblings or
//! ancestors. The gpui 0.3.14 engine this crate started on did the opposite, so
//! the gallery was built as a single view where splitting it up bought nothing.
//! With this engine a page split would shrink per-frame work, but the gallery
//! has not been restructured yet. Keep this test so the cost model stays
//! visible when the engine changes again.

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
fn notifications_rebuild_only_their_own_view(cx: &mut gpui::TestAppContext) {
    let parent_renders = Rc::new(Cell::new(0));
    let a_renders = Rc::new(Cell::new(0));
    let b_renders = Rc::new(Cell::new(0));
    let handles: Rc<RefCell<Option<Entity<Leaf>>>> = Rc::new(RefCell::new(None));
    let (parent_count, a_count, b_count) =
        (parent_renders.clone(), a_renders.clone(), b_renders.clone());
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

    // A draw with no state change reuses the retained tree.
    for _ in 0..2 {
        view.update(|window, cx| window.draw(cx).clear(cx));
    }
    assert_eq!(
        parent_renders.get(),
        parent_before,
        "an idle draw rebuilt the parent"
    );
    assert_eq!(a_renders.get(), a_before, "an idle draw rebuilt a leaf");
    assert_eq!(b_renders.get(), b_before, "an idle draw rebuilt a leaf");

    // Notifying one leaf rebuilds that view only: not the sibling, not the
    // parent that rendered both of them.
    let a = handles.borrow().clone().unwrap();
    view.update(|_, cx| a.update(cx, |_, cx| cx.notify()));
    for _ in 0..2 {
        view.update(|window, cx| window.draw(cx).clear(cx));
    }
    assert_eq!(
        a_renders.get(),
        a_before + 1,
        "the notified view should rebuild once"
    );
    assert_eq!(b_renders.get(), b_before, "the sibling rebuilt");
    assert_eq!(parent_renders.get(), parent_before, "the parent rebuilt");
}
