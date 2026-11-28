use std::time::Duration;

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
fn gallery_cascader_and_tree_select_panels_are_anchored(cx: &mut gpui::TestAppContext) {
    for (page, target_selector, panel_selector) in [
        (Page::Cascader, "cascader-basic", "cascader-panel"),
        (Page::TreeSelect, "tree-select-basic", "tree-select-panel"),
    ] {
        let (_, view) = cx.add_window_view(|_, cx| {
            Locale::English.set(cx);
            cx.set_reduce_motion(true);
            let gallery = Gallery {
                page,
                ..Gallery::default()
            };
            gallery.apply_theme(cx);
            gallery
        });
        view.simulate_resize(size(px(1040.), px(660.)));
        view.update(|window, cx| window.draw(cx).clear(cx));
        let target = view
            .debug_bounds(target_selector)
            .expect("complex selector bounds");
        view.simulate_click(target.center(), gpui::Modifiers::default());
        view.update(|window, cx| window.draw(cx).clear(cx));
        let panel = view
            .debug_bounds(panel_selector)
            .expect("complex selector panel");
        assert!(
            panel.top() >= target.bottom() && panel.top() < target.bottom() + px(20.),
            "panel={panel_selector}, panel={panel:?}, target={target:?}"
        );
    }
}

#[gpui::test]
fn gallery_color_picker_panel_opens(cx: &mut gpui::TestAppContext) {
    let (_, view) = cx.add_window_view(|_, cx| {
        Locale::English.set(cx);
        cx.set_reduce_motion(true);
        let gallery = Gallery {
            page: Page::ColorPicker,
            ..Gallery::default()
        };
        gallery.apply_theme(cx);
        gallery
    });
    view.simulate_resize(size(px(1040.), px(660.)));
    view.update(|window, cx| window.draw(cx).clear(cx));
    let target = view
        .debug_bounds("color-picker-basic")
        .expect("gallery color picker bounds");
    view.simulate_click(target.center(), gpui::Modifiers::default());
    view.update(|window, cx| window.draw(cx).clear(cx));
    assert!(view.debug_bounds("color-picker-panel").is_some());
}

#[gpui::test]
fn gallery_color_picker_text_input_commits(cx: &mut gpui::TestAppContext) {
    let (gallery, view) = cx.add_window_view(|_, cx| {
        Locale::English.set(cx);
        cx.set_reduce_motion(true);
        let gallery = Gallery {
            page: Page::ColorPicker,
            ..Gallery::default()
        };
        gallery.apply_theme(cx);
        gallery
    });
    view.simulate_resize(size(px(1040.), px(660.)));
    view.update(|window, cx| window.draw(cx).clear(cx));
    let target = view
        .debug_bounds("color-picker-basic")
        .expect("gallery color picker bounds");
    view.simulate_click(target.center(), gpui::Modifiers::default());
    view.update(|window, cx| window.draw(cx).clear(cx));
    let input = view
        .debug_bounds("color-picker-text-input")
        .expect("color picker text input");
    view.simulate_mouse_down(
        input.center(),
        gpui::MouseButton::Left,
        gpui::Modifiers::default(),
    );
    view.update(|window, cx| window.draw(cx).clear(cx));
    assert!(view.debug_bounds("color-picker-panel").is_some());
    let ctrl_a = view
        .update(|_, cx| gpui::Keystroke::parse_with_os("ctrl-a", cx.platform_info().os).unwrap());
    view.simulate_event(gpui::KeyDownEvent {
        keystroke: ctrl_a,
        is_held: false,
        prefer_character_input: false,
    });
    view.simulate_input("#112233");
    view.simulate_keystrokes("tab");
    view.update(|window, cx| window.draw(cx).clear(cx));
    let confirm = view
        .debug_bounds("color-picker-confirm")
        .expect("color picker confirm");
    view.simulate_click(confirm.center(), gpui::Modifiers::default());
    view.update(|window, cx| window.draw(cx).clear(cx));
    gallery.update(cx, |gallery, _| {
        assert_eq!(
            pages::color_picker::basic_value(&gallery.color_picker).map(|value| value.as_ref()),
            Some("#112233")
        );
    });
}

