use std::collections::HashSet;

use gpui::{Context, Div, SharedString, div, prelude::*, px, rgb};
use gpui_el::{TagEffect, TagType, el_button, el_check_tag, el_icon, el_tag, tokens::spacing};

use crate::{
    Gallery,
    copy::Copy,
    demo::{row, source},
};

pub(crate) struct State {
    toggle: bool,
    closed: HashSet<u32>,
    dynamic: Vec<DynamicTag>,
    checked: [bool; 6],
    /// Monotonic counter backing [`DynamicTag::id`]; labels may repeat after
    /// deletions, so names are never used as identity.
    next_dynamic_id: u32,
}

/// A dynamically added tag. Removal is keyed by the stable id, never by the
/// label or the array index: after a deletion the next added tag can reuse a
/// display name, which would otherwise remove two entries at once.
pub(crate) struct DynamicTag {
    id: u32,
    label: SharedString,
}

impl Default for State {
    fn default() -> Self {
        Self {
            toggle: false,
            closed: HashSet::new(),
            dynamic: ["Tag 1", "Tag 2", "Tag 3"]
                .into_iter()
                .enumerate()
                .map(|(id, label)| DynamicTag {
                    id: id as u32,
                    label: label.into(),
                })
                .collect(),
            checked: [true; 6],
            next_dynamic_id: 3,
        }
    }
}

impl State {
    /// Removes one dynamic tag by its stable id. Labels may repeat after
    /// deletions, so closing by name or array index could remove two entries
    /// at once.
    pub(crate) fn close_dynamic(&mut self, id: u32) {
        self.dynamic.retain(|item| item.id != id);
    }

    /// Appends a dynamic tag under a fresh id.
    pub(crate) fn add_dynamic(&mut self, label: SharedString) {
        let id = self.next_dynamic_id;
        self.next_dynamic_id += 1;
        self.dynamic.push(DynamicTag { id, label });
    }

    /// Labels of the dynamic tags, in display order.
    #[cfg(test)]
    pub(crate) fn dynamic_labels(&self) -> Vec<SharedString> {
        self.dynamic.iter().map(|item| item.label.clone()).collect()
    }
}

