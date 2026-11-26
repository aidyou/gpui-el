use std::sync::OnceLock;

use gpui_el::Locale;
use serde::Deserialize;

macro_rules! copy_fields {
    ($($field:ident),+ $(,)?) => {
        #[derive(Default, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub(crate) struct Copy { $(pub(crate) $field: String,)+ }
    };
}

copy_fields!(
    language,
    basics,
    navigation,
    feedback,
    button_intro,
    icon_intro,
    init_title,
    init_intro,
    init_fonts,
    container_intro,
    layout_intro,
    link_intro,
    space_intro,
    tag_intro,
    menu_intro,
    loading_intro,
    grid_basic,
    grid_gutter,
    grid_mixed,
    grid_offset,
    grid_alignment,
    grid_responsive,
    grid_resize,
    vertical_menu,
    horizontal_menu,
    menu_index_path,
    menu_collapse,
    menu_expand,
    menu_disabled,
    window_minimize,
    window_zoom,
    window_close,
    dark,
    light,
    reduce_motion,
    restore_motion,
    copy,
    show_code,
    hide_code,
    button_page,
    icon_page,
    container_page,
    layout_page,
    link_page,
    space_page,
    tag_page,
    menu_page,
    loading_page,
    icon_pack_error,
    named_icons,
    icon_search,
    icon_empty,
    icon_copied,
    specific_icons,
    container_title,
    space_title,
    space_presets,
    link_basic,
    link_disabled,
    link_underline,
    link_with_icon,
    underline_always,
    underline_hover,
    underline_never,
    link_edit,
    link_check,
    menu_first,
    menu_second,
    menu_language,
    menu_theme,
    layout_header,
    layout_aside,
    layout_main,
    layout_footer,
    show_loading,
    hide_loading,
    loading_vertical,
    loading_horizontal,
    loading_clear,
    loading_text,
    solid_title,
    plain_title,
    dashed_title,
    text_title,
    size_title,
    icon_title,
    activation_title,
    button_group,
    button_custom,
    default,
    primary,
    success,
    warning,
    danger,
    info,
    text,
    text_bg,
    link,
    large,
    compact,
    add,
    count_increment,
    disabled,
    saving,
    activation_help,
    tag_basic,
    tag_removable,
    tag_editable,
    tag_sizes,
    tag_theme,
    tag_rounded,
    tag_checkable,
    tag_effect_dark,
    tag_effect_light,
    tag_effect_plain,
    tag_hit,
    tag_add,
    tag_reset,
    tag_removed,
    tag_checked,
    tag_toggle,
    tag_label,
    tag_small,
    tag_checked_count,
    tag_editable_help,
);

pub(crate) fn strings(locale: Locale) -> &'static Copy {
    static EN: OnceLock<Copy> = OnceLock::new();
    static ZH: OnceLock<Copy> = OnceLock::new();
    let en = || EN.get_or_init(|| decode(include_str!("../i18n/en.yml")).unwrap_or_default());
    if locale == Locale::SimplifiedChinese {
        ZH.get_or_init(|| {
            decode(include_str!("../i18n/zh-Hans.yml")).unwrap_or_else(|| {
                // Only needed if an embedded translation is damaged; validated by tests.
                decode(include_str!("../i18n/en.yml")).unwrap_or_default()
            })
        })
    } else {
        en()
    }
}

fn decode(source: &str) -> Option<Copy> {
    match serde_yaml_ng::from_str(source) {
        Ok(copy) => Some(copy),
        Err(error) => {
            eprintln!("Could not load gallery translations: {error}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_catalogs_load_and_preserve_the_count_placeholder() {
        let en = decode(include_str!("../i18n/en.yml")).expect("English catalog");
        let zh = decode(include_str!("../i18n/zh-Hans.yml")).expect("Chinese catalog");
        assert_ne!(en.button_intro, zh.button_intro);
        assert_ne!(en.saving, zh.saving);
        for catalog in [en, zh] {
            assert_eq!(catalog.activation_help.matches("%{count}").count(), 1);
        }
    }
}
