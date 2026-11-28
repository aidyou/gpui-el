//! Native GPUI components following Element Plus defaults.
//!
//! ```
//! use gpui_el::el_button;
//!
//! let default = el_button().label("Cancel");
//! let compact = el_button().label("Save").primary().round().small().font_size(13.0);
//! ```
//!
//! # Errors
//!
//! Component code never panics on widget input: an unknown icon name draws
//! nothing, malformed sizes fall back to defaults, and a locale that fails to
//! parse uses the embedded English messages. Setup calls return
//! [`Result`] with the library's own [`Error`]; whether a failure is fatal is
//! the application's decision. Library sources are compiled with `unwrap`,
//! `expect`, `panic!` and index slicing denied outside tests.
//!
//! This crate uses the crates.io `gpui-fast` package (`0.1.19`, library name
//! `gpui`). Host applications must use the same GPUI package and version for
//! shared types such as `Window` and `App`. The gallery uses the matching
//! `gpui-fast-platform` desktop backend.
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
//! A Link reports activation through `onclick` and optionally opens an URL
//! through the platform handler:
//!
//! ```
//! use gpui_el::{LinkUnderline, el_link};
//! let details = el_link().label("View details").primary();
//! let external = el_link()
//!     .label("Element Plus")
//!     .href("https://element-plus.org")
//!     .underline(LinkUnderline::Never);
//! ```
//!
//! GPUI dispatches input and focus; components translate that into callbacks.
//! Buttons expose `onclick`, while menus expose `onselect`, `onopen`, and
//! `onclose` with the original index path (not the visual overflow path):
//!
//! ```
//! use gpui_el::{el_menu, el_menu_item, el_sub_menu};
//! let menu = el_menu()
//!     .default_active("overview")
//!     .entry(el_menu_item("overview", "Overview"))
//!     .entry(el_sub_menu("settings", "Settings")
//!         .entry(el_menu_item("theme", "Theme")))
//!     .onselect(|event, _, _| println!("{:?}", event.index_path));
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
//! Theme Chalk `menu.scss` baseline, with desktop-native presets:
//! [`MenuStyle::Element`] keeps the web metrics, while `Mac` (24 px rows, an
//! inset rounded accent fill, white hover text) and `Kde` (28 px flush gray
//! rows) are the compact defaults chosen per platform. `style`, `item_height`,
//! `item_radius`, `item_inset`, `hover_color`, and `popup_padding` tune
//! between them. Native differences: there is no Vue router, DOM teleport, CSS
//! class, or persistent DOM. `popup_style` overrides submenu surfaces via
//! GPUI; vertical submenu expand and collapse animate through [`Transition`]
//! like upstream's `ElCollapseTransition`, popups fade like
//! `el-zoom-in-center`, and the built-in chevron rotates in place instead of
//! swapping icons: 180° for an inline expand (∨ ↔ ∧) and 90° for a floating
//! submenu that opened beside its row (› ↔ ∨). Floating submenus nest beside
//! their parents to any depth, flipping at the window edge. Horizontal width
//! is measured from the menu itself and overflow is updated after layout.
//!
//! [`Transition`] collects Theme Chalk's `el-fade-in`, `el-fade-in-linear`,
//! `el-zoom-in-center`, `el-collapse-transition`, and
//! `horizontal-collapse-transition` into one declarative component that keeps
//! content mounted while it leaves. GPUI has no transform animation, so the
//! zoom effects reduce to their opacity channel. Motion curves and the
//! animation clock are shared with `Loading`, whose spinner turns use the
//! same easing and reduced-motion handling. Durations are the brief desktop
//! variants of Theme Chalk's web values, and [`MotionSpeed`] presets
//! (`Fast`, `Normal`, `Slow`, `Custom`) scale every animation: set it once on
//! the theme's `motion_speed` field, or per component through
//! ``fast()`/`slow()` on a component, or `Transition::speed`,
//! `Menu::speed`, and `Loading::speed` for an explicit preset.
//!
//! Table follows `table.md`, `components/table/src`, and Theme Chalk
//! `table.scss` for its basic subset: typed columns with fixed or distributed
//! widths, header and cell alignment, stripes, borders, three sizes, a fixed
//! header over a scrolling body, status row colors, row clicks, and an empty
//! state. Sorting, selection, filtering, fixed columns, tree data, and summary
//! rows are not reproduced. The gallery's API and event reference tables are
//! built with it.
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
//!
//! Link follows `link.md`, `components/link/src`, and Theme Chalk
//! `link.scss`. Native differences: `href` opens the URL through the platform
//! handler, `target` is omitted because a desktop application has no frames to
//! target, and the deprecated boolean form of `underline` is not reproduced.
//! The upstream anchor's `outline: none` is replaced by a visible focus ring
//! because the native link is keyboard-focusable.
//!
//! Text follows `text.md`, `components/text/src`, and Theme Chalk
//! `text.scss`. Native differences: the `tag` property has no GPUI equivalent,
//! `isTruncated` is a layout effect of `truncated` under a definite width
//! rather than a watched ref, and decorations come from GPUI's underline and
//! strikethrough instead of a CSS `text-decoration` shorthand.
//!
//! Splitter follows `splitter.md`, `components/splitter/src`, and Theme
//! Chalk `splitter.scss`. Panel sizes are logical pixels in the component
//! state, so a drag survives re-rendering, and the container is measured at
//! paint time. Native differences: the upstream pixel/percentage mix is not
//! reproduced, `lazy` moves only the divider line until release, the collapse
//! arrows stay visible at low opacity instead of appearing on hover, and the
//! dividers are keyboard-focusable, which the upstream `div` is not.
//!
//! Scrollbar follows `scrollbar.md`, `components/scrollbar/src`, and Theme
//! Chalk `scrollbar.scss`. GPUI overflow containers have no scrollbar chrome,
//! so the component draws the 6 px thumbs itself and reports DOM-like
//! positive scroll positions. The height or maximum height constrains the
//! scroll box itself, so a tall list never paints over the next section.
//! Native differences: content wider than the viewport must state a width
//! because GPUI has no inline-block wrapper, the gradient `mask` property,
//! `wrap`/`view` class and style hooks and `noresize` are not reproduced,
//! `setScrollTop`, `setScrollLeft` and `scrollTo` become a
//! [`gpui::ScrollHandle`] passed through `handle`, and thumbs show on hover
//! or with `always` without the upstream fade. The wheel is stopped at the
//! viewport like the other scroll containers.

