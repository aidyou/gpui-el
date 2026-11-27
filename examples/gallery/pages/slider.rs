use gpui::{Context, Div, div, prelude::*, px};
use gpui_el::{SliderMark, el_button, el_icon, el_slider};

use crate::{Gallery, copy::Copy, demo::row};

/// `Space` wraps each child in an auto-width flex item. A horizontal Slider
/// fills that item with `w_full()`, but its visual children are absolute, so
/// the item's intrinsic width is zero unless the demo supplies one.
fn horizontal_demo() -> Div {
    div().w(px(300.))
}

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
        let basic = row().child(horizontal_demo().child(slider(
            0,
            "slider-basic".into(),
            self.slider.basic,
        )));
        let sizes = row().children((0..3u32).map(|index| {
            horizontal_demo().child(
                slider(1, ("slider-sized", index).into(), self.slider.sizes)
                    .when(index == 0, |el| el.large())
                    .when(index == 2, |el| el.small()),
            )
        }));
        let steps = row().child(
            horizontal_demo().child(
                slider(2, "slider-steps".into(), self.slider.steps)
                    .step(10.)
                    .show_stops(true),
            ),
        );
        let range_view = view.clone();
        let range = row().child(
            horizontal_demo().child(
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
            ),
        );
        let vertical = row().child(
            slider(3, "slider-vertical".into(), self.slider.vertical)
                .vertical(true)
                .height(180.),
        );
        let marks = row().child(horizontal_demo().child(
            slider(4, "slider-marks".into(), self.slider.marks).marks([
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
            ]),
        ));
        let formatted = row().child(
            horizontal_demo().child(
                el_slider()
                    .id("slider-formatted")
                    .value(40.)
                    .format_tooltip(|value| format!("{value:.0}%").into()),
            ),
        );
        let readonly = row().child(
            horizontal_demo().child(el_slider().id("slider-readonly").value(70.).disabled(true)),
        );

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    #[gpui::test]
    fn horizontal_slider_has_width_and_drags_inside_a_wrapping_row(cx: &mut gpui::TestAppContext) {
        struct Harness {
            value: Rc<RefCell<f32>>,
            bounds: Rc<RefCell<Option<gpui::Bounds<gpui::Pixels>>>>,
        }
        impl gpui::Render for Harness {
            fn render(
                &mut self,
                _: &mut gpui::Window,
                _: &mut gpui::Context<Self>,
            ) -> impl IntoElement {
                let value = self.value.clone();
                let current = value.borrow().clone();
                let measured = self.bounds.clone();
                div().w(px(600.)).child(
                    row().child(
                        horizontal_demo()
                            .child(el_slider().id("row-slider").value(current).oninput(
                                move |reported, _, _| {
                                    let gpui_el::SliderValue::Single(reported) = reported else {
                                        return;
                                    };
                                    *value.borrow_mut() = reported;
                                },
                            ))
                            .child(
                                gpui::canvas(
                                    move |bounds, _, _| {
                                        *measured.borrow_mut() = Some(bounds);
                                    },
                                    |_, _, _, _| {},
                                )
                                .absolute()
                                .inset_0(),
                            ),
                    ),
                )
            }
        }
        let value = Rc::new(RefCell::new(30.));
        let bounds = Rc::new(RefCell::new(None));
        let (_, view) = cx.add_window_view({
            let value = value.clone();
            let bounds = bounds.clone();
            move |_, _| Harness { value, bounds }
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(
            bounds
                .borrow()
                .expect("slider probe should paint")
                .size
                .width,
            px(300.),
            "a horizontal slider in a wrapping row must not collapse to zero width"
        );
        view.simulate_mouse_down(
            gpui::point(px(150.), px(16.)),
            gpui::MouseButton::Left,
            gpui::Modifiers::default(),
        );
        view.simulate_mouse_up(
            gpui::point(px(150.), px(16.)),
            gpui::MouseButton::Left,
            gpui::Modifiers::default(),
        );
        assert_eq!(*value.borrow(), 50.);
    }
}
