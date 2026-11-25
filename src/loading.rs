use std::{f32::consts::TAU, sync::OnceLock, time::Instant};

use gpui::{
    AnyElement, App, ElementId, Hsla, IntoElement, ParentElement, PathBuilder, RenderOnce,
    SharedString, Window, canvas, div, point, prelude::*, px, rgb,
};

use crate::{Icon, IconPack, Locale, Theme, theme::PRIMARY};

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
    #[default]
    Uniform,
    /// Speed increases linearly from rest and resets each revolution.
    LinearAcceleration,
    /// ChatSpeed-style acceleration: CSS `ease-in`, one second per turn.
    EaseIn,
}

impl LoadingMotion {
    pub(crate) fn phase(self, elapsed: f64) -> f32 {
        let period = if self == Self::EaseIn { 1. } else { 2. };
        let progress = (elapsed % period / period) as f32;
        match self {
            Self::Uniform => progress,
            Self::LinearAcceleration => progress * progress,
            Self::EaseIn => css_easing(progress, 1.),
        }
    }
}

pub(crate) fn animation_time(window: &mut Window, cx: &App) -> f64 {
    static EPOCH: OnceLock<Instant> = OnceLock::new();
    if Theme::current(cx).reduced_motion || cx.reduce_motion() {
        return 0.;
    }
    window.request_animation_frame();
    EPOCH.get_or_init(Instant::now).elapsed().as_secs_f64()
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
        let size = if self.fullscreen { 50. } else { 42. };
        let background = self
            .background
            .unwrap_or_else(|| theme_mask(theme.dark, self.opacity));
        let indicator = match self.indicator {
            Some(Indicator::Element(element)) => element,
            Some(Indicator::Icon(icon)) => icon
                .with_default_size(size)
                .spin(self.motion)
                .into_any_element(),
            None if self.spinner == LoadingSpinner::Spokes => {
                element_plus_spinner(size, self.motion)
            }
            None => animated_spinner(size, self.spinner, self.motion),
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

fn theme_mask(dark: bool, opacity: Option<f32>) -> Hsla {
    let (color, default) = if dark { (0, 0.8) } else { (0xffffff, 0.9) };
    Hsla::from(rgb(color)).opacity(opacity.unwrap_or(default))
}

fn css_easing(progress: f32, x2: f32) -> f32 {
    // Sample x before y: ease-in-out uses x2=0.58 and ease-in x2=1.
    let (mut lo, mut hi) = (0., 1.);
    for _ in 0..16 {
        let t = (lo + hi) / 2.;
        let inv = 1. - t;
        let x = 3. * inv * inv * t * 0.42 + 3. * inv * t * t * x2 + t * t * t;
        if x < progress {
            lo = t;
        } else {
            hi = t;
        }
    }
    let t = (lo + hi) / 2.;
    match progress {
        0. | 1. => progress,
        _ => t * t * (3. - 2. * t),
    }
}

fn mask_geometry(size: f32, elapsed: f64, turn: f32) -> (f32, f32, f32, f32) {
    // SVG viewBox 50×50, radius 20 and stroke 2. Dash length and offset
    // animate independently from the two-second rotation.
    let p = (elapsed % 1.5) as f32 / 1.5;
    let (length, offset) = if p < 0.5 {
        let x = css_easing(p * 2., 0.58);
        (1. + 89. * x, 40. * x)
    } else {
        let x = css_easing((p - 0.5) * 2., 0.58);
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
    element_plus_spinner(size, LoadingMotion::Uniform)
}

fn element_plus_spinner(size: f32, motion: LoadingMotion) -> AnyElement {
    Icon::from_pack(IconPack::ElementPlus, "Loading")
        .size(size)
        .spin(motion)
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

fn animated_spinner(size: f32, style: LoadingSpinner, motion: LoadingMotion) -> AnyElement {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, cx| {
            if !bounds.intersects(&window.content_mask().bounds) {
                return;
            }
            let reduced = Theme::current(cx).reduced_motion || cx.reduce_motion();
            let elapsed = animation_time(window, cx);
            let center = bounds.center();
            let color = window.text_style().color;
            let turn = motion.phase(elapsed);
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
                        .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
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
        assert_eq!(LoadingMotion::Uniform.phase(1.), 0.5);
        assert_eq!(LoadingMotion::LinearAcceleration.phase(1.), 0.25);
        assert_eq!(LoadingMotion::LinearAcceleration.phase(2.), 0.);
        assert!((LoadingMotion::EaseIn.phase(0.5) - 0.31536).abs() < 0.0001);
        assert_eq!(LoadingMotion::EaseIn.phase(1.), 0.);
        assert!(DOTS.windows(2).all(|pair| pair[0].2 < pair[1].2));
    }

    #[test]
    fn mask_alpha_and_text_orientation_are_independent() {
        let loading = el_loading()
            .h_text("right")
            .mask_opacity(0.)
            .spinner(LoadingSpinner::Dots);
        assert!(loading.horizontal);
        assert_eq!(theme_mask(false, loading.opacity).a, 0.);
        assert_eq!(theme_mask(true, loading.opacity).a, 0.);
        assert_eq!(theme_mask(false, None).a, 0.9);
        assert_eq!(theme_mask(true, None).a, 0.8);
        let loading = loading.v_text("below");
        assert!(!loading.horizontal);
        assert_eq!(loading.spinner, LoadingSpinner::Dots);
    }
}
