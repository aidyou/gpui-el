use super::*;

/// A missing name renders as a blank gutter instead of failing, so the catalog
/// icons are checked against the pack the chrome pins them to.
#[test]
fn catalog_icons_exist_in_the_chrome_pack() {
    for name in [
        nav_icon::INIT,
        nav_icon::BASICS,
        nav_icon::DATA,
        nav_icon::NAVIGATION,
        nav_icon::FEEDBACK,
    ] {
        assert!(
            IconPack::ElementPlus
                .names()
                .any(|candidate| candidate == name),
            "{name} is missing from the Element Plus pack"
        );
    }
}

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

#[test]
fn sidebar_titles_are_english_only_in_english_mode() {
    let english = strings(Locale::English);
    for page in Page::ALL {
        let title = page.title(english);
        assert!(
            title.bytes().all(|byte| byte.is_ascii()),
            "English sidebar title contains non-ASCII text: {title}"
        );
    }
}

#[test]
fn sidebar_titles_keep_chinese_suffixes_in_chinese_mode() {
    let chinese = strings(Locale::SimplifiedChinese);
    for page in Page::ALL {
        let title = page.title(chinese);
        assert!(
            title
                .chars()
                .any(|character| ('\u{4e00}'..='\u{9fff}').contains(&character)),
            "Chinese sidebar title is missing Chinese text: {title}"
        );
    }
}

#[gpui::test]
fn gallery_select_opens_and_selects_with_pointer_and_keyboard(cx: &mut gpui::TestAppContext) {
    let (gallery, view) = cx.add_window_view(|_, cx| {
        Locale::English.set(cx);
        cx.set_reduce_motion(true);
        let gallery = Gallery {
            page: Page::Select,
            ..Gallery::default()
        };
        gallery.apply_theme(cx);
        gallery
    });
    view.simulate_resize(size(px(1040.), px(660.)));
    view.update(|window, cx| window.draw(cx).clear(cx));
    let target = view
        .debug_bounds("select-basic")
        .expect("gallery select bounds");
    view.simulate_click(target.center(), gpui::Modifiers::default());
    view.update(|window, cx| window.draw(cx).clear(cx));
    let popup = view
        .debug_bounds("select-popup")
        .expect("gallery select popup");
    assert!(
        popup.top() >= target.bottom() && popup.top() < target.bottom() + px(20.),
        "popup={popup:?}, target={target:?}"
    );
    view.simulate_keystrokes("down");
    view.simulate_keystrokes("enter");
    gallery.update(cx, |gallery, _| {
        assert_eq!(
            pages::select::basic_value(&gallery.select).map(|value| value.as_ref()),
            Some("interval")
        );
    });
}

#[gpui::test]
fn gallery_time_picker_panel_is_visible_and_selects_hours(cx: &mut gpui::TestAppContext) {
    let (gallery, view) = cx.add_window_view(|_, cx| {
        Locale::English.set(cx);
        cx.set_reduce_motion(true);
        let gallery = Gallery {
            page: Page::TimePicker,
            ..Gallery::default()
        };
        gallery.apply_theme(cx);
        gallery
    });
    view.simulate_resize(size(px(1040.), px(660.)));
    view.update(|window, cx| window.draw(cx).clear(cx));
    let target = view
        .debug_bounds("time-picker-basic")
        .expect("gallery time picker bounds");
    view.simulate_click(target.center(), gpui::Modifiers::default());
    view.update(|window, cx| window.draw(cx).clear(cx));
    let panel = view
        .debug_bounds("time-picker-panel")
        .expect("gallery time picker panel");
    assert!(
        panel.top() >= target.bottom() && panel.top() < target.bottom() + px(20.),
        "panel={panel:?}, target={target:?}"
    );
    view.simulate_click(
        gpui::point(panel.left() + px(40.), panel.top() + px(80.)),
        gpui::Modifiers::default(),
    );
    gallery.update(cx, |gallery, _| {
        assert_eq!(
            pages::time_picker::basic_value(&gallery.time_picker).map(|value| value.as_ref()),
            Some("02:30")
        );
    });
}

