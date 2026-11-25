//! Native GPUI components following Element Plus defaults.
//!
//! ```
//! use gpui_el::el_button;
//!
//! let default = el_button().label("Cancel");
//! let compact = el_button().label("Save").primary().round().small().font_size(13.0);
//! ```
//!
//! This crate uses the crates.io `gpui-pre` package (`0.3.14`, library name
//! `gpui`). Host applications must use the same GPUI package and version for
//! shared types such as `Window` and `App`. The gallery uses the matching
//! `gpui-pre-platform` desktop backend.
//!
//! From the package source, run `cargo run --example gallery`. The example
//! needs a graphical desktop session and GPUI's native libraries. On Linux,
//! this includes a working graphics backend, FreeType, and the X11/Wayland
//! libraries used by the selected backend.
//!
//! `el_icon("Plus").size(16.0)` uses the bundled Element Plus SVGs by default.
//! Configure the locale, icon pack, and optional font bytes once on startup:
//!
//! ```
//! use gpui_el::{IconPack, Locale, el_init};
//! # fn configure(cx: &mut gpui::App) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//! el_init!(cx, locale: Locale::SimplifiedChinese, icons: IconPack::Tabler)?;
//! # Ok(())
//! # }
//! ```
//!
//! `Icon::from_pack` overrides the preference for one icon. Use
//! `register_icon_font` and `Icon::from_font` for application-supplied glyphs.
//!
//! GPUI dispatches input and focus; components translate that into callbacks.
//! Buttons expose `on_click`, while menus expose `on_select`, `on_open`, and
//! `on_close` with the original index path (not the visual overflow path):
//!
//! ```
//! use gpui_el::{el_menu, el_menu_item, el_sub_menu};
//! let menu = el_menu()
//!     .default_active("overview")
//!     .entry(el_menu_item("overview", "Overview"))
//!     .entry(el_sub_menu("settings", "Settings")
//!         .entry(el_menu_item("theme", "Theme")))
//!     .on_select(|event, _, _| println!("{:?}", event.index_path));
//! ```
//!
//! Tags mark content and forward their own removal; check tags toggle:
//!
//! ```
//! use gpui_el::{el_check_tag, el_tag};
//! let tag = el_tag().label("Tag 1").success().round(true).closable(true);
//! let check = el_check_tag().label("Toggle me").checked(true);
//! ```
//!
//! Menu follows the Element Plus `menu.md`, `components/menu/src`, and
//! Theme Chalk `menu.scss` baseline. Native differences: there is no Vue
//! router, DOM teleport, CSS class, or persistent DOM. `popup_style` overrides
//! submenu surfaces via GPUI; collapse and popup visibility change immediately
//! rather than reproducing CSS enter/exit transitions. Horizontal width is
//! measured from the menu itself and overflow is updated after layout.
//!
//! Button styles were checked against Element Plus Theme Chalk's `button.scss`,
//! `mixins/_button.scss`, and light/dark variables. The native sample switches
//! pointer states immediately rather than reproducing the browser's 100 ms
//! color transition. HTML form attributes are not part of the API. The default
//! button loading indicator follows the upstream rotating spokes; the standalone
//! `Loading` uses a circular dash and a scoped mask, not a modal service.
//!
//! Tag and CheckTag follow `tag.md`, `check-tag.md`, `components/tag/src`,
//! `components/check-tag/src`, and Theme Chalk `tag.scss` / `check-tag.scss`.
//! Native differences: there is no Vue transition scope, so tags mount and
//! unmount immediately instead of playing `el-zoom-in-center`, and the
//! `disable-transitions` property is therefore not reproduced. `color` overrides
//! only the background, as upstream's inline style does. CheckTag is focusable
//! and keyboard activatable, which the upstream `span` is not.

mod button;
mod grid;
mod icon;
mod layout;
mod loading;
mod locale;
mod menu;
mod tag;
mod theme;

pub use button::{
    Button, ButtonGroup, ButtonShape, ButtonSize, ButtonType, ButtonVariant, el_button,
    el_button_group,
};
pub use grid::{Col, ColSpec, Row, RowAlign, RowJustify, el_col, el_row};
pub use icon::{Icon, IconPack, el_icon, register_icon_font};
pub use layout::{
    Aside, Container, Direction, Footer, Header, Main, Space, SpaceAlignment, SpaceSize, el_aside,
    el_container, el_footer, el_header, el_main, el_space,
};
pub use loading::{Loading, LoadingMotion, LoadingSpinner, el_loading};
pub use locale::Locale;
pub use menu::{
    Menu, MenuEntry, MenuEvent, MenuItem, MenuItemGroup, MenuMode, MenuPopupTheme, MenuState,
    MenuTrigger, SubMenu, el_menu, el_menu_item, el_menu_item_group, el_sub_menu,
};
pub use tag::{CheckTag, Tag, TagEffect, TagSize, TagType, el_check_tag, el_tag};
pub use theme::Theme;

/// Sets the defaults for one GPUI application. Individual controls may still
/// override their icon source. Register custom fonts before opening windows.
pub fn init(
    cx: &mut gpui::App,
    locale: Locale,
    icons: IconPack,
    fonts: &[&'static [u8]],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !fonts.is_empty() {
        cx.text_system().add_fonts(
            fonts
                .iter()
                .copied()
                .map(std::borrow::Cow::Borrowed)
                .collect(),
        )?;
    }
    icons.set(cx)?;
    locale.set(cx);
    Ok(())
}

/// Configures the locale, default icon pack, and optional custom font bytes.
/// Call from `Application::run` and handle the returned `Result`.
#[macro_export]
macro_rules! el_init {
    ($cx:expr, locale: $locale:expr, icons: $icons:expr $(, fonts: [$($font:expr),* $(,)?])? $(,)?) => {
        $crate::init($cx, $locale, $icons, &[$($($font),*)?])
    };
}
