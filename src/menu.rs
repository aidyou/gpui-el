use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    time::Duration,
};

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Div, ElementId, Entity, FocusHandle, Hsla, IntoElement,
    ParentElement, Pixels, RenderOnce, SharedString, Task, Window, anchored, deferred, div, point,
    prelude::*, px, rgb,
};

use crate::{
    Icon, Locale, Theme, TransitionEffect, el_icon, el_transition,
    motion::{self, MotionSpeed},
    theme::{PRIMARY, popup_priority},
    tokens,
};

const MORE: &str = "\0gpui-el:more";
const COLLAPSED_WIDTH: f32 = 64.;

/// Desktop menu surface presets.
///
/// Element keeps Theme Chalk's web metrics. Mac and Kde follow native desktop
/// conventions with compact rows; macOS draws an inset rounded accent fill and
/// KDE a flush square gray fill. The default follows the platform. Individual
/// knobs on [`Menu`] (`item_height`, `item_radius`, `item_inset`,
/// `hover_color`, `popup_padding`) override any preset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuStyle {
    /// Theme Chalk web metrics: 56 px rows, full-bleed hover fill.
    Element,
    /// macOS-like: 24 px rows, inset rounded accent-blue fill, white text.
    Mac,
    /// KDE/GNOME-like: 28 px rows, flush square gray fill.
    Kde,
}

impl Default for MenuStyle {
    fn default() -> Self {
        if cfg!(target_os = "macos") {
            Self::Mac
        } else {
            Self::Kde
        }
    }
}

#[derive(Clone, Copy)]
enum HoverFill {
    /// `--el-color-primary-light-9`, used by the web preset.
    PrimaryLight,
    /// `--el-fill-color-light`, the native gray row fill.
    FillLight,
    Custom(Hsla),
}

#[derive(Clone, Copy)]
struct Metrics {
    item_height: f32,
    /// Inline nested rows, one level down.
    sub_item_height: f32,
    /// Rows inside floating popups.
    popup_item_height: f32,
    horizontal_height: f32,
    padding_x: f32,
    popup_padding_x: f32,
    /// Horizontal margin between the hover fill and the menu edge.
    item_inset: f32,
    item_radius: f32,
    popup_padding: f32,
    popup_radius: f32,
    popup_min_width: f32,
    hover: HoverFill,
    hover_text: Option<Hsla>,
    font_size: f32,
}

impl MenuStyle {
    fn metrics(self) -> Metrics {
        match self {
            MenuStyle::Element => Metrics {
                item_height: 56.,
                sub_item_height: 50.,
                popup_item_height: 36.,
                horizontal_height: 60.,
                padding_x: 20.,
                popup_padding_x: 10.,
                item_inset: 0.,
                item_radius: 0.,
                popup_padding: 5.,
                popup_radius: tokens::radius::SMALL,
                popup_min_width: 200.,
                hover: HoverFill::PrimaryLight,
                hover_text: None,
                // The web baseline is 14 px, independent of the desktop BASE.
                font_size: 14.,
            },
            // 24 px rows at 13 px with a 20 px minimum hit target, like the
            // macOS menu bar menus.
            MenuStyle::Mac => Metrics {
                item_height: 24.,
                sub_item_height: 24.,
                popup_item_height: 24.,
                horizontal_height: 28.,
                padding_x: 10.,
                popup_padding_x: 10.,
                item_inset: 5.,
                item_radius: 4.,
                popup_padding: 5.,
                popup_radius: 6.,
                popup_min_width: 200.,
                hover: HoverFill::Custom(rgb(0x0a84ff).into()),
                hover_text: Some(rgb(0xffffff).into()),
                font_size: tokens::font_size::SMALL,
            },
            // 28 px rows at 13 px, flush square highlight, like Breeze menus.
            MenuStyle::Kde => Metrics {
                item_height: 28.,
                sub_item_height: 28.,
                popup_item_height: 28.,
                horizontal_height: 28.,
                padding_x: 10.,
                popup_padding_x: 10.,
                item_inset: 0.,
                item_radius: 0.,
                popup_padding: 4.,
                popup_radius: 3.,
                popup_min_width: 200.,
                hover: HoverFill::FillLight,
                hover_text: None,
                font_size: tokens::font_size::SMALL,
            },
        }
    }
}
/// Popup fade duration, like Theme Chalk's `el-zoom-in-center` transition.
/// Normal speed for a popup fade; [`MotionSpeed`] scales it.
const FADE_SECS: f64 = 0.2;

/// One in-flight popup fade. Vertical submenu bodies animate through
/// [`Transition`] instead.
#[derive(Clone, Copy)]
struct Fade {
    started: f64,
    opening: bool,
}

