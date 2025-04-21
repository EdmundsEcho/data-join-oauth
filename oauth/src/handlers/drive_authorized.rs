///
/// Second phase associated with retrieving a shared drive token
///
/// WIP:
///
/// ⬜ Register the token with postgREST
/// ⬜ Stop displaying the redis store credentials
/// ⬜ Figure out how to validate the state and/or use csrf
/// ⬜ Confirm whether using the same cookie key somehow prevents the browser
///    from recognizing "already done"
///
use async_redis_session::RedisSessionStore;
use axum::extract::{Extension, Path, Query, TypedHeader};
use axum::http::header::{ACCEPT, CONTENT_TYPE, COOKIE, USER_AGENT};
use axum::response::Redirect;
use oauth2::TokenResponse;
use serde_json;

use crate::config::{set_tnc_filesystem_endpoint, tnc_drive_token_endpoint};
use crate::constants::TNC_SESSION_COOKIE;
use crate::errors::AuthError;
use crate::handlers::shared;
use crate::models::auth_return::AuthReturnValues;
use crate::models::drive_clients::{DriveClient, DriveClients};
use crate::models::drive_provider::DriveProvider;
use crate::models::drive_token::Builder;
use crate::models::project_id::ProjectId;

/* -------------------------------------------------------------------------- */
///
/// 📋 Use the auth code to retrieve the token.  This is a trusted, machine to
/// machine exchange. Then go ahead and retrieve the resource (user email).
///
/// 👉 Once a token has been acquired we can redirect the user to the api that
/// triggered the drive registration process; return with a response code
/// that signals success.  This will signal that the api can retrieve the token
/// and expect to connect to the drive
///
pub async fn handle<'a>(
    Path(drive_provider): Path<DriveProvider>,
    Query(auth_return_values): Query<AuthReturnValues>,
    Extension(store): Extension<RedisSessionStore>,
    Extension(clients): Extension<DriveClients>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
) -> Result<Redirect, AuthError> {
    if let Some(DriveClient { client, .. }) = clients.get(&drive_provider) {
        //
        tracing::debug!(
            "\n📥 ...User arrived authenticated: received a code from {}:\n🔑☠️ ? code: {}",
            &drive_provider,
            &auth_return_values.code
        );

        /* ------------------------------------------------------------------ */
        // pkce required by the auth provider to prove this client
        // requested the code
        //
        // ⬜ compare csrf_state with returned values
        /* ------------------------------------------------------------------ */
        let (pkce, _csrf_state) = shared::retrieve_validators(&cookies, &store).await?;

        /* ------------------------------------------------------------------ */
        // ✅ get the resource token
        /* ------------------------------------------------------------------ */
        let token_response = shared::get_token(&auth_return_values, client, pkce).await?;
        tracing::debug!(
            "\n ✅ Token response:\n{} \n",
            serde_json::to_string_pretty(&token_response)?
        );

        let builder = Builder::new(&token_response, client.token_url());

        let project_id = ProjectId::try_from(auth_return_values.state.clone())?;

        let drive_token = builder.build(&project_id, &drive_provider);

        /* ------------------------------------------------------------------ */
        // 🎉  Access Token - store for re-use
        //
        // ⬜ 👉 go to the session endpoint used to store tokens
        // Printing rust instance as json string
        //
        /* ------------------------------------------------------------------ */
        tracing::debug!(
            "\n🗄️  {}\n📦 Body to tnc store:\n{} \n",
            &drive_provider,
            serde_json::to_string_pretty(&drive_token)?
        );
        /* ------------------------------------------------------------------------- */
        // Store the drive token in postgres
        // 🔐  * Use local kube address
        //     * Requires the reqwest client have a valid sessionId in the cookies
        /* ------------------------------------------------------------------------- */
        let api_url = tnc_drive_token_endpoint()?;

        tracing::debug!("\n🔗 tnc drive token:\n{}\n", &api_url);

        /* ------------------------------------------------------------------------- */
        // 🔖 The task of submitting the token to the server & redirecting the
        //    user agent could be done in parallel.
        /* ------------------------------------------------------------------------- */
        let client = reqwest::Client::builder().build().map_err(|err| {
            AuthError::InternalError(format!("Failed to build client: {}", err).into())
        })?;
        let (_pkce, _csrf_state) = shared::retrieve_validators(&cookies, &store).await?;
        let session_id = cookies.get(&TNC_SESSION_COOKIE).ok_or_else(|| {
            AuthError::MissingSession(
                format!("missing session cookie: {}", &TNC_SESSION_COOKIE).into(),
            )
        })?;
        let _token = client
            .post(api_url.to_string())
            .header(USER_AGENT, "Luci Drive Authorization")
            .header(COOKIE, format!("{}={}", TNC_SESSION_COOKIE, session_id))
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .json(&drive_token)
            .send()
            .await
            .map_err(|err| AuthError::TncSessionResponseError(err.to_string().into()))?;

        tracing::debug!("\n✅ Tnc registered the drive_token:\n{:#?}\n", &_token);
        tracing::debug!(
            "\n🦀 👉 Get files:\n
                http://localhost:3099/drive/{drive_provider}/{project_id}/filesystem?access_token={token}\n",
            drive_provider = &drive_provider.to_string().to_lowercase(),
            project_id = auth_return_values.state,
            token = serde_json::to_string(&token_response.access_token())
                .map_err(|err| AuthError::JsonParsingError(err.to_string().into()))?
        );

        let redirect_uri = set_tnc_filesystem_endpoint(Some(project_id))?;
        tracing::debug!("\n🔗 👉 tnc redirect uri:\n{}\n", &redirect_uri);

        // redirect - the response status should encode success (vs failing to register)
        Ok(Redirect::to(redirect_uri))
    } else {
        Err(AuthError::UnsupportedProvider(
            (&("Auth client not found")).into(),
        ))
    }
}
