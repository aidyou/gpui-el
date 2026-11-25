use gpui::{App, Global, SharedString};

/// Element Plus light/dark defaults and native rendering preferences.
#[derive(Clone, Debug)]
pub struct Theme {
    pub dark: bool,
    pub reduced_motion: bool,
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
            font_family: None,
        }
    }

    pub const fn dark() -> Self {
        Self {
            dark: true,
            reduced_motion: false,
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
        if self.dark { 0x141414 } else { WHITE }
    }

    pub(crate) fn regular_text(&self) -> u32 {
        if self.dark {
            mix(self.background(), 0xf0f5ff, 85)
        } else {
            0x606266
        }
    }

    pub(crate) fn secondary_text(&self) -> u32 {
        if self.dark {
            mix(self.background(), 0xf0f5ff, 65)
        } else {
            0x909399
        }
    }

    pub(crate) fn primary_text(&self) -> u32 {
        if self.dark {
            mix(self.background(), 0xf0f5ff, 95)
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

    pub(crate) fn fill(&self, shade: Fill) -> u32 {
        if self.dark {
            mix(
                self.background(),
                0xfafcff,
                match shade {
                    Fill::Light => 8,
                    Fill::Base => 12,
                    Fill::Dark => 16,
                },
            )
        } else {
            match shade {
                Fill::Light => 0xf5f7fa,
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

// Theme Chalk's default font stack, excluding the generic system fallback.
const FONT_FALLBACKS: [&str; 7] = [
    "Helvetica Neue",
    "Helvetica",
    "PingFang SC",
    "Hiragino Sans GB",
    "Microsoft YaHei",
    "微软雅黑",
    "Arial",
];

pub(crate) enum Fill {
    Light,
    Base,
    Dark,
}

pub(crate) const WHITE: u32 = 0xffffff;
pub(crate) const PRIMARY: u32 = 0x409eff;
pub(crate) const SUCCESS: u32 = 0x67c23a;
pub(crate) const WARNING: u32 = 0xe6a23c;
pub(crate) const DANGER: u32 = 0xf56c6c;
pub(crate) const INFO: u32 = 0x909399;

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
