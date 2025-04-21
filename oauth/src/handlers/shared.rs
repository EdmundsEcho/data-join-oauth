///
/// # Authorization Code Grant w/ PKCE
/// Manages the session required to host state between the 2 phases
/// of the oauth2 redirect flow.
///
/// Shared logic for authorize and authenticate.
///
///
/// ## About pkce
///
/// The pkce is used by the auth provider (e.g., google) to make sure the
/// web client is the same as the client to whom the user agent authorized
/// to receive the token.
/// * Recall: The redirect only had the client_id, a public value.  In the
///   event the client_secret is compromized, pkce is useful.
/// * The other failsafe is the fact that the server-to-server communication
///   must use TLS (SSL); so if a server stole the password, it won't happen
///   anonymously.
///
/// Finally, the pkce is used when exchanging the code for a token.  The token
/// itself can be used as a stand-alone.
///
use async_redis_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use http::uri::InvalidUri;
use http::Uri;
use http::{header::InvalidHeaderValue, header::SET_COOKIE, HeaderMap};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeVerifier};
use std::fmt;

use crate::constants::{AUTH_SESSION_COOKIE, CSRF_COOKIE_NAME, PKCE_COOKIE_NAME};
use crate::errors::AuthError;
use crate::models::auth_return::AuthReturnValues;

/* -------------------------------------------------------------------------------- */
///
/// Phase one: set the session value
///
pub(crate) async fn set_session(
    headers: HeaderMap,
    store: RedisSessionStore,
    verifier: PkceCodeVerifier,
    csrf_state: CsrfToken,
) -> Result<HeaderMap, AuthError> {
    //
    // 🔐 CsrfToken generates a random key that will be returned in the state key.
    //    This should be compared with csrf_state to assert the user-agent has not changed.
    //
    // 📖 create a session to store what we need to reference once authorized
    //    Requires
    //    1. create a session object
    //    2. generating a session key from Redis
    //    3. storing the session id in the user-agent's cookie jar.
    //
    let mut session = Session::new();
    session.insert(PKCE_COOKIE_NAME, verifier).map_err(|err| {
        AuthError::WriteSessionError(format!("Writing pkce verifier: {}", err).into())
    })?;
    session
        .insert(CSRF_COOKIE_NAME, csrf_state)
        .map_err(|err| {
            AuthError::WriteSessionError(format!("Writing csrf verifier: {}", err).into())
        })?;
    //
    tracing::debug!("\n📚 session:\n{:#?}", &session);
    //
    // Write to redis, retrieve and store the location of the session
    // and put a copy of the session_key in a 🍪
    //
    let session_key_cookie = store
        .store_session(session)
        .await
        .map_err(|err| AuthError::ReadSessionError(err.to_string().into()))?
        .ok_or_else(|| AuthError::MissingSession("Missing session".into()))
        .and_then(|session_key: String| {
            //
            tracing::debug!("\n📚🔑: {}", &session_key);

            // stored session, copy to user-agent's cookie jar
            format!(
                "{}={}; SameSite=Lax; Path=/",
                AUTH_SESSION_COOKIE, session_key
            )
            .parse() // parse string to HeaderValue
            .map_err(|err: InvalidHeaderValue| {
                AuthError::InternalError(format!("Failed to create HeaderValue: {}", err).into())
            })
        })?;

    //
    // 🍪 Store the session_key key in a browser cookie
    //
    let mut headers = headers;
    headers.insert(SET_COOKIE, session_key_cookie);

    Ok(headers)
}

