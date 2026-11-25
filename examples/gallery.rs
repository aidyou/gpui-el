use std::{collections::HashSet, sync::OnceLock};

use gpui::{
    App, AppContext, Bounds, ClipboardItem, Context, Div, IntoElement, MouseButton, Render,
    SharedString, TitlebarOptions, Window, WindowBounds, WindowControlArea, WindowDecorations,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_el::{
    ButtonType, ButtonVariant, Direction, Icon, IconPack, Locale, MenuMode, MenuTrigger, Space,
    Theme, el_aside, el_button, el_button_group, el_col, el_container, el_footer, el_header,
    el_icon, el_init, el_loading, el_main, el_menu, el_menu_item, el_menu_item_group, el_row,
    el_space, el_sub_menu,
};
use serde::Deserialize;

#[path = "gallery/api.rs"]
mod api;
#[path = "gallery/catalog.rs"]
mod catalog;
#[path = "gallery/search.rs"]
mod search;

#[derive(Default)]
struct Gallery {
    dark: bool,
    reduced_motion: bool,
    clicks: usize,
    page: Page,
    expanded: HashSet<&'static str>,
    masked: bool,
    menu_collapsed: bool,
    menu_selection: Option<String>,
    icon_pack_error: bool,
    icon_group: usize,
    icon_search: Option<gpui::Entity<search::Search>>,
    icon_copied: Option<&'static str>,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Page {
    #[default]
    Button,
    Icon,
    Init,
    Container,
    Layout,
    Space,
    Menu,
    Loading,
}

impl Page {
    const ALL: [Self; 8] = [
        Self::Init,
        Self::Button,
        Self::Icon,
        Self::Container,
        Self::Layout,
        Self::Space,
        Self::Menu,
        Self::Loading,
    ];

    fn description(self, s: &'static Copy) -> &'static str {
        match self {
            Self::Button => &s.button_intro,
            Self::Icon => &s.icon_intro,
            Self::Init => &s.init_intro,
            Self::Container => &s.container_intro,
            Self::Layout => &s.layout_intro,
            Self::Space => &s.space_intro,
            Self::Menu => &s.menu_intro,
            Self::Loading => &s.loading_intro,
        }
    }

    fn key(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Icon => "icon",
            Self::Init => "init",
            Self::Container => "container",
            Self::Layout => "layout",
            Self::Space => "space",
            Self::Menu => "menu",
            Self::Loading => "loading",
        }
    }

    fn title(self, s: &'static Copy) -> &'static str {
        match self {
            Self::Button => &s.button_page,
            Self::Icon => &s.icon_page,
            Self::Init => &s.init_title,
            Self::Container => &s.container_page,
            Self::Layout => &s.layout_page,
            Self::Space => &s.space_page,
            Self::Menu => &s.menu_page,
            Self::Loading => &s.loading_page,
        }
    }

    fn upstream_names(self) -> &'static [&'static str] {
        match self {
            Self::Init => &[],
            Self::Button => &["el-button", "el-button-group"],
            Self::Icon => &["el-icon"],
            Self::Container => &[
                "el-container",
                "el-header",
                "el-aside",
                "el-main",
                "el-footer",
            ],
            Self::Layout => &["el-row", "el-col"],
            Self::Space => &["el-space"],
            Self::Menu => &[
                "el-menu",
                "el-sub-menu",
                "el-menu-item",
                "el-menu-item-group",
            ],
            Self::Loading => &["v-loading"],
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Copy {
    language: String,
    basics: String,
    navigation: String,
    feedback: String,
    button_intro: String,
    icon_intro: String,
    init_title: String,
    init_intro: String,
    init_fonts: String,
    container_intro: String,
    layout_intro: String,
    space_intro: String,
    menu_intro: String,
    loading_intro: String,
    grid_basic: String,
    grid_gutter: String,
    grid_mixed: String,
    grid_offset: String,
    grid_alignment: String,
    grid_responsive: String,
    grid_resize: String,
    vertical_menu: String,
    horizontal_menu: String,
    menu_index_path: String,
    menu_collapse: String,
    menu_expand: String,
    menu_disabled: String,
    window_minimize: String,
    window_zoom: String,
    window_close: String,
    dark: String,
    light: String,
    reduce_motion: String,
    restore_motion: String,
    copy: String,
    show_code: String,
    hide_code: String,
    button_page: String,
    icon_page: String,
    container_page: String,
    layout_page: String,
    space_page: String,
    menu_page: String,
    loading_page: String,
    icon_pack_error: String,
    named_icons: String,
    icon_search: String,
    icon_empty: String,
    icon_copied: String,
    specific_icons: String,
    container_title: String,
    space_title: String,
    space_presets: String,
    menu_first: String,
    menu_second: String,
    layout_header: String,
    layout_aside: String,
    layout_main: String,
    layout_footer: String,
    show_loading: String,
    hide_loading: String,
    loading_vertical: String,
    loading_horizontal: String,
    loading_clear: String,
    loading_text: String,
    solid_title: String,
    plain_title: String,
    dashed_title: String,
    text_title: String,
    size_title: String,
    icon_title: String,
    activation_title: String,
    button_group: String,
    button_custom: String,
    button_confirm: String,
    default: String,
    primary: String,
    success: String,
    warning: String,
    danger: String,
    info: String,
    text: String,
    text_bg: String,
    link: String,
    large: String,
    compact: String,
    add: String,
    count_increment: String,
    disabled: String,
    saving: String,
    activation_help: String,
}

fn strings(locale: Locale) -> &'static Copy {
    static EN: OnceLock<Copy> = OnceLock::new();
    static ZH: OnceLock<Copy> = OnceLock::new();
    let (cache, source) = if locale == Locale::SimplifiedChinese {
        (&ZH, include_str!("i18n/zh-Hans.yml"))
    } else {
        (&EN, include_str!("i18n/en.yml"))
    };
    cache.get_or_init(|| serde_yaml_ng::from_str(source).expect("valid gallery locale"))
}

// Read the actual Rust examples, not a second hand-maintained version of each demo.
fn demo_source(name: &str) -> String {
    let source = include_str!("gallery.rs");
    let start = [format!("\n    fn {name}("), format!("\nfn {name}(")];
    let (start, indent) = start
        .iter()
        .enumerate()
        .find_map(|(i, marker)| {
            source
                .find(marker)
                .map(|start| (start + 1, if i == 0 { "    " } else { "" }))
        })
        .expect("gallery example method");
    let end_marker = format!("\n{indent}}}\n");
    let end = source[start..]
        .find(&end_marker)
        .expect("gallery example method end");
    source[start..start + end + end_marker.len() - 1]
        .lines()
        .map(|line| line.strip_prefix(indent).unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
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

fn row() -> Space {
    el_space().size(gpui_el::SpaceSize::Default).wrap(true)
}

impl Gallery {
    fn section(
        &self,
        cx: &Context<Self>,
        id: &'static str,
        title: &'static str,
        example: impl Into<SharedString>,
        controls: impl IntoElement,
    ) -> Div {
        let s = strings(Locale::current(cx));
        let expanded = self.expanded.contains(id);
        let example = example.into();
        let snippet: SharedString = if expanded {
            let example = match id {
                "solid" | "plain" | "dashed" | "text-samples" | "sizes" | "icons"
                | "activation" | "button-group" | "button-custom" => {
                    format!("{}\n\n{}", demo_source("row"), demo_source("buttons")).into()
                }
                "menu-vertical" | "menu-horizontal" => demo_source("menus").into(),
                "icon-explicit" => format!(
                    "{}\n\n{}",
                    demo_source("row"),
                    demo_source("explicit_icons")
                )
                .into(),
                "icon-catalog" => format!(
                    "{}\n\n{}\n\n{}\n\n{}\n\n// gallery/catalog.rs\n{}\n\n// gallery/search.rs\n{}",
                    demo_source("row"),
                    demo_source("icon_grid_metrics"),
                    demo_source("icon_grid_cell"),
                    demo_source("icons"),
                    include_str!("gallery/catalog.rs"),
                    include_str!("gallery/search.rs")
                )
                .into(),
                "grid-basic" | "grid-gutter" | "grid-mixed" | "grid-offset" | "grid-alignment"
                | "grid-responsive" => demo_source("grids").into(),
                "space-presets" | "space-wrap" => demo_source("spaces").into(),
                "container" => demo_source("containers").into(),
                "mask" => demo_source("loading").into(),
                _ => example,
            };
            if id == "init-example" {
                example
            } else {
                let source = include_str!("gallery.rs");
                let imports = source
                    .split_once("use serde::Deserialize;")
                    .expect("gallery imports")
                    .0;
                format!("{imports}\n{example}").into()
            }
        } else {
            example
        };
        let border = rgb(if self.dark { 0x414243 } else { 0xe4e7ed });
        let fill = rgb(if self.dark { 0x262727 } else { 0xf5f7fa });
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                div()
                    .text_size(px(18.))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(title),
            )
            .child(
                div()
                    .border_1()
                    .border_color(border)
                    .rounded(px(4.))
                    .child(div().p(px(20.)).child(controls))
                    .when(expanded, |el| {
                        el.child(
                            div()
                                .border_t_1()
                                .border_color(border)
                                .bg(fill)
                                .p(px(12.))
                                .font_family("monospace")
                                .text_size(px(13.))
                                .child(snippet.clone()),
                        )
                    })
                    .child(
                        div().border_t_1().border_color(border).p(px(6.)).child(
                            row()
                                .child(
                                    el_button()
                                        .id((id, 0_u32))
                                        .small()
                                        .text()
                                        .label(if expanded {
                                            s.hide_code.as_str()
                                        } else {
                                            s.show_code.as_str()
                                        })
                                        .icon(el_icon(if expanded {
                                            "arrow-up"
                                        } else {
                                            "arrow-down"
                                        }))
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            if !this.expanded.insert(id) {
                                                this.expanded.remove(id);
                                            }
                                            cx.notify();
                                        })),
                                )
                                .when(expanded, |el| {
                                    el.child(
                                        el_button()
                                            .id((id, 1_u32))
                                            .small()
                                            .text()
                                            .label(s.copy.as_str())
                                            .icon(el_icon("document-copy"))
                                            .on_click(move |_, _, cx| {
                                                cx.write_to_clipboard(ClipboardItem::new_string(
                                                    snippet.to_string(),
                                                ));
                                            }),
                                    )
                                }),
                        ),
                    ),
            )
    }
}

impl Gallery {
    fn apply_theme(&self, cx: &mut App) {
        let mut theme = if self.dark {
            Theme::dark()
        } else {
            Theme::light()
        };
        theme.reduced_motion = self.reduced_motion;
        theme.set(cx);
    }
}

impl Gallery {
    fn buttons(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let section = |id, title, example, controls| self.section(cx, id, title, example, controls);
        let themed = |id, title, example, variant| {
            let types = [
                (ButtonType::Default, s.default.as_str()),
                (ButtonType::Primary, &s.primary),
                (ButtonType::Success, &s.success),
                (ButtonType::Warning, &s.warning),
                (ButtonType::Danger, &s.danger),
                (ButtonType::Info, &s.info),
            ];
            section(
                id,
                title,
                example,
                row().children(types.into_iter().enumerate().map(|(i, (kind, label))| {
                    el_button()
                        .id((id, i as u32 + 2))
                        .label(label)
                        .when(kind != ButtonType::Default, |button| button.kind(kind))
                        .when(variant != ButtonVariant::Solid, |button| {
                            button.variant(variant)
                        })
                })),
            )
        };
        let text = row()
            .child(
                el_button()
                    .id("text")
                    .label(s.text.as_str())
                    .primary()
                    .text(),
            )
            .child(
                el_button()
                    .id("text-bg")
                    .label(s.text_bg.as_str())
                    .primary()
                    .text_with_background(),
            )
            .child(
                el_button()
                    .id("link")
                    .label(s.link.as_str())
                    .primary()
                    .link(),
            );
        let sizes = row()
            .child(el_button().id("large").label(s.large.as_str()).large())
            .child(
                el_button()
                    .id("round")
                    .label(s.compact.as_str())
                    .round()
                    .small()
                    .font_size(13.0),
            );
        let icons = row()
            .child(
                el_button()
                    .id("circle")
                    .circle()
                    .primary()
                    .icon(el_icon("plus"))
                    .accessibility_label(s.add.as_str()),
            )
            .child(
                el_button()
                    .id("icon")
                    .label(s.add.as_str())
                    .icon(el_icon("plus")),
            );
        let activation = row()
            .child(
                el_button()
                    .id("count")
                    .label(s.count_increment.as_str())
                    .primary()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.clicks += 1;
                        cx.notify();
                    })),
            )
            .child(
                el_button()
                    .id("disabled")
                    .label(s.disabled.as_str())
                    .danger()
                    .disabled(true),
            )
            .child(
                el_button()
                    .id("loading-demo")
                    .label(s.saving.as_str())
                    .primary()
                    .loading(true),
            )
            .child(
                s.activation_help
                    .replace("%{count}", &self.clicks.to_string()),
            );
        div()
            .flex()
            .flex_col()
            .gap(px(24.))
            .child(themed("solid", &s.solid_title, "", ButtonVariant::Solid))
            .child(themed("plain", &s.plain_title, "", ButtonVariant::Plain))
            .child(themed("dashed", &s.dashed_title, "", ButtonVariant::Dashed))
            .child(section("text-samples", &s.text_title, "", text))
            .child(section("sizes", &s.size_title, "", sizes))
            .child(section("icons", &s.icon_title, "", icons))
            .child(section("activation", &s.activation_title, "", activation))
            .child(section(
                "button-group",
                &s.button_group,
                "",
                row()
                    .child(el_button_group().kind(ButtonType::Primary).buttons([
                        el_button().label(s.default.as_str()),
                        el_button().label(s.primary.as_str()),
                    ]))
                    .child(el_button_group().direction(Direction::Vertical).buttons([
                        el_button().label(s.success.as_str()).success(),
                        el_button().label(s.info.as_str()),
                    ])),
            ))
            .child(section(
                "button-custom",
                &s.button_custom,
                "",
                row()
                    .child(el_button().label(s.primary.as_str()).color(rgb(0x626aef)))
                    .child(
                        el_button()
                            .label(s.primary.as_str())
                            .color(rgb(0x626aef))
                            .plain(),
                    )
                    .child(
                        el_button()
                            .label(s.button_confirm.as_str())
                            .auto_insert_space(true),
                    ),
            ))
    }
}

