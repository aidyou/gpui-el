//! Shared anchored-popup geometry for Tooltip, Popover, and Popconfirm.

use gpui::{
    Anchor, AnyElement, Bounds, IntoElement, PathBuilder, Pixels, Point, Size, canvas, div, point,
    prelude::*, px, rgb, size,
};

/// Twelve Element Plus popper placements.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PopupPlacement {
    Top,
    TopStart,
    TopEnd,
    #[default]
    Bottom,
    BottomStart,
    BottomEnd,
    Left,
    LeftStart,
    LeftEnd,
    Right,
    RightStart,
    RightEnd,
}

impl PopupPlacement {
    pub(crate) fn target_anchor(self) -> Anchor {
        match self {
            Self::Top => Anchor::TopCenter,
            Self::TopStart => Anchor::TopLeft,
            Self::TopEnd => Anchor::TopRight,
            Self::Bottom => Anchor::BottomCenter,
            Self::BottomStart => Anchor::BottomLeft,
            Self::BottomEnd => Anchor::BottomRight,
            Self::Left | Self::LeftStart | Self::LeftEnd => Anchor::LeftCenter,
            Self::Right | Self::RightStart | Self::RightEnd => Anchor::RightCenter,
        }
    }

    pub(crate) fn popup_anchor(self) -> Anchor {
        match self {
            Self::Top => Anchor::BottomCenter,
            Self::TopStart => Anchor::BottomLeft,
            Self::TopEnd => Anchor::BottomRight,
            Self::Bottom => Anchor::TopCenter,
            Self::BottomStart => Anchor::TopLeft,
            Self::BottomEnd => Anchor::TopRight,
            Self::Left | Self::LeftStart | Self::LeftEnd => Anchor::RightCenter,
            Self::Right | Self::RightStart | Self::RightEnd => Anchor::LeftCenter,
        }
    }

    pub(crate) fn offset(self, target: Bounds<Pixels>, distance: f32) -> Point<Pixels> {
        match self {
            Self::Top | Self::TopStart | Self::TopEnd => point(px(0.), px(-distance)),
            Self::Bottom | Self::BottomStart | Self::BottomEnd => point(px(0.), px(distance)),
            Self::Left => point(px(-distance), px(0.)),
            Self::LeftStart => point(px(-distance), -target.size.height / 2.),
            Self::LeftEnd => point(px(-distance), target.size.height / 2.),
            Self::Right => point(px(distance), px(0.)),
            Self::RightStart => point(px(distance), -target.size.height / 2.),
            Self::RightEnd => point(px(distance), target.size.height / 2.),
        }
    }

    pub(crate) fn opposite(self) -> Self {
        match self {
            Self::Top | Self::TopStart | Self::TopEnd => match self {
                Self::TopStart => Self::BottomStart,
                Self::TopEnd => Self::BottomEnd,
                _ => Self::Bottom,
            },
            Self::Bottom | Self::BottomStart | Self::BottomEnd => match self {
                Self::BottomStart => Self::TopStart,
                Self::BottomEnd => Self::TopEnd,
                _ => Self::Top,
            },
            Self::Left | Self::LeftStart | Self::LeftEnd => match self {
                Self::LeftStart => Self::RightStart,
                Self::LeftEnd => Self::RightEnd,
                _ => Self::Right,
            },
            Self::Right | Self::RightStart | Self::RightEnd => match self {
                Self::RightStart => Self::LeftStart,
                Self::RightEnd => Self::LeftEnd,
                _ => Self::Left,
            },
        }
    }

    fn origin(
        self,
        target: Bounds<Pixels>,
        popup_size: Size<Pixels>,
        distance: f32,
    ) -> Point<Pixels> {
        let popup_anchor =
            Bounds::new(point(px(0.), px(0.)), popup_size).corner(self.popup_anchor());
        let target_anchor = target.corner(self.target_anchor());
        let offset = self.offset(target, distance);
        point(
            target_anchor.x - popup_anchor.x + offset.x,
            target_anchor.y - popup_anchor.y + offset.y,
        )
    }

    fn fits(self, origin: Point<Pixels>, popup_size: Size<Pixels>, viewport: Size<Pixels>) -> bool {
        let margin = px(4.);
        origin.x >= margin
            && origin.y >= margin
            && origin.x + popup_size.width <= viewport.width - margin
            && origin.y + popup_size.height <= viewport.height - margin
    }
}

