use gpui::{
    AppContext, ClipboardItem, Context, Div, Entity, IntoElement, SharedString, Window, div,
    prelude::*, px, rgb,
};
use gpui_el::{Icon, IconPack, Locale, el_button};

use crate::{
    Gallery, catalog,
    copy::Copy,
    demo::{row, source},
    search,
};

#[derive(Default)]
pub(crate) struct State {
    pack_error: bool,
    group: usize,
    search: Option<Entity<search::Search>>,
    copied: Option<&'static str>,
}

fn icon_grid_metrics(viewport_width: f32) -> (usize, f32, f32) {
    // Match the Element Plus icon directory's responsive column count and row height.
    match viewport_width {
        w if w <= 480. => (2, 80., 12.),
        w if w <= 768. => (3, 80., 12.),
        w if w <= 992. => (4, 90., 13.),
        w if w <= 1200. => (6, 90., 13.),
        _ => (7, 90., 13.),
    }
}

fn icon_grid_cell(columns: usize, border: gpui::Rgba) -> Div {
    div()
        .flex_none()
        .w(gpui::relative(1. / columns as f32))
        .min_w_0()
        .h_full()
        .overflow_hidden()
        .border_r_1()
        .border_b_1()
        .border_color(border)
}

impl Gallery {
    pub(crate) fn icons(
        &mut self,
        window: &Window,
        s: &'static Copy,
        cx: &mut Context<Self>,
    ) -> Div {
        let pack = IconPack::current(cx);
        let locale = Locale::current(cx);
        let input = self
            .icon
            .search
            .get_or_insert_with(|| {
                let search = cx.new(search::Search::new);
                cx.observe(&search, |_, _, cx| cx.notify()).detach();
                search
            })
            .clone();
        if input.read(cx).placeholder.as_ref() != s.icon_search {
            input.update(cx, |input, cx| {
                input.placeholder = s.icon_search.as_str().into();
                cx.notify();
            });
        }
        let query = input.read(cx).query();
        let groups = catalog::groups(pack);
        let selected = self.icon.group.min(groups.len().saturating_sub(1));
        let group_name = |group: &catalog::Group| {
            if locale == Locale::SimplifiedChinese {
                group.zh
            } else {
                group.en
            }
        };
        let group = groups.get(selected);
        let names: Vec<_> = group
            .into_iter()
            .flat_map(|group| group.names.iter().copied())
            .filter(|name| catalog::matches(name, &query))
            .collect();
        let count = names.len();
        let (columns, row_height, label_size) =
            icon_grid_metrics(f32::from(window.viewport_size().width));
        let copied = self.icon.copied;
        let copy_button = std::rc::Rc::new(cx.listener(move |this, name: &&'static str, _, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string(catalog::snippet(pack, name)));
            this.icon.copied = Some(*name);
            cx.notify();
        }));
        let border = rgb(if self.dark { 0x414243 } else { 0xdcdfe6 });
        let hover_fill = rgb(if self.dark { 0x262727 } else { 0xf2f6fc });
        let previews = gpui::uniform_list(
            SharedString::from(format!(
                "icon-results-{pack:?}-{selected}-{query}-{columns}"
            )),
            count.div_ceil(columns),
            move |rows, _, _| {
                rows.map(|i| {
                    div()
                        .w_full()
                        .flex()
                        .h(px(row_height))
                        .children((0..columns).map(|column| {
                            let cell = icon_grid_cell(columns, border);
                            let Some(&name) = names.get(i * columns + column) else {
                                return cell.into_any_element();
                            };
                            let copy = copy_button.clone();
                            cell.flex()
                                .flex_col()
                                .justify_center()
                                .items_center()
                                .gap(px(8.))
                                .id(name)
                                .role(gpui::Role::Button)
                                .aria_label(name)
                                .cursor_pointer()
                                .focusable()
                                .tab_stop(true)
                                .hover(move |style| style.bg(hover_fill).text_color(rgb(0x409eff)))
                                .focus_visible(|style| style.bg(gpui::rgba(0x409eff22)))
                                .child(
                                    div()
                                        .size(px(24.))
                                        .flex_none()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(Icon::from_pack(pack, name).size(20.)),
                                )
                                .child(
                                    div()
                                        .w_full()
                                        .min_w_0()
                                        .flex_none()
                                        .px(px(8.))
                                        .truncate()
                                        .text_center()
                                        .text_size(px(label_size))
                                        .line_height(px(16.))
                                        .child(name),
                                )
                                .on_click(move |event, w, cx| {
                                    if event.standard_click() {
                                        copy(&name, w, cx);
                                    }
                                })
                                .into_any_element()
                        }))
                })
                .collect::<Vec<_>>()
            },
        )
        .h(px(count.div_ceil(columns).min(5) as f32 * row_height))
        .w_full();
        let pack_picker = row().children(
            [
                (IconPack::ElementPlus, "Element Plus"),
                (IconPack::Tabler, "Tabler"),
            ]
            .into_iter()
            .enumerate()
            .map(|(i, (option, title))| {
                el_button()
                    .id(("pack", i))
                    .label(title)
                    .when(pack == option, |button| button.primary())
                    .onclick(cx.listener(move |this, _, _, cx| {
                        this.icon.pack_error = option.set(cx).is_err();
                        if !this.icon.pack_error {
                            this.icon.group = 0;
                            this.icon.copied = None;
                        }
                        cx.notify();
                    }))
            }),
        );
        self.section(
            cx,
            "icon-catalog",
            &s.named_icons,
            || {
                format!(
                    "{}\n\n// gallery/pages/explicit_icons.rs\n{}\n\n// gallery/catalog.rs\n{}\n\n// gallery/search.rs\n{}",
                    source(
                        include_str!("icon.rs")
                            .split("\n#[cfg(test)]")
                            .next()
                            .unwrap_or_default()
                    ),
                    include_str!("explicit_icons.rs"),
                    include_str!("../catalog.rs"),
                    include_str!("../search.rs"),
                )
            },
            div()
                .flex()
                .flex_col()
                .gap(px(12.))
                .child(pack_picker)
                .when(self.icon.pack_error, |el| {
                    el.child(s.icon_pack_error.as_str())
                })
                .child(row().children(groups.iter().enumerate().map(|(i, group)| {
                    el_button()
                        .id(("icon-group", i))
                        .label(group_name(group))
                        .small()
                        .when(i == selected, |btn| btn.primary())
                        .onclick(cx.listener(move |this, _, _, cx| {
                            this.icon.group = i;
                            this.icon.copied = None;
                            cx.notify();
                        }))
                })))
                .child(input)
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .gap(px(8.))
                        .children(group.map(|group| format!("{} ({count})", group_name(group))))
                        .children(copied.map(|name| s.icon_copied.replace("%{name}", name))),
                )
                .when(count == 0, |el| el.child(s.icon_empty.as_str()))
                .when(count > 0, |el| {
                    el.child(
                        div()
                            .overflow_hidden()
                            .border_t_1()
                            .border_l_1()
                            .border_color(border)
                            .child(previews),
                    )
                }),
        )
        .child(self.section(
            cx,
            "icon-explicit",
            &s.specific_icons,
            || source(include_str!("explicit_icons.rs")),
            explicit_icons(),
        ))
    }
}

