use async_redis_session::RedisSessionStore;
use axum::extract::{Extension, Path, Query, TypedHeader};
use axum::http::header::{HeaderMap, HeaderValue, CONTENT_TYPE, SET_COOKIE, USER_AGENT};
use axum::response::Redirect;
use http::header::ACCEPT;
use serde_json::{to_string_pretty, Value};

use oauth2::TokenResponse;

use crate::config::{tnc_app_endpoint, tnc_register_endpoint};
use crate::errors::AuthError;
use crate::handlers::shared;
use crate::models::auth_return::AuthReturnValues;
use crate::models::oauth_clients::{OauthClient, OauthClients};
use crate::models::oauth_provider::OauthProvider;
use crate::models::user;
use crate::models::user_registration::UserRegistration;

///
/// This is the second phase of the Authorization Code Grant w/pkce flow
///
/// 👉 success means we have an authenticated user
///
/// Tasks:
/// 1. Verify the user-agent is the same as what initiated the flow
/// 2. Use the code to retrieve the token
/// 3. Retrieve the user profile
/// 4. Instantiate a RawUser from any of the supported oauth services
/// 5. Register-login the user
/// 6. Redirect the user to the app
///
/// This is a trusted, machine to machine exchange.
///
pub async fn handle(
    Path(oauth_provider): Path<OauthProvider>,
    Query(auth_return_values): Query<AuthReturnValues>,
    Extension(store): Extension<RedisSessionStore>,
    Extension(clients): Extension<OauthClients>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
) -> Result<(HeaderMap, Redirect), AuthError> {
    if let Some(OauthClient {
        client,
        identity_server,
        ..
    }) = clients.get(&oauth_provider)
    {
        tracing::debug!(
            "\n📥 ...User arrived authenticated: received a code from {}:\n🔑☠️ ? code: {}",
            &oauth_provider,
            &auth_return_values.code
        );

        /* ------------------------------------------------------------------------- */
        // pkce required by the auth provider to prove this client requested the code
        // ⬜ compare csrf_state with returned values
        /* ------------------------------------------------------------------------- */
        let (pkce, _csrf_state) = shared::retrieve_validators(&cookies, &store).await?;

        /* ------------------------------------------------------------------------- */
        // ✅ get the resource token
        /* ------------------------------------------------------------------------- */
        let token_response = shared::get_token(&auth_return_values, client, pkce).await?;

        tracing::debug!(
            "\n✅ Token from Auth Provider (opaque):\n{:#?}\n",
            &token_response
        );

        /* ------------------------------------------------------------------------- */
        // 👉 Fetch user data from the identity provider (protected resource)
        /* ------------------------------------------------------------------------- */
        let client = reqwest::Client::new();
        let user_data = client
            .get(identity_server)
            .bearer_auth(token_response.access_token().secret())
            .header(USER_AGENT, "Web App login")
            .header(CONTENT_TYPE, "application/json")
            .send() // convert Request to a Future
            .await
            .map_err(|err| {
                let message = format!("Identity response:\n{:?}", err);
                AuthError::InvalidResponse(message.into())
            })?;

        /* ------------------------------------------------------------------------- */
        // Parse the user data from the response body
        /* ------------------------------------------------------------------------- */
        let user_data: Value = user_data.json::<Value>().await.map_err(|err| {
            let message = format!("Unexpected user identity data: {}", err);
            AuthError::JsonParsingError(message.into())
        })?;

        // pretty print
        tracing::debug!("\n🎉 body:\n{:#?}\n", &user_data);

        /* ------------------------------------------------------------------------- */
        // Delegate based on on the auth provider
        // Provider-specific stage -> RawUser
        // 🔖 Keep it light; request detail with our own forms
        // ⬜ Clean-up the serde_json
        /* ------------------------------------------------------------------------- */
        let op = &oauth_provider.clone(); // fix the need to create a copy this way
        let raw_user: user::RawUser = match oauth_provider {
            OauthProvider::Google => Ok(From::from(
                serde_json::from_value::<user::RawFromGoogle>(user_data).unwrap(),
            )),
            OauthProvider::Azure => Ok(From::from(
                serde_json::from_value::<user::RawFromAzure>(user_data).unwrap(),
            )),
            OauthProvider::Twitter => Ok(From::from(
                serde_json::from_value::<user::RawFromTwitter>(user_data).unwrap(),
            )),
            OauthProvider::Github => Ok(From::from(
                serde_json::from_value::<user::RawFromGithub>(user_data).unwrap(),
            )),
            OauthProvider::LinkedIn => Ok(From::from(
                serde_json::from_value::<user::RawFromLinkedIn>(user_data).unwrap(),
            )),
            OauthProvider::Discord => Ok(From::from(
                serde_json::from_value::<user::RawFromDiscord>(user_data).unwrap(),
            )),
            v => Err(AuthError::UnsupportedProvider(
                format!("Missing user instance: {}", v).into(),
            )),
        }?;
        // pretty print
        tracing::debug!("\n🙂 🎉 RawUser:\n{:#?}\n", &raw_user);

        /* ------------------------------------------------------------------------- */
        // 🚧 May not need extra information about the Authenticated user.
        // ⬆  Redirect to a user-form to augment the profile as required
        // 📋 Pretty print rust instance as a json string (preview what is sent)
        /* ------------------------------------------------------------------------- */

        // convert RawUser to UserRegistration
        // e.g., UserRegistration { auth_agent: Twitter, auth_id: "43467549", email: "" }
        let user_registration: UserRegistration = raw_user.into();
        tracing::debug!(
            "\n🛡️ {} \n📦 Preview body to tnc store:\n{}\n",
            &op,
            to_string_pretty(&user_registration)?
        );

        /* ------------------------------------------------------------------------- */
        // 🙂 Register the user
        //
        // Get a tnc session
        // sessions-service - register with the postgrest api to get a session
        // 🔐  Use local kube address
        /* ------------------------------------------------------------------------- */
        let session_url = tnc_register_endpoint()?;

        tracing::debug!("\n🔗 tnc session_url:\n{}\n", &session_url);

        let session = client
            .post(session_url.to_string())
            .header(USER_AGENT, "Luci Web Authorization")
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .json(&user_registration)
            .send()
            .await
            .map_err(|err| AuthError::TncSessionResponseError((&err).into()))?;

        // Retrieve the Set-Cookie header value
        tracing::debug!("\n🎉 Response from tnc session service:\n{:#?}\n", &session);
        let session_cookie: HeaderValue = session
            .headers()
            .get(SET_COOKIE)
            .ok_or_else(|| {
                AuthError::MissingCookie("Tried to forward; missing session cookie".into())
            })?
            .to_owned();

        tracing::debug!(
            "\n🍪 Cookie (key=value) from session:\n{:?}\n",
            &session_cookie
        );

        let redirect_uri = tnc_app_endpoint()?;
        tracing::debug!("\n🔗 👉 redirect uri:\n{}\n", &redirect_uri);

        // update the headers with the Set-Cookie to complete the forwarding task
        let mut headers = HeaderMap::new();
        headers.insert(SET_COOKIE, session_cookie);

        Ok((headers, Redirect::to(redirect_uri)))
    } else {
        Err(AuthError::UnsupportedProvider(
            "Auth client not found".into(),
        ))
    }
}
