use std::{borrow::Cow, collections::BTreeMap, sync::OnceLock};

use gpui::{
    App, FontWeight, Global, Hsla, IntoElement, RenderOnce, SharedString, Window, div, prelude::*,
    px, svg,
};

include!(concat!(env!("OUT_DIR"), "/element_plus_icons.rs"));

const TABLER_FAMILY: &str = "tabler-icons";

fn element_plus(name: &str) -> Option<&'static [u8]> {
    ELEMENT_PLUS
        .binary_search_by_key(&name, |(key, _)| *key)
        .ok()
        .map(|i| ELEMENT_PLUS[i].1)
}

fn tabler_glyphs() -> &'static BTreeMap<&'static str, char> {
    static GLYPHS: OnceLock<BTreeMap<&str, char>> = OnceLock::new();
    GLYPHS.get_or_init(|| {
        include_str!("icons/tabler/codepoints.tsv")
            .lines()
            .map(|line| {
                let (name, code) = line.split_once('\t').expect("embedded Tabler mapping");
                let glyph =
                    char::from_u32(u32::from_str_radix(code, 16).expect("Tabler codepoint"))
                        .expect("valid Tabler codepoint");
                (name, glyph)
            })
            .collect()
    })
}

/// Creates a named icon using the application-wide pack. Accepts Element Plus
/// component names (`ArrowDown`) and upstream kebab-case names (`arrow-down`).
pub fn el_icon(name: impl Into<SharedString>) -> Icon {
    Icon::named(name)
}

fn normalized_name(name: SharedString) -> SharedString {
    if !name.bytes().any(|c| c.is_ascii_uppercase()) {
        return name;
    }
    let mut key = String::with_capacity(name.len());
    let chars: Vec<_> = name.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_ascii_uppercase()
            && i > 0
            && (chars[i - 1].is_ascii_lowercase()
                || chars[i - 1].is_ascii_digit()
                || (chars[i - 1].is_ascii_uppercase()
                    && chars.get(i + 1).is_some_and(char::is_ascii_lowercase)))
        {
            key.push('-');
        }
        key.push(c.to_ascii_lowercase());
    }
    key.into()
}

/// Default source for [`el_icon`]. The preference is scoped to the GPUI application.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum IconPack {
    #[default]
    ElementPlus,
    Tabler,
}

impl Global for IconPack {}

struct TablerRegistered;
impl Global for TablerRegistered {}

impl IconPack {
    /// Sets the application-wide preference and refreshes existing windows.
    /// Tabler is registered once per application. On error the preference is unchanged.
    pub fn set(self, cx: &mut App) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self == Self::Tabler {
            register_tabler(cx)?;
        }
        cx.set_global(self);
        cx.refresh_windows();
        Ok(())
    }

    pub fn current(cx: &App) -> Self {
        cx.try_global::<Self>().copied().unwrap_or_default()
    }

    /// All upstream names for this pack, sorted alphabetically.
    pub fn names(self) -> impl Iterator<Item = &'static str> {
        let svg = (self == Self::ElementPlus).then(|| ELEMENT_PLUS.iter().map(|(name, _)| *name));
        let glyphs = (self == Self::Tabler).then(|| tabler_glyphs().keys().copied());
        svg.into_iter()
            .flatten()
            .chain(glyphs.into_iter().flatten())
    }
}

fn register_tabler(cx: &mut App) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if cx.try_global::<TablerRegistered>().is_none() {
        register_icon_font(cx, include_bytes!("icons/tabler/tabler-icons.ttf"))?;
        cx.set_global(TablerRegistered);
    }
    Ok(())
}

// Shared names follow Element Plus; keep equivalents explicit rather than swapping raw codepoints.
fn tabler_equivalent(name: &str) -> &str {
    match name {
        "close" => "x",
        "arrow-down" => "chevron-down",
        "arrow-up" => "chevron-up",
        "arrow-left" => "chevron-left",
        "arrow-right" => "chevron-right",
        "document-copy" => "copy",
        "setting" => "settings",
        "warning" => "alert-circle",
        _ => name,
    }
}

