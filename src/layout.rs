use gpui::{
    AnyElement, App, Div, ElementId, Hsla, IntoElement, ParentElement, RenderOnce, Window, div,
    prelude::*, px,
};

const REGION_PADDING: f32 = 20.;
const BAR_HEIGHT: f32 = 60.;
const ASIDE_WIDTH: f32 = 300.;

fn valid_length(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value >= 0. {
        value
    } else {
        fallback
    }
}

/// Layout direction shared by Container and Space.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

pub fn el_container() -> Container {
    Container::default()
}
pub fn el_header() -> Header {
    Header::default()
}
pub fn el_footer() -> Footer {
    Footer::default()
}
#[track_caller]
pub fn el_aside() -> Aside {
    Aside::new(ElementId::CodeLocation(*std::panic::Location::caller()))
}
#[track_caller]
pub fn el_main() -> Main {
    Main::new(ElementId::CodeLocation(*std::panic::Location::caller()))
}
pub fn el_space() -> Space {
    Space::default()
}

/// Flex container. Direct Header or Footer children select vertical layout unless overridden.
#[derive(Default, IntoElement)]
pub struct Container {
    direction: Option<Direction>,
    children: Vec<AnyElement>,
}

impl Container {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = Some(direction);
        self
    }
    pub fn vertical(self) -> Self {
        self.direction(Direction::Vertical)
    }
    fn is_vertical(&mut self) -> bool {
        match self.direction {
            Some(direction) => direction == Direction::Vertical,
            None => self.children.iter_mut().any(|child| {
                child
                    .downcast_mut::<<Header as IntoElement>::Element>()
                    .is_some()
                    || child
                        .downcast_mut::<<Footer as IntoElement>::Element>()
                        .is_some()
            }),
        }
    }
}

impl ParentElement for Container {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}

impl RenderOnce for Container {
    fn render(mut self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let vertical = self.is_vertical();
        div()
            .flex()
            .flex_auto()
            .min_w_0()
            .min_h_0()
            .when(vertical, Div::flex_col)
            .children(self.children)
    }
}

macro_rules! bar {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(IntoElement)]
        pub struct $name {
            height: f32,
            background: Option<Hsla>,
            children: Vec<AnyElement>,
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    height: BAR_HEIGHT,
                    background: None,
                    children: Vec::new(),
                }
            }
        }

        impl $name {
            pub fn new() -> Self {
                Self::default()
            }
            /// Overrides the region height in logical pixels.
            pub fn height(mut self, height: f32) -> Self {
                self.height = valid_length(height, BAR_HEIGHT);
                self
            }
            /// Fills the entire region, including its padding.
            pub fn background(mut self, color: impl Into<Hsla>) -> Self {
                self.background = Some(color.into());
                self
            }
        }

        impl ParentElement for $name {
            fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
                self.children.extend(children);
            }
        }

        impl RenderOnce for $name {
            fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
                div()
                    .block()
                    .flex_shrink_0()
                    .min_w_0()
                    .px(px(REGION_PADDING))
                    .h(px(self.height))
                    .when_some(self.background, |el, color| el.bg(color))
                    .children(self.children)
            }
        }
    };
}

bar!(
    Header,
    "Container header, 60 logical pixels high by default."
);
bar!(
    Footer,
    "Container footer, 60 logical pixels high by default."
);

/// Fixed-width scrollable container region, 300 logical pixels wide by default.
#[derive(IntoElement)]
pub struct Aside {
    id: ElementId,
    width: f32,
    background: Option<Hsla>,
    end_border: Option<Hsla>,
    children: Vec<AnyElement>,
}

impl Default for Aside {
    fn default() -> Self {
        Self::new("el-aside")
    }
}

impl Aside {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            width: ASIDE_WIDTH,
            background: None,
            end_border: None,
            children: Vec::new(),
        }
    }
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
    /// Width in logical pixels; invalid values restore 300 px.
    pub fn width(mut self, width: f32) -> Self {
        self.width = valid_length(width, ASIDE_WIDTH);
        self
    }
    /// Fills the entire side region.
    pub fn background(mut self, color: impl Into<Hsla>) -> Self {
        self.background = Some(color.into());
        self
    }
    /// Adds a full-height right divider to this side region.
    pub fn end_border(mut self, color: impl Into<Hsla>) -> Self {
        self.end_border = Some(color.into());
        self
    }
}

