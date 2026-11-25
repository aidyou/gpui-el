use std::ops::Range;

use gpui::{
    App, Bounds, ClipboardItem, Context, ElementInputHandler, EntityInputHandler, FocusHandle,
    IntoElement, KeyBinding, Pixels, Point, Render, ShapedLine, SharedString, TextRun,
    UTF16Selection, Window, actions, canvas, div, point, prelude::*, px, rgb, size,
};

actions!(
    catalog_search,
    [
        Left,
        Right,
        SelectLeft,
        SelectRight,
        Home,
        End,
        Backspace,
        Delete,
        SelectAll,
        CopyText,
        Cut,
        Paste
    ]
);

pub(super) fn bind(cx: &mut App) {
    let context = Some("CatalogSearch");
    let mac = cfg!(target_os = "macos");
    cx.bind_keys([
        KeyBinding::new("left", Left, context),
        KeyBinding::new("right", Right, context),
        KeyBinding::new("shift-left", SelectLeft, context),
        KeyBinding::new("shift-right", SelectRight, context),
        KeyBinding::new("home", Home, context),
        KeyBinding::new("end", End, context),
        KeyBinding::new("backspace", Backspace, context),
        KeyBinding::new("delete", Delete, context),
        KeyBinding::new(if mac { "cmd-a" } else { "ctrl-a" }, SelectAll, context),
        KeyBinding::new(if mac { "cmd-c" } else { "ctrl-c" }, CopyText, context),
        KeyBinding::new(if mac { "cmd-x" } else { "ctrl-x" }, Cut, context),
        KeyBinding::new(if mac { "cmd-v" } else { "ctrl-v" }, Paste, context),
    ]);
}

pub(super) struct Search {
    pub(super) text: String,
    pub(super) placeholder: SharedString,
    focus: FocusHandle,
    head: usize,
    anchor: usize,
    marked: Option<Range<usize>>,
    layout: Option<(ShapedLine, Point<Pixels>)>,
    dragging: bool,
}

fn byte(text: &str, utf16: usize) -> usize {
    let mut units = 0;
    for (index, c) in text.char_indices() {
        if units + c.len_utf16() > utf16 {
            return index;
        }
        units += c.len_utf16();
    }
    text.len()
}

impl Search {
    pub(super) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            text: String::new(),
            placeholder: "Search icons".into(),
            focus: cx.focus_handle(),
            head: 0,
            anchor: 0,
            marked: None,
            layout: None,
            dragging: false,
        }
    }
    pub(super) fn query(&self) -> String {
        let mut text = self.text.clone();
        if let Some(marked) = self.marked.clone() {
            text.replace_range(marked, "");
        }
        text.trim().to_ascii_lowercase().replace('-', "")
    }
    fn range(&self) -> Range<usize> {
        self.head.min(self.anchor)..self.head.max(self.anchor)
    }
    fn utf16(&self, at: usize) -> usize {
        self.text[..at].encode_utf16().count()
    }
    fn utf16_to_bytes(&self, r: Range<usize>) -> Range<usize> {
        let a = byte(&self.text, r.start);
        a..byte(&self.text, r.end).max(a)
    }
    fn target(&self, range: Option<Range<usize>>) -> Range<usize> {
        range
            .map(|r| self.utf16_to_bytes(r))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.range())
    }
    fn step(&self, right: bool) -> usize {
        use unicode_segmentation::UnicodeSegmentation;
        let mut indices = self.text.grapheme_indices(true).map(|(i, _)| i);
        if right {
            indices.find(|&i| i > self.head).unwrap_or(self.text.len())
        } else {
            indices.take_while(|&i| i < self.head).last().unwrap_or(0)
        }
    }
    fn move_to(&mut self, at: usize, extend: bool, w: &mut Window, cx: &mut Context<Self>) {
        self.head = at;
        if !extend {
            self.anchor = at;
        }
        w.notify_text_selection_changed(&self.focus);
        cx.notify();
    }
    fn move_by(&mut self, right: bool, extend: bool, w: &mut Window, cx: &mut Context<Self>) {
        let r = self.range();
        let at = if !extend && !r.is_empty() {
            if right { r.end } else { r.start }
        } else {
            self.step(right)
        };
        self.move_to(at, extend, w, cx);
    }
    fn erase(&mut self, right: bool, w: &mut Window, cx: &mut Context<Self>) {
        if self.range().is_empty() && self.marked.is_none() {
            self.anchor = self.step(right);
        }
        self.replace_text_in_range(None, "", w, cx);
    }
    fn update_text(&mut self, w: &mut Window, cx: &mut Context<Self>) {
        self.layout = None;
        w.notify_text_input_changed(&self.focus);
        cx.notify();
    }
    fn index(&self, p: Point<Pixels>) -> Option<usize> {
        self.layout
            .as_ref()
            .map(|(line, at)| line.closest_index_for_x(p.x - at.x))
    }
    fn copy(&self, cx: &mut App) {
        if !self.range().is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.text[self.range()].into()));
        }
    }
}

