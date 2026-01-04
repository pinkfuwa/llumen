use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    /// Errors from HTTP client (reqwest)
    Http(reqwest::Error),
    /// Errors from JSON serialization/deserialization
    Serde(serde_json::Error),
    /// Upstream Openrouter API returned an error
    Api { message: String, code: Option<i32> },
    /// Malformed or unexpected response from upstream
    MalformedResponse(&'static str),
    /// incompatible upstream, not a fatal error
    Incompatible(&'static str),
    /// IO errors from streaming
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Http(e) => write!(f, "HTTP error: {}", e),
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
            Error::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Http(e) => Some(e),
            Error::Serde(e) => Some(e),
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Http(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
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