impl ParentElement for Aside {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}

impl RenderOnce for Aside {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .id(self.id)
            .flex_shrink_0()
            .min_h_0()
            .w(px(self.width))
            .when_some(self.background, |el, color| el.bg(color))
            .when_some(self.end_border, |el, color| {
                el.border_r_1().border_color(color)
            })
            .overflow_scroll()
            .children(self.children)
    }
}

/// Flexible, scrollable main region, with 20 logical pixels of padding by default.
#[derive(IntoElement)]
pub struct Main {
    id: ElementId,
    padding: f32,
    children: Vec<AnyElement>,
}

impl Default for Main {
    fn default() -> Self {
        Self::new("el-main")
    }
}

impl Main {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            padding: REGION_PADDING,
            children: Vec::new(),
        }
    }
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
    /// Padding in logical pixels; invalid values restore 20 px.
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = valid_length(padding, REGION_PADDING);
        self
    }
}

impl ParentElement for Main {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}

impl RenderOnce for Main {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .id(self.id)
            .flex_auto()
            .min_w_0()
            .min_h_0()
            .p(px(self.padding))
            .overflow_scroll()
            .children(self.children)
    }
}

/// Spacing presets from Element Plus Space, not the control-size scale.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SpaceSize {
    #[default]
    Small,
    Default,
    Large,
}

impl SpaceSize {
    fn gap(self) -> f32 {
        use crate::tokens::spacing;
        match self {
            Self::Small => spacing::SMALL,
            Self::Default => spacing::DEFAULT,
            Self::Large => spacing::LARGE,
        }
    }
}

/// Cross-axis alignment for Space.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SpaceAlignment {
    Start,
    #[default]
    Center,
    End,
    Stretch,
    Baseline,
}

/// Horizontal element spacing by default; supports wrapping and independent x/y gaps.
#[derive(Default, IntoElement)]
pub struct Space {
    size: SpaceSize,
    gap: Option<(f32, f32)>,
    vertical: bool,
    wrap: bool,
    alignment: SpaceAlignment,
    fill: bool,
    fill_ratio: Option<f32>,
    spacer: Option<Box<dyn Fn() -> AnyElement>>,
    children: Vec<AnyElement>,
}

impl Space {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn size(mut self, size: SpaceSize) -> Self {
        self.size = size;
        self
    }
    /// Uses the small spacing preset (8 logical pixels).
    pub fn small(self) -> Self {
        self.size(SpaceSize::Small)
    }
    /// Uses the large spacing preset (16 logical pixels).
    pub fn large(self) -> Self {
        self.size(SpaceSize::Large)
    }
    /// Sets horizontal and vertical gaps in logical pixels, overriding the preset.
    pub fn gap(mut self, horizontal: f32, vertical: f32) -> Self {
        self.gap = Some((horizontal, vertical));
        self
    }
    pub fn vertical(mut self) -> Self {
        self.vertical = true;
        self
    }
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }
    pub fn alignment(mut self, value: SpaceAlignment) -> Self {
        self.alignment = value;
        self
    }
    pub fn direction(mut self, value: Direction) -> Self {
        self.vertical = value == Direction::Vertical;
        self
    }
    pub fn fill(mut self, value: bool) -> Self {
        self.fill = value;
        self
    }
    /// Minimum percentage width per item when filling (0–100, default 100).
    pub fn fill_ratio(mut self, value: f32) -> Self {
        self.fill_ratio = (value.is_finite() && (0. ..=100.).contains(&value)).then_some(value);
        self
    }
    pub fn spacer<E: IntoElement + 'static>(mut self, draw: impl Fn() -> E + 'static) -> Self {
        self.spacer = Some(Box::new(move || draw().into_any_element()));
        self
    }
    fn gaps(&self) -> (f32, f32) {
        let value = self.size.gap();
        let default = if self.vertical {
            (0., value)
        } else if self.wrap || self.fill {
            (value, value)
        } else {
            (value, 0.)
        };
        self.gap.map_or(default, |(x, y)| {
            (valid_length(x, default.0), valid_length(y, default.1))
        })
    }
}

impl ParentElement for Space {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}

