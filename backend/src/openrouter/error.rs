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
    /// Model is not eligible for image generation
    ImageGenNotSupported,
    /// Model does not exist in the image generation listing
    ImageGenModelNotFound,
    /// Model cannot accept reference images
    ImageGenReferenceImagesNotSupported,
    /// Image generation response did not include any images
    ImageGenNoImagesInResponse,
    /// Model is not eligible for video generation
    VideoGenNotSupported,
    /// Model does not exist in the video generation listing
    VideoGenModelNotFound,
    /// Model cannot accept reference images for video generation
    VideoGenReferenceImagesNotSupported,
    /// Model cannot accept reference videos for video generation
    VideoGenReferenceVideosNotSupported,
    /// Too many reference images for video generation model limits
    VideoGenReferenceImagesLimitExceeded { max: usize },
    /// Too many reference videos for video generation model limits
    VideoGenReferenceVideosLimitExceeded { max: usize },
    /// Reference file was not image/video
    VideoGenInvalidReferenceFile,
    /// Video generation response did not include downloadable videos
    VideoGenNoVideosInResponse,
    /// Video generation job failed on provider
    VideoGenJobFailed(String),
    /// Video generation polling exceeded configured limits
    VideoGenPollingTimeout,
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
            Error::ImageGenNotSupported => {
                write!(f, "Model does not support image-only generation")
            }
            Error::ImageGenModelNotFound => write!(f, "Image generation model not found"),
            Error::ImageGenReferenceImagesNotSupported => {
                write!(f, "Model does not support reference images")
            }
            Error::ImageGenNoImagesInResponse => {
                write!(f, "No images in image generation response")
            }
            Error::VideoGenNotSupported => {
                write!(f, "Model does not support video generation")
            }
            Error::VideoGenModelNotFound => write!(f, "Video generation model not found"),
            Error::VideoGenReferenceImagesNotSupported => {
                write!(
                    f,
                    "Model does not support reference images for video generation"
                )
            }
            Error::VideoGenReferenceVideosNotSupported => {
                write!(
                    f,
                    "Model does not support reference videos for video generation"
                )
            }
            Error::VideoGenReferenceImagesLimitExceeded { max } => {
                write!(f, "Too many reference images for model limit ({max})")
            }
            Error::VideoGenReferenceVideosLimitExceeded { max } => {
                write!(f, "Too many reference videos for model limit ({max})")
            }
            Error::VideoGenInvalidReferenceFile => {
                write!(f, "Reference file must be an image or video")
            }
            Error::VideoGenNoVideosInResponse => {
                write!(f, "No videos in video generation response")
            }
            Error::VideoGenJobFailed(msg) => {
                write!(f, "Video generation failed: {msg}")
            }
            Error::VideoGenPollingTimeout => {
                write!(f, "Video generation polling timed out")
            }
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
