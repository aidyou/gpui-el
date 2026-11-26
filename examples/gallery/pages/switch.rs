use gpui::{Context, Div, div, prelude::*, px, rgb};
use gpui_el::{SwitchSize, el_button, el_icon, el_switch};

use crate::{Gallery, copy::Copy, demo::row};

pub(crate) struct State {
    basic: bool,
    sizes: [bool; 3],
    text: bool,
    inline: bool,
    icons: bool,
    colors: [bool; 2],
    guarded: bool,
    /// Blocks the guarded switch while on, like an unresolved
    /// `before-change` promise upstream.
    locked: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            basic: true,
            sizes: [true, true, false],
            text: false,
            inline: true,
            icons: false,
            colors: [true, false],
            guarded: false,
            locked: true,
        }
    }
}

impl Gallery {
    pub(crate) fn switches(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let basic = row().child({
            let view = cx.entity();
            el_switch()
                .id("switch-basic")
                .checked(self.switch.basic)
                .onchange(move |checked, _, cx| {
                    view.update(cx, |this, cx| {
                        this.switch.basic = checked;
                        cx.notify();
                    })
                })
        });
        let sizes = row().children(self.switch.sizes.iter().enumerate().map(|(i, &checked)| {
            let view = cx.entity();
            let size = [SwitchSize::Large, SwitchSize::Default, SwitchSize::Small]
                .get(i)
                .copied()
                .unwrap_or_default();
            el_switch()
                .id(("switch-size", i as u32))
                .size(size)
                .checked(checked)
                .onchange(move |checked, _, cx| {
                    view.update(cx, |this, cx| {
                        if let Some(slot) = this.switch.sizes.get_mut(i) {
                            *slot = checked;
                            cx.notify();
                        }
                    })
                })
        }));
        let text = row()
            .child({
                let view = cx.entity();
                el_switch()
                    .id("switch-text")
                    .active_text(s.switch_on.as_str())
                    .inactive_text(s.switch_off.as_str())
                    .checked(self.switch.text)
                    .onchange(move |checked, _, cx| {
                        view.update(cx, |this, cx| {
                            this.switch.text = checked;
                            cx.notify();
                        })
                    })
            })
            .child({
                let view = cx.entity();
                el_switch()
                    .id("switch-inline")
                    .inline_prompt(true)
                    .active_text(s.switch_on.as_str())
                    .inactive_text(s.switch_off.as_str())
                    .width(56.)
                    .checked(self.switch.inline)
                    .onchange(move |checked, _, cx| {
                        view.update(cx, |this, cx| {
                            this.switch.inline = checked;
                            cx.notify();
                        })
                    })
            });
        let icons = row()
            .child({
                let view = cx.entity();
                el_switch()
                    .id("switch-icons")
                    .active_icon(el_icon("check"))
                    .inactive_icon(el_icon("close"))
                    .checked(self.switch.icons)
                    .onchange(move |checked, _, cx| {
                        view.update(cx, |this, cx| {
                            this.switch.icons = checked;
                            cx.notify();
                        })
                    })
            })
            .child({
                let view = cx.entity();
                el_switch()
                    .id("switch-action-icons")
                    .active_action_icon(el_icon("check").size(10.))
                    .inactive_action_icon(el_icon("close").size(10.))
                    .checked(self.switch.icons)
                    .onchange(move |checked, _, cx| {
                        view.update(cx, |this, cx| {
                            this.switch.icons = checked;
                            cx.notify();
                        })
                    })
            });
        let colors = row().children(self.switch.colors.iter().enumerate().map(|(i, &checked)| {
            let view = cx.entity();
            el_switch()
                .id(("switch-color", i as u32))
                .when(i == 0, |el| {
                    el.active_color(rgb(0x13ce66)).inactive_color(rgb(0xff4949))
                })
                .checked(checked)
                .onchange(move |checked, _, cx| {
                    view.update(cx, |this, cx| {
                        if let Some(slot) = this.switch.colors.get_mut(i) {
                            *slot = checked;
                            cx.notify();
                        }
                    })
                })
        }));
        let states = row()
            .child(
                el_switch()
                    .id("switch-disabled")
                    .checked(true)
                    .disabled(true),
            )
            .child(el_switch().id("switch-loading").checked(true).loading(true));
        // The guard is consulted on every activation; while locked the switch
        // keeps its state, like upstream's `before-change` rejecting.
        let locked = self.switch.locked;
        let guarded = row()
            .child({
                let view = cx.entity();
                el_switch()
                    .id("switch-guarded")
                    .checked(self.switch.guarded)
                    .before_change(move || !locked)
                    .onchange(move |checked, _, cx| {
                        view.update(cx, |this, cx| {
                            this.switch.guarded = checked;
                            cx.notify();
                        })
                    })
            })
            .child(
                el_button()
                    .id("switch-lock")
                    .small()
                    .label(if locked {
                        s.switch_unlock.as_str()
                    } else {
                        s.switch_lock.as_str()
                    })
                    .onclick(cx.listener(|this, _, _, cx| {
                        this.switch.locked = !this.switch.locked;
                        cx.notify();
                    })),
            )
            .child(s.switch_guard_help.as_str());
        let section =
            |id, title, code: &str, controls| self.section(cx, id, title, || code.into(), controls);
        div()
            .flex()
            .flex_col()
            .gap(px(24.))
            .child(section(
                "switch-basic",
                &s.switch_basic,
                r#"let view = cx.entity();
el_switch().id("switch-basic").checked(self.switch.basic)
    .onchange(move |checked, _, cx| {
        view.update(cx, |this, cx| {
            this.switch.basic = checked;
            cx.notify();
        });
    });"#,
                basic.into_any_element(),
            ))
            .child(section(
                "switch-sizes",
                &s.switch_sizes,
                r#"el_switch().large().checked(size_large);
el_switch().checked(size_default);
el_switch().small().checked(size_small);"#,
                sizes.into_any_element(),
            ))
            .child(section(
                "switch-text",
                &s.switch_text,
                r#"el_switch().active_text("On").inactive_text("Off").checked(on);

// inline_prompt moves the text into the core; width makes room for it.
el_switch().inline_prompt(true).active_text("On").inactive_text("Off")
    .width(56.).checked(on);"#,
                text.into_any_element(),
            ))
            .child(section(
                "switch-icons",
                &s.switch_icons,
                r#"el_switch().active_icon(el_icon("check")).inactive_icon(el_icon("close"))
    .checked(on);

// Action icons sit inside the knob.
el_switch().active_action_icon(el_icon("check").size(10.))
    .inactive_action_icon(el_icon("close").size(10.)).checked(on);"#,
                icons.into_any_element(),
            ))
            .child(section(
                "switch-colors",
                &s.switch_colors,
                r#"el_switch().active_color(rgb(0x13ce66)).inactive_color(rgb(0xff4949))
    .checked(on);"#,
                colors.into_any_element(),
            ))
            .child(section(
                "switch-states",
                &s.switch_disabled,
                r#"el_switch().checked(true).disabled(true);
el_switch().checked(true).loading(true);"#,
                states.into_any_element(),
            ))
            .child(section(
                "switch-guard",
                &s.switch_guard,
                r#"let locked = self.switch.locked;
el_switch().checked(self.switch.guarded)
    .before_change(move || !locked)
    .onchange(move |checked, _, cx| {
        view.update(cx, |this, cx| {
            this.switch.guarded = checked;
            cx.notify();
        });
    });"#,
                guarded.into_any_element(),
            ))
    }
}
