//! Shared feedback semantics for Alert, Message, and Notification.

use crate::Theme;

/// Element Plus feedback type shared by alerts and transient messages.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FeedbackType {
    Primary,
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

impl FeedbackType {
    pub(crate) fn color(self) -> u32 {
        match self {
            Self::Primary => crate::theme::PRIMARY,
            Self::Info => crate::theme::INFO,
            Self::Success => crate::theme::SUCCESS,
            Self::Warning => crate::theme::WARNING,
            Self::Error => crate::theme::DANGER,
        }
    }

    pub(crate) fn tint(self, theme: &Theme, level: u32) -> u32 {
        theme.tint(self.color(), level)
    }

    pub(crate) const fn icon(self) -> &'static str {
        match self {
            Self::Primary => "info-filled",
            Self::Info => "info-filled",
            Self::Success => "success-filled",
            Self::Warning => "warning-filled",
            Self::Error => "circle-close-filled",
        }
    }
}

/// Alert color treatment.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FeedbackEffect {
    #[default]
    Light,
    Dark,
}