impl Gallery {
    fn icons(&mut self, window: &Window, s: &'static Copy, cx: &mut Context<Self>) -> Div {
        let pack = IconPack::current(cx);
        let locale = Locale::current(cx);
        if self.icon_search.is_none() {
            let search = cx.new(search::Search::new);
            cx.observe(&search, |_, _, cx| cx.notify()).detach();
            self.icon_search = Some(search);
        }
        let input = self.icon_search.as_ref().unwrap().clone();
        if input.read(cx).placeholder.as_ref() != s.icon_search {
            input.update(cx, |input, cx| {
                input.placeholder = s.icon_search.as_str().into();
                cx.notify();
            });
        }
        let query = input.read(cx).query();
        let groups = catalog::groups(pack);
        let selected = self.icon_group.min(groups.len() - 1);
        let group_name = |group: &catalog::Group| {
            if locale == Locale::SimplifiedChinese {
                group.zh
            } else {
                group.en
            }
        };
        let names: Vec<_> = groups[selected]
            .names
            .iter()
            .copied()
            .filter(|name| catalog::matches(name, &query))
            .collect();
        let count = names.len();
        let (columns, row_height, label_size) =
            icon_grid_metrics(f32::from(window.viewport_size().width));
        let copied = self.icon_copied;
        let copy_button = std::rc::Rc::new(cx.listener(move |this, name: &&'static str, _, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string(catalog::snippet(pack, name)));
            this.icon_copied = Some(*name);
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
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.icon_pack_error = option.set(cx).is_err();
                        if !this.icon_pack_error {
                            this.icon_group = 0;
                            this.icon_copied = None;
                        }
                        cx.notify();
                    }))
            }),
        );
        self.section(
            cx,
            "icon-catalog",
            &s.named_icons,
            "",
            div()
                .flex()
                .flex_col()
                .gap(px(12.))
                .child(pack_picker)
                .when(self.icon_pack_error, |el| {
                    el.child(s.icon_pack_error.as_str())
                })
                .child(row().children(groups.iter().enumerate().map(|(i, group)| {
                    el_button()
                        .id(("icon-group", i))
                        .label(group_name(group))
                        .small()
                        .when(i == selected, |btn| btn.primary())
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.icon_group = i;
                            this.icon_copied = None;
                            cx.notify();
                        }))
                })))
                .child(input)
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .gap(px(8.))
                        .child(format!("{} ({count})", group_name(&groups[selected])))
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
            "",
            self.explicit_icons(),
        ))
    }

    fn explicit_icons(&self) -> Space {
        row()
            .child(el_icon("Plus").size(24.))
            .child(Icon::from_pack(IconPack::ElementPlus, "Plus").size(24.))
            .child(Icon::from_pack(IconPack::Tabler, "plus").size(24.))
    }

    fn containers(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let (bar_color, aside, main) = if self.dark {
            (0x213d5b, 0x1d3043, 0x18222c)
        } else {
            (0xc6e2ff, 0xd9ecff, 0xecf5ff)
        };
        let labels = [
            s.layout_header.as_str(),
            s.layout_aside.as_str(),
            s.layout_main.as_str(),
            s.layout_footer.as_str(),
        ];
        let [header, side, body, footer] = labels;
        let center = |text| {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(text)
        };
        let bar = || el_header().background(rgb(bar_color)).child(center(header));
        let side = |id| {
            el_aside()
                .id(id)
                .width(200.)
                .background(rgb(aside))
                .child(center(side))
        };
        let body = || el_main().padding(0.).child(center(body).bg(rgb(main)));
        let foot = || el_footer().background(rgb(bar_color)).child(center(footer));
        let layouts = [
            (
                "Header / Main",
                210.,
                el_container().child(bar()).child(body()),
            ),
            (
                "Header / Main / Footer",
                270.,
                el_container().child(bar()).child(body()).child(foot()),
            ),
            (
                "Aside / Main / Aside",
                150.,
                el_container()
                    .child(side("left"))
                    .child(body())
                    .child(side("right")),
            ),
            (
                "Header / (Aside + Main)",
                210.,
                el_container()
                    .child(bar())
                    .child(el_container().child(side("left")).child(body())),
            ),
            (
                "Header / (Aside + (Main / Footer))",
                270.,
                el_container().child(bar()).child(
                    el_container()
                        .child(side("left"))
                        .child(el_container().child(body()).child(foot())),
                ),
            ),
            (
                "Aside + (Header / Main)",
                210.,
                el_container()
                    .child(side("left"))
                    .child(el_container().child(bar()).child(body())),
            ),
            (
                "Aside + (Header / Main / Footer)",
                270.,
                el_container()
                    .child(side("left"))
                    .child(el_container().child(bar()).child(body()).child(foot())),
            ),
            (
                "Header / (Aside + Main) / Footer",
                270.,
                el_container()
                    .child(bar())
                    .child(el_container().child(side("left")).child(body()))
                    .child(foot()),
            ),
        ];
        self.section(
            cx,
            "container",
            &s.container_title,
            "",
            div().flex().flex_col().gap(px(24.)).children(layouts.map(
                |(label, height, layout)| {
                    div().flex().flex_col().gap(px(8.)).child(label).child(
                        div()
                            .id(label)
                            .overflow_x_scroll()
                            .child(div().flex().min_w(px(480.)).h(px(height)).child(layout)),
                    )
                },
            )),
        )
    }

    fn grids(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        use gpui_el::{RowAlign, RowJustify};

        // Element Plus docs/examples/layout/index.scss: demo colors, not component defaults.
        let (panel, colors) = if self.dark {
            (0x18191a, [0x46494d, 0x667180, 0x242526])
        } else {
            (0xf9fafc, [0xd3dce6, 0xe5e9f2, 0x99a9bf])
        };
        let block =
            |tone: usize, height: f32| div().h(px(height)).rounded(px(4.)).bg(rgb(colors[tone]));
        let columns = |spans: &[u8], gutter| {
            el_row()
                .gutter(gutter)
                .cols(spans.iter().enumerate().map(|(i, &span)| {
                    let tone = if spans.len() == 1 { 2 } else { i % 2 };
                    el_col().span(span).child(block(tone, 36.))
                }))
        };
        let stack = || div().flex().flex_col().gap(px(20.));
        let basic = stack().children(
            [&[24][..], &[12, 12], &[8, 8, 8], &[6; 4], &[4; 6]].map(|spans| columns(spans, 0.)),
        );
        let gutter = columns(&[6; 4], 20.);
        let mixed = stack()
            .children([&[16, 8][..], &[8, 8, 4, 4], &[4, 16, 4]].map(|spans| columns(spans, 20.)));
        let offset = stack()
            .child(
                el_row()
                    .gutter(20.)
                    .col(el_col().span(6).child(block(0, 36.)))
                    .col(el_col().span(6).offset(6).child(block(0, 36.))),
            )
            .child(
                el_row()
                    .gutter(20.)
                    .cols((0..2).map(|_| el_col().span(6).offset(6).child(block(0, 36.)))),
            )
            .child(
                el_row()
                    .gutter(20.)
                    .col(el_col().span(12).offset(6).child(block(0, 36.))),
            )
            .child(
                el_row()
                    .gutter(20.)
                    .col(el_col().span(12).push(12).child(block(0, 36.)))
                    .col(el_col().span(12).pull(12).child(block(1, 36.))),
            );
        let alignment = stack()
            .children(
                [
                    ("start", RowJustify::Start),
                    ("center", RowJustify::Center),
                    ("end", RowJustify::End),
                    ("space-between", RowJustify::SpaceBetween),
                    ("space-around", RowJustify::SpaceAround),
                    ("space-evenly", RowJustify::SpaceEvenly),
                ]
                .map(|(name, justify)| {
                    div().flex().flex_col().gap(px(4.)).child(name).child(
                        div()
                            .py(px(10.))
                            .bg(rgb(panel))
                            .child(columns(&[6; 3], 0.).justify(justify)),
                    )
                }),
            )
            .child(
                stack().children(
                    [
                        ("top", RowAlign::Top),
                        ("middle", RowAlign::Middle),
                        ("bottom", RowAlign::Bottom),
                    ]
                    .map(|(name, align)| {
                        div().flex().flex_col().gap(px(4.)).child(name).child(
                            div().py(px(10.)).bg(rgb(panel)).child(
                                el_row()
                                    .gutter(20.)
                                    .align(align)
                                    .col(el_col().span(8).child(block(0, 24.)))
                                    .col(el_col().span(8).child(block(1, 56.)))
                                    .col(el_col().span(8).child(block(0, 36.))),
                            ),
                        )
                    }),
                ),
            );
        let responsive = el_row().gutter(10.).cols((0..4).map(|i| {
            if i == 0 || i == 3 {
                el_col().xs(8).sm(6).md(4).lg(3).xl(1)
            } else {
                el_col().xs(4).sm(6).md(8).lg(9).xl(11)
            }
            .child(block(i % 2, 36.))
        }));
        stack()
            .child(self.section(cx, "grid-basic", &s.grid_basic, "", basic))
            .child(self.section(cx, "grid-gutter", &s.grid_gutter, "", gutter))
            .child(self.section(cx, "grid-mixed", &s.grid_mixed, "", mixed))
            .child(self.section(cx, "grid-offset", &s.grid_offset, "", offset))
            .child(self.section(cx, "grid-alignment", &s.grid_alignment, "", alignment))
            .child(
                self.section(
                    cx,
                    "grid-responsive",
                    &s.grid_responsive,
                    "",
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.))
                        .child(s.grid_resize.as_str())
                        .child(responsive),
                ),
            )
    }

    fn spaces(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let buttons = |space: Space, id: &'static str| {
            space.children(
                [s.default.as_str(), s.primary.as_str(), s.success.as_str()]
                    .into_iter()
                    .enumerate()
                    .map(|(i, label)| {
                        el_button()
                            .id((id, i))
                            .label(label)
                            .when(i == 1, |button| button.primary())
                    }),
            )
        };
        let presets = [
            ("implicit", "8 px", el_space()),
            ("small", "8 px · small()", el_space().small()),
            (
                "default",
                "12 px",
                el_space().size(gpui_el::SpaceSize::Default),
            ),
            ("large", "16 px", el_space().large()),
        ];
        div()
            .flex()
            .flex_col()
            .gap(px(24.))
            .child(
                self.section(
                    cx,
                    "space-presets",
                    &s.space_presets,
                    "",
                    div().flex().flex_col().gap(px(16.)).children(presets.map(
                        |(id, label, space)| {
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(8.))
                                .child(label)
                                .child(buttons(space, id))
                        },
                    )),
                ),
            )
            .child(
                self.section(
                    cx,
                    "space-wrap",
                    &s.space_title,
                    "",
                    div()
                        .w(px(220.))
                        .max_w_full()
                        .child(buttons(el_space().large().wrap(true), "wrapped")),
                ),
            )
    }

    fn initialization(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let standard = "use gpui::App;\nuse gpui_el::{el_init, IconPack, Locale};\n\nfn configure(cx: &mut App) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {\n    el_init!(cx, locale: Locale::SimplifiedChinese, icons: IconPack::ElementPlus)\n}";
        let custom = "use gpui::App;\nuse gpui_el::{el_init, IconPack, Locale};\n\nfn configure_with_font(cx: &mut App) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {\n    el_init!(cx, locale: Locale::English, icons: IconPack::Tabler,\n        fonts: [include_bytes!(\"assets/icons.ttf\")])\n}";
        self.section(
            cx,
            "init-example",
            &s.init_title,
            format!("{standard}\n\n{custom}"),
            div()
                .flex()
                .flex_col()
                .gap(px(12.))
                .child(s.init_intro.as_str())
                .child(s.init_fonts.as_str())
                .child(div().font_family("monospace").child(standard))
                .child(div().font_family("monospace").child(custom)),
        )
    }

    fn menus(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let submenu = || {
            el_sub_menu("settings", s.menu_second.as_str())
                .icon(el_icon("Setting"))
                .entry(el_menu_item("language", s.language.as_str()))
                .entry(el_menu_item("theme", s.dark.as_str()))
                .entry(el_menu_item("disabled", s.menu_disabled.as_str()).disabled(true))
        };
        let home = || el_menu_item("overview", s.menu_first.as_str()).icon(el_icon("Grid"));
        div()
            .flex()
            .flex_col()
            .gap(px(24.))
            .child(
                self.section(
                    cx,
                    "menu-vertical",
                    &s.vertical_menu,
                    "",
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(12.))
                        .items_start()
                        .child(
                            el_button()
                                .id("collapse-menu")
                                .small()
                                .label(if self.menu_collapsed {
                                    s.menu_expand.as_str()
                                } else {
                                    s.menu_collapse.as_str()
                                })
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.menu_collapsed = !this.menu_collapsed;
                                    cx.notify();
                                })),
                        )
                        .child(
                            div().w(px(224.)).max_w_full().child(
                                el_menu()
                                    .id("vertical-demo")
                                    .collapse(self.menu_collapsed)
                                    .default_active("overview")
                                    .entry(home())
                                    .entry(submenu()),
                            ),
                        ),
                ),
            )
            .child(
                self.section(
                    cx,
                    "menu-horizontal",
                    &s.horizontal_menu,
                    "",
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(12.))
                        .child(
                            el_menu()
                                .id("horizontal-demo")
                                .mode(MenuMode::Horizontal)
                                .menu_trigger(MenuTrigger::Click)
                                .close_on_click_outside(true)
                                .default_active("overview")
                                .entry(home())
                                .entry(submenu())
                                .entry(el_menu_item("motion", s.reduce_motion.as_str()))
                                .on_select(cx.listener(
                                    |this, event: &gpui_el::MenuEvent, _, cx| {
                                        this.menu_selection = Some(
                                            event
                                                .index_path
                                                .iter()
                                                .map(|part| part.as_ref())
                                                .collect::<Vec<_>>()
                                                .join(" / "),
                                        );
                                        cx.notify();
                                    },
                                )),
                        )
                        .child(s.menu_index_path.replace(
                            "%{path}",
                            self.menu_selection.as_deref().unwrap_or("overview"),
                        )),
                ),
            )
    }

    fn loading(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        use gpui_el::{LoadingMotion, LoadingSpinner};
        let border = rgb(if self.dark { 0x414243 } else { 0xe4e7ed });
        let panel = |label| {
            div()
                .h(px(120.))
                .p(px(16.))
                .border_1()
                .border_color(border)
                .child(label)
        };
        self.section(
            cx,
            "mask",
            &s.loading_page,
            "",
            div()
                .flex()
                .flex_col()
                .gap(px(12.))
                .items_stretch()
                .child(
                    div().child(
                        el_button()
                            .id("toggle-mask")
                            .primary()
                            .label(if self.masked {
                                s.hide_loading.as_str()
                            } else {
                                s.show_loading.as_str()
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.masked = !this.masked;
                                cx.notify();
                            })),
                    ),
                )
                .child(
                    el_loading()
                        .id("element-plus-mask")
                        .spinner(LoadingSpinner::Spokes)
                        .v_text(s.loading_text.as_str())
                        .active(self.masked)
                        .child(panel(s.loading_vertical.as_str())),
                )
                .child(
                    el_loading()
                        .id("dots-mask")
                        .h_text(s.loading_text.as_str())
                        .active(self.masked)
                        .spinner(LoadingSpinner::Dots)
                        .motion(LoadingMotion::EaseIn)
                        .mask_opacity(0.40)
                        .child(panel(s.loading_horizontal.as_str())),
                )
                .child(
                    el_loading()
                        .id("custom-icon-mask")
                        .h_text(s.loading_text.as_str())
                        .active(self.masked)
                        .loading_icon(
                            Icon::from_pack(IconPack::ElementPlus, "RefreshRight").size(28.),
                        )
                        .mask_opacity(0.)
                        .child(panel(s.loading_clear.as_str())),
                ),
        )
    }
    fn titlebar(
        &self,
        window: &Window,
        cx: &Context<Self>,
        s: &'static Copy,
        locale: Locale,
    ) -> Div {
        let border = rgb(if self.dark { 0x414243 } else { 0xe4e7ed });
        let icon_button = |id, name, label| {
            el_button()
                .id(id)
                .text()
                .circle()
                .icon(Icon::from_pack(IconPack::ElementPlus, name).size(16.))
                .accessibility_label(label)
                .tooltip(label)
        };
        let drag = div()
            .id("caption-drag")
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .items_center()
            .window_control_area(WindowControlArea::Drag)
            .on_mouse_down(MouseButton::Left, |_, window, _| {
                #[cfg(not(target_os = "windows"))]
                window.start_window_move();
            })
            .on_click(|event, window, _| {
                if event.click_count() == 2 {
                    if cfg!(target_os = "macos") {
                        window.titlebar_double_click();
                    } else if !cfg!(target_os = "windows") {
                        window.zoom_window();
                    }
                }
            })
            .child(
                div()
                    .truncate()
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("gpui-el component"),
            );
        let tools = el_space()
            .gap(4., 0.)
            .child(
                el_button()
                    .id("language")
                    .text()
                    .circle()
                    .icon(Icon::from_pack(IconPack::Tabler, "language").size(16.))
                    .accessibility_label(s.language.as_str())
                    .tooltip(s.language.as_str())
                    .on_click(move |_, _, cx| {
                        if locale == Locale::SimplifiedChinese {
                            Locale::English
                        } else {
                            Locale::SimplifiedChinese
                        }
                        .set(cx);
                    }),
            )
            .child(
                icon_button(
                    "theme",
                    if self.dark { "Sunny" } else { "Moon" },
                    if self.dark {
                        s.light.as_str()
                    } else {
                        s.dark.as_str()
                    },
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.dark = !this.dark;
                    this.apply_theme(cx);
                })),
            )
            .child(
                icon_button(
                    "motion",
                    "VideoPause",
                    if self.reduced_motion {
                        s.restore_motion.as_str()
                    } else {
                        s.reduce_motion.as_str()
                    },
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.reduced_motion = !this.reduced_motion;
                    this.apply_theme(cx);
                })),
            );
        div()
            .flex()
            .items_center()
            .gap(px(12.))
            .h(px(52.))
            .px(px(16.))
            .flex_shrink_0()
            .border_b_1()
            .border_color(border)
            .when(cfg!(target_os = "macos") && !window.is_fullscreen(), |el| {
                el.pl(px(82.))
            })
            .child(drag)
            .child(tools)
            .when(
                !cfg!(target_os = "macos") && !window.is_fullscreen(),
                |el| {
                    el.child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(2.))
                            .pl(px(8.))
                            .border_l_1()
                            .border_color(border)
                            .child(
                                icon_button("minimize", "Minus", s.window_minimize.as_str())
                                    .on_click(|_, w, _| w.minimize_window()),
                            )
                            .child(
                                icon_button(
                                    "zoom",
                                    if window.is_maximized() {
                                        "CopyDocument"
                                    } else {
                                        "FullScreen"
                                    },
                                    s.window_zoom.as_str(),
                                )
                                .on_click(|_, w, _| w.zoom_window()),
                            )
                            .child(
                                icon_button("close", "Close", s.window_close.as_str())
                                    .on_click(|_, w, _| w.remove_window()),
                            ),
                    )
                },
            )
    }
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let locale = Locale::current(cx);
        let s = strings(locale);
        let page = self.page;
        let narrow = window.viewport_size().width < px(720.);
        let item = |target: Page| el_menu_item(target.key(), target.title(s));
        let nav = el_menu()
            .id("catalog")
            .active_index(page.key())
            .mode(if narrow {
                MenuMode::Horizontal
            } else {
                MenuMode::Vertical
            })
            .menu_trigger(MenuTrigger::Click)
            .close_on_click_outside(true)
            .on_select(cx.listener(|this, event: &gpui_el::MenuEvent, _, cx| {
                if let Some(page) = Page::ALL
                    .into_iter()
                    .find(|page| page.key() == event.index.as_ref())
                {
                    this.page = page;
                    cx.notify();
                }
            }));
        let nav = if narrow {
            nav.entries(Page::ALL.map(item)).into_any_element()
        } else {
            el_aside()
                .id("catalog-scroll")
                .width(196.)
                .end_border(rgb(if self.dark { 0x414243 } else { 0xdcdfe6 }))
                .child(
                    nav.border(false)
                        .entry(item(Page::Init))
                        .entry(
                            el_menu_item_group(s.basics.as_str()).items(
                                [
                                    Page::Button,
                                    Page::Icon,
                                    Page::Container,
                                    Page::Layout,
                                    Page::Space,
                                ]
                                .map(item),
                            ),
                        )
                        .entry(el_menu_item_group(s.navigation.as_str()).item(item(Page::Menu)))
                        .entry(el_menu_item_group(s.feedback.as_str()).item(item(Page::Loading))),
                )
                .into_any_element()
        };
        let content = match page {
            Page::Button => self.buttons(s, cx),
            Page::Icon => self.icons(window, s, cx),
            Page::Init => self.initialization(s, cx),
            Page::Container => self.containers(s, cx),
            Page::Layout => self.grids(s, cx),
            Page::Space => self.spaces(s, cx),
            Page::Menu => self.menus(s, cx),
            Page::Loading => self.loading(s, cx),
        };
        div()
            .id("gallery")
            .tab_group()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(if self.dark { 0x141414 } else { 0xffffff }))
            .text_color(rgb(if self.dark { 0xe5eaf3 } else { 0x303133 }))
            .text_size(px(14.))
            .child(self.titlebar(window, cx, s, locale))
            .child(
                el_container()
                    .direction(if narrow {
                        Direction::Vertical
                    } else {
                        Direction::Horizontal
                    })
                    .child(nav)
                    .child(
                        el_main()
                            .id(("gallery-page", page as u32))
                            .padding(if narrow { 20. } else { 36. })
                            .child(
                                div()
                                    .w_full()
                                    .max_w(px(820.))
                                    .mx_auto()
                                    .flex()
                                    .flex_col()
                                    .gap(px(28.))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(8.))
                                            .child(
                                                div()
                                                    .text_size(px(34.))
                                                    .font_weight(gpui::FontWeight::BOLD)
                                                    .child(page.title(s)),
                                            )
                                            .when(!page.upstream_names().is_empty(), |heading| {
                                                heading.child(
                                                    div()
                                                        .flex()
                                                        .flex_wrap()
                                                        .gap_x(px(12.))
                                                        .text_size(px(13.))
                                                        .text_color(rgb(if self.dark {
                                                            0xa3a6ad
                                                        } else {
                                                            0x606266
                                                        }))
                                                        .children(
                                                            page.upstream_names().iter().copied(),
                                                        ),
                                                )
                                            })
                                            .child(
                                                div()
                                                    .text_size(px(15.))
                                                    .text_color(rgb(if self.dark {
                                                        0xa3a6ad
                                                    } else {
                                                        0x606266
                                                    }))
                                                    .child(page.description(s)),
                                            ),
                                    )
                                    .child(content)
                                    .child(api::render(page, locale, self.dark)),
                            ),
                    ),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        el_init!(cx, locale: Locale::English, icons: IconPack::ElementPlus)
            .expect("initialize the gallery");
        search::bind(cx);
        Theme::light().set(cx);
        let bounds = Bounds::centered(None, size(px(1040.), px(760.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(360.), px(320.))),
                titlebar: Some(TitlebarOptions {
                    title: Some("gpui-el component".into()),
                    appears_transparent: true,
                    traffic_light_position: Some(gpui::point(px(12.), px(20.))),
                }),
                window_decorations: if cfg!(target_os = "macos") {
                    None
                } else {
                    Some(WindowDecorations::Client)
                },
                app_owns_titlebar_drag: true,
                ..Default::default()
            },
            |_, cx| cx.new(|_| Gallery::default()),
        )
        .expect("open the gallery window");
        cx.activate(true);
    });
}

