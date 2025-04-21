use async_redis_session::RedisSessionStore;
use async_session::SessionStore;
use axum::extract::{Extension, TypedHeader};
use axum::response::{IntoResponse, Redirect};

use crate::constants::AUTH_SESSION_COOKIE;

pub async fn handle(
    Extension(store): Extension<RedisSessionStore>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
) -> impl IntoResponse {
    let cookie = cookies.get(AUTH_SESSION_COOKIE).unwrap();
    let session = match store.load_session(cookie.to_string()).await.unwrap() {
        Some(s) => s,
        // No session active, just redirect
        None => return Redirect::to("/".parse().unwrap()),
    };

    store.destroy_session(session).await.unwrap();

    Redirect::to("/".parse().unwrap())
}
