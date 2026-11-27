use gpui::{Context, Div, div, prelude::*, px};
use gpui_el::{SliderMark, el_button, el_icon, el_slider};

use crate::{Gallery, copy::Copy, demo::row};

pub(crate) struct State {
    basic: f32,
    sizes: f32,
    steps: f32,
    range: (f32, f32),
    vertical: f32,
    marks: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            basic: 30.,
            sizes: 60.,
            steps: 20.,
            range: (20., 60.),
            vertical: 40.,
            marks: 30.,
        }
    }
}

impl Gallery {
    pub(crate) fn sliders(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let view = cx.entity();
        // One slot per demo, so a single helper wires every slider back to its
        // own state.
        let slider = |slot: usize, id: gpui::ElementId, value: f32| {
            let view = view.clone();
            el_slider()
                .id(id)
                .value(value)
                .oninput(move |reported, _, cx| {
                    let gpui_el::SliderValue::Single(reported) = reported else {
                        return;
                    };
                    view.update(cx, |this, cx| {
                        let target = match slot {
                            0 => &mut this.slider.basic,
                            1 => &mut this.slider.sizes,
                            2 => &mut this.slider.steps,
                            3 => &mut this.slider.vertical,
                            _ => &mut this.slider.marks,
                        };
                        *target = reported;
                        cx.notify();
                    })
                })
        };
        let basic = row().child(slider(0, "slider-basic".into(), self.slider.basic));
        let sizes = row().children((0..3u32).map(|index| {
            slider(1, ("slider-sized", index).into(), self.slider.sizes)
                .when(index == 0, |el| el.large())
                .when(index == 2, |el| el.small())
        }));
        let steps = row().child(
            slider(2, "slider-steps".into(), self.slider.steps)
                .step(10.)
                .show_stops(true),
        );
        let range_view = view.clone();
        let range = row().child(
            el_slider()
                .id("slider-range")
                .range(self.slider.range.0, self.slider.range.1)
                .oninput(move |reported, _, cx| {
                    let gpui_el::SliderValue::Range(low, high) = reported else {
                        return;
                    };
                    range_view.update(cx, |this, cx| {
                        this.slider.range = (low, high);
                        cx.notify();
                    })
                }),
        );
        let vertical = row().child(
            slider(3, "slider-vertical".into(), self.slider.vertical)
                .vertical(true)
                .height(180.),
        );
        let marks = row().child(slider(4, "slider-marks".into(), self.slider.marks).marks([
            SliderMark {
                point: 0.,
                label: s.slider_mark_a.as_str().into(),
            },
            SliderMark {
                point: 25.,
                label: s.slider_mark_b.as_str().into(),
            },
            SliderMark {
                point: 50.,
                label: s.slider_mark_c.as_str().into(),
            },
            SliderMark {
                point: 75.,
                label: s.slider_mark_d.as_str().into(),
            },
            SliderMark {
                point: 100.,
                label: s.slider_mark_e.as_str().into(),
            },
        ]));
        let formatted = row().child(
            el_slider()
                .id("slider-formatted")
                .value(40.)
                .format_tooltip(|value| format!("{value:.0}%").into()),
        );
        let readonly = row().child(el_slider().id("slider-readonly").value(70.).disabled(true));

        let reset = s.tag_reset.clone();
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "slider-basic",
                &s.slider_basic,
                || "el_slider().value(30.);".into(),
                basic.into_any_element(),
            ))
            .child(self.section(
                cx,
                "slider-sizes",
                &s.slider_sizes,
                || "el_slider().large();\nel_slider();\nel_slider().small();".into(),
                sizes.into_any_element(),
            ))
            .child(self.section(
                cx,
                "slider-steps",
                &s.slider_steps,
                || "el_slider().step(10.).show_stops(true);".into(),
                steps.into_any_element(),
            ))
            .child(self.section(
                cx,
                "slider-range",
                &s.slider_range,
                || "el_slider().range(20., 60.);".into(),
                range.into_any_element(),
            ))
            .child(self.section(
                cx,
                "slider-vertical",
                &s.slider_vertical,
                || "el_slider().vertical(true).height(180.);".into(),
                vertical.into_any_element(),
            ))
            .child(self.section(
                cx,
                "slider-marks",
                &s.slider_marks,
                || "el_slider().marks([\n    SliderMark { point: 25., label: \"25%\".into() },\n    SliderMark { point: 50., label: \"50%\".into() },\n]);".into(),
                marks.into_any_element(),
            ))
            .child(self.section(
                cx,
                "slider-formatted",
                &s.slider_formatted,
                || "el_slider().value(40.).format_tooltip(|v| format!(\"{v:.0}%\"));".into(),
                formatted.into_any_element(),
            ))
            .child(self.section(
                cx,
                "slider-readonly",
                &s.slider_readonly,
                || "el_slider().value(70.).disabled(true);".into(),
                readonly.into_any_element(),
            ))
            .child(
                div().flex().items_center().gap(px(12.)).child(
                    el_button()
                        .id("slider-reset")
                        .small()
                        .label(reset.as_str())
                        .icon(el_icon("refresh-left"))
                        .onclick(cx.listener(|this, _, _, cx| {
                            this.slider = Default::default();
                            cx.notify();
                        })),
                ),
            )
    }
}
