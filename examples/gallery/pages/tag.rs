use std::collections::HashSet;

use gpui::{Context, Div, SharedString, div, prelude::*, px, rgb};
use gpui_el::{TagEffect, TagType, el_button, el_check_tag, el_icon, el_tag, tokens::spacing};

use crate::{Gallery, copy::Copy, demo::row};

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
            .child(el_tag().id("tag-small").label(s.tag_small.as_str()).small());
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
        // Upstream's sixth check tag is the disabled one and uses success.
        let kinds = [
            TagType::Primary,
            TagType::Success,
            TagType::Info,
            TagType::Warning,
            TagType::Danger,
            TagType::Success,
        ];
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
                    let kind = kinds[i];
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
        let section =
            |id, title, code: &str, controls| self.section(cx, id, title, || code.into(), controls);
        div()
            .flex()
            .flex_col()
            .gap(px(24.))
            .child(section(
                "tag-basic",
                &s.tag_basic,
                r#"el_tag().primary().label("Primary");
el_tag().success().label("Success");
el_tag().info().label("Info");
el_tag().warning().label("Warning");
el_tag().danger().label("Danger");
el_tag().color(rgb(0x626aef)).label("Primary");
el_tag().hit(true).label("Highlighted border");"#,
                basic.into_any_element(),
            ))
            .child(section(
                "tag-removable",
                &s.tag_removable,
                r#"let tag = |i: u32, kind: TagType, label: &str| {
    el_tag().id(("tag-removable", i)).kind(kind).label(label).closable(true)
        .onclose(cx.listener(move |this, _, _, cx| {
            this.tag.closed.insert(i);
            cx.notify();
        }))
};

el_space()
    .child(tag(0, TagType::Primary, "Primary"))
    .child(tag(1, TagType::Success, "Success"))
    .child(tag(2, TagType::Info, "Info"))
    .child(tag(3, TagType::Warning, "Warning"))
    .child(tag(4, TagType::Danger, "Danger"))
    .child(el_button().id("tag-reset").small().label("Reset")
        .onclick(cx.listener(|this, _, _, cx| {
            this.tag.closed.clear();
            cx.notify();
        })));"#,
                removable.into_any_element(),
            ))
            .child(section(
                "tag-editable",
                &s.tag_editable,
                r#"// Each entry keeps a stable id; labels may repeat after deletions.
el_space()
    .children(self.tag.dynamic.iter().map(|tag| {
        let id = tag.id;
        el_tag().id(("tag-dynamic", id)).label(tag.label.clone()).closable(true)
            .onclose(cx.listener(move |this, _, _, cx| {
                this.tag.close_dynamic(id);
                cx.notify();
            }))
    }))
    .child(el_button().id("tag-new").small().label("New tag")
        .icon(el_icon("plus"))
        .onclick(cx.listener(|this, _, _, cx| {
            let label = format!("Tag {}", this.tag.dynamic.len() + 1).into();
            this.tag.add_dynamic(label);
            cx.notify();
        })));"#,
                editable.into_any_element(),
            ))
            .child(section(
                "tag-sizes",
                &s.tag_sizes,
                r#"el_tag().large().label("Large");
el_tag().label("Default");
el_tag().small().label("Small");"#,
                sizes.into_any_element(),
            ))
            .child(section(
                "tag-theme",
                &s.tag_theme,
                r#"let types = [
    (TagType::Primary, "Primary"),
    (TagType::Success, "Success"),
    (TagType::Info, "Info"),
    (TagType::Warning, "Warning"),
    (TagType::Danger, "Danger"),
];
let row = |effect| el_space().children(types.map(|(kind, label)| {
    el_tag().kind(kind).effect(effect).label(label)
}));

row(TagEffect::Dark);
row(TagEffect::Light); // The default.
row(TagEffect::Plain);"#,
                effect_rows("tag-theme", false).into_any_element(),
            ))
            .child(section(
                "tag-rounded",
                &s.tag_rounded,
                r#"let types = [
    (TagType::Primary, "Primary"),
    (TagType::Success, "Success"),
    (TagType::Info, "Info"),
    (TagType::Warning, "Warning"),
    (TagType::Danger, "Danger"),
];
let row = |effect| el_space().children(types.map(|(kind, label)| {
    el_tag().kind(kind).effect(effect).round(true).label(label)
}));

row(TagEffect::Dark);
row(TagEffect::Light);
row(TagEffect::Plain);"#,
                effect_rows("tag-rounded", true).into_any_element(),
            ))
            .child(section(
                "tag-checkable",
                &s.tag_checkable,
                r#"el_check_tag().label("Checked").checked(true);

let view = cx.entity();
el_check_tag().label("Toggle me").checked(self.tag.toggle)
    .onchange(move |checked, _, cx| {
        view.update(cx, |this, cx| {
            this.tag.toggle = checked;
            cx.notify();
        });
    });

el_check_tag().label("Disabled").disabled(true);

let kinds = [
    TagType::Primary,
    TagType::Success,
    TagType::Info,
    TagType::Warning,
    TagType::Danger,
    TagType::Success, // Disabled, like upstream's sixth check tag.
];

el_space().children((0..6).map(|i| {
    let view = cx.entity();
    el_check_tag().id(("tag-checked", i)).label(format!("Tag {}", i + 1))
        .kind(kinds[i]).checked(self.tag.checked[i])
        .when(i == 5, |tag| tag.disabled(true))
        .onchange(move |checked, _, cx| {
            view.update(cx, |this, cx| {
                this.tag.checked[i] = checked;
                cx.notify();
            });
        });
}));"#,
                checkable.into_any_element(),
            ))
    }
}