/// Registers an application-supplied icon font once during GPUI startup.
/// The family passed to [`Icon::from_font`] must match the font's internal name.
pub fn register_icon_font(
    cx: &App,
    bytes: &'static [u8],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    cx.text_system().add_fonts(vec![Cow::Borrowed(bytes)])?;
    Ok(())
}

#[derive(Clone)]
enum Source {
    Glyph(char, Option<SharedString>),
    Named(SharedString, Option<IconPack>),
    Svg(&'static [u8]),
}

/// A native SVG or text glyph that inherits its size and color by default.
/// A button containing only an icon should also have an accessibility label.
#[derive(Clone, IntoElement)]
#[must_use]
pub struct Icon {
    source: Source,
    size: Option<f32>,
    color: Option<Hsla>,
    spin: Option<crate::LoadingMotion>,
}

impl Icon {
    fn with_source(source: Source) -> Self {
        Self {
            source,
            size: None,
            color: None,
            spin: None,
        }
    }

    /// Creates a glyph from the inherited font and its platform fallbacks.
    pub fn new(glyph: char) -> Self {
        Self::with_source(Source::Glyph(glyph, None))
    }

    /// Creates a glyph using a registered or installed icon font.
    pub fn from_font(family: impl Into<SharedString>, glyph: char) -> Self {
        Self::with_source(Source::Glyph(glyph, Some(family.into())))
    }

    /// Resolves a kebab-case name using the app preference, then Element Plus.
    /// An unknown name reserves space without drawing a glyph.
    pub fn named(name: impl Into<SharedString>) -> Self {
        Self::with_source(Source::Named(normalized_name(name.into()), None))
    }

    /// Uses an upstream name in the specified pack, ignoring the app preference.
    pub fn from_pack(pack: IconPack, name: impl Into<SharedString>) -> Self {
        Self::with_source(Source::Named(normalized_name(name.into()), Some(pack)))
    }

    /// Uses embedded SVG bytes without installing an application asset source.
    pub fn from_svg(bytes: &'static [u8]) -> Self {
        Self::with_source(Source::Svg(bytes))
    }

    /// Overrides the glyph size in logical pixels; invalid values restore inheritance.
    pub fn size(mut self, logical_px: f32) -> Self {
        self.size = (logical_px.is_finite() && logical_px > 0.).then_some(logical_px);
        self
    }

    /// Rotates SVG icons around their center. Font glyphs stay static because
    /// GPUI does not expose glyph rotation. Honors reduced-motion settings.
    pub fn spin(mut self, motion: crate::LoadingMotion) -> Self {
        self.spin = Some(motion);
        self
    }

    pub(crate) fn with_default_size(mut self, size: f32) -> Self {
        self.size.get_or_insert(size);
        self
    }

    /// Overrides only the glyph's foreground color.
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }
}

fn resolve_named(name: &str, explicit: Option<IconPack>, cx: &mut App) -> Option<Source> {
    let pack = explicit.unwrap_or_else(|| IconPack::current(cx));
    if pack == IconPack::Tabler {
        let key = if explicit.is_some() {
            name
        } else {
            tabler_equivalent(name)
        };
        if let Some(&glyph) = tabler_glyphs()
            .get(key)
            .filter(|_| register_tabler(cx).is_ok())
        {
            return Some(Source::Glyph(glyph, Some(TABLER_FAMILY.into())));
        }
        if explicit.is_some() {
            return None;
        }
    }
    element_plus(name).map(Source::Svg)
}