#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented,
        clippy::indexing_slicing,
    )
)]

mod auto_complete;
mod button;
mod capsule;
mod cascader;
mod checkbox;
mod color_picker;
mod date_picker;
mod error;
mod grid;
mod icon;
mod input;
mod input_number;
mod layout;
mod link;
mod loading;
mod locale;
mod menu;
pub mod motion;
mod radio;
mod rate;
mod scrollbar;
mod select;
mod slider;
mod splitter;
mod switch;
mod table;
mod tag;
mod text;
mod theme;
mod time_picker;
mod transition;

pub use auto_complete::{AutoComplete, AutoCompleteOption, el_auto_complete};
pub use button::{
    Button, ButtonGroup, ButtonShape, ButtonSize, ButtonType, ButtonVariant, el_button,
    el_button_group,
};
pub use capsule::{CapsuleOption, CapsuleSegments, el_capsule_segments};
pub use cascader::{Cascader, CascaderOption, CascaderSelection, el_cascader};
pub use checkbox::{Checkbox, CheckboxGroup, CheckboxSize, el_checkbox, el_checkbox_group};
pub use color_picker::{ColorFormat, ColorPicker, el_color_picker};
pub use date_picker::{DatePicker, el_date_picker};
pub use error::{Error, Result};
pub use grid::{Col, ColSpec, Row, RowAlign, RowJustify, el_col, el_row};
pub use icon::{Icon, IconPack, el_icon, register_icon_font};
pub use input::{Input, InputRadius, InputSize, el_input};
pub use input_number::{InputNumber, InputNumberAlign, InputNumberValueOnClear, el_input_number};
pub use layout::{
    Aside, Container, Direction, Footer, Header, Main, Space, SpaceAlignment, SpaceSize, el_aside,
    el_container, el_footer, el_header, el_main, el_space,
};
pub use link::{Link, LinkType, LinkUnderline, el_link};
pub use loading::{Loading, LoadingMotion, LoadingSpinner, el_loading};
pub use locale::Locale;
pub use menu::{
    Menu, MenuEntry, MenuEvent, MenuItem, MenuItemGroup, MenuMode, MenuPopupTheme, MenuState,
    MenuStyle, MenuTrigger, SubMenu, el_menu, el_menu_item, el_menu_item_group, el_sub_menu,
};
pub use motion::MotionSpeed;
pub use radio::{Radio, RadioGroup, RadioSize, el_radio, el_radio_group};
pub use rate::{Rate, RateSize, el_rate};
pub use scrollbar::{Scrollbar, ScrollbarDirection, ScrollbarEvent, el_scrollbar};
pub use select::{Select, SelectOption, SelectValue, el_select};
pub use slider::{Slider, SliderMark, SliderSize, SliderValue, el_slider};
pub use splitter::{
    Splitter, SplitterCollapseEvent, SplitterLayout, SplitterPanel, SplitterResizeEvent,
    SplitterSide, SplitterSizes, el_splitter, el_splitter_panel,
};
pub use switch::{Switch, SwitchSize, el_switch};
pub use table::{Column, Table, TableAlign, TableSize, el_table};
pub use tag::{CheckTag, Tag, TagEffect, TagSize, TagType, el_check_tag, el_tag};
pub use text::{Text, TextSize, TextType, el_text};
pub use theme::{Theme, tokens};
pub use time_picker::{TimeFormat, TimePicker, el_time_picker};
pub use transition::{Transition, TransitionEffect, el_transition};

/// Sets the defaults for one GPUI application. Individual controls may still
/// override their icon source. Register custom fonts before opening windows.
pub fn init(
    cx: &mut gpui::App,
    locale: Locale,
    icons: IconPack,
    fonts: &[&'static [u8]],
) -> Result<()> {
    if !fonts.is_empty() {
        cx.text_system()
            .add_fonts(
                fonts
                    .iter()
                    .copied()
                    .map(std::borrow::Cow::Borrowed)
                    .collect(),
            )
            .map_err(Error::font)?;
    }
    icons.set(cx)?;
    locale.set(cx);
    Ok(())
}

/// Configures the locale, default icon pack, and optional custom font bytes.
/// Call from `Application::run` and handle the returned [`Result`].
#[macro_export]
macro_rules! el_init {
    ($cx:expr, locale: $locale:expr, icons: $icons:expr $(, fonts: [$($font:expr),* $(,)?])? $(,)?) => {
        $crate::init($cx, $locale, $icons, &[$($($font),*)?])
    };
}
