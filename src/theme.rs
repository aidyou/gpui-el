use gpui::{App, BoxShadow, Global, Hsla, SharedString, point, px, rgb};

use crate::motion::MotionSpeed;

/// Element Plus light/dark defaults and native rendering preferences.
#[derive(Clone, Debug)]
pub struct Theme {
    pub dark: bool,
    pub reduced_motion: bool,
    /// Scales every animation duration; components honor it unless they set
    /// their own [`MotionSpeed`].
    pub motion_speed: MotionSpeed,
    /// Overrides native font selection; missing glyphs still use system fallback.
    pub font_family: Option<SharedString>,
}

impl Global for Theme {}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

impl Theme {
    pub const fn light() -> Self {
        Self {
            dark: false,
            reduced_motion: false,
            motion_speed: MotionSpeed::Normal,
            font_family: None,
        }
    }

    pub const fn dark() -> Self {
        Self {
            dark: true,
            reduced_motion: false,
            motion_speed: MotionSpeed::Normal,
            font_family: None,
        }
    }

    /// Sets the application-wide theme. Existing views are refreshed.
    pub fn set(self, cx: &mut App) {
        cx.set_global(self);
        cx.refresh_windows();
    }

    pub(crate) fn current(cx: &App) -> Self {
        cx.try_global::<Self>().cloned().unwrap_or_else(|| Self {
            dark: matches!(
                cx.window_appearance(),
                gpui::WindowAppearance::Dark | gpui::WindowAppearance::VibrantDark
            ),
            ..Self::default()
        })
    }

    pub(crate) fn resolved_font(&self, cx: &mut App) -> Option<SharedString> {
        if let Some(family) = &self.font_family {
            return Some(family.clone());
        }
        if let Some(default) = cx.try_global::<DefaultFont>() {
            return default.0.clone();
        }
        let installed = cx.text_system().all_font_names();
        let family = FONT_FALLBACKS
            .iter()
            .find(|candidate| {
                installed
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(candidate))
            })
            .map(|family| SharedString::from(*family));
        cx.set_global(DefaultFont(family.clone()));
        family
    }

    pub(crate) fn background(&self) -> u32 {
        if self.dark { DARK_BACKGROUND } else { WHITE }
    }

    /// Theme Chalk `--el-bg-color-overlay` (not the translucent modal overlay).
    pub(crate) fn overlay_background(&self) -> u32 {
        if self.dark { 0x1d1e1f } else { WHITE }
    }

    /// Theme Chalk `--el-disabled-text-color`, which aliases placeholder text.
    pub(crate) fn disabled_text(&self) -> u32 {
        if self.dark {
            mix(self.background(), DARK_TEXT_BASE, 55)
        } else {
            0xa8abb2
        }
    }

    /// Theme Chalk `--el-box-shadow-light` in light and dark `var.scss`.
    pub(crate) fn light_shadow(&self) -> BoxShadow {
        BoxShadow {
            color: Hsla::from(rgb(0)).opacity(if self.dark { 0.72 } else { 0.12 }),
            offset: point(px(0.), px(0.)),
            blur_radius: px(12.),
            spread_radius: px(0.),
            inset: false,
        }
    }

    /// Theme Chalk `--el-mask-color` and `--el-mask-color-extra-light`.
    pub(crate) fn mask(&self, shade: Mask) -> Hsla {
        let opacity = match shade {
            Mask::Base if self.dark => 0.8,
            Mask::Base => 0.9,
            Mask::ExtraLight => 0.3,
        };
        Hsla::from(rgb(if self.dark { 0 } else { WHITE })).opacity(opacity)
    }

    pub(crate) fn motion_reduced(cx: &App) -> bool {
        cx.reduce_motion() || Self::current(cx).reduced_motion
    }

    pub(crate) fn regular_text(&self) -> u32 {
        if self.dark {
            mix(self.background(), DARK_TEXT_BASE, 85)
        } else {
            0x606266
        }
    }

    pub(crate) fn secondary_text(&self) -> u32 {
        if self.dark {
            mix(self.background(), DARK_TEXT_BASE, 65)
        } else {
            0x909399
        }
    }

    pub(crate) fn primary_text(&self) -> u32 {
        if self.dark {
            mix(self.background(), DARK_TEXT_BASE, 95)
        } else {
            0x303133
        }
    }

    pub(crate) fn border(&self, light: bool) -> u32 {
        if self.dark {
            mix(self.background(), 0xf5f8ff, if light { 20 } else { 25 })
        } else if light {
            0xe4e7ed
        } else {
            0xdcdfe6
        }
    }
    /// `--el-border-color-lighter`, used by Table dividers.
    pub(crate) fn border_lighter(&self) -> u32 {
        if self.dark {
            mix(self.background(), 0xf5f8ff, 15)
        } else {
            0xebeef5
        }
    }

    pub(crate) fn fill(&self, shade: Fill) -> u32 {
        if self.dark {
            mix(
                self.background(),
                0xfafcff,
                match shade {
                    Fill::Light => 8,
                    Fill::Lighter => 4,
                    Fill::Base => 12,
                    Fill::Dark => 16,
                },
            )
        } else {
            match shade {
                Fill::Light => 0xf5f7fa,
                Fill::Lighter => 0xfafafa,
                Fill::Base => 0xf0f2f5,
                Fill::Dark => 0xebedf0,
            }
        }
    }

    pub(crate) fn tint(&self, base: u32, level: u32) -> u32 {
        mix(base, self.background(), level * 10)
    }

    pub(crate) fn shade(&self, base: u32) -> u32 {
        mix(base, if self.dark { WHITE } else { 0 }, 20)
    }
}

