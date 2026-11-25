//! Button tokens from Theme Chalk `common/var.scss`, `button.scss`, and
//! `mixins/_button.scss`. Keep state colors separate from input dispatch.

use crate::theme::{DANGER, Fill, INFO, PRIMARY, SUCCESS, Theme, WARNING, WHITE};

/// Element Plus semantic button color.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonType {
    #[default]
    Default,
    Primary,
    Success,
    Warning,
    Danger,
    Info,
}

/// Visual treatment, independent of color, size and shape.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonVariant {
    #[default]
    Solid,
    Plain,
    Text,
    TextWithBackground,
    Link,
    Dashed,
}

/// Button outer height and associated insets, font size and icon spacing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonSize {
    Small,
    #[default]
    Default,
    Large,
}

/// Outer button shape.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonShape {
    #[default]
    Normal,
    Round,
    Circle,
}

pub(super) struct Metrics {
    pub height: f32,
    pub padding_x: f32,
    pub font_size: f32,
    pub radius: f32,
    pub icon_gap: f32,
}

impl ButtonSize {
    pub(super) fn metrics(self, shape: ButtonShape) -> Metrics {
        use crate::tokens::{BORDER_WIDTH, control_height, font_size, radius};
        let (height, outer_padding_x, font_size, normal_radius, icon_gap) = match self {
            Self::Small => (
                control_height::SMALL,
                12.,
                font_size::EXTRA_SMALL,
                radius::BASE - 1.,
                4.,
            ),
            Self::Default => (
                control_height::DEFAULT,
                16.,
                font_size::BASE,
                radius::BASE,
                6.,
            ),
            Self::Large => (
                control_height::LARGE,
                20.,
                font_size::BASE,
                radius::BASE,
                8.,
            ),
        };
        let radius = match shape {
            ButtonShape::Normal => normal_radius,
            ButtonShape::Round => radius::ROUND,
            ButtonShape::Circle => radius::capsule(height, height),
        };
        Metrics {
            height,
            padding_x: outer_padding_x - BORDER_WIDTH,
            font_size,
            radius,
            icon_gap,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Paint {
    pub text: u32,
    pub bg: Option<u32>,
    pub border: Option<u32>,
}

impl Paint {
    const fn framed(text: u32, bg: u32, border: u32) -> Self {
        Self {
            text,
            bg: Some(bg),
            border: Some(border),
        }
    }
}

pub(super) struct Palette {
    pub normal: Paint,
    pub hover: Paint,
    pub active: Paint,
    pub disabled: Paint,
    pub outline: u32,
}

impl Palette {
    // Theme Chalk button-custom.ts derives these states from the supplied sRGB accent.
    pub fn custom(theme: &Theme, variant: ButtonVariant, color: gpui::Rgba) -> Self {
        use ButtonVariant::*;
        let byte = |c: f32| (c.clamp(0., 1.) * 255.).round() as u32;
        let (r, g, b) = (byte(color.r), byte(color.g), byte(color.b));
        let base = (r << 16) | (g << 8) | b;
        let tint = |n| theme.tint(base, n);
        let active = crate::theme::mix(
            base,
            if theme.dark {
                WHITE
            } else {
                crate::theme::DARK_BACKGROUND
            },
            20,
        );
        let foreground = if 299 * r + 587 * g + 114 * b < 128_000 {
            WHITE
        } else {
            0
        };
        let (normal, hover, pressed, disabled) = match variant {
            Text | TextWithBackground | Link => {
                let paint = |text, fill| Paint {
                    text,
                    bg: fill,
                    border: None,
                };
                let (normal_fill, hover_fill, active_fill) = match variant {
                    Text => (None, Some(Fill::Light), Some(Fill::Base)),
                    TextWithBackground => (Some(Fill::Light), Some(Fill::Base), Some(Fill::Dark)),
                    _ => (None, None, None),
                };
                let fill = |level: Option<Fill>| level.map(|level| theme.fill(level));
                (
                    paint(base, fill(normal_fill)),
                    paint(tint(3), fill(hover_fill)),
                    paint(active, fill(active_fill)),
                    paint(tint(5), None),
                )
            }
            Plain | Dashed => (
                Paint::framed(base, tint(9), tint(5)),
                Paint::framed(WHITE, base, base),
                Paint::framed(WHITE, active, active),
                Paint::framed(tint(5), tint(9), tint(8)),
            ),
            Solid => (
                Paint::framed(foreground, base, base),
                Paint::framed(foreground, tint(3), tint(3)),
                Paint::framed(foreground, active, active),
                Paint::framed(
                    if theme.dark {
                        crate::theme::mix(tint(5), WHITE, 50)
                    } else {
                        WHITE
                    },
                    tint(5),
                    tint(5),
                ),
            ),
        };
        Self {
            normal,
            hover,
            active: pressed,
            disabled,
            outline: tint(5),
        }
    }

    pub fn resolve(theme: &Theme, kind: ButtonType, variant: ButtonVariant) -> Self {
        use ButtonVariant::*;
        let accent = match kind {
            ButtonType::Default => None,
            ButtonType::Primary => Some(PRIMARY),
            ButtonType::Success => Some(SUCCESS),
            ButtonType::Warning => Some(WARNING),
            ButtonType::Danger => Some(DANGER),
            ButtonType::Info => Some(INFO),
        };
        let base = accent.unwrap_or(PRIMARY);
        let tint = |n| theme.tint(base, n);
        let shade = theme.shade(base);
        let regular = theme.regular_text();
        let blank = theme.background();
        let (normal, hover, active, disabled) = match (accent, variant) {
            (Some(_), Solid) => (
                Paint::framed(WHITE, base, base),
                Paint::framed(WHITE, tint(3), tint(3)),
                Paint::framed(WHITE, shade, shade),
                Paint::framed(WHITE, tint(5), tint(5)),
            ),
            (Some(_), Dashed) => (
                Paint::framed(base, tint(9), tint(5)),
                Paint::framed(base, tint(9), tint(3)),
                Paint::framed(shade, tint(9), shade),
                Paint::framed(tint(5), tint(9), tint(8)),
            ),
            (Some(_), _) => (
                Paint::framed(base, tint(9), tint(5)),
                Paint::framed(WHITE, base, base),
                Paint::framed(WHITE, shade, shade),
                Paint::framed(tint(5), tint(9), tint(8)),
            ),
            (None, _) => {
                let hover_bg = if matches!(variant, Plain | Dashed) {
                    blank
                } else {
                    tint(9)
                };
                let hover_border = if matches!(variant, Plain | Dashed) {
                    base
                } else {
                    tint(7)
                };
                (
                    Paint::framed(regular, blank, theme.border(false)),
                    Paint::framed(base, hover_bg, hover_border),
                    Paint::framed(base, hover_bg, base),
                    Paint::framed(theme.disabled_text(), blank, theme.border(true)),
                )
            }
        };
        let mut result = Self {
            normal,
            hover,
            active,
            disabled,
            outline: tint(5),
        };
        if matches!(variant, Text | TextWithBackground | Link) {
            let text = normal.text;
            let (n, h, a) = match variant {
                TextWithBackground => (Some(Fill::Light), Some(Fill::Base), Some(Fill::Dark)),
                Text => (None, Some(Fill::Light), Some(Fill::Base)),
                _ => (None, None, None),
            };
            let paint = |text, fill: Option<Fill>| Paint {
                text,
                bg: fill.map(|level| theme.fill(level)),
                border: None,
            };
            let hover = if variant == Link {
                accent
                    .map(|_| tint(5))
                    .unwrap_or_else(|| theme.secondary_text())
            } else {
                text
            };
            let active = if variant == Link {
                accent
                    .map(|_| shade)
                    .unwrap_or_else(|| theme.primary_text())
            } else {
                text
            };
            result.normal = paint(text, n);
            result.hover = paint(hover, h);
            result.active = paint(active, a);
            result.disabled = paint(disabled.text, None);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_chalk_primary_state_colors() {
        let light = Palette::resolve(&Theme::light(), ButtonType::Primary, ButtonVariant::Solid);
        assert_eq!(light.normal, Paint::framed(WHITE, 0x409eff, 0x409eff));
        assert_eq!(light.hover.bg, Some(0x79bbff));
        assert_eq!(light.active.bg, Some(0x337ecc));
        assert_eq!(light.disabled.bg, Some(0xa0cfff));
        let dark = Palette::resolve(&Theme::dark(), ButtonType::Primary, ButtonVariant::Plain);
        assert_eq!(dark.normal.bg, Some(0x18222c));
        assert_eq!(dark.active.bg, Some(0x66b1ff));
    }

    #[test]
    fn normal_geometry_matches_theme_chalk_sizes() {
        for (size, expected) in [
            (ButtonSize::Small, (24., 11., 12., 3., 4.)),
            (ButtonSize::Default, (32., 15., 14., 4., 6.)),
            (ButtonSize::Large, (40., 19., 14., 4., 8.)),
        ] {
            let m = size.metrics(ButtonShape::Normal);
            assert_eq!(
                (m.height, m.padding_x, m.font_size, m.radius, m.icon_gap),
                expected
            );
        }
    }

    #[test]
    fn compact_and_shape_are_independent() {
        let small = ButtonSize::Small.metrics(ButtonShape::Round);
        assert_eq!(
            (small.height, small.radius, small.font_size),
            (24., 20., 12.)
        );
        assert_eq!(ButtonSize::Large.metrics(ButtonShape::Circle).radius, 20.);
    }
}
