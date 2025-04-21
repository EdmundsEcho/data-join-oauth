use async_redis_session::RedisSessionStore;
use axum::extract::{Extension, Path};
use axum::response::Redirect;
use http::HeaderMap;
use oauth2::{CsrfToken, PkceCodeChallenge, Scope};

use crate::errors::AuthError;
use crate::handlers::shared;
use crate::models::drive_clients::{DriveClient, DriveClients};
use crate::models::drive_provider::DriveProvider;
use crate::models::project_id::ProjectId;

use axum_macros::debug_handler;
///
/// Handle the /drive/<provider>/<project_id> endpoints
///
/// Depends on
///
/// * initialized oauth2::BasicCient values keyed by drive_providers
///   enumerated in models::drive_provider
///
/// * a valide project_id
///
#[debug_handler]
pub async fn handle(
    Path((drive_provider, project_id)): Path<(DriveProvider, ProjectId)>,
    Extension(auth_store): Extension<RedisSessionStore>,
    Extension(clients): Extension<DriveClients>,
) -> Result<(HeaderMap, Redirect), AuthError> {
    if let Some(DriveClient { client, scopes, .. }) = clients.get(&drive_provider) {
        //
        // 🟢 kick-off the process by creating the call-back url.
        //    It will include one-way keys (csrf and pkce)
        //    Create a PKCE code verifier and SHA-256 encode it as a code challenge.
        //
        tracing::debug!("\n🟢 🗄️  Authorize kick-off: {}\n", drive_provider,);

        let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

        // NEW
        let mut auth_url_builder = client
            // ⚠️  Include project_id in the state
            .authorize_url(|| CsrfToken::new(project_id.to_string()))
            .set_pkce_challenge(pkce_code_challenge)
            .add_scopes(scopes.iter().map(|s| Scope::new(s.to_string())));

        match drive_provider {
            DriveProvider::Google => {
                auth_url_builder = auth_url_builder
                    .add_extra_param("prompt", "consent")
                    .add_extra_param("access_type", "offline");
            }
            DriveProvider::DropBox => {
                auth_url_builder = auth_url_builder.add_extra_param("token_access_type", "offline");
            }
            _ => { /* no-op for other providers */ }
        }

        let (auth_url, csrf_state) = auth_url_builder
            .add_extra_param("refresh_token_key", "refresh_access") // optional naming
            .url();

        let headers =
            shared::set_session(HeaderMap::new(), auth_store, pkce_code_verifier, csrf_state)
                .await?;

        tracing::debug!("\n🍪 headers: {:?}\n", headers);
        tracing::debug!("\n>>> auth_url: {:?}\n", &auth_url);

        //
        // 📬 Redirect the client to the auth provider to authorize the user
        //
        let auth_url = shared::from_url(auth_url)?;
        shared::trace_end_part_1(
            drive_provider,
            &auth_url,
            format!("{:?}", &client.redirect_url()),
        );

        Ok((headers, Redirect::to(auth_url)))
    } else {
        Err(AuthError::UnsupportedProvider(
            format!("Auth client not found: {}", drive_provider).into(),
        ))
    }
}
