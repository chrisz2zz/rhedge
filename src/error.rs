//! Error types for hedged request operations.

use std::fmt;

/// Errors that can occur during a hedged request.
#[derive(Debug)]
pub enum HedgedError {
    /// The HTTP request could not be built (e.g. invalid URL, malformed headers).
    Build(reqwest::Error),
    /// The HTTP request was sent but failed (e.g. network error, server error).
    Request(reqwest::Error),
    /// All spawned tasks (including hedge requests) panicked.
    AllTasksPanicked,
}

impl fmt::Display for HedgedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HedgedError::Build(e) => write!(f, "request build failed: {e}"),
            HedgedError::Request(e) => write!(f, "request failed: {e}"),
            HedgedError::AllTasksPanicked => write!(f, "all requests panicked"),
        }
    }
}

impl std::error::Error for HedgedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HedgedError::Build(e) | HedgedError::Request(e) => Some(e),
            HedgedError::AllTasksPanicked => None,
        }
    }
}

impl From<reqwest::Error> for HedgedError {
    fn from(e: reqwest::Error) -> Self {
        // reqwest::Error does not expose a public way to distinguish build vs.
        // execution errors, but `is_builder()` can be used to tell them apart.
        if e.is_builder() {
            HedgedError::Build(e)
        } else {
            HedgedError::Request(e)
        }
    }
}