#[gpui::test]
fn gallery_dialog_and_drawer_overlays_open(cx: &mut gpui::TestAppContext) {
    let (dialog, dialog_view) = cx.add_window_view(|_, cx| {
        Locale::English.set(cx);
        cx.set_reduce_motion(true);
        let mut gallery = Gallery {
            page: Page::Dialog,
            ..Gallery::default()
        };
        gallery.dialog.open = true;
        gallery.apply_theme(cx);
        gallery
    });
    dialog_view.simulate_resize(size(px(1040.), px(660.)));
    dialog_view.update(|window, cx| window.draw(cx).clear(cx));
    assert!(dialog_view.debug_bounds("dialog-panel").is_some());
    dialog_view.simulate_keystrokes("escape");
    dialog.update(cx, |dialog, _| assert!(!dialog.dialog.open));

    let (drawer, drawer_view) = cx.add_window_view(|_, cx| {
        Locale::English.set(cx);
        cx.set_reduce_motion(true);
        let mut gallery = Gallery {
            page: Page::Drawer,
            ..Gallery::default()
        };
        gallery.drawer.open = true;
        gallery.drawer.direction = gpui_el::DrawerDirection::Left;
        gallery.apply_theme(cx);
        gallery
    });
    drawer_view.simulate_resize(size(px(1040.), px(660.)));
    drawer_view.update(|window, cx| window.draw(cx).clear(cx));
    let panel = drawer_view
        .debug_bounds("drawer-panel")
        .expect("drawer panel");
    assert!((panel.left() - px(0.)).abs() < px(1.));
    drawer_view.simulate_keystrokes("escape");
    drawer.update(cx, |drawer, _| assert!(!drawer.drawer.open));
}

#[gpui::test]
fn gallery_message_and_notification_stacks_render_and_clear(cx: &mut gpui::TestAppContext) {
    let (message, message_view) = cx.add_window_view(|_, cx| {
        Locale::English.set(cx);
        cx.set_reduce_motion(true);
        let gallery = Gallery {
            page: Page::Message,
            ..Gallery::default()
        };
        gallery.apply_theme(cx);
        gallery
    });
    message_view.simulate_resize(size(px(1040.), px(660.)));
    let state = message_view.update(|window, cx| {
        let state = message
            .read(cx)
            .message
            .as_ref()
            .cloned()
            .expect("message state");
        state.update(cx, |state, cx| {
            state.push(
                gpui_el::MessageOptions::new("Persistent").duration(std::time::Duration::ZERO),
                cx,
            );
        });
        window.draw(cx).clear(cx);
        state
    });
    assert!(message_view.debug_bounds("message-stack").is_some());
    message_view.update(|window, cx| {
        state.update(cx, |state, cx| state.close_all(cx));
        window.draw(cx).clear(cx);
    });
    assert!(message_view.debug_bounds("message-stack").is_none());

    let (notification, notification_view) = cx.add_window_view(|_, cx| {
        Locale::English.set(cx);
        cx.set_reduce_motion(true);
        let gallery = Gallery {
            page: Page::Notification,
            ..Gallery::default()
        };
        gallery.apply_theme(cx);
        gallery
    });
    notification_view.simulate_resize(size(px(1040.), px(660.)));
    let state = notification_view.update(|window, cx| {
        let state = notification
            .read(cx)
            .notification
            .as_ref()
            .cloned()
            .expect("notification state");
        state.update(cx, |state, cx| {
            state.push(
                gpui_el::NotificationOptions::new("Persistent").duration(std::time::Duration::ZERO),
                cx,
            );
        });
        window.draw(cx).clear(cx);
        state
    });
    assert!(
        notification_view
            .debug_bounds("notification-stack")
            .is_some()
    );
    notification_view.update(|window, cx| {
        state.update(cx, |state, cx| state.close_all(cx));
        window.draw(cx).clear(cx);
    });
    assert!(
        notification_view
            .debug_bounds("notification-stack")
            .is_none()
    );
}