#[path = "explicit_icons.rs"]
mod explicit;
use explicit::explicit_icons;

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Bounds, Render};
    use std::{cell::RefCell, rc::Rc};

    #[gpui::test]
    fn icon_grid_columns_stay_equal_with_long_names_and_empty_cells(cx: &mut gpui::TestAppContext) {
        type Rects = Rc<RefCell<Vec<(usize, usize, Bounds<gpui::Pixels>)>>>;
        struct Probe {
            viewport: f32,
            width: f32,
            rects: Rects,
        }
        impl Render for Probe {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let (columns, height, _) = icon_grid_metrics(self.viewport);
                div()
                    .w(px(self.width))
                    .flex()
                    .flex_col()
                    .children((0..2).map(|row| {
                        div()
                            .flex()
                            .w_full()
                            .h(px(height))
                            .children((0..columns).map(|column| {
                                let probe = self.rects.clone();
                                icon_grid_cell(columns, rgb(0))
                                    .relative()
                                    .when(row == 0 || column == 0, |cell| {
                                        cell.child(div().whitespace_nowrap().child(
                                            "very-long-icon-name-that-must-not-stretch-the-cell",
                                        ))
                                    })
                                    .child(
                                        gpui::canvas(
                                            move |bounds, _, _| {
                                                probe.borrow_mut().push((row, column, bounds))
                                            },
                                            |_, _, _, _| {},
                                        )
                                        .absolute()
                                        .inset_0(),
                                    )
                            }))
                    }))
            }
        }
        for (viewport, width, columns, height) in [
            (400., 317., 2, 80.),
            (768., 509., 3, 80.),
            (992., 661., 4, 90.),
            (1200., 777., 6, 90.),
            (1201., 777., 7, 90.),
        ] {
            assert_eq!(icon_grid_metrics(viewport).0, columns);
            let rects: Rects = Default::default();
            let (_, view) = cx.add_window_view({
                let rects = rects.clone();
                move |_, _| Probe {
                    viewport,
                    width,
                    rects,
                }
            });
            rects.borrow_mut().clear();
            view.update(|window, cx| window.draw(cx).clear(cx));
            let measured = rects.borrow();
            assert_eq!(measured.len(), 2 * columns);
            for &(row, column, bounds) in measured.iter() {
                let cell_width = width / columns as f32;
                assert!((f32::from(bounds.size.width) + 1. - cell_width).abs() <= 1.);
                assert_eq!(bounds.size.height, px(height - 1.));
                assert!((f32::from(bounds.origin.x) - cell_width * column as f32).abs() <= 1.);
                assert_eq!(bounds.origin.y, px(row as f32 * height));
            }
        }
    }
}