/* -------------------------------------------------------------------------------- */
///
/// ### Phase two
/// #### Retrieve the pkce
/// Required by the auth provider to prove the user agent retrieved code
/// is intended to be used by Luci; only we have access to the matching pkce.
///
pub(crate) async fn retrieve_validators(
    cookies: &headers::Cookie,
    store: &RedisSessionStore,
) -> Result<(PkceCodeVerifier, CsrfToken), AuthError> {
    //
    // debug status of the redis auth store
    let count = store.count().await.unwrap();
    tracing::debug!("\n📚 auth_store:\n{:#?}\n", &store);
    tracing::debug!("\n🧮 auth_store session count: {}\n", count);

    /* ------------------------------------------------------------------------------------- */
    // ☠️  retrieve the session to validate the user_agent that now has a code
    //    first get the id from the cookies
    /* ------------------------------------------------------------------------------------- */
    let session_id = cookies.get(AUTH_SESSION_COOKIE).ok_or_else(|| {
        AuthError::MissingSession(format!("missing session cookie: {}", AUTH_SESSION_COOKIE).into())
    })?;

    tracing::debug!(
        "\n1️⃣  {} cookie:\n{:?}\n",
        &AUTH_SESSION_COOKIE,
        &session_id
    );

    /* ------------------------------------------------------------------------------------- */
    // 🛡️ Retrieve the pkce_verifier from the sesiso
    //   A value is returned when valid?
    /* ------------------------------------------------------------------------------------- */
    let session: Session = store
        .load_session(session_id.to_string())
        .await
        .map_err(|err| AuthError::MissingSession(format!("no session store: {}", err).into()))?
        .ok_or_else(|| AuthError::MissingSession(format!("none with id: {}", session_id).into()))?;

    let verifier: PkceCodeVerifier = session
        .get(PKCE_COOKIE_NAME)
        .ok_or_else(|| AuthError::MissingChallenge("pkce_vefifier not in session".into()))?;

    let csrf_state: CsrfToken = session
        .get(CSRF_COOKIE_NAME)
        .ok_or_else(|| AuthError::MissingChallenge("csrf_state not in session".into()))?;

    tracing::debug!("\n2️⃣  verifier:\n{:?}\n", verifier.secret());

    Ok((verifier, csrf_state))
}
///
/// ### Phase two
/// #### step 2 get the session value
/// Get the token; requires providing the pkce verifier
///
pub(crate) async fn get_token(
    auth_return_values: &AuthReturnValues,
    client: &BasicClient,
    verifier: PkceCodeVerifier,
) -> Result<BasicTokenResponse, AuthError> {
    /* ------------------------------------------------------------------------------------- */
    // ✅ Got the code, now use it to get the token
    //
    // 👍 This is a secure, server to server exchange
    //
    /* ------------------------------------------------------------------------------------- */
    tracing::debug!(
        "\n👉 Exchange code for token:\n{:#?}\n🔑 code:{}",
        client.token_url(),
        &auth_return_values.code
    );
    // returns either a Token or RequestTokenError
    let token_response = client
        .exchange_code(AuthorizationCode::new(auth_return_values.code.clone()))
        .set_pkce_verifier(verifier)
        .request_async(async_http_client)
        .await
        .map_err(|err| {
            let message = format!("Access token response:\n{:#?}", err);
            AuthError::TokenCreation(message.into())
        })?;
    Ok(token_response)
}
/* -------------------------------------------------------------------------------- */
///
/// Shared support for converting Url to Uri in a type-safe manner.
/// WIP: There is likely a much more succinct and reliable way to accomplish this.
///
pub(crate) fn from_url(url: reqwest::Url) -> Result<Uri, AuthError> {
    //
    let host: url::Host<&str> = match url.host() {
        None => Err(AuthError::InvalidUrl(
            (&"Auth Url does not have a valid host").into(),
        )),
        Some(host) => Ok(host),
    }?;

    let path = url.path();

    let query = url
        .query()
        .ok_or_else(|| AuthError::InvalidUrl((&"Url missing query".to_string()).into()))?;

    let authority: Result<http::uri::Authority, AuthError> = TryFrom::try_from(host.to_string())
        .map_err(|err: InvalidUri| {
            let message = &format!("Uri builder failed:\n{}", &err.to_string());
            AuthError::InvalidUrl(message.into())
        });

    http::uri::Uri::builder()
        .scheme(url.scheme())
        .authority(authority?)
        .path_and_query(format!("{}?{}", path, query))
        .build()
        .map_err(|err| {
            let message = &format!("Uri builder failed:\n{}", &err.to_string());
            AuthError::InvalidUrl(message.into())
        })
}
/* -------------------------------------------------------------------------------- */
///
/// 📬 Redirect the client to the auth provider to auth service
///
pub(crate) fn trace_end_part_1<T, U, V>(provider: T, auth_url: U, redirect_url: V)
where
    T: fmt::Display,
    U: fmt::Display,
    V: fmt::Display,
{
    tracing::debug!("\n📬 {} endpoint redirects:", provider);
    tracing::debug!("\n🔗 😈->🙂 auth_url:\n{}", auth_url,);
    tracing::debug!("\n🔗 💫 redirect_url: {}\n", redirect_url);
}
