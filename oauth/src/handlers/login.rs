use axum::response::Html;

//------------------------------------------------------------------------------
// login
#[allow(dead_code)]
pub async fn handle() -> Html<&'static str> {
    // relative to main.rs
    // Html(include_str!("../public/login.html"))
    Html("<h1>Not yet implemented</h1>")
}
