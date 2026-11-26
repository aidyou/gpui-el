use std::collections::HashSet;

use gpui::{
    App, AppContext, Bounds, Context, IntoElement, Render, TitlebarOptions, Window, WindowBounds,
    WindowDecorations, WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_el::{
    Direction, IconPack, Locale, MenuEvent, MenuMode, MenuTrigger, Theme, el_aside, el_container,
    el_init, el_main, el_menu, el_menu_item, el_menu_item_group, tokens,
};

#[path = "gallery/api.rs"]
mod api;
#[path = "gallery/catalog.rs"]
mod catalog;
#[path = "gallery/copy.rs"]
mod copy;
#[path = "gallery/demo.rs"]
mod demo;
#[path = "gallery/pages/mod.rs"]
mod pages;
#[path = "gallery/row.rs"]
mod row;
#[path = "gallery/search.rs"]
mod search;
#[path = "gallery/syntax.rs"]
mod syntax;
#[path = "gallery/titlebar.rs"]
mod titlebar;

use copy::{Copy, strings};

#[derive(Default)]
struct Gallery {
    dark: bool,
    reduced_motion: bool,
    page: Page,
    expanded: HashSet<&'static str>,
    button: pages::button::State,
    icon: pages::icon::State,
    link: pages::link::State,
    loading: pages::loading::State,
    menu: pages::menu::State,
    tag: pages::tag::State,
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

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Page {
    #[default]
    Button,
    Icon,
    Init,
    Container,
    Layout,
    Link,
    Space,
    Tag,
    Menu,
    Loading,
}

impl Page {
    const ALL: [Self; 10] = [
        Self::Init,
        Self::Button,
        Self::Icon,
        Self::Container,
        Self::Layout,
        Self::Link,
        Self::Space,
        Self::Tag,
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
            Self::Link => &s.link_intro,
            Self::Space => &s.space_intro,
            Self::Tag => &s.tag_intro,
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
            Self::Link => "link",
            Self::Space => "space",
            Self::Tag => "tag",
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
            Self::Link => &s.link_page,
            Self::Space => &s.space_page,
            Self::Tag => &s.tag_page,
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
            Self::Link => &["el-link"],
            Self::Space => &["el-space"],
            Self::Tag => &["el-tag", "el-check-tag"],
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
            .onselect(cx.listener(|this, event: &MenuEvent, _, cx| {
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
                                    Page::Link,
                                    Page::Space,
                                    Page::Tag,
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
            Page::Link => self.links(s, cx),
            Page::Space => self.spaces(s, cx),
            Page::Tag => self.tags(s, cx),
            Page::Menu => self.menus(s, cx),
            Page::Loading => self.loading(s, cx),
        };
        let secondary = rgb(if self.dark { 0xa3a6ad } else { 0x606266 });
        let heading = div()
            .flex()
            .flex_col()
            .gap(px(tokens::spacing::SMALL))
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
                        .gap_x(px(tokens::spacing::DEFAULT))
                        .text_size(px(tokens::font_size::SMALL))
                        .text_color(secondary)
                        .children(page.upstream_names().iter().copied()),
                )
            })
            .child(
                div()
                    .text_size(px(15.))
                    .text_color(secondary)
                    .child(page.description(s)),
            );
        div()
            .id("gallery")
            .tab_group()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(if self.dark { 0x141414 } else { 0xffffff }))
            .text_color(rgb(if self.dark { 0xe5eaf3 } else { 0x303133 }))
            .text_size(px(tokens::font_size::BASE))
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
                                    .child(heading)
                                    .child(content)
                                    .child(api::render(page, locale)),
                            ),
                    ),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        if let Err(error) = el_init!(cx, locale: Locale::English, icons: IconPack::ElementPlus) {
            eprintln!("Could not initialize the gallery: {error}");
            cx.quit();
            return;
        }
        search::bind(cx);
        demo::bind(cx);
        Theme::light().set(cx);
        let bounds = Bounds::centered(None, size(px(1040.), px(760.)), cx);
        if let Err(error) = cx.open_window(
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
        ) {
            eprintln!("Could not open the gallery window: {error}");
            cx.quit();
            return;
        }
        cx.activate(true);
    });
}

#[cfg(test)]
#[path = "gallery/tests.rs"]
mod tests;