impl RenderOnce for Icon {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let inherited = window.text_style();
        let size = px(self
            .size
            .unwrap_or_else(|| f32::from(inherited.font_size.to_pixels(window.rem_size()))));
        let color = self.color.unwrap_or(inherited.color);
        let source = match self.source {
            Source::Named(name, pack) => resolve_named(&name, pack, cx),
            source => Some(source),
        };
        match source {
            Some(Source::Svg(bytes)) if self.spin.is_some() => {
                use std::hash::{DefaultHasher, Hash, Hasher};
                let motion = self.spin.unwrap_or_default();
                let mut hasher = DefaultHasher::new();
                bytes.hash(&mut hasher);
                let key = SharedString::from(format!("__binary_svg__{}", hasher.finish()));
                gpui::canvas(
                    |_, _, _| {},
                    move |bounds, _, window, cx| {
                        if !bounds.intersects(&window.content_mask().bounds) {
                            return;
                        }
                        let elapsed = crate::loading::animation_time(window, cx);
                        let angle = std::f32::consts::TAU * motion.phase(elapsed);
                        let center = bounds.center();
                        let scale = window.scale_factor();
                        let matrix = gpui::TransformationMatrix::unit()
                            .translate(center.scale(scale))
                            .rotate(gpui::radians(angle))
                            .translate(center.scale(-scale));
                        let _ = window.paint_svg(bounds, key, Some(bytes), matrix, color, cx);
                    },
                )
                .size(size)
                .flex_none()
                .into_any_element()
            }
            Some(Source::Svg(bytes)) => svg()
                .data(bytes)
                .size(size)
                .flex_none()
                .text_color(color)
                .into_any_element(),
            Some(Source::Glyph(glyph, family)) => div()
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .min_w(size)
                .min_h(size)
                .text_size(size)
                .line_height(size)
                .text_color(color)
                .when_some(family, |el, family| {
                    el.font_family(family).font_weight(FontWeight::NORMAL)
                })
                .child(glyph.to_string())
                .into_any_element(),
            _ => div().size(size).flex_none().into_any_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn app_preference_updates_named_icons_but_not_explicit_packs(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            assert_eq!(IconPack::current(cx), IconPack::ElementPlus);
            assert!(matches!(
                resolve_named("plus", None, cx),
                Some(Source::Svg(_))
            ));
            IconPack::Tabler.set(cx).unwrap();
            assert!(matches!(
                resolve_named("close", None, cx),
                Some(Source::Glyph('\u{eb55}', _))
            ));
            assert!(matches!(
                resolve_named("plus", Some(IconPack::ElementPlus), cx),
                Some(Source::Svg(_))
            ));
            assert!(matches!(
                resolve_named("melon", Some(IconPack::Tabler), cx),
                Some(Source::Glyph(..))
            ));
            assert!(resolve_named("not-an-icon", None, cx).is_none());
        });
    }

    #[test]
    fn embedded_catalogs_and_aliases_are_available() {
        assert_eq!(IconPack::ElementPlus.names().count(), 293);
        for name in IconPack::ElementPlus.names() {
            assert!(
                element_plus(name).is_some(),
                "SVG index cannot resolve {name}"
            );
        }
        assert_eq!(IconPack::Tabler.names().count(), 5555);
        for name in [
            "plus",
            "close",
            "search",
            "setting",
            "document-copy",
            "arrow-up",
            "arrow-down",
        ] {
            assert!(element_plus(name).is_some());
            assert!(tabler_glyphs().contains_key(tabler_equivalent(name)));
        }
    }

    #[test]
    fn explicit_size_and_color_are_independent_of_the_font() {
        let icon = Icon::from_font("Example Icons", '\u{e001}')
            .size(18.)
            .color(gpui::rgb(0x409eff));
        assert!(
            matches!(&icon.source, Source::Glyph('\u{e001}', Some(family)) if family.as_ref() == "Example Icons")
        );
        assert_eq!(icon.size, Some(18.));
        let icon = icon.size(f32::NAN);
        assert!(icon.size.is_none());
        assert!(icon.color.is_some());
    }
}
