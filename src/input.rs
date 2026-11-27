//! Element Plus Input and Textarea.
//!
//! Follows `input.md`, `components/input/src/input.vue`, and Theme Chalk
//! `input.scss`: three outer heights (40/32/24 px) with horizontal padding
//! 16/12/8 px, a placeholder, prefix and suffix slots, a clear button, a
//! password mask with a reveal toggle, and a character count against
//! `maxlength`. The single-line field and the multi-line textarea share native
//! caret movement, selection, word operations, clipboard shortcuts, and IME
//! composition: the field draws its own caret and selection highlight, maps
//! clicks to text positions, and implements GPUI's `InputHandler` so platform
//! IMEs can compose into it.
//!
//! Native differences: `password` replaces `type="password"`, and `textarea`
//! replaces `type="textarea"`; there are no `formatter`/`parser` props, no
//! native undo stack, and no `change` on Enter — upstream emits `change` on
//! blur and on clear, and so does this. The caret does not blink, since GPUI
//! has no CSS animation and a per-field timer is not worth the cost. Drag
//! selection tracks the pointer only inside the field, with no auto-scroll
//! past the edges, and the clear button reveals on field focus or by hovering
//! the button itself rather than anywhere in the field. The radius defaults to
//! the requested native 8 px profile rather than upstream's 4 px;
//! [`InputRadius`] exposes 0/4/8/12 px and a capsule shape.

use std::{cell::RefCell, ops::Range, rc::Rc, sync::Arc};

use gpui::{
    AnyElement, App, ClipboardItem, ElementId, FocusHandle, IntoElement, KeyDownEvent, MouseButton,
    MouseDownEvent, MouseMoveEvent, Pixels, RenderOnce, ScrollHandle, SharedString, Subscription,
    TextLayout, UnderlineStyle, Window, div, prelude::*, px, rgb,
};

use crate::{Theme, el_icon, tokens};

/// Outer height, font size, and padding, from `$input-height`,
/// `$input-font-size`, and `$input-padding-horizontal`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum InputSize {
    Large,
    #[default]
    Default,
    Small,
}

impl InputSize {
    fn height(self) -> f32 {
        match self {
            InputSize::Large => 40.,
            InputSize::Default => 32.,
            InputSize::Small => 24.,
        }
    }

    fn font_size(self) -> f32 {
        match self {
            InputSize::Small => 12.,
            InputSize::Large | InputSize::Default => 14.,
        }
    }

    fn padding(self) -> f32 {
        match self {
            InputSize::Large => 16.,
            InputSize::Default => 12.,
            InputSize::Small => 8.,
        }
    }
}

/// The field's corner profile. This is a native extension requested for the
/// gallery: upstream Input uses a fixed 4 px radius, while the default here is
/// the 8 px `Default` profile.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum InputRadius {
    Off,
    Small,
    #[default]
    Default,
    Large,
    Round,
}

impl InputRadius {
    fn pixels(self, height: f32) -> f32 {
        match self {
            InputRadius::Off => 0.,
            InputRadius::Small => 4.,
            InputRadius::Default => 8.,
            InputRadius::Large => 12.,
            InputRadius::Round => tokens::radius::capsule(height, height),
        }
    }
}

/// Per-render props and handlers, kept where the focus and blur subscriptions
/// can reach them; those subscriptions outlive any single render.
#[derive(Clone)]
struct InputProps {
    value: SharedString,
    maxlength: Option<usize>,
    readonly: bool,
    disabled: bool,
}

struct InputHandlers {
    oninput: Option<InputHandler>,
    onchange: Option<InputHandler>,
    onfocus: Option<FocusHandler>,
    onblur: Option<FocusHandler>,
    onclear: Option<InputHandler>,
    oncompositionstart: Option<FocusHandler>,
    oncompositionupdate: Option<InputHandler>,
    oncompositionend: Option<InputHandler>,
}

/// Caret, selection, composition, and measured text, keyed per field. The
/// caret and anchor are UTF-8 byte offsets into the value; the anchor is where
/// a selection started, so the two together describe both a bare caret and a
/// drag selection.
struct InputState {
    focus: FocusHandle,
    caret: usize,
    anchor: usize,
    /// The IME composing range, UTF-8 bytes.
    marked: Option<Range<usize>>,
    /// Shared with the text child, which fills it in at paint; the caret and
    /// hit testing read it afterwards. A `RefCell` because the render swaps it
    /// without notifying — the swap is not a state change the view needs to
    /// hear about.
    layout: RefCell<TextLayout>,
    password_visible: bool,
    /// Textarea scrolling, kept with the field so caret visibility can be
    /// maintained without making callers own another handle.
    scroll: ScrollHandle,
    props: Rc<RefCell<InputProps>>,
    handlers: Rc<RefCell<Option<Rc<InputHandlers>>>>,
    _subscriptions: Vec<Subscription>,
}

impl InputState {
    fn new(window: &mut Window, cx: &mut gpui::Context<Self>) -> Self {
        let focus = cx.focus_handle();
        let state = cx.entity();
        let props = Rc::new(RefCell::new(InputProps {
            value: "".into(),
            maxlength: None,
            readonly: false,
            disabled: false,
        }));
        let handlers: Rc<RefCell<Option<Rc<InputHandlers>>>> = Rc::new(RefCell::new(None));
        let mut subscriptions = Vec::new();
        // Focus and blur drive the border color, the caret, and upstream's
        // `change`-on-blur. Element listeners cannot see these transitions, so
        // the field subscribes at the window level and keeps the subscriptions
        // in its state.
        let focus_in = {
            let state = state.clone();
            let handlers = handlers.clone();
            window.on_focus_in(&focus, cx, move |window, cx| {
                if let Some(onfocus) = handlers
                    .borrow()
                    .as_ref()
                    .and_then(|handlers| handlers.onfocus.as_ref())
                {
                    onfocus(window, cx);
                }
                state.update(cx, |_, cx| cx.notify());
            })
        };
        let focus_out = {
            let state = state.clone();
            let handlers = handlers.clone();
            let props = props.clone();
            window.on_focus_out(&focus, cx, move |_event, window, cx| {
                let handlers = handlers.borrow().as_ref().cloned();
                if let Some(handlers) = handlers {
                    if let Some(onblur) = &handlers.onblur {
                        onblur(window, cx);
                    }
                    // Upstream emits `change` when the field blurs.
                    if let Some(onchange) = &handlers.onchange {
                        onchange(props.borrow().value.clone(), window, cx);
                    }
                }
                state.update(cx, |_, cx| cx.notify());
            })
        };
        subscriptions.push(focus_in);
        subscriptions.push(focus_out);
        Self {
            focus,
            caret: 0,
            anchor: 0,
            marked: None,
            layout: RefCell::new(TextLayout::default()),
            password_visible: false,
            scroll: ScrollHandle::new(),
            props,
            handlers,
            _subscriptions: subscriptions,
        }
    }

    /// The selected range, ordered low to high.
    fn selection(&self) -> Range<usize> {
        self.caret.min(self.anchor)..self.caret.max(self.anchor)
    }
}

/// An Element Plus input field.
type InputHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;
type FocusHandler = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(IntoElement)]
#[must_use]
pub struct Input {
    id: Option<ElementId>,
    value: SharedString,
    placeholder: SharedString,
    size: InputSize,
    radius: InputRadius,
    active_border_color: Option<gpui::Rgba>,
    outline: bool,
    text_color: Option<gpui::Rgba>,
    textarea: bool,
    rows: usize,
    disabled: bool,
    readonly: bool,
    clearable: bool,
    password: bool,
    show_password: bool,
    maxlength: Option<usize>,
    show_word_limit: bool,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
    oninput: Option<InputHandler>,
    onchange: Option<InputHandler>,
    onfocus: Option<FocusHandler>,
    onblur: Option<FocusHandler>,
    onclear: Option<InputHandler>,
    oncompositionstart: Option<FocusHandler>,
    oncompositionupdate: Option<InputHandler>,
    oncompositionend: Option<InputHandler>,
}

/// Creates an input field, mirroring the other `el_*` constructors.
#[track_caller]
pub fn el_input() -> Input {
    Input::new()
}

