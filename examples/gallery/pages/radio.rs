use gpui::{Context, Div, SharedString, div, prelude::*, px, rgb};
use gpui_el::{RadioOption, RadioSize, el_button, el_icon, el_radio, el_radio_group};

use crate::{Gallery, copy::Copy};

pub(crate) struct State {
    basic: usize,
    disabled: usize,
    sized: usize,
    bordered: usize,
    custom: usize,
    colors: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            basic: 0,
            disabled: 0,
            sized: 0,
            bordered: 1,
            custom: 0,
            colors: 0,
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
        // The group owns the controlled value and reports the next value.
        let make_group = |button: bool| {
            let view = view.clone();
            el_radio_group()
                .vertical()
                .gap(12.)
                .button(button)
                .value(SharedString::from(self.radio.basic.to_string()))
                .options(
                    options
                        .into_iter()
                        .enumerate()
                        .map(|(index, label)| RadioOption::new(index.to_string(), label)),
                )
                .onchange(move |value, _, cx| {
                    if let Ok(index) = value.parse::<usize>() {
                        view.update(cx, |this, cx| {
                            this.radio.basic = index;
                            cx.notify();
                        });
                    }
                })
        };
        let group = div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(make_group(false))
            .child(make_group(true));
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
        let colors = el_radio_group()
            .vertical()
            .gap(12.)
            .active_color(rgb(0x13ce66))
            .inactive_color(rgb(0xff4949))
            .radios(options.into_iter().enumerate().map(|(index, label)| {
                let view = view.clone();
                el_radio()
                    .id(("radio-color", index))
                    .label(label)
                    .checked(self.radio.colors == index)
                    .onchange(move |_, _, cx| {
                        view.update(cx, |this, cx| {
                            this.radio.colors = index;
                            cx.notify();
                        })
                    })
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
                r#"// The group owns the controlled value and reports the next value.
el_radio_group().vertical().gap(12.)
    .value(self.radio.basic.to_string())
    .options([
        RadioOption::new("0", "Prepare"),
        RadioOption::new("1", "Ship"),
        RadioOption::new("2", "Pickup"),
    ])
    .onchange(move |value, _, cx| {
        view.update(cx, |this, cx| {
            this.radio.basic = value.parse().unwrap_or(0);
            cx.notify();
        });
    });

// RadioButton uses the same controlled group API.
el_radio_group().button(true).options(options);"#,
                group.into_any_element(),
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
            .child(section(
                "radio-colors",
                &s.radio_colors,
                r#"el_radio_group().active_color(rgb(0x13ce66))
    .inactive_color(rgb(0xff4949))
    .radios(options.enumerate().map(|(index, label)| {
        el_radio().id(("radio-color", index)).label(label)
            .checked(self.radio.colors == index)
    }));"#,
                colors.into_any_element(),
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
