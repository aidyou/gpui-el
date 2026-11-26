use std::f32::consts::TAU;

use gpui::{
    AnyElement, App, ElementId, Hsla, IntoElement, ParentElement, PathBuilder, RenderOnce,
    SharedString, Window, canvas, div, point, prelude::*, px, rgb,
};

use crate::{
    Icon, IconPack, Locale, Theme,
    motion::{self, MotionSpeed},
    theme::{Mask, PRIMARY},
};

/// Built-in loading indicators. A custom `indicator` takes precedence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LoadingSpinner {
    /// Element Plus's circular dash with a changing arc length.
    #[default]
    Circle,
    /// The official Element Plus `Loading` SVG (eight spokes).
    Spokes,
    /// A ring of fourteen progressively larger dots, as in ChatSpeed.
    Dots,
}

/// Angular speed within each revolution.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LoadingMotion {
    /// Element Plus's linear rotation, two seconds per turn.
    Uniform,
    /// Speed increases linearly from rest and rests briefly each revolution.
    LinearAcceleration,
    /// ChatSpeed-style acceleration: CSS `ease-in`, one second per turn.
    EaseIn,
    /// CSS `spin 1s ease-in-out infinite`: one second per turn, slowing to a
    /// brief rest at the loop point. The desktop default.
    #[default]
    EaseInOut,
}

impl LoadingMotion {
    /// Seconds for one revolution at normal speed.
    pub(crate) fn period(self) -> f64 {
        match self {
            Self::Uniform | Self::LinearAcceleration => 2.,
            Self::EaseIn | Self::EaseInOut => 1.,
        }
    }
    pub(crate) fn phase(self, elapsed: f64, speed: MotionSpeed) -> f32 {
        let progress = motion::turn_phase(elapsed, speed.scale(self.period()));
        match self {
            Self::Uniform => progress,
            Self::LinearAcceleration => progress * progress,
            Self::EaseIn => motion::ease_in(progress),
            Self::EaseInOut => motion::ease_in_out(progress),
        }
    }
}

/// Creates an inactive scoped mask. It blocks pointers, not keyboard focus.
/// Assign a stable, sibling-unique ID when repeated at the same call site.
#[track_caller]
pub fn el_loading() -> Loading {
    Loading::new(ElementId::CodeLocation(*std::panic::Location::caller()))
}

#[derive(Default, IntoElement)]
pub struct Loading {
    id: Option<ElementId>,
    active: bool,
    fullscreen: bool,
    text: Option<SharedString>,
    horizontal: bool,
    spinner: LoadingSpinner,
    motion: LoadingMotion,
    speed: Option<MotionSpeed>,
    opacity: Option<f32>,
    background: Option<Hsla>,
    indicator: Option<Indicator>,
    children: Vec<AnyElement>,
}

enum Indicator {
    Element(AnyElement),
    Icon(Icon),
}