impl Default for Input {
    fn default() -> Self {
        Self {
            id: None,
            value: "".into(),
            placeholder: "".into(),
            size: InputSize::Default,
            radius: InputRadius::Default,
            active_border_color: None,
            outline: false,
            text_color: None,
            textarea: false,
            rows: 2,
            disabled: false,
            readonly: false,
            clearable: false,
            password: false,
            show_password: false,
            maxlength: None,
            show_word_limit: false,
            prefix: None,
            suffix: None,
            oninput: None,
            onchange: None,
            onfocus: None,
            onblur: None,
            onclear: None,
            oncompositionstart: None,
            oncompositionupdate: None,
            oncompositionend: None,
        }
    }
}

impl Input {
    /// Creates an empty field.
    #[track_caller]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a stable element key; required when several fields share a call
    /// site, since caret, selection, and focus state are keyed by it.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the controlled value.
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = value.into();
        self
    }

    /// Sets the placeholder shown while the value is empty.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn size(mut self, size: InputSize) -> Self {
        self.size = size;
        self
    }
    pub fn large(self) -> Self {
        self.size(InputSize::Large)
    }
    pub fn small(self) -> Self {
        self.size(InputSize::Small)
    }

    /// Sets the corner profile. The default is the native 8 px `Default`
    /// profile; `Round` uses half the field height for capsule ends.
    pub fn radius(mut self, radius: InputRadius) -> Self {
        self.radius = radius;
        self
    }
    pub fn radius_off(self) -> Self {
        self.radius(InputRadius::Off)
    }
    pub fn radius_small(self) -> Self {
        self.radius(InputRadius::Small)
    }
    pub fn radius_large(self) -> Self {
        self.radius(InputRadius::Large)
    }
    pub fn round(self) -> Self {
        self.radius(InputRadius::Round)
    }

    /// Sets the focused border color; primary by default.
    pub fn active_border_color(mut self, color: gpui::Rgba) -> Self {
        self.active_border_color = Some(color);
        self
    }

    /// Shows a 2 px focus ring outside the field; off by default, matching
    /// upstream's `outline: none`.
    pub fn outline(mut self, outline: bool) -> Self {
        self.outline = outline;
        self
    }

    /// Sets the normal text and caret color; the theme's regular text color by
    /// default. Disabled fields keep the disabled text token.
    pub fn text_color(mut self, color: gpui::Rgba) -> Self {
        self.text_color = Some(color);
        self
    }

    /// Switches to a multi-line textarea, like `type="textarea"`.
    pub fn textarea(mut self, textarea: bool) -> Self {
        self.textarea = textarea;
        self
    }

    /// Sets the textarea's visible row count, 2 by default. Single-line fields
    /// ignore it.
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows;
        self
    }

    /// Makes the field read-only, like upstream's `disabled`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Lets the caret and selection move without editing, like upstream's
    /// `readonly`.
    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    /// Adds a clear button that empties the field, like `clearable`.
    pub fn clearable(mut self, clearable: bool) -> Self {
        self.clearable = clearable;
        self
    }

    /// Masks the value, like `type="password"`.
    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    /// Adds a reveal toggle to a masked field, like `show-password`.
    pub fn show_password(mut self, show_password: bool) -> Self {
        self.show_password = show_password;
        self
    }

    /// Sets the character limit; typing stops there, like `maxlength`.
    pub fn maxlength(mut self, maxlength: usize) -> Self {
        self.maxlength = Some(maxlength);
        self
    }

    /// Shows a `length / maxlength` count, like `show-word-limit`.
    pub fn show_word_limit(mut self, show_word_limit: bool) -> Self {
        self.show_word_limit = show_word_limit;
        self
    }

    /// Sets the leading slot, like `prefix`.
    pub fn prefix(mut self, prefix: impl IntoElement) -> Self {
        self.prefix = Some(prefix.into_any_element());
        self
    }

    /// Sets the trailing slot, like `suffix`.
    pub fn suffix(mut self, suffix: impl IntoElement) -> Self {
        self.suffix = Some(suffix.into_any_element());
        self
    }

    /// Reports the value on every keystroke and composition step, like
    /// upstream's `input` event.
    #[track_caller]
    pub fn oninput(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.oninput = Some(Rc::new(handler));
        self.ensure_id();
        self
    }

    /// Reports the committed value when the field blurs or is cleared, like
    /// upstream's `change` event.
    #[track_caller]
    pub fn onchange(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onchange = Some(Rc::new(handler));
        self.ensure_id();
        self
    }

    /// Reports that the field gained focus, like upstream's `focus` event.
    #[track_caller]
    pub fn onfocus(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.onfocus = Some(Rc::new(handler));
        self.ensure_id();
        self
    }

    /// Reports that the field lost focus, like upstream's `blur` event.
    #[track_caller]
    pub fn onblur(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.onblur = Some(Rc::new(handler));
        self.ensure_id();
        self
    }

    /// Reports the cleared value when the clear button is pressed, like
    /// upstream's `clear` event.
    #[track_caller]
    pub fn onclear(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.onclear = Some(Rc::new(handler));
        self.ensure_id();
        self
    }

    /// Reports the start of an IME composition, like upstream's
    /// `compositionstart` event.
    #[track_caller]
    pub fn oncompositionstart(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.oncompositionstart = Some(Rc::new(handler));
        self.ensure_id();
        self
    }

    /// Reports each IME preedit update with the composing text, like
    /// upstream's `compositionupdate` event. `oninput` stays silent until the
    /// composition commits.
    #[track_caller]
    pub fn oncompositionupdate(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.oncompositionupdate = Some(Rc::new(handler));
        self.ensure_id();
        self
    }

    /// Reports the end of an IME composition with its final composing text,
    /// like upstream's `compositionend` event. The committed value follows
    /// through `oninput`.
    #[track_caller]
    pub fn oncompositionend(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.oncompositionend = Some(Rc::new(handler));
        self.ensure_id();
        self
    }

    #[track_caller]
    fn ensure_id(&mut self) {
        if self.id.is_none() {
            self.id = Some(ElementId::CodeLocation(*std::panic::Location::caller()));
        }
    }
}

/// Converts a UTF-16 offset (the IME contract's unit) to a UTF-8 byte offset.
fn utf16_to_utf8(text: &str, offset_utf16: usize) -> usize {
    let mut utf16 = 0usize;
    for (byte, char) in text.char_indices() {
        if utf16 >= offset_utf16 {
            return byte;
        }
        utf16 += char.len_utf16();
    }
    text.len()
}

/// Converts a UTF-8 byte offset to a UTF-16 offset.
fn utf8_to_utf16(text: &str, offset_utf8: usize) -> usize {
    let mut utf16 = 0usize;
    for (byte, char) in text.char_indices() {
        if byte >= offset_utf8 {
            break;
        }
        utf16 += char.len_utf16();
    }
    utf16
}

/// The character boundary one step from the index in the given direction.
/// `floor` and `ceil` alone would return the index itself when it already sits
/// on a boundary, so the step goes past it first.
fn char_boundary(text: &str, index: usize, forward: bool) -> usize {
    if forward {
        text.ceil_char_boundary((index + 1).min(text.len()))
    } else {
        text.floor_char_boundary(index.saturating_sub(1))
    }
}

