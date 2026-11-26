use std::fmt;

/// Errors reported by the component library.
///
/// Rendering never panics on bad widget input: an unknown icon name draws
/// nothing, a malformed size falls back to its default, and a locale that
/// fails to parse falls back to the embedded English messages. This type is
/// for setup calls that cannot continue without the caller's decision, and
/// the caller decides whether a failure is fatal. It is `#[non_exhaustive]`
/// so new variants stay additive.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The platform text system rejected font bytes.
    Font(Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    pub(crate) fn font(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        Self::Font(error.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Font(error) => write!(f, "the text system rejected the font: {error}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Font(error) => Some(error.as_ref()),
        }
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::Font(error)
    }
}

/// Result alias for the library's setup calls.
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_errors_name_the_failing_stage() {
        let error = Error::font(std::io::Error::other("broken ttf"));
        assert_eq!(
            error.to_string(),
            "the text system rejected the font: broken ttf"
        );
        assert!(std::error::Error::source(&error).is_some());
    }
}