#[cfg(test)]
mod tests {
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
                                "menu-horizontal",
                                "mask",
                            ]),
                            masked: true,
                            menu_collapsed: dark,
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

    #[gpui::test]
    fn icon_grid_columns_stay_equal_with_long_names_and_empty_cells(cx: &mut gpui::TestAppContext) {
        use std::{cell::RefCell, rc::Rc};
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
            // The probe measures the inside of the cell, excluding its 1 px borders.
            for &(row, column, bounds) in measured.iter() {
                let cell_width = width / columns as f32;
                assert!((f32::from(bounds.size.width) + 1. - cell_width).abs() <= 1.);
                assert_eq!(bounds.size.height, px(height - 1.));
                assert!((f32::from(bounds.origin.x) - cell_width * column as f32).abs() <= 1.);
                assert_eq!(bounds.origin.y, px(row as f32 * height));
            }
        }
    }

    #[test]
    fn both_catalogs_load_and_preserve_the_count_placeholder() {
        let en = strings(Locale::English);
        let zh = strings(Locale::SimplifiedChinese);
        assert_ne!(en.button_intro, zh.button_intro);
        assert_ne!(en.saving, zh.saving);
        for catalog in [en, zh] {
            assert_eq!(catalog.activation_help.matches("%{count}").count(), 1);
        }
    }
}