/// The next word boundary in the given direction, like a native field's
/// Ctrl+Arrow movement. Forward skips the run of the character class under the
/// caret and any whitespace behind it; backward skips the whitespace and then
/// the class run behind that, so both land on a word edge. Punctuation is its
/// own class, as native editors treat it.
fn word_boundary(text: &str, index: usize, forward: bool) -> usize {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut position = chars
        .iter()
        .position(|(byte, _)| *byte >= index)
        .unwrap_or(chars.len());
    let class_of = |char: char| {
        if char.is_alphanumeric() {
            WordClass::Word
        } else if char.is_whitespace() {
            WordClass::Space
        } else {
            WordClass::Punct
        }
    };
    let predicate = |class: WordClass| match class {
        WordClass::Word => |char: char| char.is_alphanumeric(),
        WordClass::Space => |char: char| char.is_whitespace(),
        WordClass::Punct => |char: char| !char.is_alphanumeric() && !char.is_whitespace(),
    };
    if forward {
        let class = chars
            .get(position)
            .map(|(_, char)| class_of(*char))
            .unwrap_or(WordClass::Space);
        let matches = predicate(class);
        while chars.get(position).is_some_and(|(_, char)| matches(*char)) {
            position += 1;
        }
        while chars
            .get(position)
            .is_some_and(|(_, char)| char.is_whitespace())
        {
            position += 1;
        }
    } else {
        // `wrapping_sub` makes position 0 read as "no character behind".
        let behind = |position: usize| chars.get(position.wrapping_sub(1));
        while behind(position).is_some_and(|(_, char)| char.is_whitespace()) {
            position -= 1;
        }
        if let Some((_, char)) = behind(position) {
            let matches = predicate(class_of(*char));
            while behind(position).is_some_and(|(_, previous)| matches(*previous)) {
                position -= 1;
            }
        }
    }
    chars
        .get(position)
        .map(|(byte, _)| *byte)
        .unwrap_or(text.len())
}

#[derive(Clone, Copy, PartialEq)]
enum WordClass {
    Word,
    Space,
    Punct,
}

/// The text drawn in the field: the value, or its bullet mask.
fn display_text(value: &SharedString, password: bool, visible: bool) -> SharedString {
    if password && !visible {
        SharedString::from("•".repeat(value.chars().count()))
    } else {
        value.clone()
    }
}

/// Read-only fields keep focus and selection, but never paint an editing
/// caret. Disabled fields cannot receive it either.
fn caret_visible(focused: bool, disabled: bool, readonly: bool) -> bool {
    focused && !disabled && !readonly
}

/// Maps a byte offset in the displayed text back to one in the value; the mask
/// keeps the value's character count but not its byte count, so the conversion
/// goes through characters.
fn display_byte_to_value(value: &str, display: &str, byte: usize) -> usize {
    if display.len() == value.len() {
        return byte.min(value.len());
    }
    let characters = display[..byte.min(display.len())].chars().count();
    value
        .char_indices()
        .nth(characters)
        .map(|(byte, _)| byte)
        .unwrap_or(value.len())
}

/// Maps a value byte offset to the same place in the displayed text.
fn value_byte_to_display(value: &str, display: &str, byte: usize) -> usize {
    if display.len() == value.len() {
        return byte.min(display.len());
    }
    let characters = value[..byte.min(value.len())].chars().count();
    display
        .char_indices()
        .nth(characters)
        .map(|(byte, _)| byte)
        .unwrap_or(display.len())
}

/// Reports the new value for an edit, blocked when the field is disabled or
/// read-only, and clamped to `maxlength` by character, like the native
/// attribute.
fn clamped_edit(props: &InputProps, new_value: &str) -> Option<SharedString> {
    if props.disabled || props.readonly {
        return None;
    }
    if let Some(maxlength) = props.maxlength
        && new_value.chars().count() > maxlength
    {
        return None;
    }
    Some(SharedString::from(new_value))
}

/// Applies an edit: swaps the given range for the text, collapses the caret
/// behind it, tells native text services, and reports the value.
fn replace_range(
    state: &gpui::Entity<InputState>,
    range: Range<usize>,
    text: &str,
    window: &mut Window,
    cx: &mut App,
) {
    let props = state.read(cx).props.borrow().clone();
    if props.disabled || props.readonly {
        return;
    }
    let value = props.value.as_ref();
    let mut new_value = String::with_capacity(value.len() + text.len());
    new_value.push_str(&value[..range.start.min(value.len())]);
    new_value.push_str(text);
    new_value.push_str(&value[range.end.min(value.len())..]);
    let Some(new_value) = clamped_edit(&props, &new_value) else {
        return;
    };
    let caret = range.start + text.len();
    let focus = state.read(cx).focus.clone();
    state.update(cx, |state, cx| {
        state.caret = caret;
        state.anchor = caret;
        state.marked = None;
        cx.notify();
    });
    // Keep the value the next edit builds on, in case the app has not echoed
    // the reported one back yet.
    *state.read(cx).props.borrow_mut() = InputProps {
        value: new_value.clone(),
        ..props
    };
    let handlers = state.read(cx).handlers.borrow().clone();
    if let Some(handlers) = handlers
        && let Some(oninput) = &handlers.oninput
    {
        oninput(new_value, window, cx);
    }
    window.notify_text_input_changed(&focus);
}

/// Moves the caret, optionally extending the selection, and tells native text
/// services.
fn move_caret(
    state: &gpui::Entity<InputState>,
    caret: usize,
    extend: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let focus = state.read(cx).focus.clone();
    state.update(cx, |state, cx| {
        state.caret = caret;
        if !extend {
            state.anchor = caret;
        }
        cx.notify();
    });
    window.notify_text_selection_changed(&focus);
}

/// The byte offset in the value nearest a window position, using the measured
/// text.
fn position_index(
    state: &gpui::Entity<InputState>,
    position: gpui::Point<Pixels>,
    display: &SharedString,
    cx: &App,
) -> usize {
    let state = state.read(cx);
    let value = state.props.borrow().value.clone();
    let byte = state
        .layout
        .borrow()
        .index_for_position(position)
        .unwrap_or_else(|err| err);
    display_byte_to_value(&value, display, byte)
}

/// Sets a selection from a click or drag: the nearest text position to the
/// pointer, extending the selection when asked.
fn click_position(
    state: &gpui::Entity<InputState>,
    position: gpui::Point<Pixels>,
    extend: bool,
    display: &SharedString,
    window: &mut Window,
    cx: &mut App,
) {
    move_caret(
        state,
        position_index(state, position, display, cx),
        extend,
        window,
        cx,
    );
}

/// Returns the first and last byte offsets painted on the visual line at `y`.
fn visual_line_extrema(layout: &TextLayout, text: &str, y: Pixels) -> Option<(usize, usize)> {
    let mut first = None;
    let mut last = None;
    for byte in text
        .char_indices()
        .map(|(byte, _)| byte)
        .chain(std::iter::once(text.len()))
    {
        if let Some(position) = layout.position_for_index(byte)
            && (position.y - y).abs() <= px(0.5)
        {
            first.get_or_insert(byte);
            last = Some(byte);
        }
    }
    first.zip(last)
}

/// Returns the byte offset on the visual line at `y` whose x position is
/// closest to `x`.
fn closest_index_on_line(layout: &TextLayout, text: &str, y: Pixels, x: Pixels) -> Option<usize> {
    let mut best = None;
    let mut best_distance = f32::INFINITY;
    for byte in text
        .char_indices()
        .map(|(byte, _)| byte)
        .chain(std::iter::once(text.len()))
    {
        if let Some(position) = layout.position_for_index(byte)
            && (position.y - y).abs() <= px(0.5)
        {
            let distance = (position.x - x).abs().as_f32();
            if distance < best_distance {
                best_distance = distance;
                best = Some(byte);
            }
        }
    }
    best
}

/// Moves the caret one visual line up or down in a textarea, preserving its
/// horizontal position where the target line has a matching glyph.
fn move_vertical(
    state: &gpui::Entity<InputState>,
    caret: usize,
    down: bool,
    extend: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let layout = state.read(cx).layout.borrow().clone();
    let value = state.read(cx).props.borrow().value.clone();
    let Some(origin) = layout.position_for_index(caret) else {
        return;
    };
    let target_y = if down {
        origin.y + layout.line_height()
    } else {
        origin.y - layout.line_height()
    };
    let target = closest_index_on_line(&layout, &value, target_y, origin.x).unwrap_or(if down {
        value.len()
    } else {
        0
    });
    move_caret(state, target, extend, window, cx);
}

/// Moves the caret to the start or end of the current visual line in a
/// textarea, including wrapped lines.
fn move_line_edge(
    state: &gpui::Entity<InputState>,
    caret: usize,
    end: bool,
    extend: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let layout = state.read(cx).layout.borrow().clone();
    let value = state.read(cx).props.borrow().value.clone();
    let Some(origin) = layout.position_for_index(caret) else {
        return;
    };
    let target = visual_line_extrema(&layout, &value, origin.y)
        .map(|(start, last)| if end { last } else { start })
        .unwrap_or(if end { value.len() } else { 0 });
    move_caret(state, target, extend, window, cx);
}

