use std::sync::OnceLock;

use gpui::{App, Global};
use serde::Deserialize;

/// Language for text supplied by the library. Caller-provided labels are unaffected.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum Locale {
    #[default]
    English,
    SimplifiedChinese,
}

impl Global for Locale {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Messages {
    button: ButtonMessages,
    menu: MenuMessages,
    tag: TagMessages,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MenuMessages {
    more: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ButtonMessages {
    loading: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TagMessages {
    close: String,
}

impl Messages {
    /// Fallback for a broken embedded locale file. The values mirror
    /// `i18n/en.yml`, and a unit test fails when the two drift apart, so a
    /// malformed resource degrades to English instead of panicking.
    fn fallback() -> Self {
        Self {
            button: ButtonMessages {
                loading: "Loading".into(),
            },
            menu: MenuMessages {
                more: "More".into(),
            },
            tag: TagMessages {
                close: "Close".into(),
            },
        }
    }
}

impl Locale {
    /// Sets the application-wide locale and refreshes open windows.
    pub fn set(self, cx: &mut App) {
        cx.set_global(self);
        cx.refresh_windows();
    }

    /// Returns the current locale, defaulting to English.
    pub fn current(cx: &App) -> Self {
        cx.try_global::<Self>().copied().unwrap_or_default()
    }

    pub(crate) fn button_loading(self) -> &'static str {
        &self.messages().button.loading
    }
    pub(crate) fn menu_more(self) -> &'static str {
        &self.messages().menu.more
    }
    pub(crate) fn tag_close(self) -> &'static str {
        &self.messages().tag.close
    }

    fn messages(self) -> &'static Messages {
        static ENGLISH: OnceLock<Messages> = OnceLock::new();
        static CHINESE: OnceLock<Messages> = OnceLock::new();
        let (cache, source) = match self {
            Self::English => (&ENGLISH, include_str!("../i18n/en.yml")),
            Self::SimplifiedChinese => (&CHINESE, include_str!("../i18n/zh-Hans.yml")),
        };
        cache.get_or_init(|| {
            serde_yaml_ng::from_str(source).unwrap_or_else(|_| Messages::fallback())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_built_in_locales_define_loading() {
        assert_eq!(Locale::default().button_loading(), "Loading");
        assert_eq!(Locale::SimplifiedChinese.button_loading(), "加载中");
        assert_eq!(Locale::default().tag_close(), "Close");
        assert_eq!(Locale::SimplifiedChinese.tag_close(), "关闭");
    }

    #[test]
    fn fallback_matches_embedded_english() {
        let embedded = serde_yaml_ng::from_str::<Messages>(include_str!("../i18n/en.yml"))
            .expect("embedded English locale parses in tests");
        let fallback = Messages::fallback();
        assert_eq!(fallback.button.loading, embedded.button.loading);
        assert_eq!(fallback.menu.more, embedded.menu.more);
        assert_eq!(fallback.tag.close, embedded.tag.close);
    }
}
