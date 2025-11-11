// This module defines the error types and result type aliases used throughout
// the Llumen backend. It follows a pattern where errors are converted to JSON
// responses via the WithKind trait, ensuring consistent error formatting for
// the frontend.
//
// Error propagation strategy:
// 1. Internal errors (anyhow::Error) are caught at the API boundary
// 2. Converted to AppError (Json<Error>) with appropriate ErrorKind
// 3. Frontend receives structured error with kind and reason
//
// The WithKind trait enables ergonomic error conversion:
//   some_operation().kind(ErrorKind::ApiFail)?
//
// This is preferred over manual Result wrapping because it:
// - Maintains consistent error formatting
// - Reduces boilerplate in route handlers
// - Makes error types clear at the call site

use axum::Json;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

/// Error response structure sent to the frontend.
///
/// This is the standard JSON response for all error cases.
/// The frontend uses the `error` field to determine error handling behavior,
/// and the `reason` field to display user-friendly messages.
///
/// # Example
/// ```json
/// {
///   "error": "unauthorized",
///   "reason": "Invalid or expired token"
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
#[typeshare]
pub struct Error {
    /// Error classification used for client-side error handling.
    /// Determines UI behavior, logging level, and user messaging.
    pub error: ErrorKind,
    /// Human-readable error description from the underlying error context.
    /// Should be descriptive enough for logging but safe to show users.
    pub reason: String,
}

/// Enumeration of all possible error types in the Llumen API.
///
/// Each variant represents a category of error that can occur during request processing.
/// The frontend uses these to implement error-specific handling (e.g., redirecting to
/// login on Unauthorized).
///
/// When adding new error types:
/// 1. Consider if an existing variant applies first
/// 2. Add to the enum with a clear, descriptive name
/// 3. Update error conversion logic in route handlers
/// 4. Consider what frontend UI behavior should result from this error
#[derive(Debug, Clone, Deserialize, Serialize)]
#[typeshare]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// User is not authenticated or session token is missing.
    /// Frontend should redirect to login page.
    Unauthorized,

    /// Session token is malformed, expired, or tampered with.
    /// Usually means the user needs to log in again.
    MalformedToken,

    /// Request body doesn't match expected schema.
    /// Indicates a client-side bug or API version mismatch.
    MalformedRequest,

    /// Unexpected internal server error.
    /// Indicates a bug in Llumen or an unhandled edge case.
    /// Frontend should show generic error message and suggest reporting.
    Internal,

    /// User login failed (incorrect credentials).
    /// Frontend should show specific error message to user.
    LoginFail,

    /// Requested resource (chat, message, user, etc.) not found.
    /// May indicate:
    /// - Resource was deleted by another client
    /// - User lacks permission to access resource
    /// - Resource ID is invalid
    ResourceNotFound,

    /// OpenRouter API request failed.
    /// Indicates network issue, rate limiting, or API problem.
    /// Frontend should suggest retrying or checking API status.
    ApiFail,

    /// Tool execution failed (web search, code execution, etc.).
    /// May indicate:
    /// - Network connectivity issue
    /// - Tool service is down
    /// - Tool rejected input parameters
    /// Frontend should show tool-specific error context.
    ToolCallFail,
}

pub type JsonResult<T> = Result<Json<T>, Json<Error>>;

pub type AppError = Json<Error>;

/// Trait for ergonomic error type conversion using the `.kind()` method.
///
/// This trait enables a convenient syntax for converting any Result type
/// to a JsonResult by specifying the error kind to use.
///
/// # Example
/// ```ignore
/// // Without WithKind (verbose):
/// let user = user_lookup().await.map_err(|e| {
///     Json(Error {
///         error: ErrorKind::Internal,
///         reason: e.to_string(),
///     })
/// })?;
///
/// // With WithKind (concise):
/// let user = user_lookup().await.kind(ErrorKind::Internal)?;
/// ```
///
/// # Error Propagation Semantics
///
/// `.kind()` should be used when:
/// - Converting between different error types at API boundaries
/// - Wrapping internal errors for external consumption
/// - You have a specific error category to assign
///
/// `.raw_kind()` should be used when:
/// - You need an Error struct without Json wrapping (for custom responses)
/// - Building error responses in middleware
pub trait WithKind<T> {
    fn kind(self, kind: ErrorKind) -> Result<T, Json<Error>>;
    fn raw_kind(self, kind: ErrorKind) -> Result<T, Error>;
}

impl<T, E> WithKind<T> for Result<T, E>
where
    E: ToString,
{
    fn kind(self, kind: ErrorKind) -> Result<T, Json<Error>> {
        self.map_err(|e| {
            Json(Error {
                error: kind,
                reason: e.to_string(),
            })
        })
    }

    fn raw_kind(self, kind: ErrorKind) -> Result<T, Error> {
        self.map_err(|e| Error {
            error: kind,
            reason: e.to_string(),
        })
    }
}

/// Union type for representing responses that can be either Ok or Err.
///
/// This is useful for endpoints that sometimes need to return partial successes
/// or want to include error information in the response body alongside other data.
///
/// Field name error is forbidden for other response type(`#[typeshare]`)
///
/// Client will check for error field when HTTP(200) is received
///
/// # Example
/// ```json
/// {
///   "error": "internal",
///   "reason": "optional reason",
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum JsonUnion<T, E> {
    /// Successful result variant
    Ok(T),
    /// Error result variant
    Err(E),
}

impl<T, E> From<Result<T, E>> for JsonUnion<T, E> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(v) => Self::Ok(v),
            Err(e) => Self::Err(e),
        }
    }
}
