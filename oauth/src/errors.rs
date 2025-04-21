use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
// use oauth2::RequestTokenError;
use serde_json;
use serde_json::json;
use std::fmt;
// use std::convert::From;
use thiserror::Error;

use crate::models::message::Message;

pub type Result<T> = std::result::Result<T, AuthError>;

///
/// ⬜ Checkout reqwest Errors module for how to best create an Error
///
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("{:?}", .0)]
    MissingProperty(Message),
    #[error("{:?}", .0)]
    MissingSession(Message),
    #[error("{:?}", .0)]
    JsonParsingError(Message),
    #[error("{:?}", .0)]
    MissingCookie(Message),
    #[error("{:?}", .0)]
    TokenCreation(Message),
    #[error("{:?}", .0)]
    MissingChallenge(Message),
    #[error("{:?}", .0)]
    MissingParameter(Message),
    #[error("{:?}", .0)]
    ReadSessionError(Message),
    #[error("{:?}", .0)]
    WriteSessionError(Message),
    #[error("{:?}", .0)]
    InvalidUrl(Message),
    #[error("{:?}", .0)]
    UnsupportedProvider(Message),
    #[error("{:?}", .0)]
    InternalError(Message),
    #[error("{:?}", .0)]
    InvalidResponse(Message),
    #[error("{:?}", .0)]
    TncSessionResponseError(Message),
    #[error("{:?}", .0)]
    ConfigError(Message),
    #[error("{:?}", .0)]
    DriveTokenError(Message),
    #[error("{:?}", .0)]
    Unauthorized(Message),
    #[error("{:?}", .0)]
    InvalidHeaderValue(Message),
    #[error("{:?}", .0)]
    ProjectIdError(Message),
    #[error("{:?}", .0)]
    MissingQuery(Message),
}

/// Modeled after reqwest Error
/// ⬜ How use Display fmt
#[allow(dead_code)]
pub(crate) fn decode<E: fmt::Display>(e: E) -> AuthError {
    AuthError::JsonParsingError(e.to_string().into())
}
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error, message) = match self {
            AuthError::ReadSessionError(msg) => {
                (StatusCode::UNAUTHORIZED, "Could not read from session", msg)
            }
            AuthError::JsonParsingError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Parsing the response body failed",
                msg,
            ),
            AuthError::MissingCookie(msg) => (
                StatusCode::UNAUTHORIZED,
                "Failed to request a session token",
                msg,
            ),
            AuthError::InvalidResponse(msg) => {
                (StatusCode::UNAUTHORIZED, "Response failed to validate", msg)
            }
            AuthError::WriteSessionError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not write to session",
                msg,
            ),
            AuthError::TncSessionResponseError(msg) => {
                (StatusCode::BAD_REQUEST, "Could not create a session", msg)
            }
            AuthError::MissingChallenge(msg) => {
                (StatusCode::UNAUTHORIZED, "Missing credentials", msg)
            }
            AuthError::MissingSession(msg) => (StatusCode::NO_CONTENT, "Missing session", msg),
            AuthError::InvalidHeaderValue(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed HeaderValue", msg)
            }
            AuthError::MissingProperty(msg) => (
                StatusCode::BAD_REQUEST,
                "Missing data from the provider",
                msg,
            ),
            AuthError::MissingParameter(msg) => (
                StatusCode::BAD_REQUEST,
                "The request is missing a parameter",
                msg,
            ),
            AuthError::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error", msg)
            }
            AuthError::TokenCreation(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Token creation error",
                msg,
            ),
            AuthError::InvalidUrl(msg) => (StatusCode::NOT_FOUND, "Malformed url", msg),
            AuthError::UnsupportedProvider(msg) => {
                (StatusCode::BAD_REQUEST, "Invalid oauth provider", msg)
            }
            AuthError::ProjectIdError(msg) => {
                (StatusCode::BAD_REQUEST, "Require a valid project id", msg)
            }
            AuthError::ConfigError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Configuration error",
                msg,
            ),
            AuthError::MissingQuery(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Missing query", msg)
            }
            AuthError::DriveTokenError(msg) => {
                (StatusCode::UNAUTHORIZED, "Failed to retrieve token", msg)
            }
            AuthError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "Missing credentials", msg),
        };
        let body = Json(json!({
            "error": error,
            "message": message,
        }));
        (status, body).into_response()
    }
}

impl From<serde_json::Error> for AuthError {
    fn from(err: serde_json::Error) -> AuthError {
        AuthError::JsonParsingError(err.to_string().into())
    }
}
/*
impl<RE, T> From<RequestTokenError<RE, T>> for AuthError
where
    RE: Error + 'static,
    T: ErrorResponse + 'static,
{
    fn from(err: BasicRequestTokenError<RE>) -> AuthError {
        match err {
            ServerResponse(t) => AuthError::InvalidResponse(t.to_string.into()),
            Request(re) => AuthError::InvalidRequest(re.to_string.into()),
            Parse(err, msg) => {
                let msg = format!("{:?}\n{:#?}", err, msg);
                AuthError::InvalidRequest(re.to_string.into());
            }
            Other(str) => AuthError::InvalidRequest(str.to_string.into()),
        }
    }
}
*/

impl AuthError {
    pub fn trace(self) -> Self {
        tracing::error!("\n❌ Error:\n{:#?}", &self);
        self
    }
    pub fn trace_with_more<E, S>(self, message: Option<&S>, err: Option<E>) -> Self
    where
        E: std::fmt::Debug,
        S: std::convert::AsRef<str> + std::fmt::Display,
    {
        let message = match message {
            None => format!("\n❌ Error:\n{:?}", &err),
            Some(msg) => format!("\n❌ {}:\n{:?}", &msg, &err),
        };
        tracing::error!("{:?}\n{}", self, message);

        self
    }
    pub fn new<E>(err: E, message: Option<String>) -> Self
    where
        E: std::fmt::Debug,
    {
        let message = match message {
            None => format!("{:#?}", err),
            Some(msg) => format!("{}\n{:#?}", msg, err),
        };
        AuthError::InternalError((&message).into())
    }
}