/// The range an IME call should replace. Linux and Wayland pass `None` for
/// both preedit updates and the final commit, so an existing marked range
/// takes precedence over the caret selection.
fn ime_replacement_range(
    state: &gpui::Entity<InputState>,
    range_utf16: Option<Range<usize>>,
    cx: &App,
) -> Range<usize> {
    if let Some(range_utf16) = range_utf16 {
        let value = state.read(cx).props.borrow().value.clone();
        return utf16_to_utf8(&value, range_utf16.start)..utf16_to_utf8(&value, range_utf16.end);
    }
    let state = state.read(cx);
    state.marked.clone().unwrap_or_else(|| state.selection())
}

/// Ends an IME composition and emits `compositionend` once. `final_text` is
/// supplied by commit calls; unmark-only paths report the marked text.
fn finish_composition(
    state: &gpui::Entity<InputState>,
    final_text: Option<&str>,
    window: &mut Window,
    cx: &mut App,
) -> bool {
    let (value, marked) = {
        let state = state.read(cx);
        (state.props.borrow().value.clone(), state.marked.clone())
    };
    let Some(marked) = marked else {
        return false;
    };
    let text = final_text
        .map(SharedString::from)
        .or_else(|| value.get(marked).map(SharedString::from));
    state.update(cx, |state, cx| {
        state.marked = None;
        cx.notify();
    });
    if let Some(text) = text {
        let handlers = state.read(cx).handlers.borrow().clone();
        if let Some(handlers) = handlers
            && let Some(oncompositionend) = &handlers.oncompositionend
        {
            oncompositionend(text, window, cx);
        }
    }
    true
}

/// The GPUI `InputHandler` for a field, so platform IMEs can compose into it.
/// Ranges are UTF-16, per the native text protocol.
struct InputHandlerImpl {
    state: gpui::Entity<InputState>,
}

