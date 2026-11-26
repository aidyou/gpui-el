use std::ops::Range;

use gpui::{
    App, ClipboardItem, Context, Div, FocusHandle, IntoElement, KeyBinding, MouseButton, Pixels,
    Point, RenderOnce, SharedString, StyledText, TextLayout, Window, actions, div, prelude::*, px,
    rgb, rgba,
};
use gpui_el::{Locale, el_button, el_icon, tokens};

pub(crate) use crate::row::row;
use crate::{Gallery, copy::strings, syntax};

actions!(gallery_code, [CopySelection, SelectAll]);

pub(crate) fn bind(cx: &mut App) {
    let (copy, all) = if cfg!(target_os = "macos") {
        ("cmd-c", "cmd-a")
    } else {
        ("ctrl-c", "ctrl-a")
    };
    cx.bind_keys([
        KeyBinding::new(copy, CopySelection, Some("GalleryCode")),
        KeyBinding::new(all, SelectAll, Some("GalleryCode")),
    ]);
}

struct Selection {
    text: SharedString,
    focus: FocusHandle,
    anchor: usize,
    head: usize,
    dragging: bool,
}

impl Selection {
    fn range(&self) -> Range<usize> {
        self.anchor.min(self.head)..self.anchor.max(self.head)
    }

    fn index(&self, layout: &TextLayout, position: Point<Pixels>) -> usize {
        let bounds = layout.bounds();
        if position.y < bounds.top() {
            return 0;
        }
        if position.y >= bounds.bottom() {
            return self.text.len();
        }
        let index = layout
            .index_for_position(position)
            .unwrap_or_else(|edge| edge);
        self.text.floor_char_boundary(index.min(self.text.len()))
    }
}

#[derive(IntoElement)]
struct CodeBlock {
    id: &'static str,
    text: SharedString,
    dark: bool,
}

impl RenderOnce for CodeBlock {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = (self.id, 2_u32);
        let state = window.use_keyed_state(id, cx, |_, cx| Selection {
            text: self.text.clone(),
            focus: cx.focus_handle(),
            anchor: 0,
            head: 0,
            dragging: false,
        });
        if state.read(cx).text != self.text {
            state.update(cx, |state, _| {
                state.text = self.text.clone();
                state.anchor = 0;
                state.head = 0;
                state.dragging = false;
            });
        }
        let selection = state.read(cx);
        let focus = selection.focus.clone();
        let range = selection.range();
        // Token colors and the selection background merge into one sorted,
        // disjoint range list: StyledText computes one text run per range and
        // requires them to be ordered and non-overlapping.
        let selected = (!range.is_empty()).then(|| {
            (
                range,
                rgba(if self.dark { 0x409eff66 } else { 0x409eff40 }).into(),
            )
        });
        let highlights = syntax::highlights(&self.text, selected, self.dark);
        let styled = StyledText::new(self.text.clone()).with_highlights(highlights);
        let line_count = self.text.matches('\n').count() + 1;
        let gutter = (1..=line_count)
            .map(|number| format!("{number:>2}"))
            .collect::<Vec<_>>()
            .join("\n");
        let layout = styled.layout().clone();
        let (down, move_to, up, outside, select_all, copy) = (
            state.clone(),
            state.clone(),
            state.clone(),
            state.clone(),
            state.clone(),
            state,
        );
        let (down_layout, move_layout, up_layout, out_layout) =
            (layout.clone(), layout.clone(), layout.clone(), layout);
        div()
            .id((self.id, 2_u32))
            .key_context("GalleryCode")
            .track_focus(&focus)
            .tab_stop(true)
            .aria_label(self.text)
            .role(gpui::Role::Label)
            .w_full()
            .min_w_0()
            .flex()
            .p(px(11.))
            .gap(px(12.))
            .border_1()
            .border_color(rgba(0))
            .focus_visible(|s| s.border_color(rgb(0x409eff)))
            .font_family("monospace")
            .text_size(px(13.))
            .line_height(px(20.))
            .cursor_text()
            .child(
                // Line numbers stay fixed while the code scrolls horizontally.
                div()
                    .flex_none()
                    .w(px(22.))
                    .text_color(rgb(if self.dark { 0x8b949e } else { 0x909399 }))
                    .child(gutter),
            )
            .child(
                // Horizontal scrolling keeps every line on a single row so the
                // gutter stays aligned.
                div()
                    .id((self.id, 3_u32))
                    .flex_1()
                    .min_w_0()
                    .overflow_x_scroll()
                    .child(styled),
            )
            .on_action(move |_: &CopySelection, _, cx| {
                let state = copy.read(cx);
                if let Some(text) = state
                    .text
                    .get(state.range())
                    .filter(|text| !text.is_empty())
                {
                    cx.write_to_clipboard(ClipboardItem::new_string(text.to_owned()));
                }
            })
            .on_action(move |_: &SelectAll, window, cx| {
                select_all.update(cx, |state, _| {
                    state.anchor = 0;
                    state.head = state.text.len();
                    state.dragging = false;
                });
                window.refresh();
            })
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                down.update(cx, |state, cx| {
                    state.focus.focus(window, cx);
                    let index = state.index(&down_layout, event.position);
                    if !event.modifiers.shift {
                        state.anchor = index;
                    }
                    state.head = index;
                    state.dragging = true;
                });
                window.refresh();
            })
            .on_mouse_move(move |event, window, cx| {
                if !move_to.read(cx).dragging {
                    return;
                }
                if !event.dragging() {
                    move_to.update(cx, |state, _| state.dragging = false);
                    return;
                }
                let index = move_to.read(cx).index(&move_layout, event.position);
                if move_to.read(cx).head != index {
                    move_to.update(cx, |state, _| state.head = index);
                    window.refresh();
                }
            })
            .on_mouse_up(MouseButton::Left, move |event, window, cx| {
                if up.read(cx).dragging {
                    up.update(cx, |state, _| {
                        state.head = state.index(&up_layout, event.position);
                        state.dragging = false;
                    });
                    window.refresh();
                }
            })
            .on_mouse_up_out(MouseButton::Left, move |event, window, cx| {
                if outside.read(cx).dragging {
                    outside.update(cx, |state, _| {
                        state.head = state.index(&out_layout, event.position);
                        state.dragging = false;
                    });
                    window.refresh();
                }
            })
    }
}

