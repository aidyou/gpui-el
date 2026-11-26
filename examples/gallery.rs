use std::collections::HashSet;

use gpui::{
    AnyElement, App, AppContext, Bounds, Context, Div, IntoElement, MouseButton, Render,
    ResizeEdge, Stateful, TitlebarOptions, Window, WindowAppearance, WindowBounds,
    WindowDecorations, WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_el::{
    Direction, IconPack, Locale, MenuEvent, MenuMode, MenuTrigger, Theme, el_aside, el_container,
    el_init, el_main, el_menu, el_menu_item, el_sub_menu, tokens,
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
#[cfg(test)]
#[path = "gallery/perf.rs"]
mod perf;
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
    /// Press origin of a title-bar drag, used to tell a drag from a double
    /// click before the compositor takes over the pointer.
    caption_press: Option<gpui::Point<gpui::Pixels>>,
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

/// Catalog icons: thin Element Plus outlines of comparable ink weight, so the
/// group rows read as one family instead of mixing solid and line glyphs.
mod nav_icon {
    pub const INIT: &str = "guide";
    pub const BASICS: &str = "collection";
    pub const DATA: &str = "pie-chart";
    pub const NAVIGATION: &str = "compass";
    pub const FEEDBACK: &str = "chat-line-round";
}

/// The catalog chrome stays on one pack even when the icon page switches the
/// application preference, so the group icons never mix stroke and fill styles.
fn nav_glyph(name: &str) -> gpui_el::Icon {
    gpui_el::Icon::from_pack(IconPack::ElementPlus, name)
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
    Text,
    Space,
    Table,
    Tag,
    Menu,
    Loading,
}

impl Page {
    const ALL: [Self; 12] = [
        Self::Init,
        Self::Button,
        Self::Icon,
        Self::Container,
        Self::Layout,
        Self::Link,
        Self::Text,
        Self::Space,
        Self::Table,
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
            Self::Text => &s.text_intro,
            Self::Space => &s.space_intro,
            Self::Table => &s.table_intro,
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
            Self::Text => "text",
            Self::Space => "space",
            Self::Table => "table",
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
            Self::Text => &s.text_page,
            Self::Space => &s.space_page,
            Self::Table => &s.table_page,
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
            Self::Text => &["el-text"],
            Self::Space => &["el-space"],
            Self::Table => &["el-table", "el-table-column"],
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
                .width(208.)
                .background(rgb(if self.dark { 0x191a1b } else { 0xf7f8fa }))
                .end_border(rgb(if self.dark { 0x2a2b2c } else { 0xe4e7ed }))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .h(px(44.))
                                .px(px(16.))
                                .child(
                                    div()
                                        .text_size(px(tokens::font_size::BASE))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .child("gpui-el"),
                                )
                                .child(
                                    div()
                                        .text_size(px(tokens::font_size::EXTRA_SMALL))
                                        .text_color(rgb(if self.dark {
                                            0x8d9095
                                        } else {
                                            0x909399
                                        }))
                                        .child(env!("CARGO_PKG_VERSION")),
                                ),
                        )
                        .child(
                            nav.border(false)
                                .entry(item(Page::Init).icon(nav_glyph(nav_icon::INIT)))
                                .entry(
                                    el_sub_menu("basics", s.basics.as_str())
                                        .icon(nav_glyph(nav_icon::BASICS))
                                        .entries(
                                            [
                                                Page::Button,
                                                Page::Icon,
                                                Page::Container,
                                                Page::Layout,
                                                Page::Link,
                                                Page::Text,
                                                Page::Space,
                                            ]
                                            .map(item),
                                        ),
                                )
                                .entry(
                                    el_sub_menu("data", s.data.as_str())
                                        .icon(nav_glyph(nav_icon::DATA))
                                        .entries([Page::Table, Page::Tag].map(item)),
                                )
                                .entry(
                                    el_sub_menu("navigation", s.navigation.as_str())
                                        .icon(nav_glyph(nav_icon::NAVIGATION))
                                        .entry(item(Page::Menu)),
                                )
                                .entry(
                                    el_sub_menu("feedback", s.feedback.as_str())
                                        .icon(nav_glyph(nav_icon::FEEDBACK))
                                        .entry(item(Page::Loading)),
                                ),
                        ),
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
            Page::Text => self.texts(s, cx),
            Page::Space => self.spaces(s, cx),
            Page::Table => self.tables(s, cx),
            Page::Tag => self.tags(s, cx),
            Page::Menu => self.menus(s, cx),
            Page::Loading => self.loading(s, cx),
        };
        let secondary = rgb(if self.dark { 0xa3a6ad } else { 0x606266 });
        let chip = rgb(if self.dark { 0x262727 } else { 0xf2f3f5 });
        let heading = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(
                div()
                    .text_size(px(28.))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child(page.title(s)),
            )
            .when(!page.upstream_names().is_empty(), |heading| {
                heading.child(div().flex().flex_wrap().gap(px(6.)).children(
                    page.upstream_names().iter().map(|name| {
                        div()
                            .px(px(6.))
                            .py(px(2.))
                            .rounded(px(tokens::radius::SMALL))
                            .bg(chip)
                            .font_family("monospace")
                            .text_size(px(tokens::font_size::EXTRA_SMALL))
                            .text_color(secondary)
                            .child(*name)
                    }),
                ))
            })
            .child(
                div()
                    .text_size(px(tokens::font_size::BASE + 1.))
                    .line_height(px(21.))
                    .text_color(secondary)
                    .child(page.description(s)),
            );
        div()
            .id("gallery")
            .tab_group()
            .relative()
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
                                    .max_w(px(760.))
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
            .children(resize_handles(window))
    }
}