#[gpui::test]
fn gallery_navigation_group_renders_and_interacts(cx: &mut gpui::TestAppContext) {
    let (_, breadcrumb_view) = cx.add_window_view(|_, cx| {
        Locale::English.set(cx);
        cx.set_reduce_motion(true);
        let gallery = Gallery {
            page: Page::Breadcrumb,
            ..Gallery::default()
        };
        gallery.apply_theme(cx);
        gallery
    });
    breadcrumb_view.simulate_resize(size(px(1040.), px(900.)));
    breadcrumb_view.update(|window, cx| window.draw(cx).clear(cx));
    assert!(breadcrumb_view.debug_bounds("breadcrumb").is_some());

    let (_, dropdown_view) = cx.add_window_view(|_, cx| {
        Locale::English.set(cx);
        cx.set_reduce_motion(true);
        let gallery = Gallery {
            page: Page::Dropdown,
            ..Gallery::default()
        };
        gallery.apply_theme(cx);
        gallery
    });
    dropdown_view.simulate_resize(size(px(1040.), px(900.)));
    dropdown_view.update(|window, cx| window.draw(cx).clear(cx));
    let trigger = dropdown_view
        .debug_bounds("dropdown-basic-area")
        .expect("dropdown area");
    dropdown_view.simulate_mouse_move(trigger.center(), None, Default::default());
    dropdown_view.run_until_parked();
    dropdown_view
        .executor()
        .advance_clock(Duration::from_millis(170));
    dropdown_view.run_until_parked();
    dropdown_view.update(|window, cx| window.draw(cx).clear(cx));
    assert!(dropdown_view.debug_bounds("dropdown-menu").is_some());
    let item = dropdown_view
        .debug_bounds("dropdown-item-0")
        .expect("dropdown item");
    dropdown_view.simulate_click(item.center(), gpui::Modifiers::default());
    dropdown_view.update(|window, cx| window.draw(cx).clear(cx));
    assert!(dropdown_view.debug_bounds("dropdown-menu").is_none());

    let (_, steps_view) = cx.add_window_view(|_, cx| {
        Locale::English.set(cx);
        cx.set_reduce_motion(true);
        let gallery = Gallery {
            page: Page::Steps,
            ..Gallery::default()
        };
        gallery.apply_theme(cx);
        gallery
    });
    steps_view.simulate_resize(size(px(1040.), px(900.)));
    steps_view.update(|window, cx| window.draw(cx).clear(cx));
    assert!(steps_view.debug_bounds("step-0").is_some());
    assert!(steps_view.debug_bounds("step-2").is_some());

    let (_, tabs_view) = cx.add_window_view(|_, cx| {
        Locale::English.set(cx);
        cx.set_reduce_motion(true);
        let gallery = Gallery {
            page: Page::Tabs,
            ..Gallery::default()
        };
        gallery.apply_theme(cx);
        gallery
    });
    tabs_view.simulate_resize(size(px(1040.), px(900.)));
    tabs_view.update(|window, cx| window.draw(cx).clear(cx));
    assert!(tabs_view.debug_bounds("tabs-nav").is_some());
    assert!(tabs_view.debug_bounds("pane-active-0").is_some());
}