impl Loading {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self::default().id(id)
    }
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }
    /// Places text below the spinner (the Element Plus default).
    pub fn v_text(mut self, text: impl Into<SharedString>) -> Self {
        self.horizontal = false;
        self.text = Some(text.into());
        self
    }
    /// Places text to the right of the spinner.
    pub fn h_text(mut self, text: impl Into<SharedString>) -> Self {
        self.horizontal = true;
        self.text = Some(text.into());
        self
    }
    pub fn spinner(mut self, spinner: LoadingSpinner) -> Self {
        self.spinner = spinner;
        self
    }
    pub fn motion(mut self, motion: LoadingMotion) -> Self {
        self.motion = motion;
        self
    }
    /// Overrides the theme's [`MotionSpeed`] for the spinner rotation.
    pub fn speed(mut self, speed: MotionSpeed) -> Self {
        self.speed = Some(speed);
        self
    }
    /// Sets the theme mask alpha; 0 keeps pointer blocking but draws no mask.
    /// Finite values are clamped to 0–1; an explicit `background` takes precedence.
    pub fn mask_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.is_finite().then(|| opacity.clamp(0., 1.));
        self
    }
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
    /// Uses the fullscreen spinner size. Mount at the window root to cover it.
    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }
    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.text = Some(text.into());
        self
    }
    pub fn background(mut self, color: impl Into<Hsla>) -> Self {
        self.background = Some(color.into());
        self
    }
    /// Replaces the built-in spinner with an element. Its animation, if any,
    /// is controlled by the element itself.
    pub fn indicator(mut self, indicator: impl IntoElement) -> Self {
        self.indicator = Some(Indicator::Element(indicator.into_any_element()));
        self
    }

    /// Replaces the spinner with an icon and applies the selected motion.
    /// SVG icons rotate; font glyphs remain static.
    ///
    /// ```
    /// use gpui_el::{Icon, IconPack, LoadingMotion, el_loading};
    /// let mask = el_loading()
    ///     .active(true)
    ///     .h_text("Refreshing")
    ///     .loading_icon(Icon::from_pack(IconPack::ElementPlus, "RefreshRight").size(28.))
    ///     .motion(LoadingMotion::EaseIn);
    /// ```
    pub fn loading_icon(mut self, icon: Icon) -> Self {
        self.indicator = Some(Indicator::Icon(icon));
        self
    }
}

impl ParentElement for Loading {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}

impl RenderOnce for Loading {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let root = div()
            .relative()
            .when(self.fullscreen, |el| el.size_full())
            .children(self.children);
        let finish = |root: gpui::Div| match self.id {
            Some(id) => root.id(id).into_any_element(),
            None => root.into_any_element(),
        };
        if !self.active {
            return finish(root);
        }
        let theme = Theme::current(cx);
        let speed = self.speed.unwrap_or(theme.motion_speed);
        let size = if self.fullscreen { 50. } else { 42. };
        let background = self
            .background
            .unwrap_or_else(|| mask_background(&theme, self.opacity));
        let indicator = match self.indicator {
            Some(Indicator::Element(element)) => element,
            Some(Indicator::Icon(icon)) => icon
                .with_default_size(size)
                .spin(self.motion)
                .spin_speed(speed)
                .into_any_element(),
            None if self.spinner == LoadingSpinner::Spokes => {
                element_plus_spinner(size, self.motion, speed)
            }
            None => animated_spinner(size, self.spinner, self.motion, speed),
        };
        let text = self
            .text
            .clone()
            .map(|text| div().min_w_0().text_size(px(14.)).text_center().child(text));
        let aria = self
            .text
            .clone()
            .unwrap_or_else(|| Locale::current(cx).button_loading().into());
        // Center the horizontal pair, or center the spinner with text beneath it.
        finish(
            root.child(
                div()
                    .absolute()
                    .inset_0()
                    .occlude()
                    .bg(background)
                    .id("loading-mask")
                    .role(gpui::Role::Status)
                    .aria_label(aria)
                    .text_color(rgb(PRIMARY))
                    .child(
                        div()
                            .absolute()
                            .when(self.horizontal, |el| {
                                el.inset_0().justify_center().gap(px(8.)).px(px(8.))
                            })
                            .when(!self.horizontal, |el| {
                                el.top(gpui::relative(0.5))
                                    .mt(px(-size / 2.))
                                    .flex_col()
                                    .gap(px(3.))
                            })
                            .w_full()
                            .flex()
                            .items_center()
                            .child(indicator)
                            .children(text),
                    ),
            ),
        )
    }
}

fn mask_background(theme: &Theme, opacity: Option<f32>) -> Hsla {
    let mask = theme.mask(Mask::Base);
    Hsla {
        a: opacity.unwrap_or(mask.a),
        ..mask
    }
}

fn mask_geometry(size: f32, elapsed: f64, turn: f32) -> (f32, f32, f32, f32) {
    // SVG viewBox 50×50, radius 20 and stroke 2. Dash length and offset
    // animate independently from the two-second rotation.
    let p = (elapsed % 1.5) as f32 / 1.5;
    let (length, offset) = if p < 0.5 {
        let x = motion::ease_in_out(p * 2.);
        (1. + 89. * x, 40. * x)
    } else {
        let x = motion::ease_in_out((p - 0.5) * 2.);
        (90., 40. + 80. * x)
    };
    (
        size * 0.4,
        size * 0.04,
        turn + offset / (20. * TAU),
        length / (20. * TAU),
    )
}