/// Resolves the requested placement, then caller-provided fallbacks, and finally
/// clamps the requested placement to the window.
pub(crate) fn resolve_placement(
    requested: PopupPlacement,
    fallbacks: &[PopupPlacement],
    target: Bounds<Pixels>,
    popup_size: Option<Size<Pixels>>,
    viewport: Size<Pixels>,
    distance: f32,
) -> (PopupPlacement, Point<Pixels>) {
    let popup_size = popup_size.unwrap_or_else(|| size(px(150.), px(40.)));
    for placement in std::iter::once(requested)
        .chain(fallbacks.iter().copied())
        .chain([
            requested.opposite(),
            PopupPlacement::Bottom,
            PopupPlacement::Top,
            PopupPlacement::Right,
            PopupPlacement::Left,
        ])
    {
        let origin = placement.origin(target, popup_size, distance);
        if placement.fits(origin, popup_size, viewport) {
            return (placement, origin);
        }
    }
    let origin = requested.origin(target, popup_size, distance);
    (
        requested,
        point(
            origin.x.clamp(
                px(4.),
                (viewport.width - popup_size.width - px(4.)).max(px(4.)),
            ),
            origin.y.clamp(
                px(4.),
                (viewport.height - popup_size.height - px(4.)).max(px(4.)),
            ),
        ),
    )
}

/// Draws the ten-pixel popper arrow as a filled native path.
pub(crate) fn arrow(placement: PopupPlacement, popup_size: Size<Pixels>, color: u32) -> AnyElement {
    let mut element = div().absolute().w(px(10.)).h(px(10.)).child(
        canvas(
            move |_, _, _| placement,
            move |_, placement, window, _| {
                let mut path = PathBuilder::fill();
                match placement {
                    PopupPlacement::Top | PopupPlacement::TopStart | PopupPlacement::TopEnd => {
                        path.move_to(point(px(0.), px(0.)));
                        path.line_to(point(px(10.), px(0.)));
                        path.line_to(point(px(5.), px(10.)));
                    }
                    PopupPlacement::Bottom
                    | PopupPlacement::BottomStart
                    | PopupPlacement::BottomEnd => {
                        path.move_to(point(px(0.), px(10.)));
                        path.line_to(point(px(10.), px(10.)));
                        path.line_to(point(px(5.), px(0.)));
                    }
                    PopupPlacement::Left | PopupPlacement::LeftStart | PopupPlacement::LeftEnd => {
                        path.move_to(point(px(0.), px(0.)));
                        path.line_to(point(px(0.), px(10.)));
                        path.line_to(point(px(10.), px(5.)));
                    }
                    PopupPlacement::Right
                    | PopupPlacement::RightStart
                    | PopupPlacement::RightEnd => {
                        path.move_to(point(px(10.), px(0.)));
                        path.line_to(point(px(10.), px(10.)));
                        path.line_to(point(px(0.), px(5.)));
                    }
                }
                path.close();
                if let Ok(path) = path.build() {
                    window.paint_path(path, rgb(color));
                }
            },
        )
        .size(px(10.)),
    );
    element = match placement {
        PopupPlacement::Top | PopupPlacement::TopStart | PopupPlacement::TopEnd => {
            element.bottom(px(-5.))
        }
        PopupPlacement::Bottom | PopupPlacement::BottomStart | PopupPlacement::BottomEnd => {
            element.top(px(-5.))
        }
        PopupPlacement::Left | PopupPlacement::LeftStart | PopupPlacement::LeftEnd => {
            element.right(px(-5.))
        }
        PopupPlacement::Right | PopupPlacement::RightStart | PopupPlacement::RightEnd => {
            element.left(px(-5.))
        }
    };
    element = match placement {
        PopupPlacement::TopStart | PopupPlacement::BottomStart => element.left(px(12.)),
        PopupPlacement::TopEnd | PopupPlacement::BottomEnd => element.right(px(12.)),
        PopupPlacement::Top | PopupPlacement::Bottom => {
            element.left((popup_size.width - px(10.)) / 2.)
        }
        PopupPlacement::LeftStart | PopupPlacement::RightStart => element.top(px(12.)),
        PopupPlacement::LeftEnd | PopupPlacement::RightEnd => element.bottom(px(12.)),
        PopupPlacement::Left | PopupPlacement::Right => {
            element.top((popup_size.height - px(10.)) / 2.)
        }
    };
    element.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placements_flip_and_align_start_end() {
        let target = Bounds::new(point(px(400.), px(300.)), size(px(100.), px(40.)));
        let viewport = size(px(800.), px(600.));
        let popup = size(px(150.), px(40.));
        let (placement, origin) =
            resolve_placement(PopupPlacement::Top, &[], target, Some(popup), viewport, 12.);
        assert_eq!(placement, PopupPlacement::Top);
        assert_eq!(origin, point(px(375.), px(248.)));
        let (placement, origin) = resolve_placement(
            PopupPlacement::LeftStart,
            &[],
            target,
            Some(popup),
            viewport,
            12.,
        );
        assert_eq!(placement, PopupPlacement::LeftStart);
        assert_eq!(origin, point(px(238.), px(280.)));
    }

    #[test]
    fn crowded_targets_use_the_opposite_side() {
        let target = Bounds::new(point(px(20.), px(20.)), size(px(80.), px(30.)));
        let viewport = size(px(320.), px(240.));
        let popup = size(px(150.), px(60.));
        let (placement, _) =
            resolve_placement(PopupPlacement::Top, &[], target, Some(popup), viewport, 12.);
        assert_eq!(placement, PopupPlacement::Right);
    }
}