impl gpui::InputHandler for InputHandlerImpl {
    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        cx: &mut App,
    ) -> Option<gpui::UTF16Selection> {
        let state = self.state.read(cx);
        let value = state.props.borrow().value.clone();
        let range = state.selection();
        Some(gpui::UTF16Selection {
            range: utf8_to_utf16(&value, range.start)..utf8_to_utf16(&value, range.end),
            reversed: state.anchor > state.caret,
        })
    }

    fn marked_text_range(&mut self, _window: &mut Window, cx: &mut App) -> Option<Range<usize>> {
        let state = self.state.read(cx);
        let value = state.props.borrow().value.clone();
        state
            .marked
            .as_ref()
            .map(|range| utf8_to_utf16(&value, range.start)..utf8_to_utf16(&value, range.end))
    }

    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut App,
    ) -> Option<String> {
        let value = self.state.read(cx).props.borrow().value.clone();
        let start = utf16_to_utf8(&value, range_utf16.start);
        let end = utf16_to_utf8(&value, range_utf16.end);
        value.get(start..end).map(|text| text.to_string())
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        window: &mut Window,
        cx: &mut App,
    ) {
        // Linux and Wayland commit with `None`; replace the marked preedit
        // range rather than inserting after it.
        let range = ime_replacement_range(&self.state, range_utf16, cx);
        finish_composition(&self.state, Some(text), window, cx);
        replace_range(&self.state, range, text, window, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut App,
    ) {
        let value = self.state.read(cx).props.borrow().value.clone();
        let was_marked = self.state.read(cx).marked.is_some();
        let range = ime_replacement_range(&self.state, range_utf16, cx);
        let props = self.state.read(cx).props.borrow().clone();
        let mut new_value = String::with_capacity(value.len() + new_text.len());
        new_value.push_str(&value[..range.start.min(value.len())]);
        new_value.push_str(new_text);
        new_value.push_str(&value[range.end.min(value.len())..]);
        let Some(new_value) = clamped_edit(&props, &new_value) else {
            return;
        };
        let marked_end = range.start + new_text.len();
        let selected = new_selected_range.map(|range| {
            utf16_to_utf8(&new_value, range.start)..utf16_to_utf8(&new_value, range.end)
        });
        let focus = self.state.read(cx).focus.clone();
        self.state.update(cx, |state, cx| {
            state.marked = Some(range.start..marked_end);
            match selected {
                Some(selected) => {
                    state.caret = selected.end;
                    state.anchor = selected.start;
                }
                None => {
                    state.caret = marked_end;
                    state.anchor = marked_end;
                }
            }
            cx.notify();
        });
        *self.state.read(cx).props.borrow_mut() = InputProps {
            value: new_value,
            ..props
        };
        // Upstream suppresses `input` while an IME composition is in progress;
        // the committed value is reported by `unmark_text` or the final
        // `replace_text_in_range` call.
        let handlers = self.state.read(cx).handlers.borrow().clone();
        if let Some(handlers) = handlers {
            if !was_marked && let Some(oncompositionstart) = &handlers.oncompositionstart {
                oncompositionstart(window, cx);
            }
            if let Some(oncompositionupdate) = &handlers.oncompositionupdate {
                oncompositionupdate(SharedString::from(new_text), window, cx);
            }
        }
        window.notify_text_input_changed(&focus);
        window.notify_text_selection_changed(&focus);
    }

    fn unmark_text(&mut self, window: &mut Window, cx: &mut App) {
        let focus = self.state.read(cx).focus.clone();
        if finish_composition(&self.state, None, window, cx) {
            let value = self.state.read(cx).props.borrow().value.clone();
            let handlers = self.state.read(cx).handlers.borrow().clone();
            if let Some(handlers) = handlers
                && let Some(oninput) = &handlers.oninput
            {
                oninput(value, window, cx);
            }
            window.notify_text_input_changed(&focus);
        }
        window.notify_text_selection_changed(&focus);
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _window: &mut Window,
        cx: &mut App,
    ) -> Option<gpui::Bounds<Pixels>> {
        let state = self.state.read(cx);
        let value = state.props.borrow().value.clone();
        let start = utf16_to_utf8(&value, range_utf16.start);
        let end = utf16_to_utf8(&value, range_utf16.end);
        let layout = state.layout.borrow().clone();
        let start_point = layout.position_for_index(start)?;
        let end_point = layout.position_for_index(end)?;
        Some(gpui::Bounds::new(
            start_point,
            gpui::size(
                (end_point.x - start_point.x).max(px(1.)),
                layout.line_height(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _window: &mut Window,
        cx: &mut App,
    ) -> Option<usize> {
        let state = self.state.read(cx);
        let value = state.props.borrow().value.clone();
        let byte = state
            .layout
            .borrow()
            .index_for_position(point)
            .unwrap_or_else(|err| err);
        Some(utf8_to_utf16(&value, byte))
    }
}

impl RenderOnce for Input {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Input {
            id,
            value: prop_value,
            placeholder,
            size,
            radius,
            active_border_color,
            outline,
            text_color: configured_text_color,
            textarea,
            rows,
            disabled,
            readonly,
            clearable,
            password,
            show_password,
            maxlength,
            show_word_limit,
            prefix,
            suffix,
            oninput,
            onchange,
            onfocus,
            onblur,
            onclear,
            oncompositionstart,
            oncompositionupdate,
            oncompositionend,
        } = self;
        let theme = Theme::current(cx);
        let Some(id) = id else {
            return div().into_any_element();
        };
        let state = window.use_keyed_state(id.clone(), cx, InputState::new);

        // Refresh the props and handlers the focus subscriptions read, without
        // notifying: this runs on every render. While an IME composition is
        // active, keep the optimistic composing text instead of replacing it
        // with the last committed prop value.
        {
            let state = state.read(cx);
            let value = if state.marked.is_some() {
                state.props.borrow().value.clone()
            } else {
                prop_value.clone()
            };
            *state.props.borrow_mut() = InputProps {
                value: value.clone(),
                maxlength,
                readonly,
                disabled,
            };
            *state.handlers.borrow_mut() = Some(Rc::new(InputHandlers {
                oninput,
                onchange,
                onfocus,
                onblur,
                onclear,
                oncompositionstart,
                oncompositionupdate,
                oncompositionend,
            }));
        }
        let value = prop_value;
        let value = if state.read(cx).marked.is_some() {
            state.read(cx).props.borrow().value.clone()
        } else {
            value
        };

        let focused = state.read(cx).focus.is_focused(window);
        let password_visible = state.read(cx).password_visible;
        let marked = state.read(cx).marked.clone();
        let len = value.len();
        let caret = state.read(cx).caret.min(len);
        let anchor = state.read(cx).anchor.min(len);
        let selection = caret.min(anchor)..caret.max(anchor);

        let display = if textarea {
            value.clone()
        } else {
            display_text(&value, password, password_visible)
        };
        let showing_placeholder = value.is_empty() && !placeholder.is_empty();

        let active_border = active_border_color.unwrap_or(rgb(crate::theme::PRIMARY));
        let border = if disabled {
            rgb(theme.border(true))
        } else if focused {
            active_border
        } else {
            rgb(theme.border(false))
        };
        let hover_border = rgb(if theme.dark {
            crate::theme::mix(theme.background(), 0xf5f8ff, 40)
        } else {
            0xc0c4cc
        });
        let exceed = show_word_limit
            && maxlength.is_some()
            && value.chars().count() > maxlength.unwrap_or(usize::MAX);
        let border = if exceed {
            rgb(crate::theme::DANGER)
        } else {
            border
        };
        let bg = if disabled {
            rgb(theme.fill(crate::theme::Fill::Light))
        } else {
            rgb(theme.background())
        };
        let text_color = if disabled {
            rgb(theme.disabled_text())
        } else {
            configured_text_color.unwrap_or(rgb(theme.regular_text()))
        };
        let caret_color = text_color;
        let placeholder_color = rgb(theme.disabled_text());
        let icon_color = rgb(theme.disabled_text());
        let line_height = size.font_size() * 1.5;
        let root_height = if textarea {
            rows.max(1) as f32 * line_height + 10. + tokens::BORDER_WIDTH * 2.
        } else {
            size.height()
        };
        let corner_radius = radius.pixels(root_height);

        // The text itself: laid out by the child, measured for the caret and
        // hit testing through the shared layout.
        let mut highlights = Vec::new();
        if !selection.is_empty() && !showing_placeholder {
            let display_selection = value_byte_to_display(&value, &display, selection.start)
                ..value_byte_to_display(&value, &display, selection.end);
            highlights.push((
                display_selection,
                gpui::HighlightStyle {
                    background_color: Some(gpui::hsla(210., 1., 0.63, 0.3)),
                    ..Default::default()
                },
            ));
        }
        if let Some(marked) = marked
            && !showing_placeholder
        {
            let display_marked = value_byte_to_display(&value, &display, marked.start)
                ..value_byte_to_display(&value, &display, marked.end);
            highlights.push((
                display_marked,
                gpui::HighlightStyle {
                    underline: Some(UnderlineStyle {
                        color: Some(gpui::Hsla::from(text_color)),
                        thickness: px(1.),
                        wavy: false,
                    }),
                    ..Default::default()
                },
            ));
        }
        let text = gpui::StyledText::new(if showing_placeholder {
            placeholder.clone()
        } else {
            display.clone()
        })
        .with_highlights(highlights);
        *state.read(cx).layout.borrow_mut() = text.layout().clone();

        let text_color = if showing_placeholder {
            placeholder_color
        } else {
            text_color
        };

        let mut suffix_items: Vec<AnyElement> = Vec::new();
        if show_word_limit {
            let count = if disabled {
                rgb(theme.disabled_text())
            } else if exceed {
                rgb(crate::theme::DANGER)
            } else {
                rgb(theme.secondary_text())
            };
            suffix_items.push(
                div()
                    .text_size(px(12.))
                    .text_color(count)
                    .child(SharedString::from(format!(
                        "{} / {}",
                        value.chars().count(),
                        maxlength.map(|max| max.to_string()).unwrap_or_default()
                    )))
                    .into_any_element(),
            );
        }
        if clearable && !disabled && !readonly {
            suffix_items.push(
                div()
                    .id(ElementId::NamedChild(Arc::new(id.clone()), "clear".into()))
                    .flex()
                    .items_center()
                    .cursor_pointer()
                    .child(el_icon("circle-close").size(size.font_size()))
                    .text_color(icon_color)
                    .hover(|style| style.text_color(rgb(theme.secondary_text())))
                    .when(focused, |el| el.text_color(rgb(theme.secondary_text())))
                    .when(value.is_empty(), |el| el.opacity(0.))
                    .on_mouse_down(MouseButton::Left, {
                        let state = state.clone();
                        move |_event, window, cx| {
                            cx.stop_propagation();
                            let props = state.read(cx).props.borrow().clone();
                            if props.disabled || props.readonly || props.value.is_empty() {
                                return;
                            }
                            let focus = state.read(cx).focus.clone();
                            state.update(cx, |state, cx| {
                                state.caret = 0;
                                state.anchor = 0;
                                cx.notify();
                            });
                            *state.read(cx).props.borrow_mut() = InputProps {
                                value: "".into(),
                                ..props
                            };
                            let handlers = state.read(cx).handlers.borrow().clone();
                            if let Some(handlers) = handlers {
                                if let Some(oninput) = &handlers.oninput {
                                    oninput("".into(), window, cx);
                                }
                                if let Some(onchange) = &handlers.onchange {
                                    onchange("".into(), window, cx);
                                }
                                if let Some(onclear) = &handlers.onclear {
                                    onclear("".into(), window, cx);
                                }
                            }
                            window.notify_text_input_changed(&focus);
                        }
                    })
                    .into_any_element(),
            );
        }
        if show_password && password && !disabled {
            let visible = password_visible;
            suffix_items.push(
                div()
                    .id(ElementId::NamedChild(
                        Arc::new(id.clone()),
                        "password".into(),
                    ))
                    .flex()
                    .items_center()
                    .cursor_pointer()
                    .child(el_icon(if visible { "view" } else { "hide" }).size(size.font_size()))
                    .text_color(icon_color)
                    .hover(|style| style.text_color(rgb(theme.secondary_text())))
                    .when(value.is_empty() && !focused, |el| el.opacity(0.))
                    .on_mouse_down(MouseButton::Left, {
                        let state = state.clone();
                        move |_event, _window, cx| {
                            cx.stop_propagation();
                            state.update(cx, |state, cx| {
                                state.password_visible = !state.password_visible;
                                cx.notify();
                            });
                        }
                    })
                    .into_any_element(),
            );
        }
        if let Some(suffix) = suffix {
            suffix_items.push(suffix);
        }
        let has_suffix = !suffix_items.is_empty();

        let handler_state = state.clone();
        let root = div()
            .id(id.clone())
            .track_focus(&state.read(cx).focus)
            .tab_stop(true)
            .flex()
            .items_center()
            .when(textarea, |el| el.items_stretch())
            .h(px(root_height))
            .w_full()
            .px(px(size.padding() - tokens::BORDER_WIDTH))
            .when(textarea, |el| el.py(px(5.)))
            .rounded(px(corner_radius))
            .border_1()
            .border_color(border)
            .bg(bg)
            .overflow_hidden()
            .text_size(px(size.font_size()))
            .when(focused && outline && !disabled, |el| {
                el.shadow(vec![gpui::BoxShadow {
                    color: gpui::Hsla::from(border),
                    offset: gpui::point(px(0.), px(0.)),
                    blur_radius: px(0.),
                    spread_radius: px(2.),
                    inset: false,
                }])
            })
            .when(!disabled && !readonly, |el| {
                el.cursor_text().when(!focused, |el| {
                    el.hover(|style| style.border_color(hover_border))
                })
            })
            .when(disabled, |el| el.cursor_not_allowed())
            .on_mouse_down(MouseButton::Left, {
                let state = state.clone();
                let display = display.clone();
                move |event: &MouseDownEvent, window, cx| {
                    if disabled {
                        return;
                    }
                    let focus = state.read(cx).focus.clone();
                    focus.focus(window, cx);
                    let extend = event.modifiers.shift;
                    match event.click_count {
                        // Double-click selects the word at the click, like a
                        // native field; triple-click selects everything.
                        2 => {
                            let value = state.read(cx).props.borrow().value.clone();
                            let position = position_index(&state, event.position, &display, cx);
                            let start = word_boundary(&value, position, false);
                            let end = word_boundary(&value, position, true);
                            let focus = state.read(cx).focus.clone();
                            state.update(cx, |state, cx| {
                                state.caret = end;
                                state.anchor = start;
                                cx.notify();
                            });
                            window.notify_text_selection_changed(&focus);
                        }
                        3 => {
                            let len = state.read(cx).props.borrow().value.len();
                            let focus = state.read(cx).focus.clone();
                            state.update(cx, |state, cx| {
                                state.caret = len;
                                state.anchor = 0;
                                cx.notify();
                            });
                            window.notify_text_selection_changed(&focus);
                        }
                        _ => click_position(&state, event.position, extend, &display, window, cx),
                    }
                    cx.stop_propagation();
                }
            })
            .on_mouse_move({
                let state = state.clone();
                let display = display.clone();
                move |event: &MouseMoveEvent, window, cx| {
                    if disabled || event.pressed_button != Some(MouseButton::Left) {
                        return;
                    }
                    // A drag extends the selection from the press anchor.
                    click_position(&state, event.position, true, &display, window, cx);
                }
            })
            .on_key_down({
                let state = state.clone();
                move |event: &KeyDownEvent, window, cx| {
                    if disabled {
                        return;
                    }
                    let key = event.keystroke.key.as_str();
                    let modifiers = event.keystroke.modifiers;
                    let editable = !readonly && state.read(cx).marked.is_none();
                    let value = state.read(cx).props.borrow().value.clone();
                    let selection = state.read(cx).selection();
                    let caret = state.read(cx).caret.min(value.len());
                    let handled = match key {
                        "backspace" if editable => {
                            delete(
                                &state,
                                caret,
                                selection,
                                false,
                                modifiers.control,
                                window,
                                cx,
                            );
                            true
                        }
                        "delete" if editable => {
                            delete(
                                &state,
                                caret,
                                selection,
                                true,
                                modifiers.control,
                                window,
                                cx,
                            );
                            true
                        }
                        "left" => {
                            let target = if modifiers.control {
                                word_boundary(&value, caret, false)
                            } else {
                                char_boundary(&value, caret, false)
                            };
                            move_caret(&state, target, modifiers.shift, window, cx);
                            true
                        }
                        "right" => {
                            let target = if modifiers.control {
                                word_boundary(&value, caret, true)
                            } else {
                                char_boundary(&value, caret, true)
                            };
                            move_caret(&state, target, modifiers.shift, window, cx);
                            true
                        }
                        "up" if textarea => {
                            move_vertical(&state, caret, false, modifiers.shift, window, cx);
                            true
                        }
                        "down" if textarea => {
                            move_vertical(&state, caret, true, modifiers.shift, window, cx);
                            true
                        }
                        "home" => {
                            if textarea {
                                move_line_edge(&state, caret, false, modifiers.shift, window, cx);
                            } else {
                                move_caret(&state, 0, modifiers.shift, window, cx);
                            }
                            true
                        }
                        "end" => {
                            if textarea {
                                move_line_edge(&state, caret, true, modifiers.shift, window, cx);
                            } else {
                                move_caret(&state, value.len(), modifiers.shift, window, cx);
                            }
                            true
                        }
                        "enter" if textarea && editable => {
                            replace_range(&state, selection, "\n", window, cx);
                            true
                        }
                        "a" if modifiers.control => {
                            let focus = state.read(cx).focus.clone();
                            state.update(cx, |state, cx| {
                                state.caret = state.props.borrow().value.len();
                                state.anchor = 0;
                                cx.notify();
                            });
                            window.notify_text_selection_changed(&focus);
                            true
                        }
                        "c" if modifiers.control => {
                            if !selection.is_empty() {
                                cx.write_to_clipboard(ClipboardItem::new_string(
                                    value[selection].to_string(),
                                ));
                            }
                            true
                        }
                        "x" if modifiers.control && editable => {
                            if !selection.is_empty() {
                                cx.write_to_clipboard(ClipboardItem::new_string(
                                    value[selection.clone()].to_string(),
                                ));
                                replace_range(&state, selection, "", window, cx);
                            }
                            true
                        }
                        "v" if modifiers.control && editable => {
                            if let Some(item) = cx.read_from_clipboard()
                                && let Some(text) = item.text()
                            {
                                replace_range(&state, selection, &text, window, cx);
                            }
                            true
                        }
                        _ => {
                            // Printable characters arrive as `key_char`; the IME
                            // path handles keys while composing.
                            if !editable
                                || modifiers.control
                                || modifiers.alt
                                || modifiers.platform
                                || state.read(cx).marked.is_some()
                            {
                                false
                            } else {
                                match event.keystroke.key_char.as_ref() {
                                    Some(typed) => {
                                        let mut characters = typed.chars();
                                        match characters.next() {
                                            Some(character)
                                                if characters.next().is_none()
                                                    && !character.is_control() =>
                                            {
                                                replace_range(
                                                    &state,
                                                    selection,
                                                    &character.to_string(),
                                                    window,
                                                    cx,
                                                );
                                                true
                                            }
                                            _ => false,
                                        }
                                    }
                                    None => false,
                                }
                            }
                        }
                    };
                    if handled {
                        cx.stop_propagation();
                    }
                }
            })
            .when_some(prefix, |el, prefix| {
                el.child(
                    div()
                        .flex()
                        .items_center()
                        .mr(px(8.))
                        .text_color(icon_color)
                        .child(prefix),
                )
            })
            .child(
                div()
                    .id(ElementId::NamedChild(Arc::new(id.clone()), "text".into()))
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .when(textarea, |el| {
                        el.flex()
                            .flex_col()
                            .justify_start()
                            .whitespace_normal()
                            .line_height(px(line_height))
                            .overflow_scroll()
                            .track_scroll(&state.read(cx).scroll)
                            .on_scroll(|_, _, cx| cx.stop_propagation())
                            .when(has_suffix, |el| el.pr(px(24.)))
                    })
                    .when(!textarea, |el| el.flex().items_center().whitespace_nowrap())
                    .text_color(text_color)
                    .child(text),
            )
            .children({
                let suffix = div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .children(suffix_items)
                    .when(textarea, |el| {
                        el.absolute()
                            .right(px(11.))
                            .top(px(5.))
                            .flex_col()
                            .items_end()
                    })
                    .when(!textarea, |el| el.ml(px(8.)));
                if has_suffix { Some(suffix) } else { None }
            })
            .child(
                // Keeps the textarea caret visible, paints the caret, and
                // registers the IME handler after the text child has measured
                // itself.
                gpui::canvas(
                    {
                        let state = state.clone();
                        move |_, window, cx| {
                            if !textarea {
                                return;
                            }
                            let state = state.read(cx);
                            if !state.focus.is_focused(window) {
                                return;
                            }
                            let props = state.props.borrow().clone();
                            if props.disabled {
                                return;
                            }
                            let layout = state.layout.borrow().clone();
                            let caret = state.caret.min(props.value.len());
                            let Some(origin) = layout.position_for_index(caret) else {
                                return;
                            };
                            let scroll = state.scroll.clone();
                            let viewport = scroll.bounds();
                            let max_offset = scroll.max_offset();
                            let mut offset = scroll.offset();
                            let line_height = layout.line_height();
                            if origin.y < viewport.top() {
                                offset.y += viewport.top() - origin.y;
                            } else if origin.y + line_height > viewport.bottom() {
                                offset.y += viewport.bottom() - origin.y - line_height;
                            }
                            offset.y = offset.y.clamp(-max_offset.y, px(0.));
                            if offset != scroll.offset() {
                                scroll.set_offset(offset);
                            }
                        }
                    },
                    {
                        let state = state.clone();
                        move |_bounds, _, window, cx| {
                            let state = state.read(cx);
                            let focused = state.focus.is_focused(window);
                            let props = state.props.borrow().clone();
                            let display = if textarea {
                                props.value.clone()
                            } else {
                                display_text(&props.value, password, state.password_visible)
                            };
                            if caret_visible(focused, props.disabled, props.readonly) {
                                let layout = state.layout.borrow().clone();
                                let caret = value_byte_to_display(
                                    &props.value,
                                    &display,
                                    state.caret.min(props.value.len()),
                                );
                                if let Some(origin) = layout.position_for_index(caret) {
                                    window.paint_quad(gpui::fill(
                                        gpui::Bounds::new(
                                            origin,
                                            gpui::size(px(1.5), layout.line_height()),
                                        ),
                                        caret_color,
                                    ));
                                }
                            }
                            if focused && !props.disabled && !props.readonly {
                                window.handle_input(
                                    &state.focus,
                                    InputHandlerImpl {
                                        state: handler_state.clone(),
                                    },
                                    cx,
                                );
                            }
                        }
                    },
                )
                .absolute()
                .inset_0(),
            );

        root.into_any_element()
    }
}

/// Deletes the selection, or one character or word beside the caret.
fn delete(
    state: &gpui::Entity<InputState>,
    caret: usize,
    selection: Range<usize>,
    forward: bool,
    word: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let value = state.read(cx).props.borrow().value.clone();
    let range = if !selection.is_empty() {
        selection
    } else {
        let target = if word {
            word_boundary(&value, caret, forward)
        } else {
            char_boundary(&value, caret, forward)
        };
        if forward {
            caret..target
        } else {
            target..caret
        }
    };
    replace_range(state, range, "", window, cx);
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Modifiers;

    #[test]
    fn sizes_map_to_upstream_tokens() {
        assert_eq!(InputSize::Large.height(), 40.);
        assert_eq!(InputSize::Default.height(), 32.);
        assert_eq!(InputSize::Small.height(), 24.);
        assert_eq!(InputSize::Large.padding(), 16.);
        assert_eq!(InputSize::Default.padding(), 12.);
        assert_eq!(InputSize::Small.padding(), 8.);
        assert_eq!(InputSize::Small.font_size(), 12.);
        assert_eq!(InputSize::Default.font_size(), 14.);
    }

    #[test]
    fn utf16_round_trips_through_ascii_and_bmp_pairs() {
        // "a𝄞b": the surrogate pair takes two UTF-16 units.
        let text = "a𝄞b";
        assert_eq!(utf8_to_utf16(text, 0), 0);
        assert_eq!(utf8_to_utf16(text, 1), 1);
        assert_eq!(utf8_to_utf16(text, 5), 3);
        assert_eq!(utf16_to_utf8(text, 0), 0);
        assert_eq!(utf16_to_utf8(text, 1), 1);
        assert_eq!(utf16_to_utf8(text, 3), 5);
        assert_eq!(utf16_to_utf8(text, 10), 6);
    }

    #[test]
    fn char_boundaries_never_split_a_unicode_scalar() {
        let text = "a𝄞b";
        assert_eq!(char_boundary(text, 0, true), 1);
        assert_eq!(char_boundary(text, 1, true), 5);
        assert_eq!(char_boundary(text, 5, false), 1);
        assert_eq!(char_boundary(text, 4, false), 1);
    }

    #[test]
    fn word_boundaries_land_on_word_edges() {
        let text = "foo bar, baz";
        // Forward: from "foo" past the space to "bar", from "bar" to the
        // comma, from the comma to "baz".
        assert_eq!(word_boundary(text, 0, true), 4);
        assert_eq!(word_boundary(text, 4, true), 7);
        assert_eq!(word_boundary(text, 7, true), 9);
        // Backward: from the end to "baz", from "baz" to the comma, from
        // "bar" to "foo".
        assert_eq!(word_boundary(text, 12, false), 9);
        assert_eq!(word_boundary(text, 9, false), 7);
        assert_eq!(word_boundary(text, 4, false), 0);
    }

    #[test]
    fn masked_display_keeps_character_offsets() {
        let value: SharedString = "abc".into();
        let display = display_text(&value, true, false);
        assert_eq!(display.as_ref(), "•••");
        // Three bullets are nine bytes; the conversion still maps characters.
        assert_eq!(display_byte_to_value("abc", &display, 3), 1);
        assert_eq!(value_byte_to_display("abc", &display, 1), 3);
    }

    #[test]
    fn clamped_edit_enforces_maxlength_by_character() {
        let props = InputProps {
            value: "".into(),
            maxlength: Some(3),
            readonly: false,
            disabled: false,
        };
        assert_eq!(
            clamped_edit(&props, "ab").map(|value| value.to_string()),
            Some("ab".into())
        );
        assert!(clamped_edit(&props, "abcd").is_none());
        // A surrogate pair counts as one character, like the web attribute.
        let props = InputProps {
            value: "".into(),
            maxlength: Some(2),
            readonly: false,
            disabled: false,
        };
        assert!(clamped_edit(&props, "𝄞𝄞").is_some());
        assert!(clamped_edit(&props, "𝄞𝄞𝄞").is_none());
        let readonly = InputProps {
            value: "".into(),
            maxlength: None,
            readonly: true,
            disabled: false,
        };
        assert!(clamped_edit(&readonly, "a").is_none());
    }

    #[test]
    fn readonly_and_disabled_fields_do_not_paint_a_caret() {
        assert!(caret_visible(true, false, false));
        assert!(!caret_visible(false, false, false));
        assert!(!caret_visible(true, true, false));
        assert!(!caret_visible(true, false, true));
    }

    #[test]
    fn radius_profiles_match_the_requested_native_scale() {
        assert_eq!(InputRadius::Off.pixels(32.), 0.);
        assert_eq!(InputRadius::Small.pixels(32.), 4.);
        assert_eq!(InputRadius::Default.pixels(32.), 8.);
        assert_eq!(InputRadius::Large.pixels(32.), 12.);
        assert_eq!(InputRadius::Round.pixels(32.), 16.);
        assert_eq!(InputRadius::Round.pixels(54.), 27.);
    }

    #[test]
    fn style_and_mode_setters_are_independent() {
        let active = rgb(0x67c23a);
        let text = rgb(0x303133);
        let input = el_input()
            .small()
            .radius_off()
            .active_border_color(active)
            .outline(true)
            .text_color(text)
            .readonly(true)
            .textarea(true)
            .rows(4);
        assert_eq!(input.size, InputSize::Small);
        assert_eq!(input.radius, InputRadius::Off);
        assert_eq!(input.active_border_color, Some(active));
        assert!(input.outline);
        assert_eq!(input.text_color, Some(text));
        assert!(input.readonly);
        assert!(input.textarea);
        assert_eq!(input.rows, 4);
    }

    #[gpui::test]
    fn ime_composition_is_optimistic_and_commits_once(cx: &mut gpui::TestAppContext) {
        struct Harness {
            state: gpui::Entity<InputState>,
            reported: Rc<RefCell<Vec<String>>>,
            events: Rc<RefCell<Vec<String>>>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                div()
            }
        }
        let reported = Rc::new(RefCell::new(Vec::new()));
        let events = Rc::new(RefCell::new(Vec::new()));
        let (harness, view) = cx.add_window_view({
            let reported = reported.clone();
            let events = events.clone();
            move |window, cx| {
                let reported_for_handler = reported.clone();
                let events_for_handler = events.clone();
                let state = cx.new(|cx| InputState::new(window, cx));
                state.update(cx, |state, _| {
                    *state.props.borrow_mut() = InputProps {
                        value: "".into(),
                        maxlength: None,
                        readonly: false,
                        disabled: false,
                    };
                    *state.handlers.borrow_mut() = Some(Rc::new(InputHandlers {
                        oninput: Some(Rc::new(move |value, _, _| {
                            reported_for_handler.borrow_mut().push(value.to_string());
                        })),
                        onchange: None,
                        onfocus: None,
                        onblur: None,
                        onclear: None,
                        oncompositionstart: Some(Rc::new({
                            let events = events_for_handler.clone();
                            move |_, _| {
                                events.borrow_mut().push("start".into());
                            }
                        })),
                        oncompositionupdate: Some(Rc::new({
                            let events = events_for_handler.clone();
                            move |value, _, _| {
                                events.borrow_mut().push(format!("update:{value}"));
                            }
                        })),
                        oncompositionend: Some(Rc::new({
                            let events = events_for_handler.clone();
                            move |value, _, _| {
                                events.borrow_mut().push(format!("end:{value}"));
                            }
                        })),
                    }));
                });
                Harness {
                    state,
                    reported,
                    events,
                }
            }
        });
        use gpui::InputHandler as _;
        view.update(|window, cx| {
            let state = harness.read(cx).state.clone();
            let reported = harness.read(cx).reported.clone();
            let events = harness.read(cx).events.clone();
            let mut handler = InputHandlerImpl {
                state: state.clone(),
            };
            handler.replace_and_mark_text_in_range(Some(0..0), "你", None, window, cx);
            assert!(reported.borrow().is_empty());
            assert_eq!(state.read(cx).props.borrow().value.as_ref(), "你");
            assert_eq!(state.read(cx).marked, Some(0..3));

            // Linux and Wayland pass None again: replace the marked preedit,
            // not insert a second copy after it.
            handler.replace_and_mark_text_in_range(None, "你好", None, window, cx);
            assert!(reported.borrow().is_empty());
            assert_eq!(state.read(cx).props.borrow().value.as_ref(), "你好");
            assert_eq!(state.read(cx).marked, Some(0..6));
            assert_eq!(
                events.borrow().as_slice(),
                ["start", "update:你", "update:你好"]
            );

            handler.unmark_text(window, cx);
            assert_eq!(reported.borrow().as_slice(), ["你好"]);
            assert_eq!(events.borrow().last().map(String::as_str), Some("end:你好"));
            // A later unmark cannot report the same composition twice.
            handler.unmark_text(window, cx);
            assert_eq!(reported.borrow().len(), 1);

            // The platform's normal commit path is replace_text_in_range with
            // None, followed by unmark. It must replace the preedit and emit
            // end -> input exactly once.
            handler.replace_and_mark_text_in_range(None, "世", None, window, cx);
            handler.replace_text_in_range(None, "世界", window, cx);
            assert_eq!(state.read(cx).props.borrow().value.as_ref(), "你好世界");
            assert_eq!(state.read(cx).marked, None);
            assert_eq!(reported.borrow().as_slice(), ["你好", "你好世界"]);
            assert_eq!(events.borrow().last().map(String::as_str), Some("end:世界"));
            handler.unmark_text(window, cx);
            assert_eq!(reported.borrow().len(), 2);
        });
    }

    #[gpui::test]
    fn textarea_handles_newlines_and_visual_line_navigation(cx: &mut gpui::TestAppContext) {
        use std::cell::RefCell as StdCell;
        struct Harness(Rc<StdCell<String>>);
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let value = self.0.clone();
                let current = value.borrow().clone();
                div().w(px(300.)).child(
                    el_input()
                        .id("textarea")
                        .value(current)
                        .textarea(true)
                        .rows(3)
                        .oninput(move |reported, _, _| {
                            *value.borrow_mut() = reported.to_string();
                        }),
                )
            }
        }
        let value = Rc::new(StdCell::new("ab\ncd".into()));
        let (_, view) = cx.add_window_view({
            let value = value.clone();
            move |_, _| Harness(value)
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        view.simulate_mouse_down(
            gpui::point(px(20.), px(10.)),
            MouseButton::Left,
            Modifiers::default(),
        );
        press_key(view, "ctrl-a");
        press_key(view, "end");
        press_key(view, "up");
        type_text(view, "X");
        assert_eq!(value.borrow().as_str(), "abX\ncd");

        press_key(view, "down");
        type_text(view, "Y");
        assert_eq!(value.borrow().as_str(), "abX\ncdY");

        press_key(view, "home");
        type_text(view, "Z");
        assert_eq!(value.borrow().as_str(), "abX\nZcdY");

        press_key(view, "enter");
        assert_eq!(value.borrow().as_str(), "abX\nZ\ncdY");
    }

    fn press_key(view: &mut gpui::VisualTestContext, key: &str) {
        let keystroke = view
            .update(|_, cx| gpui::Keystroke::parse_with_os(key, cx.platform_info().os).unwrap());
        view.simulate_event(gpui::KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        view.simulate_event(gpui::KeyUpEvent { keystroke });
    }

    #[gpui::test]
    fn typing_selecting_and_clearing_report_through_the_handlers(cx: &mut gpui::TestAppContext) {
        use std::cell::RefCell as StdCell;
        struct Harness {
            value: Rc<StdCell<String>>,
            committed: Rc<StdCell<String>>,
            cleared: Rc<StdCell<bool>>,
        }
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let value = self.value.clone();
                let committed = self.committed.clone();
                let cleared = self.cleared.clone();
                let current = value.borrow().clone();
                div().id("input-harness").w(px(400.)).child(
                    el_input()
                        .id("typed")
                        .value(current)
                        .placeholder("Type here")
                        .clearable(true)
                        .oninput(move |reported, _, _| *value.borrow_mut() = reported.to_string())
                        .onchange(move |reported, _, _| {
                            *committed.borrow_mut() = reported.to_string()
                        })
                        .onclear(move |_, _, _| *cleared.borrow_mut() = true),
                )
            }
        }
        let value = Rc::new(StdCell::new(String::new()));
        let committed = Rc::new(StdCell::new(String::new()));
        let cleared = Rc::new(StdCell::new(false));
        let (_, view) = cx.add_window_view({
            let value = value.clone();
            let committed = committed.clone();
            let cleared = cleared.clone();
            move |_, _| Harness {
                value,
                committed,
                cleared,
            }
        });
        let draw = |view: &mut gpui::VisualTestContext| {
            view.update(|window, cx| window.draw(cx).clear(cx));
        };
        draw(view);

        // Click the field to focus it, then type.
        view.simulate_mouse_down(
            gpui::point(px(20.), px(16.)),
            MouseButton::Left,
            Modifiers::default(),
        );
        type_text(view, "hi");
        assert_eq!(value.borrow().as_str(), "hi");

        // Select all with Ctrl+A, then typing replaces the selection.
        let ctrl_a = view.update(|_, cx| {
            gpui::Keystroke::parse_with_os("ctrl-a", cx.platform_info().os).unwrap()
        });
        view.simulate_event(gpui::KeyDownEvent {
            keystroke: ctrl_a,
            is_held: false,
            prefer_character_input: false,
        });
        type_text(view, "yo");
        assert_eq!(value.borrow().as_str(), "yo");

        // Backspace deletes, and the left arrow then moves the caret without
        // editing.
        let backspace = view.update(|_, cx| {
            gpui::Keystroke::parse_with_os("backspace", cx.platform_info().os).unwrap()
        });
        view.simulate_event(gpui::KeyDownEvent {
            keystroke: backspace,
            is_held: false,
            prefer_character_input: false,
        });
        assert_eq!(value.borrow().as_str(), "y");

        // The clear button sits at the field's right edge.
        view.simulate_mouse_down(
            gpui::point(px(382.), px(16.)),
            MouseButton::Left,
            Modifiers::default(),
        );
        assert_eq!(value.borrow().as_str(), "");
        assert!(*cleared.borrow());

        // Blur reports the committed value.
        view.simulate_mouse_down(
            gpui::point(px(20.), px(80.)),
            MouseButton::Left,
            Modifiers::default(),
        );
        assert_eq!(committed.borrow().as_str(), "");
    }

    fn type_text(view: &mut gpui::VisualTestContext, text: &str) {
        for char in text.chars() {
            view.simulate_event(gpui::KeyDownEvent {
                keystroke: gpui::Keystroke {
                    modifiers: Modifiers::default(),
                    key: char.to_string(),
                    key_char: Some(char.to_string()),
                },
                is_held: false,
                prefer_character_input: false,
            });
        }
    }
}