impl EntityInputHandler for Search {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        adjusted: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let r = self.utf16_to_bytes(range);
        *adjusted = Some(self.utf16(r.start)..self.utf16(r.end));
        Some(self.text[r].into())
    }
    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let r = self.range();
        Some(UTF16Selection {
            range: self.utf16(r.start)..self.utf16(r.end),
            reversed: self.head < self.anchor,
        })
    }
    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked
            .as_ref()
            .map(|r| self.utf16(r.start)..self.utf16(r.end))
    }
    fn unmark_text(&mut self, w: &mut Window, cx: &mut Context<Self>) {
        self.marked = None;
        self.update_text(w, cx);
    }
    fn replace_text_in_range(
        &mut self,
        r: Option<Range<usize>>,
        text: &str,
        w: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let r = self.target(r);
        let text = text.replace(['\n', '\r'], " ");
        self.head = r.start + text.len();
        self.anchor = self.head;
        self.text.replace_range(r, &text);
        self.marked = None;
        self.update_text(w, cx);
    }
    fn replace_and_mark_text_in_range(
        &mut self,
        r: Option<Range<usize>>,
        text: &str,
        selection: Option<Range<usize>>,
        w: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let r = self.target(r);
        let start = r.start;
        let text = text.replace(['\n', '\r'], " ");
        self.text.replace_range(r, &text);
        self.marked = (!text.is_empty()).then_some(start..start + text.len());
        let default = text.encode_utf16().count();
        let selection = selection.unwrap_or(default..default);
        self.anchor = start + byte(&text, selection.start);
        self.head = start + byte(&text, selection.end);
        self.update_text(w, cx);
    }
    fn bounds_for_range(
        &mut self,
        r: Range<usize>,
        bounds: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let r = self.utf16_to_bytes(r);
        let (line, at) = self.layout.as_ref()?;
        let left = line.x_for_index(r.start);
        Some(Bounds::new(
            point(at.x + left, at.y),
            size(
                (line.x_for_index(r.end) - left).max(px(1.)),
                bounds.size.height,
            ),
        ))
    }
    fn character_index_for_point(
        &mut self,
        p: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        self.index(p).map(|i| self.utf16(i))
    }
}

