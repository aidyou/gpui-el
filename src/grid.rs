use gpui::{
    AnyElement, App, Div, IntoElement, ParentElement, RenderOnce, Window, div, prelude::*, px,
    relative,
};

pub fn el_row() -> Row {
    Row::default()
}
pub fn el_col() -> Col {
    Col::default()
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RowJustify {
    #[default]
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RowAlign {
    #[default]
    Stretch,
    Top,
    Middle,
    Bottom,
}

/// Element Plus 24-column flex row. A gutter is split across each column and
/// offset on both edges of the row. Use `.col()` to attach typed columns.
#[derive(Default, IntoElement)]
pub struct Row {
    gutter: f32,
    justify: RowJustify,
    align: RowAlign,
    columns: Vec<Col>,
}

impl Row {
    /// Gutter in logical pixels. Negative/non-finite values reset to zero.
    pub fn gutter(mut self, gutter: f32) -> Self {
        self.gutter = if gutter.is_finite() {
            gutter.max(0.)
        } else {
            0.
        };
        self
    }
    pub fn justify(mut self, value: RowJustify) -> Self {
        self.justify = value;
        self
    }
    pub fn align(mut self, value: RowAlign) -> Self {
        self.align = value;
        self
    }
    pub fn col(mut self, col: Col) -> Self {
        self.columns.push(col);
        self
    }
    pub fn cols(mut self, cols: impl IntoIterator<Item = Col>) -> Self {
        self.columns.extend(cols);
        self
    }
}

impl RenderOnce for Row {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        let width = f32::from(window.viewport_size().width);
        let half = self.gutter / 2.;
        div()
            .flex()
            .flex_wrap()
            .mx(px(-half))
            .map(|el| match self.justify {
                RowJustify::Start => el.justify_start(),
                RowJustify::End => el.justify_end(),
                RowJustify::Center => el.justify_center(),
                RowJustify::SpaceBetween => el.justify_between(),
                RowJustify::SpaceAround => el.justify_around(),
                RowJustify::SpaceEvenly => el.justify_evenly(),
            })
            .map(|el| match self.align {
                RowAlign::Stretch => el.items_stretch(),
                RowAlign::Top => el.items_start(),
                RowAlign::Middle => el.items_center(),
                RowAlign::Bottom => el.items_end(),
            })
            .children(
                self.columns
                    .into_iter()
                    .filter_map(|col| col.resolve(width, half)),
            )
    }
}

/// Optional overrides at one breakpoint. Unspecified properties inherit the preceding breakpoint.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ColSpec {
    pub span: Option<u8>,
    pub offset: Option<u8>,
    pub push: Option<u8>,
    pub pull: Option<u8>,
}
impl From<u8> for ColSpec {
    fn from(span: u8) -> Self {
        Self {
            span: Some(span),
            ..Self::default()
        }
    }
}

#[derive(IntoElement)]
pub struct Col {
    base: ColSpec,
    breakpoints: [ColSpec; 5],
    children: Vec<AnyElement>,
}
impl Default for Col {
    fn default() -> Self {
        Self {
            base: 24.into(),
            breakpoints: [ColSpec::default(); 5],
            children: Vec::new(),
        }
    }
}
impl Col {
    pub fn span(mut self, span: u8) -> Self {
        self.base.span = Some(span);
        self
    }
    pub fn offset(mut self, offset: u8) -> Self {
        self.base.offset = Some(offset);
        self
    }
    pub fn push(mut self, push: u8) -> Self {
        self.base.push = Some(push);
        self
    }
    pub fn pull(mut self, pull: u8) -> Self {
        self.base.pull = Some(pull);
        self
    }
    pub fn xs(mut self, spec: impl Into<ColSpec>) -> Self {
        self.breakpoints[0] = spec.into();
        self
    }
    pub fn sm(mut self, spec: impl Into<ColSpec>) -> Self {
        self.breakpoints[1] = spec.into();
        self
    }
    pub fn md(mut self, spec: impl Into<ColSpec>) -> Self {
        self.breakpoints[2] = spec.into();
        self
    }
    pub fn lg(mut self, spec: impl Into<ColSpec>) -> Self {
        self.breakpoints[3] = spec.into();
        self
    }
    pub fn xl(mut self, spec: impl Into<ColSpec>) -> Self {
        self.breakpoints[4] = spec.into();
        self
    }

