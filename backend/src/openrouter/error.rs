use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    /// Errors from HTTP client (reqwest)
    Http(reqwest::Error),
    /// Errors from event streaming (eventsource_stream)
    EventSource(eventsource_stream::EventStreamError<reqwest::Error>),
    /// Errors from JSON serialization/deserialization
    Serde(serde_json::Error),
    /// Upstream Openrouter API returned an error
    Api { message: String, code: Option<i32> },
    /// Malformed or unexpected response from upstream
    MalformedResponse(&'static str),
    /// incompatible upstream, not a fatal error
    Incompatible(&'static str),
    /// Model does not support text output
    TextOutputNotSupported,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Http(e) => write!(f, "HTTP error: {}", e),
            Error::EventSource(e) => write!(f, "EventSource error: {}", e),
            Error::Serde(e) => write!(f, "Serde error: {}", e),
            Error::Api { message, code } => {
                if let Some(code) = code {
                    write!(f, "Openrouter API error (code {}): {}", code, message)
                } else {
                    write!(f, "Openrouter API error: {}", message)
                }
            }
            Error::MalformedResponse(msg) => write!(f, "Malformed response: {}", msg),
            Error::Incompatible(msg) => write!(f, "Incompatible upstream: {}", msg),
            Error::TextOutputNotSupported => write!(f, "Model does not support text output"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Http(e) => Some(e),
            Error::EventSource(e) => Some(e),
            Error::Serde(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Http(e)
    }
}

impl From<eventsource_stream::EventStreamError<reqwest::Error>> for Error {
    fn from(e: eventsource_stream::EventStreamError<reqwest::Error>) -> Self {
        Error::EventSource(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serde(e)
    }
}

// For converting Openrouter API error responses
impl From<crate::openrouter::raw::ErrorInfo> for Error {
    fn from(e: crate::openrouter::raw::ErrorInfo) -> Self {
        Error::Api {
            message: e.message,
            code: e.code,
        }
    }
}

// Optional: for anyhow interop if needed
