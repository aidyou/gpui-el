use std::collections::HashSet;

use gpui::{Context, Div, SharedString, div, prelude::*, px, rgb};
use gpui_el::{CheckboxOption, CheckboxSize, el_button, el_checkbox, el_checkbox_group, el_icon};

use crate::{Gallery, copy::Copy, demo::row};

pub(crate) struct State {
    basic: bool,
    sized: [bool; 3],
    bordered: [bool; 3],
    /// The options the indeterminate parent controls.
    children: [bool; 3],
    group: HashSet<String>,
    custom: bool,
    colors: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            basic: true,
            sized: [true, true, false],
            bordered: [true, false, true],
            children: [true, true, false],
            group: HashSet::from(["A".to_string(), "B".to_string()]),
            custom: true,
            colors: true,
        }
    }
}

impl State {
    /// The parent box is checked when every child is, unchecked when none is,
    /// and indeterminate in between, like upstream's check-all demo.
    fn parent(&self) -> (bool, bool) {
        let count = self.children.iter().filter(|&&child| child).count();
        match count {
            0 => (false, false),
            3 => (true, false),
            _ => (false, true),
        }
    }
}

impl Gallery {
    pub(crate) fn checkboxes(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let view = cx.entity();
        let basic = row().child({
            let view = view.clone();
            el_checkbox()
                .id("checkbox-basic")
                .label(s.checkbox_option.as_str())
                .checked(self.checkbox.basic)
                .onchange(move |checked, _, cx| {
                    view.update(cx, |this, cx| {
                        this.checkbox.basic = checked;
                        cx.notify();
                    })
                })
        });
        let disabled = row()
            .child(
                el_checkbox()
                    .id("checkbox-disabled-on")
                    .label(s.checkbox_option.as_str())
                    .checked(true)
                    .disabled(true),
            )
            .child(
                el_checkbox()
                    .id("checkbox-disabled-off")
                    .label(s.checkbox_option.as_str())
                    .disabled(true),
            );
        let (parent_checked, indeterminate) = self.checkbox.parent();
        let parent = row()
            .child({
                let view = view.clone();
                el_checkbox()
                    .id("checkbox-parent")
                    .label(s.checkbox_all.as_str())
                    .checked(parent_checked)
                    .indeterminate(indeterminate)
                    .onchange(move |checked, _, cx| {
                        view.update(cx, |this, cx| {
                            this.checkbox.children.fill(checked);
                            cx.notify();
                        })
                    })
            })
            .children(
                self.checkbox
                    .children
                    .iter()
                    .enumerate()
                    .map(|(i, &child)| {
                        let view = view.clone();
                        el_checkbox()
                            .id(("checkbox-child", i as u32))
                            .label(format!("{} {}", s.checkbox_option, i + 1))
                            .checked(child)
                            .onchange(move |checked, _, cx| {
                                view.update(cx, |this, cx| {
                                    if let Some(slot) = this.checkbox.children.get_mut(i) {
                                        *slot = checked;
                                        cx.notify();
                                    }
                                })
                            })
                    }),
            );
        let sizes = row().children(self.checkbox.sized.iter().enumerate().map(|(i, &checked)| {
            let view = view.clone();
            let size = [
                CheckboxSize::Large,
                CheckboxSize::Default,
                CheckboxSize::Small,
            ]
            .get(i)
            .copied()
            .unwrap_or_default();
            el_checkbox()
                .id(("checkbox-size", i as u32))
                .label(s.checkbox_option.as_str())
                .size(size)
                .checked(checked)
                .onchange(move |checked, _, cx| {
                    view.update(cx, |this, cx| {
                        if let Some(slot) = this.checkbox.sized.get_mut(i) {
                            *slot = checked;
                            cx.notify();
                        }
                    })
                })
        }));
        let bordered = row().children(self.checkbox.bordered.iter().enumerate().map(
            |(i, &checked)| {
                let view = view.clone();
                el_checkbox()
                    .id(("checkbox-bordered", i as u32))
                    .label(s.checkbox_option.as_str())
                    .border(true)
                    .checked(checked)
                    .onchange(move |checked, _, cx| {
                        view.update(cx, |this, cx| {
                            if let Some(slot) = this.checkbox.bordered.get_mut(i) {
                                *slot = checked;
                                cx.notify();
                            }
                        })
                    })
            },
        ));
        let values: [&str; 3] = ["A", "B", "C"];
        let make_group = |button: bool| {
            let view = view.clone();
            el_checkbox_group()
                .vertical()
                .gap(12.)
                .button(button)
                .values(
                    self.checkbox
                        .group
                        .iter()
                        .map(|value| SharedString::from(value.clone())),
                )
                .options(values.into_iter().map(|value| {
                    CheckboxOption::new(value, format!("{} {}", s.checkbox_option, value))
                }))
                .onchange(move |next, _, cx| {
                    view.update(cx, |this, cx| {
                        this.checkbox.group =
                            next.into_iter().map(|value| value.to_string()).collect();
                        cx.notify();
                    })
                })
        };
        let group = div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(make_group(false))
            .child(make_group(true));
        let custom = row().child({
            let view = view.clone();
            el_checkbox()
                .id("checkbox-custom")
                .checked(self.checkbox.custom)
                .onchange(move |checked, _, cx| {
                    view.update(cx, |this, cx| {
                        this.checkbox.custom = checked;
                        cx.notify();
                    })
                })
                .content(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .child(el_icon("picture"))
                        .child(s.checkbox_custom.as_str()),
                )
        });
        let colors = row().child({
            let view = view.clone();
            el_checkbox()
                .id("checkbox-colors")
                .label(s.checkbox_option.as_str())
                .checked(self.checkbox.colors)
                .active_color(rgb(0x13ce66))
                .inactive_color(rgb(0xff4949))
                .onchange(move |checked, _, cx| {
                    view.update(cx, |this, cx| {
                        this.checkbox.colors = checked;
                        cx.notify();
                    })
                })
        });
        let section =
            |id, title, code: &str, controls| self.section(cx, id, title, || code.into(), controls);
        div()
            .flex()
            .flex_col()
            .gap(px(24.))
            .child(section(
                "checkbox-basic",
                &s.checkbox_basic,
                r#"let view = cx.entity();
el_checkbox().id("checkbox-basic").label("Option")
    .checked(self.checkbox.basic)
    .onchange(move |checked, _, cx| {
        view.update(cx, |this, cx| {
            this.checkbox.basic = checked;
            cx.notify();
        });
    });"#,
                basic.into_any_element(),
            ))
            .child(section(
                "checkbox-disabled",
                &s.checkbox_disabled,
                r#"el_checkbox().label("Option").checked(true).disabled(true);
el_checkbox().label("Option").disabled(true);"#,
                disabled.into_any_element(),
            ))
            .child(section(
                "checkbox-indeterminate",
                &s.checkbox_indeterminate,
                r#"// The parent is checked when all children are, unchecked when none
// are, and indeterminate in between.
el_checkbox().label("All").checked(all).indeterminate(some)
    .onchange(move |checked, _, cx| {
        view.update(cx, |this, cx| {
            this.checkbox.children.fill(checked);
            cx.notify();
        });
    });"#,
                parent.into_any_element(),
            ))
            .child(section(
                "checkbox-sizes",
                &s.checkbox_sizes,
                r#"el_checkbox().large().label("Option");
el_checkbox().label("Option");
el_checkbox().small().label("Option");"#,
                sizes.into_any_element(),
            ))
            .child(section(
                "checkbox-bordered",
                &s.checkbox_bordered,
                r#"el_checkbox().border(true).label("Option");"#,
                bordered.into_any_element(),
            ))
            .child(section(
                "checkbox-group",
                &s.checkbox_group,
                r#"// The group owns the controlled value array and reports the next array.
el_checkbox_group().vertical().gap(12.)
    .values(self.checkbox.group.iter().map(SharedString::from))
    .options([
        CheckboxOption::new("A", "Option A"),
        CheckboxOption::new("B", "Option B"),
        CheckboxOption::new("C", "Option C"),
    ])
    .onchange(move |next, _, cx| {
        view.update(cx, |this, cx| {
            this.checkbox.group = next.into_iter().map(|v| v.to_string()).collect();
            cx.notify();
        });
    });

// CheckboxButton uses the same controlled group API.
el_checkbox_group().button(true).options(options);"#,
                group.into_any_element(),
            ))
            .child(section(
                "checkbox-content",
                &s.checkbox_content,
                r#"el_checkbox().content(
    div().flex().items_center().gap(6.)
        .child(el_icon("picture"))
        .child("Custom content"),
);"#,
                custom.into_any_element(),
            ))
            .child(section(
                "checkbox-colors",
                &s.checkbox_colors,
                r#"el_checkbox().label("Option").checked(on)
    .active_color(rgb(0x13ce66)).inactive_color(rgb(0xff4949));"#,
                colors.into_any_element(),
            ))
            .child(
                div().flex().items_center().gap(px(12.)).child(
                    el_button()
                        .id("checkbox-reset")
                        .small()
                        .label(s.tag_reset.as_str())
                        .onclick(cx.listener(|this, _, _, cx| {
                            this.checkbox = Default::default();
                            cx.notify();
                        })),
                ),
            )
    }
}