fn fade_progress(fade: Option<Fade>, reduced: bool, secs: f64) -> f32 {
    match fade {
        Some(fade) if !reduced => {
            let t = ((motion::elapsed() - fade.started) / secs).clamp(0., 1.) as f32;
            if fade.opening {
                motion::element_plus(t)
            } else {
                1. - motion::element_plus(t)
            }
        }
        _ => 1.,
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MenuMode {
    #[default]
    Vertical,
    Horizontal,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MenuTrigger {
    #[default]
    Hover,
    Click,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MenuPopupTheme {
    #[default]
    Dark,
    Light,
}

/// One selection or submenu event. `index_path` includes the item itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MenuEvent {
    pub index: SharedString,
    pub index_path: Vec<SharedString>,
}

type Listener = Rc<dyn Fn(&MenuEvent, &mut Window, &mut App)>;
type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
type Content = Rc<dyn Fn() -> AnyElement>;
type PopupStyle = Rc<dyn Fn(Div) -> Div>;

pub fn el_menu() -> Menu {
    Menu::default()
}
pub fn el_menu_item(index: impl Into<SharedString>, label: impl Into<SharedString>) -> MenuItem {
    let index = index.into();
    MenuItem::new(index.clone(), label).index(index)
}
pub fn el_sub_menu(index: impl Into<SharedString>, title: impl Into<SharedString>) -> SubMenu {
    SubMenu {
        index: index.into(),
        title: title.into(),
        disabled: false,
        icon: None,
        title_slot: None,
        nodes: Vec::new(),
        offset: None,
        show_timeout: None,
        hide_timeout: None,
        expand_icons: None,
        collapse_icons: None,
        popup_style: None,
    }
}
pub fn el_menu_item_group(title: impl Into<SharedString>) -> MenuItemGroup {
    MenuItemGroup {
        title: title.into(),
        title_slot: None,
        items: Vec::new(),
    }
}

/// A typed tree preserves logical index paths even when entries move into a popup.
#[derive(Clone)]
pub enum MenuEntry {
    Item(MenuItem),
    SubMenu(Box<SubMenu>),
    Group(MenuItemGroup),
}
impl From<MenuItem> for MenuEntry {
    fn from(value: MenuItem) -> Self {
        Self::Item(value)
    }
}
impl From<SubMenu> for MenuEntry {
    fn from(value: SubMenu) -> Self {
        Self::SubMenu(Box::new(value))
    }
}
impl From<MenuItemGroup> for MenuEntry {
    fn from(value: MenuItemGroup) -> Self {
        Self::Group(value)
    }
}

#[derive(Clone)]
pub struct SubMenu {
    index: SharedString,
    title: SharedString,
    disabled: bool,
    icon: Option<Icon>,
    title_slot: Option<Content>,
    nodes: Vec<MenuEntry>,
    offset: Option<f32>,
    show_timeout: Option<Duration>,
    hide_timeout: Option<Duration>,
    expand_icons: Option<(Icon, Icon)>,
    collapse_icons: Option<(Icon, Icon)>,
    popup_style: Option<PopupStyle>,
}
impl SubMenu {
    pub fn entry(mut self, entry: impl Into<MenuEntry>) -> Self {
        self.nodes.push(entry.into());
        self
    }
    pub fn entries(mut self, entries: impl IntoIterator<Item = impl Into<MenuEntry>>) -> Self {
        self.nodes.extend(entries.into_iter().map(Into::into));
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn icon(mut self, value: Icon) -> Self {
        self.icon = Some(value);
        self
    }
    pub fn title_slot<E: IntoElement>(mut self, draw: impl Fn() -> E + 'static) -> Self {
        self.title_slot = Some(Rc::new(move || draw().into_any_element()));
        self
    }
    /// Overrides the popup offset in logical pixels (nonnegative).
    pub fn popper_offset(mut self, value: f32) -> Self {
        self.offset = (value.is_finite() && value >= 0.).then_some(value);
        self
    }
    pub fn show_timeout(mut self, value: Duration) -> Self {
        self.show_timeout = Some(value);
        self
    }
    pub fn hide_timeout(mut self, value: Duration) -> Self {
        self.hide_timeout = Some(value);
        self
    }
    pub fn expand_icons(mut self, closed: Icon, open: Icon) -> Self {
        self.expand_icons = Some((closed, open));
        self
    }
    pub fn collapse_icons(mut self, closed: Icon, open: Icon) -> Self {
        self.collapse_icons = Some((closed, open));
        self
    }
    pub fn popup_style(mut self, style: impl Fn(Div) -> Div + 'static) -> Self {
        self.popup_style = Some(Rc::new(style));
        self
    }
}

#[derive(Clone)]
pub struct MenuItemGroup {
    title: SharedString,
    title_slot: Option<Content>,
    items: Vec<MenuItem>,
}
impl MenuItemGroup {
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items.extend(items);
        self
    }
    pub fn title_slot<E: IntoElement>(mut self, draw: impl Fn() -> E + 'static) -> Self {
        self.title_slot = Some(Rc::new(move || draw().into_any_element()));
        self
    }
}

/// One chevron rotation, from a sampled angle to the target.
#[derive(Clone, Copy)]
struct Arrow {
    started: f64,
    from: f32,
    to: f32,
}

impl Arrow {
    /// Normal speed for a chevron rotation; [`MotionSpeed`] scales it.
    const SECS: f64 = 0.15;

    fn sample(&self, now: f64, secs: f64) -> f32 {
        let delta = ((now - self.started) / secs).clamp(0., 1.) as f32;
        self.from + (self.to - self.from) * motion::ease_in_out(delta)
    }

    fn running(&self, now: f64, secs: f64) -> bool {
        now - self.started < secs
    }
}

/// Persistent menu data. `Menu::state` exposes programmatic control; otherwise
/// the component keeps this state under its own stable element ID.
#[derive(Default)]
pub struct MenuState {
    active: Option<SharedString>,
    opened: HashSet<SharedString>,
    initialized: bool,
    default_active: Option<SharedString>,
    popup: bool,
    paths: HashMap<SharedString, Vec<SharedString>>,
    branches: HashMap<SharedString, Vec<SharedString>>,
    focus_request: Option<(SharedString, FocusHandle)>,
    frame: std::rc::Weak<Frame>,
    focus: HashMap<SharedString, FocusHandle>,
    current: Option<SharedString>,
    bounds: HashMap<SharedString, Bounds<Pixels>>,
    popup_bounds: HashMap<SharedString, Bounds<Pixels>>,
    width: Option<Pixels>,
    pending: HashMap<SharedString, Task<()>>,
    unique_opened: bool,
    /// In-flight popup fade-ins and fade-outs.
    fading: HashMap<SharedString, Fade>,
    /// In-flight chevron rotations, sampled from 0 (closed) to 1 (open).
    arrows: HashMap<SharedString, Arrow>,
}
impl MenuState {
    /// Records the chevron rotation and, for popups, the fade of a submenu
    /// branch.
    fn animate(&mut self, key: &SharedString, opening: bool) {
        let now = motion::elapsed();
        let from = self
            .arrows
            .get(key)
            .map_or(if opening { 0. } else { 1. }, |arrow| {
                arrow.sample(now, Arrow::SECS)
            });
        let to = if opening { 1. } else { 0. };
        if from != to {
            self.arrows.insert(
                key.clone(),
                Arrow {
                    started: now,
                    from,
                    to,
                },
            );
        }
        if self.popup {
            self.fading.insert(
                key.clone(),
                Fade {
                    started: motion::elapsed(),
                    opening,
                },
            );
        }
    }
    pub fn active_index(&self) -> Option<&str> {
        self.active.as_deref()
    }
    pub fn is_open(&self, index: &str) -> bool {
        self.opened.contains(index)
    }
    /// Opens a registered submenu and its ancestors, without emitting user-intent events.
    pub fn open(&mut self, index: &str, window: &mut Window) {
        if let Some(path) = self.branches.get(index).cloned() {
            if self.unique_opened || self.popup {
                self.opened.retain(|key| path.contains(key));
            }
            self.opened.extend(path);
            self.animate(&index.into(), true);
            self.focus_request = None;
            window.refresh();
        }
    }
    pub fn close(&mut self, index: &str, window: &mut Window) {
        let index: SharedString = index.into();
        self.opened.remove(&index);
        self.animate(&index, false);
        if self.popup {
            self.opened.retain(|key| {
                !self
                    .branches
                    .get(key)
                    .is_some_and(|path| path.iter().any(|part| part == &index))
            });
        }
        self.pending.clear();
        self.focus_request = None;
        window.refresh();
    }
    pub fn update_active_index(&mut self, index: impl Into<SharedString>, window: &mut Window) {
        self.active = Some(index.into());
        self.reveal_active();
        window.refresh();
    }
    fn reveal_active(&mut self) {
        if !self.popup
            && let Some(path) = self.active.as_ref().and_then(|key| self.paths.get(key))
        {
            self.opened.extend(
                path.iter()
                    .filter(|key| self.branches.contains_key(*key))
                    .cloned(),
            );
        }
    }
}

#[derive(Clone)]
struct Options {
    mode: MenuMode,
    trigger: MenuTrigger,
    collapse: bool,
    ellipsis: bool,
    ellipsis_icon: Option<Icon>,
    unique: bool,
    outside: bool,
    border: bool,
    offset: f32,
    show: Duration,
    hide: Duration,
    popup_theme: MenuPopupTheme,
    popup_style: Option<PopupStyle>,
    bg: Option<Hsla>,
    text: Option<Hsla>,
    active: Option<Hsla>,
    onselect: Option<Listener>,
    onopen: Option<Listener>,
    onclose: Option<Listener>,
    metrics: Metrics,
    speed: Option<MotionSpeed>,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            mode: MenuMode::Vertical,
            trigger: MenuTrigger::Hover,
            collapse: false,
            ellipsis: true,
            ellipsis_icon: None,
            unique: false,
            outside: false,
            border: true,
            offset: 6.,
            show: Duration::from_millis(300),
            hide: Duration::from_millis(300),
            popup_theme: MenuPopupTheme::Dark,
            popup_style: None,
            bg: None,
            text: None,
            active: None,
            onselect: None,
            onopen: None,
            onclose: None,
            metrics: MenuStyle::default().metrics(),
            // Desktop menus open and close quickly; `normal()` restores the
            // Element Plus 1x timings.
            speed: Some(MotionSpeed::Fast),
        }
    }
}
impl Options {
    fn horizontal(&self) -> bool {
        self.mode == MenuMode::Horizontal
    }
    fn collapsed(&self) -> bool {
        !self.horizontal() && self.collapse
    }
    fn popup(&self) -> bool {
        self.horizontal() || self.collapsed()
    }
    fn hover(&self) -> bool {
        self.collapsed() || self.horizontal() && self.trigger == MenuTrigger::Hover
    }
}

/// Vertical or horizontal navigation following Element Plus Menu. Supply a
/// sibling-unique `.id()` for multiple menus and stable unique indices for entries.
#[derive(Default, IntoElement)]
pub struct Menu {
    id: Option<ElementId>,
    nodes: Vec<MenuEntry>,
    state: Option<Entity<MenuState>>,
    default_active: Option<SharedString>,
    active: Option<SharedString>,
    default_openeds: Vec<SharedString>,
    options: Options,
}
impl Menu {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }
    pub fn state(mut self, state: &Entity<MenuState>) -> Self {
        self.state = Some(state.clone());
        self
    }
    pub fn entry(mut self, node: impl Into<MenuEntry>) -> Self {
        self.nodes.push(node.into());
        self
    }
    pub fn entries(mut self, nodes: impl IntoIterator<Item = impl Into<MenuEntry>>) -> Self {
        self.nodes.extend(nodes.into_iter().map(Into::into));
        self
    }
    pub fn mode(mut self, value: MenuMode) -> Self {
        self.options.mode = value;
        self
    }
    pub fn collapse(mut self, value: bool) -> Self {
        self.options.collapse = value;
        self
    }
    pub fn ellipsis(mut self, value: bool) -> Self {
        self.options.ellipsis = value;
        self
    }
    pub fn ellipsis_icon(mut self, value: Icon) -> Self {
        self.options.ellipsis_icon = Some(value);
        self
    }
    pub fn unique_opened(mut self, value: bool) -> Self {
        self.options.unique = value;
        self
    }
    /// Explicitly sets the [`MotionSpeed`] for the collapse, popup fade, and
    /// chevron rotation. Menus use [`MotionSpeed::Fast`] by default, the
    /// desktop-regular timing; [`Self::normal`] restores the Element Plus 1x
    /// durations.
    pub fn speed(mut self, speed: MotionSpeed) -> Self {
        self.options.speed = Some(speed);
        self
    }
    /// Runs this animation at jQuery's `fast` speed (0.6×), already the
    /// default.
    pub fn fast(mut self) -> Self {
        self.options.speed = Some(MotionSpeed::Fast);
        self
    }

    /// Runs this animation at jQuery's `slow` speed (1.6×); omit for the
    /// theme's speed.
    pub fn slow(mut self) -> Self {
        self.options.speed = Some(MotionSpeed::Slow);
        self
    }

    /// Uses the Element Plus 1x timings instead of the default desktop speed.
    pub fn normal(mut self) -> Self {
        self.options.speed = Some(MotionSpeed::Normal);
        self
    }

    /// Chooses a desktop surface preset; individual knobs still override it.
    pub fn style(mut self, style: MenuStyle) -> Self {
        self.options.metrics = style.metrics();
        self
    }
    /// Sets the vertical row height for top-level and inline nested rows.
    pub fn item_height(mut self, height: f32) -> Self {
        self.options.metrics.item_height = height;
        self.options.metrics.sub_item_height = height;
        self
    }
    /// Rounds the inset hover fill.
    pub fn item_radius(mut self, radius: f32) -> Self {
        self.options.metrics.item_radius = radius;
        self
    }
    /// Sets the horizontal margin between hover fills and the menu edge.
    pub fn item_inset(mut self, inset: f32) -> Self {
        self.options.metrics.item_inset = inset;
        self
    }
    /// Overrides the hover fill color.
    pub fn hover_color(mut self, color: impl Into<Hsla>) -> Self {
        self.options.metrics.hover = HoverFill::Custom(color.into());
        self
    }
    /// Sets the text color while a row is hovered, like macOS white on accent.
    pub fn hover_text_color(mut self, color: impl Into<Hsla>) -> Self {
        self.options.metrics.hover_text = Some(color.into());
        self
    }
    /// Sets the vertical padding inside floating popups.
    pub fn popup_padding(mut self, padding: f32) -> Self {
        self.options.metrics.popup_padding = padding;
        self
    }
    /// Sets the menu font size.
    pub fn font_size(mut self, size: f32) -> Self {
        self.options.metrics.font_size = size;
        self
    }
    pub fn menu_trigger(mut self, value: MenuTrigger) -> Self {
        self.options.trigger = value;
        self
    }
    /// Draws the menu's edge divider (enabled by default).
    pub fn border(mut self, value: bool) -> Self {
        self.options.border = value;
        self
    }
    pub fn close_on_click_outside(mut self, value: bool) -> Self {
        self.options.outside = value;
        self
    }
    /// Popup offset in logical pixels (nonnegative, default 6).
    pub fn popper_offset(mut self, value: f32) -> Self {
        self.options.offset = if value.is_finite() && value >= 0. {
            value
        } else {
            6.
        };
        self
    }
    pub fn show_timeout(mut self, value: Duration) -> Self {
        self.options.show = value;
        self
    }
    pub fn hide_timeout(mut self, value: Duration) -> Self {
        self.options.hide = value;
        self
    }
    pub fn popper_effect(mut self, value: MenuPopupTheme) -> Self {
        self.options.popup_theme = value;
        self
    }
    pub fn popup_style(mut self, value: impl Fn(Div) -> Div + 'static) -> Self {
        self.options.popup_style = Some(Rc::new(value));
        self
    }
    pub fn background_color(mut self, value: impl Into<Hsla>) -> Self {
        self.options.bg = Some(value.into());
        self
    }
    pub fn text_color(mut self, value: impl Into<Hsla>) -> Self {
        self.options.text = Some(value.into());
        self
    }
    pub fn active_text_color(mut self, value: impl Into<Hsla>) -> Self {
        self.options.active = Some(value.into());
        self
    }
    pub fn default_active(mut self, value: impl Into<SharedString>) -> Self {
        self.default_active = Some(value.into());
        self
    }
    /// Controlled selection. Handle `onselect` and pass the accepted index back.
    pub fn active_index(mut self, value: impl Into<SharedString>) -> Self {
        self.active = Some(value.into());
        self
    }
    pub fn default_openeds(
        mut self,
        values: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.default_openeds = values.into_iter().map(Into::into).collect();
        self
    }
    pub fn onselect(
        mut self,
        callback: impl Fn(&MenuEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.options.onselect = Some(Rc::new(callback));
        self
    }
    pub fn onopen(
        mut self,
        callback: impl Fn(&MenuEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.options.onopen = Some(Rc::new(callback));
        self
    }
    pub fn onclose(
        mut self,
        callback: impl Fn(&MenuEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.options.onclose = Some(Rc::new(callback));
        self
    }
}

#[derive(Clone, IntoElement)]
pub struct MenuItem {
    id: ElementId,
    index: SharedString,
    label: SharedString,
    selected: bool,
    disabled: bool,
    icon: Option<Icon>,
    slot: Option<Content>,
    click: Option<ClickHandler>,
    on_click: Option<Listener>,
}
impl MenuItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        let id = id.into();
        Self {
            index: id.to_string().into(),
            id,
            label: label.into(),
            selected: false,
            disabled: false,
            icon: None,
            slot: None,
            click: None,
            on_click: None,
        }
    }
    pub fn index(mut self, index: impl Into<SharedString>) -> Self {
        self.index = index.into();
        self
    }
    pub fn selected(mut self, value: bool) -> Self {
        self.selected = value;
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn icon(mut self, value: Icon) -> Self {
        self.icon = Some(value);
        self
    }
    pub fn title_slot<E: IntoElement>(mut self, draw: impl Fn() -> E + 'static) -> Self {
        self.slot = Some(Rc::new(move || draw().into_any_element()));
        self
    }
    pub fn onclick(
        mut self,
        callback: impl Fn(&MenuEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(callback));
        self
    }
    /// Compatibility callback that receives the GPUI activation event.
    pub fn onselect(
        mut self,
        callback: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.click = Some(Rc::new(callback));
        self
    }
}

impl RenderOnce for MenuItem {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let metrics = MenuStyle::default().metrics();
        let mut el = div()
            .id(self.id)
            .role(gpui::Role::MenuItem)
            .aria_label(self.label.clone())
            .aria_selected(self.selected)
            .h(px(metrics.item_height))
            .flex()
            .items_center()
            .mx(px(metrics.item_inset))
            .rounded(px(metrics.item_radius))
            .px(px(metrics.padding_x))
            .gap(px(5.))
            .text_size(px(metrics.font_size))
            .text_color(rgb(if self.selected {
                PRIMARY
            } else {
                theme.primary_text()
            }));
        if let Some(icon) = self.icon {
            el = el.child(icon.size(18.));
        }
        el = el.child(title(&self.label, &self.slot));
        if self.disabled {
            return el.opacity(0.25).cursor_not_allowed();
        }
        let event = MenuEvent {
            index: self.index.clone(),
            index_path: vec![self.index],
        };
        let hover = theme.tint(PRIMARY, 9);
        el.focusable()
            .tab_stop(true)
            .cursor_pointer()
            .hover(move |s| s.bg(rgb(hover)))
            .focus_visible(|s| s.border_1().border_color(rgb(PRIMARY)))
            .on_click(move |click, window, cx| {
                if click.standard_click() {
                    if let Some(cb) = &self.on_click {
                        cb(&event, window, cx);
                    }
                    if let Some(cb) = &self.click {
                        cb(click, window, cx);
                    }
                }
            })
    }
}

fn title(label: &SharedString, slot: &Option<Content>) -> AnyElement {
    slot.as_ref()
        .map_or_else(|| label.clone().into_any_element(), |draw| draw())
}

#[derive(Clone)]
struct Node {
    index: SharedString,
    label: SharedString,
    icon: Option<Icon>,
    slot: Option<Content>,
    disabled: bool,
    item: Option<MenuItem>,
    branch: Option<SubMenu>,
    children: Vec<Node>,
    group: bool,
}
fn nodes(
    entries: Vec<MenuEntry>,
    path: &[SharedString],
    paths: &mut HashMap<SharedString, Vec<SharedString>>,
) -> Vec<Node> {
    entries
        .into_iter()
        .filter_map(|entry| {
            let (key, mut node) = match entry {
                MenuEntry::Group(group) => {
                    return Some(Node {
                        index: SharedString::default(),
                        label: group.title,
                        slot: group.title_slot,
                        icon: None,
                        disabled: true,
                        item: None,
                        branch: None,
                        group: true,
                        children: nodes(
                            group.items.into_iter().map(MenuEntry::Item).collect(),
                            path,
                            paths,
                        ),
                    });
                }
                MenuEntry::Item(item) => (
                    item.index.clone(),
                    Node {
                        index: item.index.clone(),
                        label: item.label.clone(),
                        icon: item.icon.clone(),
                        slot: item.slot.clone(),
                        disabled: item.disabled,
                        item: Some(item),
                        branch: None,
                        children: vec![],
                        group: false,
                    },
                ),
                MenuEntry::SubMenu(branch) => (
                    branch.index.clone(),
                    Node {
                        index: branch.index.clone(),
                        label: branch.title.clone(),
                        icon: branch.icon.clone(),
                        slot: branch.title_slot.clone(),
                        disabled: branch.disabled,
                        item: None,
                        branch: Some(*branch),
                        children: vec![],
                        group: false,
                    },
                ),
            };
            if key.is_empty() || key.as_ref() == MORE || paths.contains_key(&key) {
                return None;
            }
            let mut full = path.to_vec();
            full.push(key.clone());
            paths.insert(key, full.clone());
            if let Some(branch) = &mut node.branch {
                node.children = nodes(std::mem::take(&mut branch.nodes), &full, paths);
            }
            Some(node)
        })
        .collect()
}

fn overflow_at(widths: &[Pixels], available: Pixels, more: Pixels) -> usize {
    if widths.iter().copied().sum::<Pixels>() <= available {
        return widths.len();
    }
    let mut sum = more;
    widths
        .iter()
        .take_while(|&&w| {
            sum += w;
            sum <= available
        })
        .count()
}

impl RenderOnce for Menu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.unwrap_or_else(|| "el-menu".into());
        let options = Rc::new(self.options);
        let theme = Theme::current(cx);
        let font = theme.resolved_font(cx);
        let state = self
            .state
            .unwrap_or_else(|| window.use_keyed_state(id.clone(), cx, |_, _| MenuState::default()));
        let mut paths = HashMap::new();
        let mut roots = nodes(self.nodes, &[], &mut paths);
        let available = state.read(cx).width.unwrap_or(window.viewport_size().width);
        if options.horizontal() && options.ellipsis && !roots.is_empty() {
            let more_label = Locale::current(cx).menu_more();
            let mut more = Node {
                index: MORE.into(),
                label: more_label.into(),
                icon: Some(
                    options
                        .ellipsis_icon
                        .clone()
                        .unwrap_or_else(|| el_icon("More")),
                ),
                slot: None,
                disabled: false,
                item: None,
                branch: Some(el_sub_menu(MORE, more_label)),
                children: vec![],
                group: false,
            };
            let natural = |node: &Node, window: &mut Window, cx: &mut App| {
                let mut row = div()
                    .flex()
                    .items_center()
                    .gap(px(5.))
                    .px(px(options.metrics.padding_x))
                    .text_size(px(options.metrics.font_size))
                    .whitespace_nowrap()
                    .when_some(font.clone(), |el, font| el.font_family(font));
                if node.icon.is_some() {
                    row = row.child(div().w(px(24.)));
                }
                if node.index.as_ref() != MORE {
                    row = row.child(title(&node.label, &node.slot));
                }
                if node.branch.is_some() && node.index.as_ref() != MORE {
                    row = row.child(div().w(px(12.)));
                }
                row.into_any_element()
                    .layout_as_root(gpui::AvailableSpace::min_size(), window, cx)
                    .width
            };
            let widths: Vec<_> = roots.iter().map(|n| natural(n, window, cx)).collect();
            let split = overflow_at(&widths, available, natural(&more, window, cx));
            if split < roots.len() {
                more.children = roots.split_off(split);
                paths.insert(MORE.into(), vec![MORE.into()]);
                roots.push(more);
            }
        }
        let popup = options.popup();
        let overflow = find_node(&roots, MORE).map(|more| &more.children);
        let visual_paths: HashMap<_, _> = paths
            .iter()
            .map(|(key, path)| {
                let mut path = path.clone();
                if overflow.is_some_and(|nodes| find_node(nodes, key).is_some()) {
                    path.insert(0, MORE.into());
                }
                (key.clone(), path)
            })
            .collect();
        let branches: HashMap<_, _> = visual_paths
            .into_iter()
            .filter(|(_, path)| {
                path.iter().all(|part| {
                    find_node(&roots, part)
                        .is_some_and(|node| !node.disabled && node.branch.is_some())
                })
            })
            .collect();
        let controlled = self.active.is_some();
        let focused = state.read(cx).focus.iter().find_map(|(key, handle)| {
            handle
                .is_focused(window)
                .then(|| (key.clone(), handle.clone()))
        });
        state.update(cx, |s, cx| {
            let changed_mode = s.initialized && s.popup != popup;
            if changed_mode {
                s.opened.clear();
                s.pending.clear();
            }
            s.popup = popup;
            s.unique_opened = options.unique;
            if !s.initialized && !options.collapsed() {
                s.opened.extend(self.default_openeds);
            }
            let active_changed = !s.initialized
                || s.default_active != self.default_active
                || controlled && s.active != self.active;
            if active_changed {
                s.default_active = self.default_active;
                s.active = self.active.or_else(|| s.default_active.clone());
            }
            s.paths = paths;
            s.opened.retain(|key| {
                branches.contains_key(key)
                    && (!s.initialized || s.branches.get(key) == branches.get(key))
            });
            s.pending.retain(|key, _| {
                options.hover()
                    && branches.contains_key(key)
                    && s.branches.get(key) == branches.get(key)
            });
            s.bounds.retain(|key, _| s.paths.contains_key(key));
            s.arrows.retain(|key, _| s.paths.contains_key(key));
            s.branches = branches;
            if popup {
                let opened = s.opened.clone();
                s.opened
                    .retain(|key| s.branches[key].iter().all(|part| opened.contains(part)));
            }
            s.popup_bounds.clear();
            s.focus.retain(|key, _| s.paths.contains_key(key));
            if active_changed || changed_mode {
                s.reveal_active();
            }
            s.initialized = true;
            for key in s.paths.keys() {
                s.focus
                    .entry(key.clone())
                    .or_insert_with(|| cx.focus_handle());
            }
        });
        let visible = visible_rows(&roots, &state.read(cx).opened, None);
        state.update(cx, |s, _| {
            let enabled =
                |key: &SharedString| visible.iter().any(|row| !row.disabled && &row.index == key);
            if s.focus_request
                .as_ref()
                .is_some_and(|(key, _)| !enabled(key))
            {
                s.focus_request = None;
            }
            if let Some((key, origin)) = focused
                && !enabled(&key)
            {
                let ancestor = s
                    .paths
                    .get(&key)
                    .and_then(|path| path.iter().rev().find(|key| enabled(key)));
                let more = find_node(&roots, MORE)
                    .filter(|more| find_node(&more.children, &key).is_some());
                let target = ancestor
                    .or_else(|| more.map(|more| &more.index))
                    .or_else(|| {
                        visible
                            .iter()
                            .find(|row| !row.disabled)
                            .map(|row| &row.index)
                    });
                s.focus_request = target.cloned().map(|target| (target, origin));
            }
        });
        let first = visible
            .iter()
            .find(|row| !row.disabled)
            .map(|row| row.index.clone());
        let roving = state
            .read(cx)
            .current
            .as_ref()
            .filter(|key| {
                visible
                    .iter()
                    .any(|row| !row.disabled && &row.index == *key)
            })
            .cloned()
            .or_else(|| {
                state
                    .read(cx)
                    .active
                    .as_ref()
                    .filter(|key| {
                        visible
                            .iter()
                            .any(|row| !row.disabled && &row.index == *key)
                    })
                    .cloned()
            })
            .or(first);
        let shared = Rc::new(Frame {
            state,
            options,
            theme,
            font,
            roots,
            roving,
            controlled,
            max_popup_height: (window.viewport_size().height - px(16.)).max(px(36.)),
        });
        shared
            .state
            .update(cx, |state, _| state.frame = Rc::downgrade(&shared));
        if shared.state.read(cx).focus_request.is_some() {
            let state = shared.state.downgrade();
            window.defer(cx, move |window, cx| {
                let Some(state) = state.upgrade() else {
                    return;
                };
                let Some((key, origin)) = state.read(cx).focus_request.clone() else {
                    return;
                };
                let handle = state.read(cx).focus.get(&key).cloned();
                if !origin.is_focused(window) {
                    state.update(cx, |s, _| s.focus_request = None);
                } else if let Some(handle) = handle.filter(|handle| handle.is_rendered(window)) {
                    state.update(cx, |s, _| {
                        s.focus_request = None;
                        s.current = Some(key);
                    });
                    handle.focus(window, cx);
                    window.refresh();
                }
            });
        }
        let background = shared
            .options
            .bg
            .unwrap_or_else(|| rgb(shared.theme.background()).into());
        let mut root = div()
            .flex()
            .min_w_0()
            .bg(background)
            .border_color(rgb(shared.theme.border(false)))
            .when(shared.options.horizontal(), |el| {
                el.w_full()
                    .h(px(shared.options.metrics.horizontal_height))
                    .when(shared.options.border, |el| el.border_b_1())
                    .items_center()
            })
            .when(!shared.options.horizontal(), |el| {
                el.flex_col()
                    .when(shared.options.border, |el| el.border_r_1())
            })
            .when(shared.options.collapsed(), |el| el.w(px(COLLAPSED_WIDTH)))
            .when(!shared.options.collapsed(), |el| el.w_full());
        if shared.options.horizontal() {
            let state = shared.state.clone();
            root = root.relative().child(
                gpui::canvas(
                    move |bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App| {
                        if state.read(cx).width != Some(bounds.size.width) {
                            state.update(cx, |s, _| s.width = Some(bounds.size.width));
                            // Repainting alone is not enough: the overflow split
                            // runs in render, so the owning view has to rebuild
                            // its element tree before the new width is visible.
                            let owner = window.current_view();
                            cx.notify(owner);
                            window.refresh();
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            );
        }
        let mut root = root.id(id).role(gpui::Role::Menu);
        if shared.options.outside && shared.options.popup() {
            root = root.on_mouse_down_out({
                let shared = shared.clone();
                move |ev, window, cx| {
                    let inside = shared
                        .state
                        .read(cx)
                        .popup_bounds
                        .iter()
                        .any(|(key, bounds)| {
                            shared.state.read(cx).opened.contains(key)
                                && bounds.contains(&ev.position)
                        });
                    // Popup surface bounds are tracked separately from trigger bounds.
                    if !inside {
                        shared.close_all(window, cx);
                    }
                }
            });
        }
        root.children(
            shared
                .roots
                .iter()
                .map(|node| shared.draw(node, 0, false, cx)),
        )
    }
}

struct Frame {
    state: Entity<MenuState>,
    options: Rc<Options>,
    theme: Theme,
    font: Option<SharedString>,
    roots: Vec<Node>,
    roving: Option<SharedString>,
    controlled: bool,
    max_popup_height: Pixels,
}

struct Visible {
    index: SharedString,
    parent: Option<SharedString>,
    branch: bool,
    disabled: bool,
}
fn visible_rows(
    roots: &[Node],
    opened: &HashSet<SharedString>,
    parent: Option<SharedString>,
) -> Vec<Visible> {
    let mut out = Vec::new();
    for node in roots {
        if node.group {
            out.extend(visible_rows(&node.children, opened, parent.clone()));
            continue;
        }
        out.push(Visible {
            index: node.index.clone(),
            parent: parent.clone(),
            branch: node.branch.is_some(),
            disabled: node.disabled,
        });
        if !node.disabled && opened.contains(&node.index) {
            out.extend(visible_rows(
                &node.children,
                opened,
                Some(node.index.clone()),
            ));
        }
    }
    out
}
fn find_node<'a>(roots: &'a [Node], key: &str) -> Option<&'a Node> {
    roots.iter().find_map(|node| {
        if !node.group && node.index.as_ref() == key {
            Some(node)
        } else {
            find_node(&node.children, key)
        }
    })
}
impl Frame {
    fn event(&self, key: &SharedString, cx: &App) -> Option<MenuEvent> {
        (key.as_ref() != MORE)
            .then(|| self.state.read(cx).paths.get(key).cloned())
            .flatten()
            .map(|index_path| MenuEvent {
                index: key.clone(),
                index_path,
            })
    }
    fn set_open(&self, key: &SharedString, open: bool, window: &mut Window, cx: &mut App) {
        let Some(path) = self.state.read(cx).branches.get(key).cloned() else {
            return;
        };
        let changed = self.state.update(cx, |s, _| {
            if open {
                if s.opened.contains(key)
                    || !path[..path.len() - 1]
                        .iter()
                        .all(|part| s.opened.contains(part))
                {
                    return false;
                }
                s.focus_request = None;
                if self.options.unique || s.popup {
                    // Animate the branches that exclusivity closes.
                    let closed: Vec<_> = s
                        .opened
                        .iter()
                        .filter(|other| !path.contains(other))
                        .cloned()
                        .collect();
                    for other in &closed {
                        s.animate(other, false);
                    }
                    s.opened.retain(|other| path.contains(other));
                    s.pending.retain(|other, _| path.contains(other));
                }
                s.animate(key, true);
                s.opened.insert(key.clone())
            } else if s.opened.remove(key) {
                s.focus_request = None;
                s.animate(key, false);
                if s.popup {
                    s.opened.retain(|other| !s.branches[other].contains(key));
                    s.pending
                        .retain(|other, _| !s.branches.get(other).is_some_and(|p| p.contains(key)));
                }
                true
            } else {
                false
            }
        });
        if changed {
            self.restore_hidden_focus(window, cx);
            window.refresh();
            let listener = if open {
                &self.options.onopen
            } else {
                &self.options.onclose
            };
            if let (Some(listener), Some(event)) = (listener, self.event(key, cx)) {
                listener(&event, window, cx);
            }
        }
    }
    fn close_all(&self, window: &mut Window, cx: &mut App) {
        let mut list: Vec<_> = self.state.read(cx).opened.iter().cloned().collect();
        list.sort_by_key(|key| {
            (
                std::cmp::Reverse(self.state.read(cx).paths.get(key).map_or(0, Vec::len)),
                key.to_string(),
            )
        });
        let events: Vec<_> = list.iter().filter_map(|key| self.event(key, cx)).collect();
        self.state.update(cx, |s, _| {
            for key in &list {
                s.animate(key, false);
            }
            s.pending.clear();
            s.opened.clear();
            s.focus_request = None;
        });
        if !list.is_empty() {
            self.restore_hidden_focus(window, cx);
            window.refresh();
        }
        if let Some(listener) = &self.options.onclose {
            for event in events {
                listener(&event, window, cx);
            }
        }
    }
    fn restore_hidden_focus(&self, window: &mut Window, cx: &mut App) {
        let s = self.state.read(cx);
        let rows = visible_rows(&self.roots, &s.opened, None);
        let focused = s.focus.iter().find(|(_, handle)| handle.is_focused(window));
        let Some((key, _)) = focused else {
            return;
        };
        let visible =
            |key: &SharedString| rows.iter().any(|row| !row.disabled && &row.index == key);
        if visible(key) {
            return;
        }
        let target = s
            .paths
            .get(key)
            .and_then(|path| path.iter().rev().find(|key| visible(key)))
            .or_else(|| {
                rows.iter()
                    .find(|row| !row.disabled && row.index.as_ref() == MORE)
                    .map(|row| &row.index)
            })
            .or_else(|| rows.iter().find(|row| !row.disabled).map(|row| &row.index))
            .cloned();
        if let Some(target) = target {
            self.focus(&target, window, cx);
        }
    }
    fn focus(&self, key: &SharedString, window: &mut Window, cx: &mut App) {
        let handle = self.state.read(cx).focus.get(key).cloned();
        self.state.update(cx, |s, _| s.current = Some(key.clone()));
        if let Some(handle) = handle {
            handle.focus(window, cx);
        }
        window.refresh();
    }
    fn click(&self, key: &SharedString, event: &ClickEvent, window: &mut Window, cx: &mut App) {
        if !event.standard_click() {
            return;
        }
        let Some(node) = find_node(&self.roots, key).filter(|node| !node.disabled) else {
            return;
        };
        self.focus(key, window, cx);
        self.state.update(cx, |s, _| {
            s.pending.remove(key);
        });
        if node.branch.is_some() {
            let open = !self.state.read(cx).is_open(key);
            self.set_open(key, open, window, cx);
            return;
        }
        let Some(payload) = self.event(key, cx) else {
            return;
        };
        self.state.update(cx, |s, _| {
            if !self.controlled {
                s.active = Some(key.clone());
            }
            if self.options.popup() {
                // Fade the open popups out instead of removing them instantly.
                let closing: Vec<_> = s.opened.iter().cloned().collect();
                for key in &closing {
                    s.animate(key, false);
                }
                s.opened.clear();
                s.pending.clear();
            }
        });
        if self.options.popup() {
            let rows = visible_rows(&self.roots, &HashSet::new(), None);
            if !rows.iter().any(|r| &r.index == key) {
                let fallback = payload
                    .index_path
                    .first()
                    .filter(|key| rows.iter().any(|r| &r.index == *key))
                    .cloned()
                    .unwrap_or_else(|| MORE.into());
                self.focus(&fallback, window, cx);
            }
        }
        if let Some(listener) = &self.options.onselect {
            listener(&payload, window, cx);
        }
        if let Some(item) = &node.item {
            if let Some(listener) = &item.on_click {
                listener(&payload, window, cx);
            }
            if let Some(listener) = &item.click {
                listener(event, window, cx);
            }
        }
    }
    fn hover_later(
        self: &Rc<Self>,
        key: &SharedString,
        open: bool,
        window: &mut Window,
        cx: &mut App,
    ) {
        let Some(branch) = find_node(&self.roots, key).and_then(|node| node.branch.as_ref()) else {
            return;
        };
        let delay = if open {
            branch.show_timeout.unwrap_or(self.options.show)
        } else {
            branch.hide_timeout.unwrap_or(self.options.hide)
        };
        self.state.update(cx, |s, _| {
            s.pending.remove(key);
        });
        let state = self.state.downgrade();
        let task_key = key.clone();
        let task = window.spawn(cx, async move |cx| {
            cx.background_executor().timer(delay).await;
            let _ = cx.update(|window, cx| {
                let Some(state) = state.upgrade() else {
                    return;
                };
                let frame = state.read(cx).frame.upgrade();
                let Some(frame) = frame else {
                    return;
                };
                let s = state.read(cx);
                let pointer = window.mouse_position();
                let over_trigger = s
                    .bounds
                    .get(&task_key)
                    .is_some_and(|b| b.contains(&pointer));
                let over_popup = s.popup_bounds.iter().any(|(key, bounds)| {
                    s.opened.contains(key)
                        && bounds.contains(&pointer)
                        && s.branches
                            .get(key)
                            .is_some_and(|path| path.contains(&task_key))
                });
                if if open {
                    over_trigger
                } else {
                    !over_trigger && !over_popup
                } {
                    frame.set_open(&task_key, open, window, cx);
                }
            });
        });
        self.state.update(cx, |s, _| {
            s.pending.insert(key.clone(), task);
        });
    }
    fn key(&self, key: &SharedString, pressed: &str, window: &mut Window, cx: &mut App) -> bool {
        let rows = visible_rows(&self.roots, &self.state.read(cx).opened, None);
        let Some(index) = rows
            .iter()
            .position(|row| !row.disabled && &row.index == key)
        else {
            return false;
        };
        let row = &rows[index];
        let at_horizontal = self.options.horizontal() && row.parent.is_none();
        let neighbors: Vec<_> = rows
            .iter()
            .filter(|r| {
                !r.disabled
                    && if self.options.popup() {
                        r.parent == row.parent
                    } else {
                        true
                    }
            })
            .collect();
        let current = neighbors.iter().position(|r| &r.index == key).unwrap_or(0);
        let next = match pressed {
            "home" => neighbors.first().map(|r| &r.index),
            "end" => neighbors.last().map(|r| &r.index),
            "left" if at_horizontal => neighbors
                .get((current + neighbors.len() - 1) % neighbors.len())
                .map(|r| &r.index),
            "right" if at_horizontal => neighbors
                .get((current + 1) % neighbors.len())
                .map(|r| &r.index),
            "up" if !at_horizontal => neighbors
                .get((current + neighbors.len() - 1) % neighbors.len())
                .map(|r| &r.index),
            "down" if !at_horizontal => neighbors
                .get((current + 1) % neighbors.len())
                .map(|r| &r.index),
            "down" | "up" | "right" if row.branch => {
                self.state.update(cx, |s, _| {
                    if let Some(path) = s.branches.get(key) {
                        s.pending.retain(|part, _| !path.contains(part));
                    }
                });
                self.set_open(key, true, window, cx);
                let refreshed = visible_rows(&self.roots, &self.state.read(cx).opened, None);
                let mut children = refreshed
                    .iter()
                    .filter(|r| !r.disabled && r.parent.as_ref() == Some(key));
                let target = if pressed == "up" {
                    children.next_back()
                } else {
                    children.next()
                };
                if let (Some(target), Some(origin)) = (target, window.focused(cx)) {
                    let target = target.index.clone();
                    self.state
                        .update(cx, |s, _| s.focus_request = Some((target, origin)));
                    window.refresh();
                }
                return true;
            }
            "escape" | "left" => {
                if self.state.read(cx).is_open(key) {
                    self.set_open(key, false, window, cx);
                } else if let Some(parent) = &row.parent {
                    self.set_open(parent, false, window, cx);
                    self.focus(parent, window, cx);
                } else {
                    return false;
                }
                return true;
            }
            _ => return false,
        };
        if let Some(next) = next {
            self.focus(next, window, cx);
        }
        true
    }
    fn draw(self: &Rc<Self>, node: &Node, depth: usize, floating: bool, cx: &App) -> AnyElement {
        if node.group {
            return div()
                .flex()
                .flex_col()
                .child(
                    div()
                        .pt(px(7.))
                        .mx(px(self.options.metrics.item_inset))
                        .px(px(self.options.metrics.padding_x * (depth + 1) as f32))
                        .text_size(px(12.))
                        .text_color(rgb(self.theme.secondary_text()))
                        .child(title(&node.label, &node.slot)),
                )
                .children(
                    node.children
                        .iter()
                        .map(|n| self.draw(n, depth, floating, cx)),
                )
                .into_any_element();
        }
        let key = &node.index;
        let state = self.state.read(cx);
        let speed = self.options.speed.unwrap_or(self.theme.motion_speed);
        let arrow_secs = speed.scale(Arrow::SECS);
        let open = state.is_open(key);
        let selected = node.item.as_ref().is_some_and(|item| item.selected)
            || state.active.as_ref().is_some_and(|active| {
                if key.as_ref() == MORE {
                    find_node(&node.children, active).is_some()
                } else {
                    state
                        .paths
                        .get(active)
                        .is_some_and(|path| path.contains(key))
                }
            });
        let top = depth == 0 && self.options.horizontal();
        let collapsed = depth == 0 && self.options.collapsed();
        let compact = floating && self.options.horizontal();
        let metrics = self.options.metrics;
        let row_height = if top {
            metrics.horizontal_height
        } else if compact {
            metrics.popup_item_height
        } else if depth > 0 && !floating {
            metrics.sub_item_height
        } else {
            metrics.item_height
        };
        let mut row = div()
            .id(key.clone())
            .role(gpui::Role::MenuItem)
            .aria_label(node.label.clone())
            .aria_selected(selected)
            .h(px(row_height))
            .flex()
            .items_center()
            .flex_shrink_0()
            .gap(px(5.))
            .text_size(px(metrics.font_size))
            .whitespace_nowrap()
            // Native desktop menus inset the hover fill; the web preset keeps
            // it full-bleed with square corners.
            .rounded(px(metrics.item_radius))
            .when(!collapsed, |el| el.mx(px(metrics.item_inset)))
            .px(px(if compact {
                metrics.popup_padding_x
            } else if floating || top {
                metrics.padding_x
            } else {
                metrics.padding_x * (depth + 1) as f32
            }))
            .when(collapsed, |el| {
                el.w(px(COLLAPSED_WIDTH)).px_0().justify_center()
            })
            .when_some(self.font.clone(), |el, font| el.font_family(font))
            .text_color(if selected {
                self.options.active.unwrap_or_else(|| rgb(PRIMARY).into())
            } else {
                self.options
                    .text
                    .unwrap_or_else(|| rgb(self.theme.primary_text()).into())
            })
            .when(top, |el| {
                el.border_b_2().border_color(if selected {
                    self.options.active.unwrap_or_else(|| rgb(PRIMARY).into())
                } else {
                    gpui::transparent_black()
                })
            });
        if let Some(icon) = &node.icon {
            row = row.child(
                div()
                    .w(px(24.))
                    .flex()
                    .justify_center()
                    .child(icon.clone().size(18.)),
            );
        }
        if !collapsed && key.as_ref() != MORE {
            row = row.child(title(&node.label, &node.slot));
        } else if collapsed && node.icon.is_none() {
            row = row.child(
                div()
                    .max_w_full()
                    .px(px(4.))
                    .truncate()
                    .child(node.label.clone()),
            );
        }
        if let Some(branch) = &node.branch
            && key.as_ref() != MORE
            && !collapsed
        {
            let custom = if floating {
                &branch.collapse_icons
            } else {
                &branch.expand_icons
            };
            // The built-in chevron rotates instead of swapping icons: 180°
            // for an inline expand, 90° for a floating submenu opened beside
            // its row. Custom icon pairs still swap.
            let angle = if custom.is_some() {
                0.
            } else {
                let target = if open { 1. } else { 0. };
                let sampled = if Theme::motion_reduced(cx) {
                    target
                } else {
                    state
                        .arrows
                        .get(key)
                        .map_or(target, |arrow| arrow.sample(motion::elapsed(), arrow_secs))
                };
                sampled
                    * if floating {
                        std::f32::consts::FRAC_PI_2
                    } else {
                        std::f32::consts::PI
                    }
            };
            let arrow = custom
                .as_ref()
                .map(|(closed, opened)| if open { opened.clone() } else { closed.clone() })
                .unwrap_or_else(|| {
                    el_icon(if floating { "ArrowRight" } else { "ArrowDown" }).rotate(angle)
                });
            if !top {
                row = row.child(div().flex_1());
            }
            row = row.child(arrow.size(12.));
        }
        if node.disabled {
            return row.opacity(0.25).cursor_not_allowed().into_any_element();
        }
        let hover: Hsla = match metrics.hover {
            HoverFill::PrimaryLight => rgb(self.theme.tint(PRIMARY, 9)).into(),
            HoverFill::FillLight => rgb(self.theme.fill(crate::theme::Fill::Light)).into(),
            HoverFill::Custom(color) => color,
        };
        let hover_text = metrics.hover_text;
        row = row
            .cursor_pointer()
            .hover(move |s| {
                let s = s.bg(hover);
                match hover_text {
                    Some(color) => s.text_color(color),
                    None => s,
                }
            })
            .focus_visible(|s| s.bg(rgb(0x409eff)).text_color(rgb(0xffffff)))
            .when_some(state.focus.get(key), |el, handle| {
                el.track_focus(handle)
                    .tab_stop(self.roving.as_ref() == Some(key))
            })
            .on_click({
                let this = self.clone();
                let key = key.clone();
                move |event, window, cx| this.click(&key, event, window, cx)
            })
            .on_key_down({
                let this = self.clone();
                let key = key.clone();
                move |event, window, cx| {
                    if this.key(&key, &event.keystroke.key, window, cx) {
                        cx.stop_propagation();
                    }
                }
            });
        if collapsed && node.branch.is_none() && node.icon.is_some() {
            let label = node.label.clone();
            let dark = self.options.popup_theme == MenuPopupTheme::Dark;
            row = row.tooltip(move |_, cx| {
                cx.new(|_| CollapsedLabel {
                    label: label.clone(),
                    dark,
                })
                .into()
            });
        }
        // Record the row bounds for popup placement and hover hit-testing.
        let owner = self.state.clone();
        let row_key = key.clone();
        row = row.relative().child(
            gpui::canvas(
                move |bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App| {
                    if owner.read(cx).bounds.get(&row_key) != Some(&bounds) {
                        owner.update(cx, |s, _| {
                            s.bounds.insert(row_key.clone(), bounds);
                        });
                        if owner.read(cx).is_open(&row_key) {
                            window.on_next_frame(|window, _| window.refresh());
                        }
                    }
                    if owner
                        .read(cx)
                        .arrows
                        .get(&row_key)
                        .is_some_and(|arrow| arrow.running(motion::elapsed(), arrow_secs))
                    {
                        window.request_animation_frame();
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0(),
        );
        if node.branch.is_none() {
            return row.into_any_element();
        }
        if self.options.hover() {
            row = row.on_hover({
                let this = self.clone();
                let key = key.clone();
                move |hover, window, cx| this.hover_later(&key, *hover, window, cx)
            });
        }
        if !self.options.popup() {
            let body = div().flex().flex_col().children(
                node.children
                    .iter()
                    .map(|n| self.draw(n, depth + 1, false, cx)),
            );
            // The transition keeps the body mounted while it collapses and
            // drops it once settled, like upstream's ElCollapseTransition.
            let body_id: SharedString = format!("\u{2}{key}").into();
            return div()
                .flex()
                .flex_col()
                .child(row)
                .child(
                    el_transition(body_id, open, TransitionEffect::Collapse)
                        .speed(speed)
                        .child(body),
                )
                .into_any_element();
        }
        let prior = state.bounds.get(key).copied();
        let fade = self.state.read(cx).fading.get(key).copied();
        let fade_secs = speed.scale(FADE_SECS);
        let progress = fade_progress(fade, Theme::motion_reduced(cx), fade_secs);
        // Inside the wrapper (a flex row) the row grows to the menu width, so
        // the flex spacer pushes a floating submenu's chevron to the right edge.
        let row = if !top && !collapsed {
            row.flex_grow(1.)
        } else {
            row
        };
        let wrapper = div().relative().flex().flex_shrink_0().child(row);
        wrapper
            .children(
                prior
                    .filter(|_| open || fade.is_some())
                    .map(|target| self.popup(node, depth, top, target, progress, cx)),
            )
            .into_any_element()
    }
    fn popup(
        self: &Rc<Self>,
        node: &Node,
        depth: usize,
        top: bool,
        target: Bounds<Pixels>,
        progress: f32,
        cx: &App,
    ) -> AnyElement {
        let key = node.index.clone();
        let popup_id: SharedString = format!("\u{1}{key}").into();
        let branch = node.branch.as_ref().expect("popup branch");
        let offset = px(branch.offset.unwrap_or(self.options.offset));
        let metrics = self.options.metrics;
        let fade_secs = self
            .options
            .speed
            .unwrap_or(self.theme.motion_speed)
            .scale(FADE_SECS);
        let base = div()
            .flex()
            .flex_col()
            .py(px(metrics.popup_padding))
            .min_w(px(metrics.popup_min_width))
            .max_h(self.max_popup_height)
            .rounded(px(metrics.popup_radius))
            .shadow(self.theme.overlay_shadow())
            .bg(rgb(self.theme.overlay_background()));
        let styled = if let Some(style) = branch
            .popup_style
            .as_ref()
            .or(self.options.popup_style.as_ref())
        {
            style(base)
        } else {
            base
        };
        let popup = styled
            .relative()
            .opacity(progress)
            .child(
                gpui::canvas(
                    {
                        let owner = self.state.clone();
                        let key = key.clone();
                        move |bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App| {
                            let reduced = Theme::motion_reduced(cx);
                            let mut finished = false;
                            owner.update(cx, |s, _| {
                                s.popup_bounds.insert(key.clone(), bounds);
                                let done = s.fading.get(&key).is_some_and(|fade| {
                                    reduced || (motion::elapsed() - fade.started) >= fade_secs
                                });
                                if done {
                                    s.fading.remove(&key);
                                    finished = true;
                                }
                            });
                            if finished {
                                window.refresh();
                            } else if owner.read(cx).fading.contains_key(&key) {
                                window.request_animation_frame();
                            }
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            )
            .id(popup_id)
            .overflow_y_scroll()
            .on_scroll(|_, _, cx| cx.stop_propagation())
            .occlude()
            .when(self.options.hover(), |el| {
                el.on_hover({
                    let this = self.clone();
                    let key = key.clone();
                    move |hover, window, cx| {
                        let chain = this
                            .state
                            .read(cx)
                            .branches
                            .get(&key)
                            .cloned()
                            .unwrap_or_default();
                        if *hover {
                            this.state.update(cx, |s, _| {
                                for key in chain {
                                    s.pending.remove(&key);
                                    // Cancel an in-flight close fade so hovering
                                    // the popup keeps it open.
                                    if s.fading.get(&key).is_some_and(|fade| !fade.opening)
                                        && !s.opened.contains(&key)
                                    {
                                        s.fading.remove(&key);
                                        s.opened.insert(key);
                                    }
                                }
                            });
                        } else {
                            for ancestor in chain.into_iter().rev() {
                                this.hover_later(&ancestor, false, window, cx);
                            }
                        }
                    }
                })
            })
            .children(
                node.children
                    .iter()
                    .map(|n| self.draw(n, depth + 1, true, cx)),
            );
        deferred(
            anchored()
                .attach(
                    target,
                    if top {
                        gpui::Anchor::BottomLeft
                    } else {
                        gpui::Anchor::TopRight
                    },
                )
                .offset(if top {
                    point(px(0.), offset)
                } else {
                    point(offset, px(0.))
                })
                .snap_to_window_with_margin(px(4.))
                .child(popup),
        )
        .with_priority(popup_priority(depth))
        .into_any_element()
    }
}

struct CollapsedLabel {
    label: SharedString,
    dark: bool,
}

impl gpui::Render for CollapsedLabel {
    fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
        div()
            .rounded(px(4.))
            .border_1()
            .border_color(rgb(if self.dark { 0x303133 } else { 0xe4e7ed }))
            .bg(rgb(if self.dark { 0x303133 } else { 0xffffff }))
            .text_color(rgb(if self.dark { 0xffffff } else { 0x303133 }))
            .text_size(px(12.))
            .px(px(10.))
            .py(px(5.))
            .child(self.label.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type EventLog = Rc<std::cell::RefCell<Vec<(&'static str, Vec<SharedString>)>>>;

    struct Harness {
        state: Entity<MenuState>,
        events: EventLog,
        width: f32,
        trigger: MenuTrigger,
    }

    impl gpui::Render for Harness {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            let listener = |kind: &'static str| {
                let events = self.events.clone();
                move |event: &MenuEvent, _: &mut Window, _: &mut App| {
                    events.borrow_mut().push((kind, event.index_path.clone()));
                }
            };
            div().id("menu-test").tab_group().size_full().child(
                div().w(px(self.width)).child(
                    el_menu()
                        .state(&self.state)
                        .mode(MenuMode::Horizontal)
                        .menu_trigger(self.trigger)
                        .close_on_click_outside(true)
                        .entry(el_menu_item("home", "Home"))
                        .entry(
                            el_sub_menu("settings", "Settings")
                                .entry(el_menu_item("disabled", "Disabled").disabled(true))
                                .entry(el_menu_item("theme", "Theme").onclick(listener("click"))),
                        )
                        .onselect(listener("select"))
                        .onopen(listener("open"))
                        .onclose(listener("close")),
                ),
            )
        }
    }

    #[gpui::test]
    fn popup_keyboard_and_pointer_preserve_logical_paths(cx: &mut gpui::TestAppContext) {
        let events = Rc::new(std::cell::RefCell::new(Vec::new()));
        let state = cx.update(|cx| cx.new(|_| MenuState::default()));
        let (_, view) = cx.add_window_view({
            let events = events.clone();
            let state = state.clone();
            move |_, _| Harness {
                state,
                events,
                width: 500.,
                trigger: MenuTrigger::Click,
            }
        });
        let draw = |view: &mut gpui::VisualTestContext| view.update(|w, cx| w.draw(cx).clear(cx));
        let key = |view: &mut gpui::VisualTestContext, key: &str| {
            let keystroke = view.update(|_, cx| {
                gpui::Keystroke::parse_with_os(key, cx.platform_info().os).unwrap()
            });
            view.simulate_event(gpui::KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            });
            view.simulate_event(gpui::KeyUpEvent { keystroke });
        };
        draw(view);
        view.update(|w, cx| state.read(cx).focus["settings"].clone().focus(w, cx));
        draw(view);
        key(view, "down");
        draw(view);
        view.run_until_parked();
        view.update(|w, cx| {
            let s = state.read(cx);
            assert!(s.is_open("settings"));
            assert!(s.focus["theme"].is_rendered(w));
            assert!(s.focus["theme"].is_focused(w));
        });
        key(view, "escape");
        draw(view);
        view.update(|w, cx| {
            assert!(!state.read(cx).is_open("settings"));
            assert!(state.read(cx).focus["settings"].is_focused(w));
        });
        let trigger = view.update(|_, cx| state.read(cx).bounds["settings"].center());
        view.simulate_click(trigger, gpui::Modifiers::default());
        draw(view);
        let theme_row = view.update(|_, cx| state.read(cx).bounds["theme"].center());
        view.simulate_click(theme_row, gpui::Modifiers::default());
        view.update(|_, cx| assert_eq!(state.read(cx).active_index(), Some("theme")));
        assert_eq!(
            *events.borrow(),
            vec![
                ("open", vec!["settings".into()]),
                ("close", vec!["settings".into()]),
                ("open", vec!["settings".into()]),
                ("select", vec!["settings".into(), "theme".into()]),
                ("click", vec!["settings".into(), "theme".into()]),
            ]
        );
    }

    #[gpui::test]
    fn overflow_popup_selects_original_path_and_outside_closes(cx: &mut gpui::TestAppContext) {
        let events = EventLog::default();
        let state = cx.update(|cx| cx.new(|_| MenuState::default()));
        let (_, view) = cx.add_window_view({
            let events = events.clone();
            let state = state.clone();
            move |_, _| Harness {
                state,
                events,
                width: 80.,
                trigger: MenuTrigger::Click,
            }
        });
        for _ in 0..2 {
            view.update(|w, cx| w.draw(cx).clear(cx));
        }
        let more = view.update(|_, cx| state.read(cx).bounds[MORE].center());
        view.simulate_click(more, gpui::Modifiers::default());
        view.update(|w, cx| w.draw(cx).clear(cx));
        let settings = view.update(|_, cx| state.read(cx).bounds["settings"].center());
        view.simulate_click(settings, gpui::Modifiers::default());
        view.update(|w, cx| w.draw(cx).clear(cx));
        let theme_row = view.update(|_, cx| state.read(cx).bounds["theme"].center());
        view.simulate_click(theme_row, gpui::Modifiers::default());
        view.update(|w, cx| {
            assert_eq!(state.read(cx).active_index(), Some("theme"));
            assert!(state.read(cx).focus[MORE].is_focused(w));
            w.draw(cx).clear(cx);
        });
        view.simulate_click(more, gpui::Modifiers::default());
        view.update(|w, cx| w.draw(cx).clear(cx));
        view.simulate_click(settings, gpui::Modifiers::default());
        view.update(|w, cx| w.draw(cx).clear(cx));
        view.simulate_click(point(px(650.), px(440.)), gpui::Modifiers::default());
        view.update(|_, cx| assert!(state.read(cx).opened.is_empty()));
        assert_eq!(
            *events.borrow(),
            vec![
                ("open", vec!["settings".into()]),
                ("select", vec!["settings".into(), "theme".into()]),
                ("click", vec!["settings".into(), "theme".into()]),
                ("open", vec!["settings".into()]),
                ("close", vec!["settings".into()]),
            ]
        );
    }

    #[test]
    fn speed_chainables_are_optional() {
        assert_eq!(el_menu().options.speed, Some(MotionSpeed::Fast));
        assert_eq!(el_menu().fast().options.speed, Some(MotionSpeed::Fast));
        assert_eq!(el_menu().slow().options.speed, Some(MotionSpeed::Slow));
        assert_eq!(
            el_menu().slow().normal().options.speed,
            Some(MotionSpeed::Normal)
        );
    }

    #[test]
    fn chevron_rotation_animates_between_closed_and_open() {
        let arrow = Arrow {
            started: 10.,
            from: 0.,
            to: 1.,
        };
        assert_eq!(arrow.sample(10., Arrow::SECS), 0.);
        assert_eq!(arrow.sample(10. + Arrow::SECS, Arrow::SECS), 1.);
        let mid = arrow.sample(10. + Arrow::SECS / 2., Arrow::SECS);
        assert!(mid > 0.1 && mid < 0.9, "midway angle {mid}");
        assert!(arrow.running(10. + Arrow::SECS / 2., Arrow::SECS));

        let mut state = MenuState::default();
        let key: SharedString = "one".into();
        state.animate(&key, true);
        let opened = state.arrows[&key];
        assert_eq!((opened.from, opened.to), (0., 1.));
        // Closing without a running animation starts from the open angle.
        state.animate(&key, false);
        assert_eq!(state.arrows[&key].to, 0.);
        assert!(state.arrows[&key].from > 0.);
    }

    #[gpui::test]
    fn nested_popups_open_a_third_level_and_report_the_full_path(cx: &mut gpui::TestAppContext) {
        let events: EventLog = Rc::new(std::cell::RefCell::new(Vec::new()));
        let state = cx.update(|cx| cx.new(|_| MenuState::default()));
        struct Nested {
            state: Entity<MenuState>,
            events: EventLog,
        }
        impl gpui::Render for Nested {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let events = self.events.clone();
                div().id("menu-test").tab_group().size_full().child(
                    el_menu()
                        .state(&self.state)
                        .mode(MenuMode::Horizontal)
                        .menu_trigger(MenuTrigger::Click)
                        .entry(el_menu_item("home", "Home"))
                        .entry(
                            el_sub_menu("settings", "Settings")
                                .entry(el_menu_item("language", "Language"))
                                .entry(
                                    el_sub_menu("theme", "Theme")
                                        .entry(el_menu_item("light", "Light"))
                                        .entry(el_menu_item("dark", "Dark")),
                                ),
                        )
                        .onselect(move |event: &MenuEvent, _, _| {
                            events
                                .borrow_mut()
                                .push(("select", event.index_path.clone()));
                        }),
                )
            }
        }
        let (_, view) = cx.add_window_view({
            let state = state.clone();
            let events = events.clone();
            move |_, cx| {
                cx.set_reduce_motion(true);
                Nested { state, events }
            }
        });
        let draw = |view: &mut gpui::VisualTestContext| {
            view.update(|window, cx| window.draw(cx).clear(cx));
        };
        draw(view);
        let settings = view.update(|_, cx| state.read(cx).bounds["settings"].center());
        view.simulate_click(settings, gpui::Modifiers::default());
        draw(view);
        let theme = view.update(|_, cx| state.read(cx).bounds["theme"].center());
        view.simulate_click(theme, gpui::Modifiers::default());
        draw(view);
        view.update(|_, cx| {
            let s = state.read(cx);
            assert!(s.is_open("settings"), "the first popup stays open");
            assert!(s.is_open("theme"), "a grandchild popup opens beside it");
            assert!(s.popup_bounds.contains_key("settings"));
            assert!(s.popup_bounds.contains_key("theme"));
            // Branch rows fill the popup so the chevron sits at the right edge.
            assert_eq!(
                s.bounds["theme"].size.width,
                s.popup_bounds["settings"].size.width
            );
        });
        let dark = view.update(|_, cx| state.read(cx).bounds["dark"].center());
        view.simulate_click(dark, gpui::Modifiers::default());
        draw(view);
        view.update(|_, cx| assert_eq!(state.read(cx).active_index(), Some("dark")));
        assert_eq!(
            *events.borrow(),
            vec![(
                "select",
                vec!["settings".into(), "theme".into(), "dark".into()]
            )]
        );
    }

    #[gpui::test]
    fn hover_respects_delay_and_keeps_popup_open_when_entered(cx: &mut gpui::TestAppContext) {
        let events = EventLog::default();
        let state = cx.update(|cx| cx.new(|_| MenuState::default()));
        let (_, view) = cx.add_window_view({
            let events = events.clone();
            let state = state.clone();
            move |_, _| Harness {
                state,
                events,
                width: 500.,
                trigger: MenuTrigger::Hover,
            }
        });
        let draw = |view: &mut gpui::VisualTestContext| view.update(|w, cx| w.draw(cx).clear(cx));
        let wait = |view: &mut gpui::VisualTestContext, millis| {
            view.run_until_parked();
            view.executor().advance_clock(Duration::from_millis(millis));
            view.run_until_parked();
            draw(view);
        };
        draw(view);
        let trigger = view.update(|_, cx| state.read(cx).bounds["settings"].center());
        view.simulate_mouse_move(trigger, None, Default::default());
        wait(view, 299);
        assert!(events.borrow().is_empty());
        wait(view, 2);
        let popup = view.update(|_, cx| state.read(cx).popup_bounds["settings"].center());
        view.simulate_mouse_move(popup, None, Default::default());
        wait(view, 350);
        view.update(|_, cx| assert!(state.read(cx).is_open("settings")));
        view.simulate_mouse_move(point(px(650.), px(440.)), None, Default::default());
        wait(view, 350);
        view.update(|_, cx| assert!(!state.read(cx).is_open("settings")));
        assert_eq!(
            *events.borrow(),
            vec![
                ("open", vec!["settings".into()]),
                ("close", vec!["settings".into()])
            ]
        );
    }

    #[gpui::test]
    fn keyboard_focus_survives_overflow_changes_and_discards_closed_requests(
        cx: &mut gpui::TestAppContext,
    ) {
        let events = EventLog::default();
        let state = cx.update(|cx| cx.new(|_| MenuState::default()));
        let (root, view) = cx.add_window_view({
            let state = state.clone();
            let events = events.clone();
            move |_, _| Harness {
                state,
                events,
                width: 500.,
                trigger: MenuTrigger::Click,
            }
        });
        let draw = |view: &mut gpui::VisualTestContext| {
            for _ in 0..2 {
                view.update(|w, cx| w.draw(cx).clear(cx));
                view.run_until_parked();
            }
        };
        draw(view);
        view.update(|w, cx| {
            let frame = state.read(cx).frame.upgrade().unwrap();
            let settings = "settings".into();
            frame.focus(&settings, w, cx);
            assert!(frame.key(&settings, "down", w, cx));
            assert!(state.read(cx).focus_request.is_some());
            assert!(frame.key(&settings, "escape", w, cx));
            assert!(state.read(cx).focus_request.is_none());
            assert!(frame.key(&settings, "down", w, cx));
        });
        draw(view);
        view.update(|w, cx| assert!(state.read(cx).focus["theme"].is_focused(w)));
        let prior_events = events.borrow().len();
        for (width, target) in [(80., MORE), (500., "home")] {
            view.update(|_, cx| {
                root.update(cx, |h, cx| {
                    h.width = width;
                    cx.notify();
                })
            });
            draw(view);
            view.update(|w, cx| {
                let focus = &state.read(cx).focus[target];
                assert!(focus.is_rendered(w));
                assert!(
                    focus.is_focused(w),
                    "expected focus on {target} at width {width}"
                );
            });
        }
        assert_eq!(events.borrow().len(), prior_events);
    }

    #[test]
    fn overflow_reserves_space_only_when_needed() {
        let widths = [px(40.), px(60.), px(100.)];
        assert_eq!(overflow_at(&widths, px(200.), px(40.)), 3);
        assert_eq!(overflow_at(&widths, px(139.), px(40.)), 1);
        assert_eq!(overflow_at(&widths, px(60.), px(40.)), 0);
    }

    #[gpui::test]
    fn clipped_container_still_measures_natural_height(cx: &mut gpui::TestAppContext) {
        use std::{cell::Cell, rc::Rc};

        struct Probe(Rc<Cell<f32>>);
        impl gpui::Render for Probe {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let height = self.0.clone();
                div().h(px(0.)).overflow_hidden().child(
                    div()
                        .relative()
                        .child(div().h(px(46.)).child("content"))
                        .child(
                            gpui::canvas(
                                move |bounds, _, _| height.set(f32::from(bounds.size.height)),
                                |_, _, _, _| {},
                            )
                            .absolute()
                            .inset_0(),
                        ),
                )
            }
        }
        let height = Rc::new(Cell::new(0.));
        let (_, view) = cx.add_window_view({
            let height = height.clone();
            move |_, _| Probe(height)
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(height.get(), 46., "inner block keeps its natural height");
    }

    #[gpui::test]
    fn vertical_submenu_expand_keeps_rows_laid_out(cx: &mut gpui::TestAppContext) {
        struct Vertical {
            state: Entity<MenuState>,
        }
        impl gpui::Render for Vertical {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                div().id("menu-test").tab_group().size_full().child(
                    el_menu()
                        .state(&self.state)
                        .entry(el_menu_item("home", "Home"))
                        .entry(
                            el_sub_menu("one", "One")
                                .entry(el_menu_item("one-a", "Item A"))
                                .entry(el_menu_item("one-b", "Item B")),
                        ),
                )
            }
        }
        let state = cx.update(|cx| cx.new(|_| MenuState::default()));
        let (_, view) = cx.add_window_view({
            let state = state.clone();
            move |_, cx| {
                cx.set_reduce_motion(true);
                Vertical { state }
            }
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        let title = view.update(|_, cx| state.read(cx).bounds["one"].center());
        view.simulate_click(title, gpui::Modifiers::default());
        view.update(|window, cx| window.draw(cx).clear(cx));
        view.update(|window, cx| {
            assert!(state.read(cx).is_open("one"));
            assert!(
                state.read(cx).focus["one-a"].is_rendered(window),
                "submenu rows stay laid out while expanded"
            );
        });
        // Collapsing removes the submenu from the open set right away; the
        // transition unmounts the content once the collapse settles.
        view.simulate_click(title, gpui::Modifiers::default());
        view.update(|window, cx| {
            assert!(!state.read(cx).is_open("one"));
            window.draw(cx).clear(cx);
        });
        view.update(|window, cx| {
            assert!(
                !state.read(cx).focus["one-a"].is_rendered(window),
                "submenu rows unmount after the collapse settles"
            );
        });
    }
}
