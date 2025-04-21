use axum::http::uri::Uri;
use axum::response::{IntoResponse, Redirect, Response};
use std::fmt;

use crate::errors::AuthError;

///
/// Retry upon failure; this will need to be limited
/// WIP: Make more generic
///
pub struct AuthFailedRedirect {
    uri: Uri,
}
impl AuthFailedRedirect {
    #[allow(dead_code)]
    pub fn new<T>(provider: &T) -> Result<Self, AuthError>
    where
        T: fmt::Display,
    {
        let uri = format!("/auth/{}", &provider);
        let uri =
            Uri::try_from(uri).map_err(|err| AuthError::InvalidUrl(err.to_string().into()))?;
        Ok(AuthFailedRedirect { uri })
    }
}
impl IntoResponse for AuthFailedRedirect {
    fn into_response(self) -> Response {
        Redirect::temporary(self.uri).into_response()
    }
}