pub(crate) fn inline_spinner(size: f32) -> AnyElement {
    element_plus_spinner(size, LoadingMotion::Uniform, MotionSpeed::Normal)
}

fn element_plus_spinner(size: f32, motion: LoadingMotion, speed: MotionSpeed) -> AnyElement {
    Icon::from_pack(IconPack::ElementPlus, "Loading")
        .size(size)
        .spin(motion)
        .spin_speed(speed)
        .into_any_element()
}

// Centers and diameters in the 1024-unit view box of ChatSpeed's vue-loading.svg.
// Draw circles directly rather than depending on the host application's font.
const DOTS: [(f32, f32, f32); 14] = [
    (833., 744., 11.),
    (717., 877., 22.),
    (559., 945., 32.),
    (389., 936., 43.),
    (238., 851., 54.),
    (136., 707., 65.),
    (100., 530., 76.),
    (139., 354., 88.),
    (245., 212., 98.),
    (397., 131., 110.),
    (568., 126., 120.),
    (725., 198., 131.),
    (837., 333., 142.),
    (886., 508., 153.),
];

fn animated_spinner(
    size: f32,
    style: LoadingSpinner,
    motion: LoadingMotion,
    speed: MotionSpeed,
) -> AnyElement {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, cx| {
            if !bounds.intersects(&window.content_mask().bounds) {
                return;
            }
            let reduced = Theme::motion_reduced(cx);
            let elapsed = motion::animation_time(window, cx);
            let center = bounds.center();
            let color = window.text_style().color;
            let turn = motion.phase(elapsed, speed);
            if style == LoadingSpinner::Dots {
                let (sin, cos) = (TAU * turn).sin_cos();
                let scale = size / 1024.;
                for (x, y, diameter) in DOTS {
                    let (x, y) = (x - 512., y - 512.);
                    let at = center
                        + point(
                            px((x * cos - y * sin) * scale),
                            px((x * sin + y * cos) * scale),
                        );
                    let dot = px(diameter * scale);
                    let bounds = gpui::Bounds::centered_at(at, gpui::size(dot, dot));
                    window.paint_quad(gpui::fill(bounds, color).corner_radii(dot / 2.));
                }
                return;
            }
            let (radius, stroke, start, sweep) = if reduced {
                let (radius, stroke, _, _) = mask_geometry(size, 0., 0.);
                (radius, stroke, 0., 0.75)
            } else {
                mask_geometry(size, elapsed, turn)
            };
            let mut path = PathBuilder::stroke(px(stroke));
            for step in 0..=32 {
                let angle = TAU * (start + sweep * step as f32 / 32.);
                let p = center + point(px(radius * angle.cos()), px(radius * angle.sin()));
                if step == 0 {
                    path.move_to(p);
                } else {
                    path.line_to(p);
                }
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, window.text_style().color);
            }
        },
    )
    .size(px(size))
    .flex_none()
    .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn active_mask_blocks_pointer_only_inside_its_region(cx: &mut gpui::TestAppContext) {
        use std::{cell::Cell, rc::Rc};

        struct Harness(Rc<Cell<u32>>);
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let button = |id| {
                    let clicks = self.0.clone();
                    crate::Button::new(id)
                        .label("Activate")
                        .onclick(move |_, _, _| clicks.set(clicks.get() + 1))
                };
                div()
                    .flex()
                    .flex_col()
                    .size_full()
                    .child(Loading::new("masked").active(true).child(button("covered")))
                    .child(Loading::new("unmasked").child(button("uncovered")))
            }
        }
        let clicks = Rc::new(Cell::new(0));
        let (_, view) = cx.add_window_view({
            let clicks = clicks.clone();
            move |_, cx| {
                cx.set_reduce_motion(true);
                Harness(clicks)
            }
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        view.simulate_click(point(px(20.), px(16.)), gpui::Modifiers::default());
        assert_eq!(clicks.get(), 0);
        view.simulate_click(point(px(20.), px(48.)), gpui::Modifiers::default());
        assert_eq!(clicks.get(), 1);
    }

    #[gpui::test]
    fn horizontal_indicator_precedes_text(cx: &mut gpui::TestAppContext) {
        use std::{cell::Cell, rc::Rc};

        struct Harness(Rc<Cell<Option<gpui::Bounds<gpui::Pixels>>>>);
        impl gpui::Render for Harness {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let bounds = self.0.clone();
                div().w(px(240.)).child(
                    el_loading()
                        .active(true)
                        .h_text("Loading data")
                        .indicator(
                            gpui::canvas(move |rect, _, _| bounds.set(Some(rect)), |_, _, _, _| {})
                                .size(px(42.))
                                .flex_none(),
                        )
                        .child(div().h(px(100.))),
                )
            }
        }

        let bounds = Rc::new(Cell::new(None));
        let (_, view) = cx.add_window_view({
            let bounds = bounds.clone();
            move |_, _| Harness(bounds)
        });
        view.update(|window, cx| window.draw(cx).clear(cx));
        let indicator = bounds.get().expect("indicator laid out");
        assert!(indicator.center().x < px(120.));
        assert_eq!(indicator.center().y, px(50.));
    }

    #[test]
    fn mask_dash_and_inline_rotation_use_separate_periods() {
        let (_, _, start, initial) = mask_geometry(42., 0., 0.);
        let (_, _, later, peak) = mask_geometry(42., 0.75, 0.375);
        assert_eq!(start, 0.);
        assert!(later > start && peak > initial);
        assert_eq!(mask_geometry(42., 1.5, 0.75).3, initial);
        let normal = MotionSpeed::Normal;
        assert_eq!(LoadingMotion::Uniform.phase(1., normal), 0.5);
        assert_eq!(LoadingMotion::LinearAcceleration.phase(1., normal), 0.25);
        assert_eq!(LoadingMotion::LinearAcceleration.phase(2., normal), 0.);
        assert!((LoadingMotion::EaseIn.phase(0.5, normal) - 0.31536).abs() < 0.0001);
        assert_eq!(LoadingMotion::EaseIn.phase(1., normal), 0.);
        // The desktop default: CSS `spin 1s ease-in-out infinite`.
        assert!((LoadingMotion::EaseInOut.phase(0.5, normal) - 0.5).abs() < 1e-5);
        assert_eq!(LoadingMotion::EaseInOut.phase(1., normal), 0.);
        assert!(LoadingMotion::EaseInOut.phase(0.1, normal) < 0.1);
        // A faster speed completes a turn sooner.
        assert_eq!(LoadingMotion::EaseInOut.phase(0.6, MotionSpeed::Fast), 0.);
        assert!(DOTS.windows(2).all(|pair| pair[0].2 < pair[1].2));
    }

    #[test]
    fn mask_alpha_and_text_orientation_are_independent() {
        let loading = el_loading()
            .h_text("right")
            .mask_opacity(0.)
            .spinner(LoadingSpinner::Dots);
        assert!(loading.horizontal);
        assert_eq!(mask_background(&Theme::light(), loading.opacity).a, 0.);
        assert_eq!(mask_background(&Theme::dark(), loading.opacity).a, 0.);
        assert_eq!(mask_background(&Theme::light(), None).a, 0.9);
        assert_eq!(mask_background(&Theme::dark(), None).a, 0.8);
        assert_eq!(mask_background(&Theme::light(), Some(1.)).a, 1.);
        assert_eq!(mask_background(&Theme::dark(), Some(1.)).a, 1.);
        let loading = loading.v_text("below");
        assert!(!loading.horizontal);
        assert_eq!(loading.spinner, LoadingSpinner::Dots);
    }
}
