use gpui::{Context, Div, MouseButton, Window, WindowControlArea, div, prelude::*, px, rgb};
use gpui_el::{Icon, IconPack, Locale, el_button, el_space, tokens};

use crate::{Gallery, copy::Copy};

impl Gallery {
    pub(crate) fn titlebar(
        &self,
        window: &Window,
        cx: &Context<Self>,
        s: &'static Copy,
        locale: Locale,
    ) -> Div {
        let border = rgb(if self.dark { 0x414243 } else { 0xe4e7ed });
        let icon_button = |id, name, label| {
            el_button()
                .id(id)
                .text()
                .circle()
                .icon(Icon::from_pack(IconPack::ElementPlus, name).size(16.))
                .accessibility_label(label)
                .tooltip(label)
        };
        let drag = div()
            .id("caption-drag")
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .items_center()
            .window_control_area(WindowControlArea::Drag)
            .on_mouse_down(MouseButton::Left, |_, window, _| {
                #[cfg(not(target_os = "windows"))]
                window.start_window_move();
            })
            .on_click(|event, window, _| {
                if event.click_count() == 2 {
                    if cfg!(target_os = "macos") {
                        window.titlebar_double_click();
                    } else if !cfg!(target_os = "windows") {
                        window.zoom_window();
                    }
                }
            })
            .child(
                div()
                    .truncate()
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("gpui-el component"),
            );
        let tools = el_space()
            .gap(4., 0.)
            .child(
                el_button()
                    .id("language")
                    .text()
                    .circle()
                    .icon(Icon::from_pack(IconPack::Tabler, "language").size(16.))
                    .accessibility_label(s.language.as_str())
                    .tooltip(s.language.as_str())
                    .onclick(move |_, _, cx| {
                        if locale == Locale::SimplifiedChinese {
                            Locale::English
                        } else {
                            Locale::SimplifiedChinese
                        }
                        .set(cx);
                    }),
            )
            .child(
                icon_button(
                    "theme",
                    if self.dark { "Sunny" } else { "Moon" },
                    if self.dark {
                        s.light.as_str()
                    } else {
                        s.dark.as_str()
                    },
                )
                .onclick(cx.listener(|this, _, _, cx| {
                    this.dark = !this.dark;
                    this.apply_theme(cx);
                })),
            )
            .child(
                icon_button(
                    "motion",
                    "VideoPause",
                    if self.reduced_motion {
                        s.restore_motion.as_str()
                    } else {
                        s.reduce_motion.as_str()
                    },
                )
                .onclick(cx.listener(|this, _, _, cx| {
                    this.reduced_motion = !this.reduced_motion;
                    this.apply_theme(cx);
                })),
            );
        div()
            .flex()
            .items_center()
            .gap(px(tokens::spacing::DEFAULT))
            .h(px(52.))
            .px(px(16.))
            .flex_shrink_0()
            .border_b_1()
            .border_color(border)
            .when(cfg!(target_os = "macos") && !window.is_fullscreen(), |el| {
                el.pl(px(82.))
            })
            .child(drag)
            .child(tools)
            .when(
                !cfg!(target_os = "macos") && !window.is_fullscreen(),
                |el| {
                    el.child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(2.))
                            .pl(px(8.))
                            .border_l_1()
                            .border_color(border)
                            .child(
                                icon_button("minimize", "Minus", s.window_minimize.as_str())
                                    .onclick(|_, w, _| w.minimize_window()),
                            )
                            .child(
                                icon_button(
                                    "zoom",
                                    if window.is_maximized() {
                                        "CopyDocument"
                                    } else {
                                        "FullScreen"
                                    },
                                    s.window_zoom.as_str(),
                                )
                                .onclick(|_, w, _| w.zoom_window()),
                            )
                            .child(
                                icon_button("close", "Close", s.window_close.as_str())
                                    .onclick(|_, w, _| w.remove_window()),
                            ),
                    )
                },
            )
    }
}
