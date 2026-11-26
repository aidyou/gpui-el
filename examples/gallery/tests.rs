use super::*;

#[test]
fn dynamic_tags_are_removed_by_stable_id_not_label() {
    let mut state = pages::tag::State::default();
    // Close "Tag 2", then add a tag that reuses the label "Tag 3".
    state.close_dynamic(1);
    state.add_dynamic(format!("Tag {}", state.dynamic_labels().len() + 1).into());
    assert_eq!(
        state
            .dynamic_labels()
            .iter()
            .map(|label| label.to_string())
            .collect::<Vec<_>>(),
        ["Tag 1", "Tag 3", "Tag 3"]
    );
    // Closing the original "Tag 3" must leave the newly added one in place.
    state.close_dynamic(2);
    assert_eq!(
        state
            .dynamic_labels()
            .iter()
            .map(|label| label.to_string())
            .collect::<Vec<_>>(),
        ["Tag 1", "Tag 3"]
    );
}

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
                            "tag-basic",
                            "tag-removable",
                            "tag-editable",
                            "tag-sizes",
                            "tag-theme",
                            "tag-rounded",
                            "tag-checkable",
                            "menu-vertical",
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