#[gpui::test]
fn gallery_tooltip_family_opens_and_closes(cx: &mut gpui::TestAppContext) {
    let (_, tooltip_view) = cx.add_window_view(|_, cx| {
        Locale::English.set(cx);
        cx.set_reduce_motion(true);
        let gallery = Gallery {
            page: Page::Tooltip,
            ..Gallery::default()
        };
        gallery.apply_theme(cx);
        gallery
    });
    tooltip_view.simulate_resize(size(px(1040.), px(900.)));
    tooltip_view.update(|window, cx| window.draw(cx).clear(cx));
    let trigger = tooltip_view
        .debug_bounds("tooltip-basic-area")
        .expect("tooltip area");
    tooltip_view.simulate_mouse_move(trigger.center(), None, Default::default());
    tooltip_view.update(|window, cx| window.draw(cx).clear(cx));
    assert!(tooltip_view.debug_bounds("tooltip-popup").is_some());
    tooltip_view.simulate_click(gpui::point(px(1000.), px(850.)), gpui::Modifiers::default());
    tooltip_view.update(|window, cx| window.draw(cx).clear(cx));
    assert!(tooltip_view.debug_bounds("tooltip-popup").is_none());

    let (_, popover_view) = cx.add_window_view(|_, cx| {
        Locale::English.set(cx);
        cx.set_reduce_motion(true);
        let gallery = Gallery {
            page: Page::Popover,
            ..Gallery::default()
        };
        gallery.apply_theme(cx);
        gallery
    });
    popover_view.simulate_resize(size(px(1040.), px(900.)));
    popover_view.update(|window, cx| window.draw(cx).clear(cx));
    let trigger = popover_view
        .debug_bounds("popover-basic-area")
        .expect("popover area");
    popover_view.simulate_mouse_move(trigger.center(), None, Default::default());
    popover_view.update(|window, cx| window.draw(cx).clear(cx));
    assert!(popover_view.debug_bounds("popover-title").is_some());

    let (_, popconfirm_view) = cx.add_window_view(|_, cx| {
        Locale::English.set(cx);
        cx.set_reduce_motion(true);
        let gallery = Gallery {
            page: Page::Popconfirm,
            ..Gallery::default()
        };
        gallery.apply_theme(cx);
        gallery
    });
    popconfirm_view.simulate_resize(size(px(1040.), px(900.)));
    popconfirm_view.update(|window, cx| window.draw(cx).clear(cx));
    let trigger = popconfirm_view
        .debug_bounds("popconfirm-basic-area")
        .expect("popconfirm area");
    popconfirm_view.simulate_click(trigger.center(), gpui::Modifiers::default());
    popconfirm_view.update(|window, cx| window.draw(cx).clear(cx));
    assert!(popconfirm_view.debug_bounds("popconfirm-content").is_some());
    let cancel = popconfirm_view
        .debug_bounds("popconfirm-cancel")
        .expect("cancel action");
    popconfirm_view.simulate_click(cancel.center(), gpui::Modifiers::default());
    popconfirm_view.update(|window, cx| window.draw(cx).clear(cx));
    assert!(popconfirm_view.debug_bounds("popconfirm-content").is_none());
}

#[gpui::test]
fn gallery_select_can_create_a_new_option(cx: &mut gpui::TestAppContext) {
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
    view.simulate_resize(size(px(1040.), px(900.)));
    view.update(|window, cx| window.draw(cx).clear(cx));
    let target = view
        .debug_bounds("select-create")
        .expect("gallery creatable select bounds");
    view.simulate_click(target.center(), gpui::Modifiers::default());
    view.simulate_input("new");
    view.update(|window, cx| window.draw(cx).clear(cx));
    let create = view
        .debug_bounds("select-create-option")
        .expect("create option row");
    view.simulate_click(create.center(), gpui::Modifiers::default());
    gallery.update(cx, |gallery, _| {
        assert_eq!(
            pages::select::created_value(&gallery.select).map(|value| value.as_ref()),
            Some("new")
        );
    });
}

#[gpui::test]
fn gallery_autocomplete_popup_is_anchored_and_selects(cx: &mut gpui::TestAppContext) {
    let (gallery, view) = cx.add_window_view(|_, cx| {
        Locale::English.set(cx);
        cx.set_reduce_motion(true);
        let gallery = Gallery {
            page: Page::AutoComplete,
            ..Gallery::default()
        };
        gallery.apply_theme(cx);
        gallery
    });
    view.simulate_resize(size(px(1040.), px(660.)));
    view.update(|window, cx| window.draw(cx).clear(cx));
    let target = view
        .debug_bounds("auto-complete-basic")
        .expect("gallery autocomplete bounds");
    view.simulate_click(target.center(), gpui::Modifiers::default());
    view.update(|window, cx| window.draw(cx).clear(cx));
    let popup = view
        .debug_bounds("autocomplete-popup")
        .expect("autocomplete popup");
    assert!(
        popup.top() >= target.bottom() && popup.top() < target.bottom() + px(20.),
        "popup={popup:?}, target={target:?}"
    );
    view.simulate_keystrokes("down");
    view.simulate_keystrokes("enter");
    gallery.update(cx, |gallery, _| {
        assert_eq!(
            pages::auto_complete::basic_value(&gallery.auto_complete).as_ref(),
            "gpui"
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