    fn at(&self, width: f32) -> ColSpec {
        let mut spec = self.base;
        let applies = [
            width < 768.,
            width >= 768.,
            width >= 992.,
            width >= 1200.,
            width >= 1920.,
        ];
        for (next, _) in self
            .breakpoints
            .iter()
            .zip(applies)
            .filter(|(_, applies)| *applies)
        {
            spec = ColSpec {
                span: next.span.or(spec.span),
                offset: next.offset.or(spec.offset),
                push: next.push.or(spec.push),
                pull: next.pull.or(spec.pull),
            };
        }
        spec
    }

    fn resolve(self, width: f32, half: f32) -> Option<Div> {
        let spec = self.at(width);
        let span = spec.span.unwrap_or(24).min(24);
        if span == 0 {
            return None;
        }
        let percent = |value: Option<u8>| f32::from(value.unwrap_or(0).min(24)) / 24.;
        let shift = percent(spec.push) - percent(spec.pull);
        Some(
            div()
                .flex_none()
                .w(relative(f32::from(span) / 24.))
                .ml(relative(percent(spec.offset)))
                .when(shift != 0., |el| el.relative().left(relative(shift)))
                .when(half > 0., |el| el.px(px(half)).min_h(px(1.)))
                .children(self.children),
        )
    }
}
impl ParentElement for Col {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}
impl RenderOnce for Col {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        self.resolve(f32::from(window.viewport_size().width), 0.)
            .map(IntoElement::into_any_element)
            .unwrap_or_else(|| div().hidden().into_any_element())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn grid_geometry_matches_flex_spans_gutters_and_placement(cx: &mut gpui::TestAppContext) {
        use gpui::{Bounds, Context, Pixels, Render};
        use std::{cell::RefCell, rc::Rc};

        type Rects = Rc<RefCell<[Option<Bounds<Pixels>>; 11]>>;
        struct Probe(Rects);
        impl Render for Probe {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let mark = |index: usize| {
                    let rects = self.0.clone();
                    gpui::canvas(
                        move |rect, _, _| rects.borrow_mut()[index] = Some(rect),
                        |_, _, _, _| {},
                    )
                    .h(px(if index == 9 { 40. } else { 20. }))
                };
                div()
                    .w(px(240.))
                    .flex()
                    .flex_col()
                    .child(
                        el_row()
                            .gutter(20.)
                            .cols((0..4).map(|i| el_col().span(6).child(mark(i)))),
                    )
                    .child(
                        el_row()
                            .col(el_col().span(6).offset(6).child(mark(4)))
                            .col(el_col().span(6).push(6).child(mark(5)))
                            .col(el_col().span(6).pull(6).child(mark(6))),
                    )
                    .child(
                        el_row()
                            .justify(RowJustify::End)
                            .col(el_col().span(18).child(mark(7)))
                            .col(el_col().span(12).child(mark(8))),
                    )
                    .child(
                        el_row()
                            .align(RowAlign::Middle)
                            .col(el_col().span(12).child(mark(9)))
                            .col(el_col().span(12).child(mark(10))),
                    )
            }
        }

        let rects: Rects = Default::default();
        let (_, view) = cx.add_window_view({
            let rects = rects.clone();
            move |_, _| Probe(rects)
        });
        view.update(|w, cx| w.draw(cx).clear(cx));
        let bounds = rects
            .borrow()
            .map(|rect| rect.expect("column content laid out"));
        let actual = bounds.map(|rect| {
            (
                f32::from(rect.origin.x),
                f32::from(rect.origin.y),
                f32::from(rect.size.width),
            )
        });
        assert_eq!(
            actual,
            [
                (0., 0., 45.),
                (65., 0., 45.),
                (130., 0., 45.),
                (195., 0., 45.),
                (60., 20., 60.),
                (180., 20., 60.),
                (120., 20., 60.),
                (60., 40., 180.),
                (120., 60., 120.),
                (0., 80., 120.),
                (120., 90., 120.),
            ]
        );
    }

    #[test]
    fn responsive_columns_follow_viewport_breakpoints_and_merge_properties() {
        let col = el_col()
            .span(24)
            .xs(0)
            .sm(12)
            .md(ColSpec {
                offset: Some(3),
                ..Default::default()
            })
            .xl(6);
        assert_eq!(col.at(767.9).span, Some(0));
        assert_eq!(col.at(768.).span, Some(12));
        assert_eq!(col.at(991.9).offset, None);
        assert_eq!(col.at(992.).offset, Some(3));
        assert_eq!(col.at(1919.9).span, Some(12));
        assert_eq!(col.at(1920.).span, Some(6));
    }
}