impl RenderOnce for Space {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let (x, y) = self.gaps();
        let count = self.children.len();
        let children = self
            .children
            .into_iter()
            .enumerate()
            .flat_map(|(i, child)| {
                let item = div()
                    .when(self.fill, |el| {
                        el.flex_grow(1.)
                            .min_w(gpui::relative(self.fill_ratio.unwrap_or(100.) / 100.))
                    })
                    .child(child)
                    .into_any_element();
                [
                    Some(item),
                    (i + 1 < count)
                        .then(|| self.spacer.as_ref().map(|draw| draw()))
                        .flatten(),
                ]
                .into_iter()
                .flatten()
            });
        div()
            .flex()
            .map(|el| match self.alignment {
                SpaceAlignment::Start => el.items_start(),
                SpaceAlignment::Center => el.items_center(),
                SpaceAlignment::End => el.items_end(),
                SpaceAlignment::Stretch => el.items_stretch(),
                SpaceAlignment::Baseline => el.items_baseline(),
            })
            .gap_x(px(x))
            .gap_y(px(y))
            .when(self.vertical, Div::flex_col)
            .when(self.wrap || self.fill, Div::flex_wrap)
            .children(children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_auto_detects_direct_regions_and_honors_override() {
        assert!(el_container().child(el_header()).is_vertical());
        assert!(el_container().child(el_footer()).is_vertical());
        assert!(
            !el_container()
                .child(el_container().child(el_header()))
                .is_vertical()
        );
        assert!(!el_container().child(el_aside()).is_vertical());
        assert!(
            !el_container()
                .direction(Direction::Horizontal)
                .child(el_header())
                .is_vertical()
        );
    }

    #[gpui::test]
    fn nested_regions_share_width_and_fill_sidebar_height(cx: &mut gpui::TestAppContext) {
        use gpui::{Bounds, Context, Pixels, Render, canvas};
        use std::{cell::RefCell, rc::Rc};

        type Rects = Rc<RefCell<[Option<Bounds<Pixels>>; 4]>>;
        struct Probe(Rects);
        impl Render for Probe {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let measure = |i: usize| {
                    let rects = self.0.clone();
                    canvas(
                        move |bounds, _, _| rects.borrow_mut()[i] = Some(bounds),
                        |_, _, _, _| {},
                    )
                    .size_full()
                };
                div().flex().w(px(380.)).h(px(270.)).child(
                    el_container()
                        .child(el_header().child(measure(0)))
                        .child(
                            el_container()
                                .child(el_aside().width(120.).child(measure(1)))
                                .child(el_main().padding(0.).child(measure(2))),
                        )
                        .child(el_footer().child(measure(3))),
                )
            }
        }
        let rects = Rects::default();
        let (_, view) = cx.add_window_view({
            let rects = rects.clone();
            move |_, _| Probe(rects)
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        let [header, aside, main, footer] = rects.borrow().map(Option::unwrap);
        assert_eq!(header.size, gpui::size(px(340.), px(60.)));
        assert_eq!(footer.size, header.size);
        assert_eq!(header.origin.x, aside.origin.x + px(20.));
        assert_eq!(aside.size, gpui::size(px(120.), px(150.)));
        assert_eq!(main.size, gpui::size(px(260.), px(150.)));
        assert_eq!(main.origin.x, aside.origin.x + aside.size.width);
        assert_eq!(aside.origin.y, header.origin.y + header.size.height);
        assert_eq!(footer.origin.y, aside.origin.y + aside.size.height);
    }

    #[test]
    fn spacing_presets_and_overrides_match_contract() {
        assert_eq!(Space::new().gaps(), (8., 0.));
        assert_eq!(el_space().small().gaps(), (8., 0.));
        assert_eq!(
            el_space().size(SpaceSize::Default).wrap(true).gaps(),
            (12., 12.)
        );
        assert_eq!(el_space().large().gaps(), (16., 0.));
        assert_eq!(el_space().large().small().gaps(), (8., 0.));
        assert_eq!(el_space().gap(5., 7.).large().gaps(), (5., 7.));
        assert_eq!(
            Space::new()
                .gap(0., f32::NAN)
                .size(SpaceSize::Large)
                .vertical()
                .gaps(),
            (0., 16.)
        );
    }
}
