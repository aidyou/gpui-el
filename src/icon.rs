use std::{
    borrow::Cow,
    collections::BTreeMap,
    fmt::Write,
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, Mutex, OnceLock},
};

use gpui::{
    App, FontWeight, Global, Hsla, IntoElement, RenderOnce, SharedString, Window, div, prelude::*,
    px, svg,
};
use ttf_parser::OutlineBuilder;

include!(concat!(env!("OUT_DIR"), "/element_plus_icons.rs"));

const TABLER_FAMILY: &str = "tabler-icons";

static REGISTERED_FONTS: OnceLock<Mutex<BTreeMap<String, &'static [u8]>>> = OnceLock::new();
type GlyphSvgCache = BTreeMap<(String, char), Arc<[u8]>>;
static GLYPH_SVGS: OnceLock<Mutex<GlyphSvgCache>> = OnceLock::new();

fn element_plus(name: &str) -> Option<&'static [u8]> {
    ELEMENT_PLUS
        .binary_search_by_key(&name, |(key, _)| *key)
        .ok()
        .and_then(|i| ELEMENT_PLUS.get(i))
        .map(|(_, bytes)| *bytes)
}

fn tabler_glyphs() -> &'static BTreeMap<&'static str, char> {
    static GLYPHS: OnceLock<BTreeMap<&str, char>> = OnceLock::new();
    // A malformed line in the embedded table is skipped rather than aborting
    // the application; the tests pin that the shipped table parses cleanly.
    GLYPHS.get_or_init(|| {
        include_str!("icons/tabler/codepoints.tsv")
            .lines()
            .filter_map(|line| {
                let (name, code) = line.split_once('\t')?;
                let glyph = char::from_u32(u32::from_str_radix(code.trim(), 16).ok()?)?;
                Some((name, glyph))
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
    let mut previous = None;
    for (i, &c) in chars.iter().enumerate() {
        let boundary = previous.is_some_and(|p: char| {
            p.is_ascii_lowercase()
                || p.is_ascii_digit()
                || (p.is_ascii_uppercase()
                    && chars.get(i + 1).is_some_and(char::is_ascii_lowercase))
        });
        if c.is_ascii_uppercase() && boundary {
            key.push('-');
        }
        key.push(c.to_ascii_lowercase());
        previous = Some(c);
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
    pub fn set(self, cx: &mut App) -> crate::Result<()> {
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

fn register_tabler(cx: &mut App) -> crate::Result<()> {
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
/// A rejected font is reported as [`crate::Error::Font`].
pub fn register_icon_font(cx: &App, bytes: &'static [u8]) -> crate::Result<()> {
    cx.text_system()
        .add_fonts(vec![Cow::Borrowed(bytes)])
        .map_err(crate::Error::font)?;
    remember_icon_font(bytes);
    Ok(())
}

fn registered_fonts() -> &'static Mutex<BTreeMap<String, &'static [u8]>> {
    REGISTERED_FONTS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn remember_icon_font(bytes: &'static [u8]) {
    let Ok(face) = ttf_parser::Face::parse(bytes, 0) else {
        return;
    };
    // Family, typographic family, and WWS family names are all valid ways to
    // address a font in GPUI. Keep the bytes so transformed glyphs can use
    // outlines even though GPUI's SVG renderer has a separate font database.
    let names: Vec<_> = face
        .names()
        .into_iter()
        .filter(|name| matches!(name.name_id, 1 | 16 | 21))
        .filter_map(|name| name.to_string())
        .collect();
    if names.is_empty() {
        return;
    }
    let mut fonts = registered_fonts()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for name in names {
        fonts.insert(name, bytes);
    }
}

fn registered_font_bytes(family: &str) -> Option<&'static [u8]> {
    registered_fonts()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(family)
        .copied()
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
    spin_speed: Option<crate::MotionSpeed>,
    rotate: f32,
}

impl Icon {
    fn with_source(source: Source) -> Self {
        Self {
            source,
            size: None,
            color: None,
            spin: None,
            spin_speed: None,
            rotate: 0.,
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

    /// Applies the size and color unless the icon already chose its own, so
    /// embedding components can keep their defaults without overriding
    /// explicit choices.
    pub(crate) fn fallback(mut self, size: f32, color: gpui::Hsla) -> Self {
        if self.size.is_none() {
            self.size = Some(size);
        }
        if self.color.is_none() {
            self.color = Some(color);
        }
        self
    }

    /// Overrides the glyph size in logical pixels; invalid values restore inheritance.
    pub fn size(mut self, logical_px: f32) -> Self {
        self.size = (logical_px.is_finite() && logical_px > 0.).then_some(logical_px);
        self
    }

    /// Rotates SVG and font glyph icons around their center. Font glyphs are
    /// rasterized through SVG so GPUI can transform them. Honors reduced-motion
    /// settings.
    pub fn spin(mut self, motion: crate::LoadingMotion) -> Self {
        self.spin = Some(motion);
        self
    }

    /// Overrides the theme's [`MotionSpeed`](crate::MotionSpeed) for [`Self::spin`].
    pub fn spin_speed(mut self, speed: crate::MotionSpeed) -> Self {
        self.spin_speed = Some(speed);
        self
    }

    /// Rotates an SVG or font glyph clockwise around its center, in radians.
    /// Callers animate this by re-rendering with a new angle, as the menu
    /// chevron does.
    pub fn rotate(mut self, radians: f32) -> Self {
        self.rotate = if radians.is_finite() { radians } else { 0. };
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

#[derive(Default)]
struct SvgPathBuilder {
    path: String,
}

impl OutlineBuilder for SvgPathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let _ = write!(self.path, "M {x} {y} ");
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let _ = write!(self.path, "L {x} {y} ");
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let _ = write!(self.path, "Q {x1} {y1} {x} {y} ");
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let _ = write!(self.path, "C {x1} {y1} {x2} {y2} {x} {y} ");
    }

    fn close(&mut self) {
        self.path.push_str("Z ");
    }
}

fn glyph_path_svg(bytes: &[u8], glyph: char) -> Option<Vec<u8>> {
    let face = ttf_parser::Face::parse(bytes, 0).ok()?;
    let glyph_id = face.glyph_index(glyph)?;
    let mut builder = SvgPathBuilder::default();
    let bbox = face.outline_glyph(glyph_id, &mut builder)?;
    let x_min = bbox.x_min as f32;
    let x_max = bbox.x_max as f32;
    let y_min = bbox.y_min as f32;
    let y_max = bbox.y_max as f32;
    let width = x_max - x_min;
    let height = y_max - y_min;
    if !width.is_finite() || !height.is_finite() || width <= 0. || height <= 0. {
        return None;
    }
    let scale = 100. / width.max(height);
    let x = (100. - width * scale) / 2.;
    let y = (100. - height * scale) / 2.;
    Some(
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100"><path d="{}" transform="translate({x} {y}) scale({scale} {}) translate({} {})"/></svg>"#,
            builder.path, -scale, -x_min, -y_max
        )
        .into_bytes(),
    )
}

fn xml_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn svg_font_family(family: &str) -> &str {
    match family {
        ".SystemUIFont" => "sans-serif",
        ".ZedSans" | "Zed Plex Sans" => "IBM Plex Sans",
        ".ZedMono" | "Zed Plex Mono" => "Lilex",
        _ => family,
    }
}

fn glyph_text_svg(glyph: char, family: &str) -> Vec<u8> {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100"><text x="50" y="50" text-anchor="middle" dominant-baseline="central" font-family="{}" font-size="100">{}</text></svg>"#,
        xml_escape(svg_font_family(family)),
        xml_escape(&glyph.to_string())
    )
    .into_bytes()
}

fn transformed_glyph_svg(glyph: char, family: &str) -> Arc<[u8]> {
    let key = (family.to_owned(), glyph);
    let cache = GLYPH_SVGS.get_or_init(|| Mutex::new(BTreeMap::new()));
    if let Some(svg) = cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&key)
    {
        return Arc::clone(svg);
    }
    let svg = Arc::from(
        registered_font_bytes(family)
            .and_then(|bytes| glyph_path_svg(bytes, glyph))
            .unwrap_or_else(|| glyph_text_svg(glyph, family)),
    );
    cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(key, Arc::clone(&svg));
    svg
}

fn spinning_svg<D>(
    data: D,
    size: gpui::Pixels,
    color: Hsla,
    motion: crate::LoadingMotion,
    speed: crate::MotionSpeed,
) -> gpui::AnyElement
where
    D: AsRef<[u8]> + 'static,
{
    let mut hasher = DefaultHasher::new();
    data.as_ref().hash(&mut hasher);
    let key = SharedString::from(format!("__binary_svg__{}", hasher.finish()));
    gpui::canvas(
        |_, _, _| {},
        move |bounds, _, window, cx| {
            if !bounds.intersects(&window.content_mask().bounds) {
                return;
            }
            let elapsed = crate::motion::animation_time(window, cx);
            let angle = std::f32::consts::TAU * motion.phase(elapsed, speed);
            let center = bounds.center();
            let scale = window.scale_factor();
            let matrix = gpui::TransformationMatrix::unit()
                .translate(center.scale(scale))
                .rotate(gpui::radians(angle))
                .translate(center.scale(-scale));
            let _ = window.paint_svg(bounds, key, Some(data.as_ref()), matrix, color, cx);
        },
    )
    .size(size)
    .flex_none()
    .into_any_element()
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
                let motion = self.spin.unwrap_or_default();
                let speed = self
                    .spin_speed
                    .unwrap_or_else(|| crate::Theme::current(cx).motion_speed);
                spinning_svg(bytes, size, color, motion, speed)
            }
            Some(Source::Svg(bytes)) => svg()
                .data(bytes)
                .size(size)
                .flex_none()
                .text_color(color)
                .with_transformation(
                    gpui::Transformation::default().with_rotation(gpui::radians(self.rotate)),
                )
                .into_any_element(),
            Some(Source::Glyph(glyph, family)) => {
                let family_name = family.as_deref().unwrap_or(inherited.font_family.as_str());
                if self.spin.is_some() || self.rotate != 0. {
                    let data = transformed_glyph_svg(glyph, family_name);
                    if let Some(motion) = self.spin {
                        let speed = self
                            .spin_speed
                            .unwrap_or_else(|| crate::Theme::current(cx).motion_speed);
                        spinning_svg(data, size, color, motion, speed)
                    } else {
                        svg()
                            .data(data.as_ref())
                            .size(size)
                            .flex_none()
                            .text_color(color)
                            .with_transformation(
                                gpui::Transformation::default()
                                    .with_rotation(gpui::radians(self.rotate)),
                            )
                            .into_any_element()
                    }
                } else {
                    div()
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
                        .into_any_element()
                }
            }
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
    fn embedded_tabler_table_maps_every_line() {
        // The runtime parser skips malformed rows so a bad table can never
        // abort a host application; this pins that the shipped table is whole.
        let lines = include_str!("icons/tabler/codepoints.tsv")
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count();
        assert!(lines > 5000, "unexpected table size: {lines}");
        assert_eq!(tabler_glyphs().len(), lines);
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

    #[test]
    fn glyph_text_svg_escapes_xml_and_maps_virtual_fonts() {
        let svg = String::from_utf8(glyph_text_svg('☯', ".SystemUIFont")).unwrap();
        assert!(svg.contains("font-family=\"sans-serif\""));
        assert!(svg.contains(">☯</text>"));

        let svg = String::from_utf8(glyph_text_svg('<', "A\"&<")).unwrap();
        assert!(svg.contains("font-family=\"A&quot;&amp;&lt;\""));
        assert!(svg.contains(">&lt;</text>"));
    }

    #[test]
    fn registered_tabler_glyphs_have_svg_outlines() {
        let svg = glyph_path_svg(include_bytes!("icons/tabler/tabler-icons.ttf"), '\u{ec35}')
            .expect("Tabler yin-yang glyph should have an outline");
        let svg = String::from_utf8(svg).unwrap();
        assert!(svg.contains("<path d=\"M "));
        assert!(svg.contains("transform=\"translate("));
        assert!(svg.contains("scale("));
    }
}
