use async_redis_session::RedisSessionStore;
use axum::extract::{Extension, Path};
use axum::response::Redirect;
use http::HeaderMap;
use oauth2::{CsrfToken, PkceCodeChallenge, Scope};

use crate::errors::AuthError;
use crate::handlers::shared;
use crate::models::oauth_clients::{OauthClient, OauthClients};
use crate::models::oauth_provider::OauthProvider;

use axum_macros::debug_handler;
///
/// Handle the /auth/<provider> endpoints
///
/// Depends on initialized oauth2::BasicCient values keyed by oauth_providers
/// enumerated in models::oauth_provider
///
#[debug_handler]
pub async fn handle(
    Path(oauth_provider): Path<OauthProvider>,
    Extension(auth_store): Extension<RedisSessionStore>,
    Extension(clients): Extension<OauthClients>,
) -> Result<(HeaderMap, Redirect), AuthError> {
    if let Some(OauthClient { client, scope, .. }) = clients.get(&oauth_provider) {
        //
        // 🟢 kick-off the process by creating the call-back url.
        //    It will include one-way keys (csrf and pkce)
        //    Create a PKCE code verifier and SHA-256 encode it as a code challenge.
        //
        tracing::debug!("\n🟢 Authentication kick-off: {}\n", &oauth_provider,);
        let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

        let mut auth_req = client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_code_challenge)
            .add_scope(Scope::new(scope.to_string()));

        match oauth_provider {
            OauthProvider::Google => {
                auth_req = auth_req
                    .add_extra_param("prompt", "consent")
                    .add_extra_param("access_type", "offline");
            }
            _ => { /* no-op for other providers */ }
        }

        // Finalize the URL + CSRF state
        let (auth_url, csrf_state) = auth_req.url();

        let headers =
            shared::set_session(HeaderMap::new(), auth_store, pkce_code_verifier, csrf_state)
                .await?;

        tracing::debug!("\n📖🍪 headers: {:?}\n", headers);

        //
        // 📬 Redirect the client to the auth provider to authorize the user
        //
        let auth_url = shared::from_url(auth_url)?;
        shared::trace_end_part_1(
            oauth_provider,
            &auth_url,
            format!("{:?}", &client.redirect_url()),
        );

        Ok((headers, Redirect::to(auth_url)))
    } else {
        Err(AuthError::UnsupportedProvider(
            (&("Auth client not found")).into(),
        ))
    }
}
