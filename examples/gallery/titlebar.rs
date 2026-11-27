use gpui::{
    Context, Div, MouseButton, MouseDownEvent, MouseMoveEvent, Window, WindowControlArea, div,
    prelude::*, px, rgb,
};
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
        let border = rgb(if self.dark { 0x2a2b2c } else { 0xe4e7ed });
        let muted = rgb(if self.dark { 0x8d9095 } else { 0x909399 });
        let icon_button = |id, name, label| {
            el_button()
                .id(id)
                .text()
                .circle()
                .font_size(15.0)
                .icon(Icon::from_pack(IconPack::ElementPlus, name).size(16.))
                .accessibility_label(label)
                .tooltip(label)
        };
        // Desktop title bar: drag to move, double-click to zoom, right-click for
        // the window menu. Starting a compositor move on the first click would
        // swallow the second click, so the double-click is handled here.
        let drag = div()
            .id("caption-drag")
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .items_center()
            .gap(px(8.))
            .window_control_area(WindowControlArea::Drag)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, window, _| {
                    if event.click_count == 2 {
                        this.caption_press = None;
                        window.zoom_window();
                    } else {
                        this.caption_press = Some(event.position);
                    }
                }),
            )
            // A compositor move grabs the pointer, so it only starts once the
            // pointer actually travels; otherwise the second click of a
            // double-click would never reach the app.
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, window, _| {
                let Some(origin) = this.caption_press else {
                    return;
                };
                if !event.dragging() {
                    this.caption_press = None;
                    return;
                }
                let delta = event.position - origin;
                if f32::from(delta.x).abs() > 4. || f32::from(delta.y).abs() > 4. {
                    this.caption_press = None;
                    #[cfg(not(target_os = "windows"))]
                    window.start_window_move();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, _| {
                    this.caption_press = None;
                }),
            )
            .on_mouse_down(MouseButton::Right, |event, window, _| {
                #[cfg(not(target_os = "windows"))]
                window.show_window_menu(event.position);
            })
            .child(
                div()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("gpui-el"),
            )
            .child(
                div()
                    .truncate()
                    .text_size(px(tokens::font_size::SMALL))
                    .text_color(muted)
                    .child(self.page.title(s)),
            );
        let tools = el_space()
            .gap(4., 0.)
            .child(
                // The translate glyph reads as a language switch at every size.
                el_button()
                    .id("language")
                    .text()
                    .circle()
                    .font_size(15.0)
                    .icon(Icon::from_svg(include_bytes!("assets/translate.svg")).size(16.))
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
                    cx.notify();
                })),
            )
            .child(
                // Pause while motion runs; a close glyph once it is disabled,
                // so the icon states what is on rather than what clicking does.
                icon_button(
                    "motion",
                    if self.reduced_motion {
                        "Close"
                    } else {
                        "VideoPause"
                    },
                    if self.reduced_motion {
                        s.restore_motion.as_str()
                    } else {
                        s.reduce_motion.as_str()
                    },
                )
                .onclick(cx.listener(|this, _, _, cx| {
                    this.reduced_motion = !this.reduced_motion;
                    this.apply_theme(cx);
                    cx.notify();
                })),
            );
        div()
            .flex()
            .items_center()
            .gap(px(tokens::spacing::SMALL))
            .h(px(48.))
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
