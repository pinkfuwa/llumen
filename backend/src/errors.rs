use axum::Json;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize)]
#[typeshare]
pub struct Error {
    pub error: ErrorKind,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[typeshare]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    Unauthorized,
    MalformedToken,
    MalformedRequest,
    Internal,
    UnknownUser,
}

pub type JsonResult<T> = Result<Json<T>, Json<Error>>;

pub trait WithKind<T> {
    fn kind(self, kind: ErrorKind) -> Result<T, Json<Error>>;
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
}