impl Gallery {
    pub(crate) fn section(
        &self,
        cx: &Context<Self>,
        id: &'static str,
        title: &str,
        example: impl FnOnce() -> String,
        controls: impl IntoElement,
    ) -> Div {
        let s = strings(Locale::current(cx));
        let snippet: Option<SharedString> = self.expanded.contains(id).then(|| example().into());
        let expanded = snippet.is_some();
        let border = rgb(if self.dark { 0x414243 } else { 0xe4e7ed });
        let fill = rgb(if self.dark { 0x262727 } else { 0xf5f7fa });
        div()
            .flex()
            .flex_col()
            .gap(px(tokens::spacing::DEFAULT))
            .child(
                div()
                    .text_size(px(18.))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(title.to_owned()),
            )
            .child(
                div()
                    .border_1()
                    .border_color(border)
                    .rounded(px(tokens::radius::BASE))
                    .child(div().p(px(20.)).child(controls))
                    .when_some(snippet.clone(), |el, snippet| {
                        el.child(div().border_t_1().border_color(border).bg(fill).child(
                            CodeBlock {
                                id,
                                text: snippet,
                                dark: self.dark,
                            },
                        ))
                    })
                    .child(
                        div().border_t_1().border_color(border).p(px(6.)).child(
                            row()
                                .child(
                                    el_button()
                                        .id((id, 0_u32))
                                        .small()
                                        .text()
                                        .label(if expanded {
                                            s.hide_code.as_str()
                                        } else {
                                            s.show_code.as_str()
                                        })
                                        .icon(el_icon(if expanded {
                                            "arrow-up"
                                        } else {
                                            "arrow-down"
                                        }))
                                        .onclick(cx.listener(move |this, _, _, cx| {
                                            if !this.expanded.insert(id) {
                                                this.expanded.remove(id);
                                            }
                                            cx.notify();
                                        })),
                                )
                                .when_some(snippet, |el, snippet| {
                                    el.child(
                                        el_button()
                                            .id((id, 1_u32))
                                            .small()
                                            .text()
                                            .label(s.copy.as_str())
                                            .icon(el_icon("document-copy"))
                                            .onclick(move |_, _, cx| {
                                                cx.write_to_clipboard(ClipboardItem::new_string(
                                                    snippet.to_string(),
                                                ))
                                            }),
                                    )
                                }),
                        ),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{
        KeyDownEvent, KeyUpEvent, Keystroke, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Render,
        point,
    };

    const SAMPLE: &str = "    确认\nel_tag().small()\nnext line";

    struct SnippetDemo;

    impl Render for SnippetDemo {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(300.)).child(CodeBlock {
                id: "test-code",
                text: SAMPLE.into(),
                dark: false,
            })
        }
    }

    #[gpui::test]
    fn drag_copies_only_selected_source_and_select_all_copies_everything(
        cx: &mut gpui::TestAppContext,
    ) {
        let (_, mut view) = cx.add_window_view(|_, cx| {
            bind(cx);
            SnippetDemo
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        let start = point(px(50.), px(22.));
        let end = point(px(250.), px(42.));
        view.simulate_event(MouseDownEvent {
            button: MouseButton::Left,
            position: end,
            click_count: 1,
            ..Default::default()
        });
        view.simulate_event(MouseMoveEvent {
            position: start,
            pressed_button: Some(MouseButton::Left),
            ..Default::default()
        });
        view.simulate_event(MouseUpEvent {
            button: MouseButton::Left,
            position: start,
            click_count: 1,
            ..Default::default()
        });
        let press = |view: &mut gpui::VisualTestContext, key| {
            let keystroke =
                view.update(|_, cx| Keystroke::parse_with_os(key, cx.platform_info().os).unwrap());
            view.simulate_event(KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            });
            view.simulate_event(KeyUpEvent { keystroke });
        };
        let (copy, select_all) = if cfg!(target_os = "macos") {
            ("cmd-c", "cmd-a")
        } else {
            ("ctrl-c", "ctrl-a")
        };
        let clipboard = |view: &mut gpui::VisualTestContext| {
            view.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text()))
        };
        view.update(|window, cx| window.draw(cx).clear(cx));
        press(&mut view, copy);
        assert_eq!(
            clipboard(&mut view).as_deref(),
            Some("    确认\nel_tag().small()")
        );
        press(&mut view, select_all);
        press(&mut view, copy);
        assert_eq!(clipboard(&mut view).as_deref(), Some(SAMPLE));
    }
}
