use super::*;

#[gpui::test]
fn all_pages_draw_at_wide_and_narrow_sizes_in_both_languages_and_themes(
    cx: &mut gpui::TestAppContext,
) {
    for locale in [Locale::English, Locale::SimplifiedChinese] {
        for (dark, width) in [(false, 1040.), (true, 400.)] {
            for page in Page::ALL {
                let window = cx.add_window(move |_, cx| {
                    locale.set(cx);
                    cx.set_reduce_motion(true);
                    let gallery = Gallery {
                        page,
                        dark,
                        expanded: HashSet::from([
                            "solid",
                            "icon-catalog",
                            "icon-explicit",
                            "init-example",
                            "container",
                            "grid-basic",
                            "grid-responsive",
                            "space-presets",
                            "space-wrap",
                            "tag-editable",
                            "menu-horizontal",
                            "mask",
                        ]),
                        loading: pages::loading::State { masked: true },
                        menu: pages::menu::State::with_collapse(dark),
                        ..Gallery::default()
                    };
                    gallery.apply_theme(cx);
                    gallery
                });
                cx.simulate_window_resize(window.into(), size(px(width), px(660.)));
                let window_handle: gpui::AnyWindowHandle = window.into();
                window_handle
                    .update(cx, |_, window, cx| {
                        assert_eq!(window.viewport_size().width, px(width));
                        for _ in 0..2 {
                            window.draw(cx).clear(cx);
                        }
                        window.remove_window();
                    })
                    .unwrap();
            }
        }
    }
}