impl Gallery {
    pub(crate) fn tags(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let types = [
            (TagType::Primary, s.primary.as_str()),
            (TagType::Success, s.success.as_str()),
            (TagType::Info, s.info.as_str()),
            (TagType::Warning, s.warning.as_str()),
            (TagType::Danger, s.danger.as_str()),
        ];
        let typed = |prefix: &'static str, effect, round| {
            row().children(
                types
                    .into_iter()
                    .enumerate()
                    .map(move |(i, (kind, label))| {
                        el_tag()
                            .id((prefix, i as u32))
                            .label(label)
                            .kind(kind)
                            .when(effect != TagEffect::Light, |tag| tag.effect(effect))
                            .when(round, |tag| tag.round(true))
                    }),
            )
        };
        let basic = typed("tag-basic", TagEffect::Light, false)
            .child(
                el_tag()
                    .id("tag-custom-color")
                    .label(s.primary.as_str())
                    .color(rgb(0x626aef)),
            )
            .child(el_tag().id("tag-hit").label(s.tag_hit.as_str()).hit(true));
        let removable = row()
            .children(types.into_iter().enumerate().map(|(i, (kind, label))| {
                let index = i as u32;
                if self.tag.closed.contains(&index) {
                    return div().into_any_element();
                }
                el_tag()
                    .id(("tag-removable", index))
                    .label(label)
                    .kind(kind)
                    .closable(true)
                    .onclose(cx.listener(move |this, _, _, cx| {
                        this.tag.closed.insert(index);
                        cx.notify();
                    }))
                    .into_any_element()
            }))
            .child(
                el_button()
                    .id("tag-reset")
                    .small()
                    .label(s.tag_reset.as_str())
                    .onclick(cx.listener(|this, _, _, cx| {
                        this.tag.closed.clear();
                        cx.notify();
                    })),
            )
            .child(
                s.tag_removed
                    .replace("%{count}", &self.tag.closed.len().to_string()),
            );
        let editable = row()
            .children(self.tag.dynamic.iter().map(|tag| {
                let id = tag.id;
                el_tag()
                    .id(("tag-dynamic", id))
                    .label(tag.label.clone())
                    .closable(true)
                    .onclose(cx.listener(move |this, _, _, cx| {
                        this.tag.close_dynamic(id);
                        cx.notify();
                    }))
            }))
            .child(
                el_button()
                    .id("tag-new")
                    .small()
                    .label(s.tag_add.as_str())
                    .icon(el_icon("plus"))
                    .onclick(cx.listener(move |this, _, _, cx| {
                        let label =
                            format!("{} {}", s.tag_label, this.tag.dynamic.len() + 1).into();
                        this.tag.add_dynamic(label);
                        cx.notify();
                    })),
            )
            .child(s.tag_editable_help.as_str());
        let sizes = row()
            .child(el_tag().id("tag-large").label(s.large.as_str()).large())
            .child(el_tag().id("tag-default-size").label(s.default.as_str()))
            .child(el_tag().id("tag-small").label(s.compact.as_str()).small());
        let effects = [
            (s.tag_effect_dark.as_str(), TagEffect::Dark),
            (s.tag_effect_light.as_str(), TagEffect::Light),
            (s.tag_effect_plain.as_str(), TagEffect::Plain),
        ];
        let effect_rows = |id, round| {
            div()
                .flex()
                .flex_col()
                .gap(px(spacing::DEFAULT))
                .children(effects.map(|(label, effect)| {
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(spacing::SMALL))
                        .child(label)
                        .child(typed(id, effect, round))
                }))
        };
        let checked_count =
            self.tag.toggle as usize + self.tag.checked.iter().filter(|&&checked| checked).count();
        let view = cx.entity();
        let checkable = div()
            .flex()
            .flex_col()
            .gap(px(spacing::DEFAULT))
            .child(
                row()
                    .child(
                        el_check_tag()
                            .id("tag-always-checked")
                            .label(s.tag_checked.as_str())
                            .checked(true),
                    )
                    .child(
                        el_check_tag()
                            .id("tag-toggle")
                            .label(s.tag_toggle.as_str())
                            .checked(self.tag.toggle)
                            .onchange(move |checked, _, cx| {
                                view.update(cx, |this, cx| {
                                    this.tag.toggle = checked;
                                    cx.notify();
                                })
                            }),
                    )
                    .child(
                        el_check_tag()
                            .id("tag-check-disabled")
                            .label(s.disabled.as_str())
                            .disabled(true),
                    ),
            )
            .child(
                row().children(self.tag.checked.iter().enumerate().map(|(i, &checked)| {
                    let kind = types.get(i).map_or(TagType::Danger, |(kind, _)| *kind);
                    let view = cx.entity();
                    el_check_tag()
                        .id(("tag-checked", i as u32))
                        .label(format!("{} {}", s.tag_label, i + 1))
                        .kind(kind)
                        .checked(checked)
                        .when(i == 5, |tag| tag.disabled(true))
                        .onchange(move |checked, _, cx| {
                            view.update(cx, |this, cx| {
                                if let Some(value) = this.tag.checked.get_mut(i) {
                                    *value = checked;
                                    cx.notify();
                                }
                            })
                        })
                })),
            )
            .child(
                s.tag_checked_count
                    .replace("%{count}", &checked_count.to_string()),
            );
        let snippet = || source(include_str!("tag.rs"));
        div()
            .flex()
            .flex_col()
            .gap(px(24.))
            .child(self.section(cx, "tag-basic", &s.tag_basic, snippet, basic))
            .child(self.section(cx, "tag-removable", &s.tag_removable, snippet, removable))
            .child(self.section(cx, "tag-editable", &s.tag_editable, snippet, editable))
            .child(self.section(cx, "tag-sizes", &s.tag_sizes, snippet, sizes))
            .child(self.section(
                cx,
                "tag-theme",
                &s.tag_theme,
                snippet,
                effect_rows("tag-theme", false),
            ))
            .child(self.section(
                cx,
                "tag-rounded",
                &s.tag_rounded,
                snippet,
                effect_rows("tag-rounded", true),
            ))
            .child(self.section(cx, "tag-checkable", &s.tag_checkable, snippet, checkable))
    }
}
