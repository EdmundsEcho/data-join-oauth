use crate::models::user::RawGoogleUser;
use axum::response::IntoResponse;

// Valid user session required. If there is none, redirect to the auth page
pub async fn handle(user: RawGoogleUser) -> impl IntoResponse {
    format!(
        "Welcome to the protected area :)\nHere's your info:\n{:?}",
        user
    )
}
