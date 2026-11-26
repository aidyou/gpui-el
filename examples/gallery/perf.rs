//! Frame-cost probe for the gallery example.
//!
//! GPUI 0.3.14 renders the whole view tree on every draw, no matter which
//! entity notified (`tests/view_scope.rs` pins that behaviour), so this probe
//! measures the cost of a whole-page rebuild rather than of a single widget.
//!
//! Run it with:
//!
//! ```text
//! cargo test --release --example gallery perf_draw -- --ignored --nocapture
//! ```
//!
//! Release versus debug matters a lot: the same page costs roughly 10x more in
//! a dev build, which is what makes hover and drag-selection feel sluggish
//! there.

use super::*;

/// Every demo card id used by the example pages, expanded at once.
const ALL_SECTIONS: [&str; 41] = [
    "solid",
    "plain",
    "dashed",
    "text-samples",
    "sizes",
    "icons",
    "activation",
    "button-group",
    "button-custom",
    "icon-catalog",
    "icon-explicit",
    "init-example",
    "container",
    "grid-basic",
    "grid-gutter",
    "grid-mixed",
    "grid-offset",
    "grid-alignment",
    "grid-responsive",
    "space-presets",
    "space-wrap",
    "table-basic",
    "table-stripe",
    "table-fixed",
    "table-status",
    "table-empty",
    "link-basic",
    "link-disabled",
    "link-underline",
    "link-with-icon",
    "tag-basic",
    "tag-removable",
    "tag-editable",
    "tag-sizes",
    "tag-theme",
    "tag-rounded",
    "tag-checkable",
    "menu-vertical",
    "menu-nested",
    "menu-horizontal",
    "mask",
];

#[gpui::test]
#[ignore = "timing probe, run with --release --ignored"]
fn perf_draw(cx: &mut gpui::TestAppContext) {
    for (label, expanded) in [
        ("default", HashSet::default()),
        ("all-expanded", HashSet::from(ALL_SECTIONS)),
    ] {
        for page in Page::ALL {
            let expanded = expanded.clone();
            let window = cx.add_window(move |_, cx| {
                Locale::English.set(cx);
                cx.set_reduce_motion(true);
                let gallery = Gallery {
                    page,
                    expanded,
                    menu: pages::menu::State::with_collapse(false),
                    ..Gallery::default()
                };
                gallery.apply_theme(cx);
                gallery
            });
            cx.simulate_window_resize(window.into(), size(px(1440.), px(1000.)));
            let handle: gpui::AnyWindowHandle = window.into();
            handle
                .update(cx, |_, window, cx| {
                    for _ in 0..3 {
                        window.draw(cx).clear(cx);
                    }
                    let draws = 30;
                    let start = std::time::Instant::now();
                    for _ in 0..draws {
                        window.draw(cx).clear(cx);
                    }
                    let per_draw = start.elapsed().as_secs_f64() * 1000.0 / f64::from(draws);
                    println!("PERF {label:<13} {:<9} {per_draw:.3} ms/draw", page.key());
                })
                .unwrap();
        }
    }
}