#[gpui::test]
fn gallery_date_picker_switches_views_and_range_selects(cx: &mut gpui::TestAppContext) {
    let (gallery, view) = cx.add_window_view(|_, cx| {
        Locale::English.set(cx);
        cx.set_reduce_motion(true);
        let gallery = Gallery {
            page: Page::DatePicker,
            ..Gallery::default()
        };
        gallery.apply_theme(cx);
        gallery
    });
    view.simulate_resize(size(px(1040.), px(660.)));
    view.update(|window, cx| window.draw(cx).clear(cx));
    let basic = view
        .debug_bounds("date-picker-basic")
        .expect("gallery date picker bounds");
    view.simulate_click(basic.center(), gpui::Modifiers::default());
    view.update(|window, cx| window.draw(cx).clear(cx));
    let date_panel = view.debug_bounds("date-picker-panel").expect("date panel");
    let year_label = view
        .debug_bounds("date-picker-year-label-left")
        .expect("year label");
    view.simulate_click(year_label.center(), gpui::Modifiers::default());
    view.update(|window, cx| window.draw(cx).clear(cx));
    let year_panel = view.debug_bounds("date-picker-panel").expect("year panel");
    assert!(
        year_panel.size.height < date_panel.size.height,
        "date={date_panel:?}, year={year_panel:?}"
    );
    let year = view
        .debug_bounds("date-picker-left-Year-6")
        .expect("2026 year cell");
    view.simulate_click(year.center(), gpui::Modifiers::default());
    view.update(|window, cx| window.draw(cx).clear(cx));
    let month = view
        .debug_bounds("date-picker-left-Month-10")
        .expect("November month cell");
    view.simulate_click(month.center(), gpui::Modifiers::default());
    view.update(|window, cx| window.draw(cx).clear(cx));
    assert!(view.debug_bounds("date-picker-month-label-left").is_some());
    view.simulate_keystrokes("escape");

    let range = view
        .debug_bounds("date-picker-range")
        .expect("gallery date range bounds");
    view.simulate_click(range.center(), gpui::Modifiers::default());
    view.update(|window, cx| window.draw(cx).clear(cx));
    let range_panel = view
        .debug_bounds("date-picker-panel")
        .expect("date range panel");
    assert!(range_panel.size.width > px(500.), "range={range_panel:?}");
    let start = view
        .debug_bounds("date-picker-left-day-3")
        .expect("range start cell");
    view.simulate_click(start.center(), gpui::Modifiers::default());
    view.update(|window, cx| window.draw(cx).clear(cx));
    let end = view
        .debug_bounds("date-picker-left-day-28")
        .expect("range end cell");
    view.simulate_click(end.center(), gpui::Modifiers::default());
    gallery.update(cx, |gallery, _| {
        let (start, end) = pages::date_picker::range_value(&gallery.date_picker);
        assert_eq!(
            start.as_ref().map(|value| value.as_ref()),
            Some("2026-11-04")
        );
        assert_eq!(end.as_ref().map(|value| value.as_ref()), Some("2026-11-29"));
    });
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
                            "table-basic",
                            "table-stripe",
                            "table-fixed",
                            "table-status",
                            "table-empty",
                            "link-basic",
                            "link-disabled",
                            "link-underline",
                            "link-with-icon",
                            "text-basic",
                            "text-sizes",
                            "text-truncated",
                            "text-clamp",
                            "text-decorations",
                            "text-mixed",
                            "scrollbar-basic",
                            "scrollbar-horizontal",
                            "scrollbar-max-height",
                            "scrollbar-manual",
                            "scrollbar-infinite",
                            "splitter-basic",
                            "splitter-vertical",
                            "splitter-collapsible",
                            "splitter-disabled",
                            "splitter-size",
                            "splitter-lazy",
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
