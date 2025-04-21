use async_redis_session::RedisSessionStore;

// use axum::error_handling::HandleErrorLayer;

use axum::extract::Extension;

use axum::http::header::{HeaderValue, USER_AGENT};

use axum::{
    handler::Handler, http::StatusCode, response::IntoResponse, routing::get, AddExtensionLayer,
    Router,
};

use secrecy::ExposeSecret;

use tokio::sync::mpsc;

use tower::ServiceBuilder;

use tower_http::set_header::SetRequestHeaderLayer;

use tower_http::trace::TraceLayer;

#[path = "constants.rs"]
mod constants;

#[path = "handlers/mod.rs"]
mod handlers;

#[path = "models/mod.rs"]
mod models;

#[path = "middleware/mod.rs"]
mod middleware;

#[path = "errors.rs"]
mod errors;

#[path = "state.rs"]
mod state;

#[path = "config.rs"]
pub mod config;

use crate::config::config_get;

use crate::errors::{AuthError, Result};

use crate::handlers::*;

use crate::models::drive_clients;

use crate::models::oauth_clients;

pub fn app() -> Result<Router> {
    let redis_uri = config_get()?.options.redis_db.expose_secret().clone();

    let redis_client = redis::Client::open(redis_uri).unwrap();

    let auth_store = RedisSessionStore::from_client(redis_client);

    let oauth_clients = oauth_clients::init()?;

    let drive_clients = drive_clients::init()?;

    let middleware_stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http()) // tower_http=trace to activate
        .layer(AddExtensionLayer::new(auth_store))
        .layer(AddExtensionLayer::new(oauth_clients))
        .layer(AddExtensionLayer::new(drive_clients))
        .layer(SetRequestHeaderLayer::overriding(
            USER_AGENT,
            HeaderValue::from_static("Luci Auth Service"),
        ))
        // .layer(SetSensitiveHeadersLayer::new(iter::once(AUTHORIZATION)))
        .into_inner();

    // debug only

    // .layer(axum_middleware::from_fn(middleware::print_response::print))

    // the app
    let router = Router::new()
        .route("/livez", get(health_check))
        .route("/reload", get(reload))
        .route("/favicon.ico", get(favicon::go))
        .route("/auth/:auth_provider", get(authenticate::handle))
        .route("/drive/:auth_provider/:project_id", get(authorize::handle))
        .route(
            "/auth/authorized/:auth_provider",
            get(login_authorized::handle),
        )
        .route(
            "/drive/authorized/:auth_provider",
            get(drive_authorized::handle),
        )
        // testing how to access files
        .route(
            "/drive/:auth_provider/:project_id/filesystem",
            get(filesystem::handle),
        )
        .route("/api/logout", get(logout::handle))
        .fallback(handler_404.into_service())
        .layer(middleware_stack);

    Ok(router)
}

// see https://kubernetes.io/docs/reference/using-api/health-checks/
async fn health_check() -> impl IntoResponse {
    "ok"
}

// global fallback
async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Auth service 404")
}

// for dev purposes; reload the config files
async fn reload(Extension(shutdown_handle): Extension<mpsc::Sender<()>>) -> impl IntoResponse {
    shutdown_handle
        .send(())
        .await
        .map_err(|e| AuthError::InternalError(e.to_string().into()))
}