struct DefaultFont(Option<SharedString>);
impl Global for DefaultFont {}

// Native desktop UI faces first, then CJK coverage and the Theme Chalk web
// stack, excluding the generic system fallback.
const FONT_FALLBACKS: &[&str] = &[
    "SF Pro Text",
    ".AppleSystemUIFont",
    "Segoe UI Variable Text",
    "Segoe UI",
    "Inter",
    "Noto Sans",
    "Cantarell",
    "Ubuntu",
    "PingFang SC",
    "Hiragino Sans GB",
    "Noto Sans CJK SC",
    "Microsoft YaHei",
    "微软雅黑",
    "Helvetica Neue",
    "Helvetica",
    "Arial",
];

pub(crate) enum Mask {
    Base,
    ExtraLight,
}

pub(crate) enum Fill {
    Light,
    Lighter,
    Base,
    Dark,
}

pub(crate) const WHITE: u32 = 0xffffff;
pub(crate) const PRIMARY: u32 = 0x409eff;
pub(crate) const SUCCESS: u32 = 0x67c23a;
pub(crate) const WARNING: u32 = 0xe6a23c;
pub(crate) const DANGER: u32 = 0xf56c6c;
pub(crate) const INFO: u32 = 0x909399;
pub(crate) const DARK_BACKGROUND: u32 = 0x141414;
const DARK_TEXT_BASE: u32 = 0xf0f5ff;

/// Shared Theme Chalk defaults. Component-specific recipes still follow
/// their own maps rather than equating every `small` preset.
pub mod tokens {
    /// Desktop control heights. Apple's desktop guidance uses a 28 pt default
    /// control with a 20 pt minimum; KDE Breeze tool buttons and inputs sit at
    /// 28 px as well.
    pub mod control_height {
        pub const SMALL: f32 = 20.;
        pub const DEFAULT: f32 = 28.;
        pub const LARGE: f32 = 32.;
    }

    /// Element Plus Space `SIZE_MAP` (`components/space/src/use-space.ts`).
    pub mod spacing {
        pub const SMALL: f32 = 8.;
        pub const DEFAULT: f32 = 12.;
        pub const LARGE: f32 = 16.;
    }

    /// Desktop corner radii. macOS standard controls use about 6 pt and KDE
    /// Breeze about 3 px; `BASE` takes the middle of that range.
    pub mod radius {
        pub const SMALL: f32 = 3.;
        pub const BASE: f32 = 5.;
        /// Upstream `round`; used by Button. There is no global `large` key.
        pub const ROUND: f32 = 20.;

        /// Fully rounds the ends using the shorter outer edge; equivalent to
        /// the oversized CSS radius used by components such as rounded Tag.
        pub fn capsule(width: f32, height: f32) -> f32 {
            width.min(height).max(0.) / 2.
        }
    }

    /// Theme Chalk `$border-width`.
    pub const BORDER_WIDTH: f32 = 1.;

    /// Theme Chalk `--el-font-weight-primary` (500).
    pub const FONT_WEIGHT_PRIMARY: gpui::FontWeight = gpui::FontWeight::MEDIUM;

    /// Desktop type scale. macOS body text is 13 pt and KDE's default font is
    /// a 10 pt face (~13 px), so `BASE` is the regular UI size.
    pub mod font_size {
        pub const EXTRA_SMALL: f32 = 11.;
        pub const SMALL: f32 = 12.;
        pub const BASE: f32 = 13.;
        pub const MEDIUM: f32 = 15.;
        pub const LARGE: f32 = 17.;
        pub const EXTRA_LARGE: f32 = 20.;
    }
}

/// Native order within GPUI deferred drawing, not the CSS `z-index` scale.
/// Local overlays such as Loading remain in their own element tree.
pub(crate) fn popup_priority(depth: usize) -> usize {
    const POPPER: usize = 10;
    POPPER + depth.min(100)
}

// Theme Chalk mixes opaque sRGB channels, rounded once at the end.
pub(crate) fn mix(base: u32, target: u32, weight: u32) -> u32 {
    let channel = |shift: u32| {
        ((((base >> shift) & 0xff_u32) * (100 - weight)
            + ((target >> shift) & 0xff_u32) * weight
            + 50)
            / 100)
            << shift
    };
    channel(16) | channel(8) | channel(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_layer_styles_match_theme_chalk() {
        let light = Theme::light();
        let dark = Theme::dark();
        assert_eq!(light.overlay_background(), 0xffffff);
        assert_eq!(dark.overlay_background(), 0x1d1e1f);
        assert_eq!(light.disabled_text(), 0xa8abb2);
        assert_eq!(dark.disabled_text(), 0x8d9095);
        for (theme, shadow_alpha) in [(light, 0.12), (dark, 0.72)] {
            let shadow = theme.light_shadow();
            assert_eq!(shadow.color.a, shadow_alpha);
            assert_eq!(shadow.blur_radius, px(12.));
            assert_eq!(theme.mask(Mask::ExtraLight).a, 0.3);
        }
        assert!(popup_priority(1) > popup_priority(0));
    }
}
