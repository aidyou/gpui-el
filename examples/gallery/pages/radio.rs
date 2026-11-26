use gpui::{Context, Div, div, prelude::*, px};
use gpui_el::{RadioSize, el_button, el_icon, el_radio, el_radio_group};

use crate::{Gallery, copy::Copy};

pub(crate) struct State {
    basic: usize,
    disabled: usize,
    sized: usize,
    bordered: usize,
    custom: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            basic: 0,
            disabled: 0,
            sized: 0,
            bordered: 1,
            custom: 0,
        }
    }
}

impl Gallery {
    pub(crate) fn radios(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let view = cx.entity();
        let options = [
            s.radio_prepare.as_str(),
            s.radio_ship.as_str(),
            s.radio_pickup.as_str(),
        ];
        // A whole group shares one selection, so the builder captures the
        // chosen index and each radio reports its own activation.
        let group = |selection: usize| {
            let view = view.clone();
            el_radio_group()
                .vertical()
                .gap(12.)
                .radios(options.into_iter().enumerate().map(|(index, label)| {
                    let view = view.clone();
                    el_radio()
                        .id(("radio", index))
                        .label(label)
                        .checked(selection == index)
                        .onchange(move |_, _, cx| {
                            view.update(cx, |this, cx| {
                                this.radio.basic = index;
                                cx.notify();
                            })
                        })
                }))
        };
        let disabled =
            el_radio_group()
                .vertical()
                .gap(12.)
                .radios(options.into_iter().enumerate().map(|(index, label)| {
                    el_radio()
                        .id(("radio-disabled", index))
                        .label(label)
                        .checked(self.radio.disabled == index)
                        .disabled(index == 1)
                }));
        let sizes = el_radio_group().radios(
            [RadioSize::Large, RadioSize::Default, RadioSize::Small]
                .into_iter()
                .enumerate()
                .map(|(index, size)| {
                    let view = view.clone();
                    el_radio()
                        .id(("radio-size", index))
                        .label(s.checkbox_option.as_str())
                        .size(size)
                        .checked(self.radio.sized == index)
                        .onchange(move |_, _, cx| {
                            view.update(cx, |this, cx| {
                                this.radio.sized = index;
                                cx.notify();
                            })
                        })
                }),
        );
        let bordered = el_radio_group()
            .border(true)
            .radios(options.into_iter().enumerate().map(|(index, label)| {
                let view = view.clone();
                el_radio()
                    .id(("radio-bordered", index))
                    .label(label)
                    .checked(self.radio.bordered == index)
                    .onchange(move |_, _, cx| {
                        view.update(cx, |this, cx| {
                            this.radio.bordered = index;
                            cx.notify();
                        })
                    })
            }));
        let custom =
            el_radio_group().radios(options.into_iter().enumerate().map(|(index, label)| {
                let view = view.clone();
                el_radio()
                    .id(("radio-custom", index))
                    .checked(self.radio.custom == index)
                    .onchange(move |_, _, cx| {
                        view.update(cx, |this, cx| {
                            this.radio.custom = index;
                            cx.notify();
                        })
                    })
                    .content(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .child(el_icon("box"))
                            .child(label),
                    )
            }));
        let section =
            |id, title, code: &str, controls| self.section(cx, id, title, || code.into(), controls);
        div()
            .flex()
            .flex_col()
            .gap(px(24.))
            .child(section(
                "radio-basic",
                &s.radio_basic,
                r#"// One selection per group: the application stores the index and each
// radio reports its own activation.
el_radio_group().vertical().gap(12.).radios(
    options.enumerate().map(|(index, label)| {
        let view = cx.entity();
        el_radio().id(("radio", index)).label(label)
            .checked(self.radio.basic == index)
            .onchange(move |_, _, cx| {
                view.update(cx, |this, cx| {
                    this.radio.basic = index;
                    cx.notify();
                });
            })
    })
);"#,
                group(self.radio.basic).into_any_element(),
            ))
            .child(section(
                "radio-disabled",
                &s.radio_disabled,
                r#"el_radio().label("Prepare").checked(selected == 0);
el_radio().label("Ship").disabled(true);
el_radio().label("Pickup");"#,
                disabled.into_any_element(),
            ))
            .child(section(
                "radio-sizes",
                &s.radio_sizes,
                r#"el_radio_group().radios([RadioSize::Large, RadioSize::Default, RadioSize::Small]
    .enumerate().map(|(index, size)| {
        let view = cx.entity();
        el_radio().id(("radio-size", index)).label("Option").size(size)
            .checked(self.radio.sized == index)
            .onchange(move |_, _, cx| {
                view.update(cx, |this, cx| {
                    this.radio.sized = index;
                    cx.notify();
                });
            })
    }));"#,
                sizes.into_any_element(),
            ))
            .child(section(
                "radio-bordered",
                &s.radio_bordered,
                r#"el_radio_group().border(true).radios(options.enumerate().map(|(index, label)| {
    let view = cx.entity();
    el_radio().id(("radio-bordered", index)).label(label)
        .checked(self.radio.bordered == index)
        .onchange(move |_, _, cx| {
            view.update(cx, |this, cx| {
                this.radio.bordered = index;
                cx.notify();
            });
        })
}));"#,
                bordered.into_any_element(),
            ))
            .child(section(
                "radio-custom",
                &s.radio_custom,
                r#"el_radio().content(
    div().flex().items_center().gap(6.)
        .child(el_icon("box"))
        .child("Ship"),
);"#,
                custom.into_any_element(),
            ))
            .child(
                div().flex().items_center().gap(px(12.)).child(
                    el_button()
                        .id("radio-reset")
                        .small()
                        .label(s.tag_reset.as_str())
                        .onclick(cx.listener(|this, _, _, cx| {
                            this.radio = Default::default();
                            cx.notify();
                        })),
                ),
            )
    }
}