impl Render for Search {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let prepaint = entity.clone();
        let placeholder = self.placeholder.clone();
        div()
            .id("search-input")
            .role(gpui::Role::TextInput)
            .aria_label(placeholder.clone())
            .key_context("CatalogSearch")
            .track_focus(&self.focus)
            .tab_stop(true)
            .w_full()
            .h(px(40.))
            .px(px(12.))
            .border_1()
            .border_color(rgb(0x909399))
            .rounded(px(4.))
            .overflow_hidden()
            .focus(|s| s.border_color(rgb(0x409eff)))
            .cursor_text()
            .on_action(cx.listener(|s, _: &Left, w, cx| s.move_by(false, false, w, cx)))
            .on_action(cx.listener(|s, _: &Right, w, cx| s.move_by(true, false, w, cx)))
            .on_action(cx.listener(|s, _: &SelectLeft, w, cx| s.move_by(false, true, w, cx)))
            .on_action(cx.listener(|s, _: &SelectRight, w, cx| s.move_by(true, true, w, cx)))
            .on_action(cx.listener(|s, _: &Home, w, cx| s.move_to(0, false, w, cx)))
            .on_action(cx.listener(|s, _: &End, w, cx| s.move_to(s.text.len(), false, w, cx)))
            .on_action(cx.listener(|s, _: &Backspace, w, cx| s.erase(false, w, cx)))
            .on_action(cx.listener(|s, _: &Delete, w, cx| s.erase(true, w, cx)))
            .on_action(cx.listener(|s, _: &SelectAll, w, cx| {
                s.anchor = 0;
                s.move_to(s.text.len(), true, w, cx);
            }))
            .on_action(cx.listener(|s, _: &CopyText, _, cx| s.copy(cx)))
            .on_action(cx.listener(|s, _: &Cut, w, cx| {
                s.copy(cx);
                if !s.range().is_empty() {
                    s.replace_text_in_range(None, "", w, cx);
                }
            }))
            .on_action(cx.listener(|s, _: &Paste, w, cx| {
                if let Some(text) = cx.read_from_clipboard().and_then(|it| it.text()) {
                    s.replace_text_in_range(None, &text, w, cx);
                }
            }))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|s, ev: &gpui::MouseDownEvent, w, cx| {
                    s.focus.focus(w, cx);
                    s.dragging = true;
                    if let Some(i) = s.index(ev.position) {
                        s.move_to(i, ev.modifiers.shift, w, cx);
                    }
                }),
            )
            .on_mouse_move(cx.listener(|s, ev: &gpui::MouseMoveEvent, w, cx| {
                if ev.pressed_button.is_none() {
                    s.dragging = false;
                }
                if s.dragging
                    && let Some(i) = s.index(ev.position)
                {
                    s.move_to(i, true, w, cx);
                }
            }))
            .on_mouse_up(
                gpui::MouseButton::Left,
                cx.listener(|s, _, _, _| s.dragging = false),
            )
            .on_mouse_up_out(
                gpui::MouseButton::Left,
                cx.listener(|s, _, _, _| s.dragging = false),
            )
            .child(
                canvas(
                    move |bounds, w, cx| {
                        let s = prepaint.read(cx);
                        let style = w.text_style();
                        let shape = |text: SharedString, color| {
                            let runs = [TextRun {
                                len: text.len(),
                                font: style.font(),
                                color,
                                background_color: None,
                                underline: None,
                                strikethrough: None,
                            }];
                            w.text_system().shape_line(text, px(14.), &runs, None)
                        };
                        let line = shape(s.text.clone().into(), style.color);
                        let hint = s
                            .text
                            .is_empty()
                            .then(|| shape(placeholder, rgb(0x909399).into()));
                        let offset = (line.x_for_index(s.head) - bounds.size.width + px(2.))
                            .max(px(0.))
                            .min((line.width - bounds.size.width + px(2.)).max(px(0.)));
                        let at = point(
                            bounds.origin.x - offset,
                            bounds.origin.y + (bounds.size.height - px(20.)) / 2.,
                        );
                        (
                            line,
                            hint,
                            at,
                            s.range(),
                            s.marked.clone(),
                            s.head,
                            s.focus.clone(),
                        )
                    },
                    move |bounds, (line, hint, at, selection, marked, head, focus), w, cx| {
                        if focus.is_focused(w) {
                            w.handle_input(
                                &focus,
                                ElementInputHandler::new(bounds, entity.clone()),
                                cx,
                            );
                            let rect = |r: Range<usize>, y, h| {
                                let x = line.x_for_index(r.start);
                                Bounds::new(
                                    point(at.x + x, at.y + px(y)),
                                    size((line.x_for_index(r.end) - x).max(px(1.)), px(h)),
                                )
                            };
                            if !selection.is_empty() {
                                w.paint_quad(gpui::fill(
                                    rect(selection, 0., 20.),
                                    gpui::rgba(0x409eff55),
                                ));
                            } else {
                                w.paint_quad(gpui::fill(rect(head..head, 0., 20.), rgb(0x409eff)));
                            }
                            if let Some(marked) = marked {
                                w.paint_quad(gpui::fill(rect(marked, 19., 1.), rgb(0x409eff)));
                            }
                        }
                        let _ = hint.as_ref().unwrap_or(&line).paint(
                            at,
                            px(20.),
                            gpui::TextAlign::Left,
                            None,
                            w,
                            cx,
                        );
                        entity.update(cx, |s, _| s.layout = Some((line, at)));
                    },
                )
                .size_full(),
            )
    }
}

#[test]
fn interprets_platform_utf16_at_unicode_boundaries() {
    assert_eq!(byte("中文😀a", 3), 6);
    assert_eq!(byte("中文😀a", 4), 10);
}
