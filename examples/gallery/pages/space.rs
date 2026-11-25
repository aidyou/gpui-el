use gpui::{Context, Div, div, prelude::*, px};
use gpui_el::{Space, SpaceSize, el_button, el_space, tokens::spacing};

use crate::{Gallery, copy::Copy};

impl Gallery {
    pub(crate) fn spaces(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let buttons = |space: Space, id: &'static str| {
            space.children(
                [s.default.as_str(), s.primary.as_str(), s.success.as_str()]
                    .into_iter()
                    .enumerate()
                    .map(|(i, label)| {
                        el_button()
                            .id((id, i))
                            .label(label)
                            .when(i == 1, |button| button.primary())
                    }),
            )
        };
        let presets = [
            ("implicit", format!("{} px", spacing::SMALL), el_space()),
            (
                "small",
                format!("{} px · small()", spacing::SMALL),
                el_space().small(),
            ),
            (
                "default",
                format!("{} px", spacing::DEFAULT),
                el_space().size(SpaceSize::Default),
            ),
            (
                "large",
                format!("{} px", spacing::LARGE),
                el_space().large(),
            ),
        ];
        let source = || include_str!("space.rs").into();
        div()
            .flex()
            .flex_col()
            .gap(px(24.))
            .child(
                self.section(
                    cx,
                    "space-presets",
                    &s.space_presets,
                    source,
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(spacing::LARGE))
                        .children(presets.map(|(id, label, space)| {
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(spacing::SMALL))
                                .child(label)
                                .child(buttons(space, id))
                        })),
                ),
            )
            .child(
                self.section(
                    cx,
                    "space-wrap",
                    &s.space_title,
                    source,
                    div()
                        .w(px(220.))
                        .max_w_full()
                        .child(buttons(el_space().large().wrap(true), "wrapped")),
                ),
            )
    }
}
