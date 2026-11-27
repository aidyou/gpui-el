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
    table: TableMessages,
    input_number: InputNumberMessages,
    select: SelectMessages,
    auto_complete: AutoCompleteMessages,
    color_picker: ColorPickerMessages,
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TableMessages {
    copy: String,
    copied: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InputNumberMessages {
    increase: String,
    decrease: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectMessages {
    loading: String,
    no_data: String,
    no_match: String,
    clear: String,
    placeholder: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AutoCompleteMessages {
    loading: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ColorPickerMessages {
    clear: String,
    confirm: String,
    default_label: String,
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
            table: TableMessages {
                copy: "Copy as markdown".into(),
                copied: "Copied".into(),
            },
            input_number: InputNumberMessages {
                increase: "Increase".into(),
                decrease: "Decrease".into(),
            },
            select: SelectMessages {
                loading: "Loading".into(),
                no_data: "No data".into(),
                no_match: "No matching data".into(),
                clear: "Clear selection".into(),
                placeholder: "Select".into(),
            },
            auto_complete: AutoCompleteMessages {
                loading: "Loading".into(),
            },
            color_picker: ColorPickerMessages {
                clear: "Clear".into(),
                confirm: "Confirm".into(),
                default_label: "Color picker".into(),
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
    pub(crate) fn table_copy(self) -> &'static str {
        &self.messages().table.copy
    }
    pub(crate) fn table_copied(self) -> &'static str {
        &self.messages().table.copied
    }
    pub(crate) fn input_number_increase(self) -> &'static str {
        &self.messages().input_number.increase
    }
    pub(crate) fn input_number_decrease(self) -> &'static str {
        &self.messages().input_number.decrease
    }
    pub(crate) fn select_loading(self) -> &'static str {
        &self.messages().select.loading
    }
    pub(crate) fn select_no_data(self) -> &'static str {
        &self.messages().select.no_data
    }
    pub(crate) fn select_no_match(self) -> &'static str {
        &self.messages().select.no_match
    }
    pub(crate) fn select_clear(self) -> &'static str {
        &self.messages().select.clear
    }
    pub(crate) fn select_placeholder(self) -> &'static str {
        &self.messages().select.placeholder
    }
    pub(crate) fn auto_complete_loading(self) -> &'static str {
        &self.messages().auto_complete.loading
    }
    pub(crate) fn color_picker_clear(self) -> &'static str {
        &self.messages().color_picker.clear
    }
    pub(crate) fn color_picker_confirm(self) -> &'static str {
        &self.messages().color_picker.confirm
    }
    pub(crate) fn color_picker_default_label(self) -> &'static str {
        &self.messages().color_picker.default_label
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
        assert_eq!(Locale::default().table_copy(), "Copy as markdown");
        assert_eq!(Locale::SimplifiedChinese.table_copy(), "复制为 Markdown");
        assert_eq!(Locale::default().table_copied(), "Copied");
        assert_eq!(Locale::SimplifiedChinese.table_copied(), "复制成功");
        assert_eq!(Locale::default().input_number_increase(), "Increase");
        assert_eq!(Locale::SimplifiedChinese.input_number_increase(), "增加");
        assert_eq!(Locale::default().input_number_decrease(), "Decrease");
        assert_eq!(Locale::SimplifiedChinese.input_number_decrease(), "减少");
        assert_eq!(Locale::default().select_loading(), "Loading");
        assert_eq!(Locale::SimplifiedChinese.select_loading(), "加载中");
        assert_eq!(Locale::default().select_no_data(), "No data");
        assert_eq!(Locale::SimplifiedChinese.select_no_data(), "暂无数据");
        assert_eq!(Locale::default().select_no_match(), "No matching data");
        assert_eq!(Locale::SimplifiedChinese.select_no_match(), "无匹配数据");
        assert_eq!(Locale::default().select_clear(), "Clear selection");
        assert_eq!(Locale::SimplifiedChinese.select_clear(), "清空选择");
        assert_eq!(Locale::default().select_placeholder(), "Select");
        assert_eq!(Locale::SimplifiedChinese.select_placeholder(), "请选择");
        assert_eq!(Locale::default().auto_complete_loading(), "Loading");
        assert_eq!(Locale::SimplifiedChinese.auto_complete_loading(), "加载中");
        assert_eq!(Locale::default().color_picker_clear(), "Clear");
        assert_eq!(Locale::SimplifiedChinese.color_picker_clear(), "清空");
        assert_eq!(Locale::default().color_picker_confirm(), "Confirm");
        assert_eq!(Locale::SimplifiedChinese.color_picker_confirm(), "确定");
        assert_eq!(
            Locale::default().color_picker_default_label(),
            "Color picker"
        );
        assert_eq!(
            Locale::SimplifiedChinese.color_picker_default_label(),
            "颜色选择器"
        );
    }

    #[test]
    fn fallback_matches_embedded_english() {
        let embedded = serde_yaml_ng::from_str::<Messages>(include_str!("../i18n/en.yml"))
            .expect("embedded English locale parses in tests");
        let fallback = Messages::fallback();
        assert_eq!(fallback.button.loading, embedded.button.loading);
        assert_eq!(fallback.menu.more, embedded.menu.more);
        assert_eq!(fallback.tag.close, embedded.tag.close);
        assert_eq!(fallback.table.copy, embedded.table.copy);
        assert_eq!(fallback.table.copied, embedded.table.copied);
        assert_eq!(
            fallback.input_number.increase,
            embedded.input_number.increase
        );
        assert_eq!(
            fallback.input_number.decrease,
            embedded.input_number.decrease
        );
        assert_eq!(fallback.select.loading, embedded.select.loading);
        assert_eq!(fallback.select.no_data, embedded.select.no_data);
        assert_eq!(fallback.select.no_match, embedded.select.no_match);
        assert_eq!(fallback.select.clear, embedded.select.clear);
        assert_eq!(fallback.select.placeholder, embedded.select.placeholder);
        assert_eq!(
            fallback.auto_complete.loading,
            embedded.auto_complete.loading
        );
        assert_eq!(fallback.color_picker.clear, embedded.color_picker.clear);
        assert_eq!(fallback.color_picker.confirm, embedded.color_picker.confirm);
        assert_eq!(
            fallback.color_picker.default_label,
            embedded.color_picker.default_label
        );
    }
}