/// Edge and corner strips that give the client-decorated window native
/// drag-to-resize behavior on every side.
fn resize_handles(window: &Window) -> Vec<AnyElement> {
    if window.is_maximized() || window.is_fullscreen() {
        return Vec::new();
    }
    fn handle(
        id: &'static str,
        edge: ResizeEdge,
        place: impl FnOnce(Stateful<Div>) -> Stateful<Div>,
        cursor: impl FnOnce(Stateful<Div>) -> Stateful<Div>,
    ) -> AnyElement {
        cursor(place(div().id(id).absolute().on_mouse_down(
            MouseButton::Left,
            move |_, window, _| {
                window.start_window_resize(edge);
            },
        )))
        .into_any_element()
    }
    vec![
        handle(
            "resize-top",
            ResizeEdge::Top,
            |el| el.top_0().left(px(10.)).right(px(10.)).h(px(5.)),
            |el| el.cursor_ns_resize(),
        ),
        handle(
            "resize-bottom",
            ResizeEdge::Bottom,
            |el| el.bottom_0().left(px(10.)).right(px(10.)).h(px(5.)),
            |el| el.cursor_ns_resize(),
        ),
        handle(
            "resize-left",
            ResizeEdge::Left,
            |el| el.left_0().top(px(10.)).bottom(px(10.)).w(px(5.)),
            |el| el.cursor_ew_resize(),
        ),
        handle(
            "resize-right",
            ResizeEdge::Right,
            |el| el.right_0().top(px(10.)).bottom(px(10.)).w(px(5.)),
            |el| el.cursor_ew_resize(),
        ),
        handle(
            "resize-top-left",
            ResizeEdge::TopLeft,
            |el| el.top_0().left_0().size(px(12.)),
            |el| el.cursor_nwse_resize(),
        ),
        handle(
            "resize-top-right",
            ResizeEdge::TopRight,
            |el| el.top_0().right_0().size(px(12.)),
            |el| el.cursor_nesw_resize(),
        ),
        handle(
            "resize-bottom-right",
            ResizeEdge::BottomRight,
            |el| el.bottom_0().right_0().size(px(12.)),
            |el| el.cursor_nwse_resize(),
        ),
        handle(
            "resize-bottom-left",
            ResizeEdge::BottomLeft,
            |el| el.bottom_0().left_0().size(px(12.)),
            |el| el.cursor_nesw_resize(),
        ),
    ]
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
        let bounds = Bounds::centered(None, size(px(1040.), px(760.)), cx);
        if let Err(error) = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(640.), px(480.))),
                app_id: Some("gpui-el-gallery".into()),
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
            |window, cx| {
                // Start in the system appearance; the toolbar still offers an
                // explicit light/dark switch for reviewing the component set.
                let dark = matches!(
                    window.appearance(),
                    WindowAppearance::Dark | WindowAppearance::VibrantDark
                );
                let gallery = Gallery {
                    dark,
                    ..Gallery::default()
                };
                gallery.apply_theme(cx);
                cx.new(|_| gallery)
            },
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
