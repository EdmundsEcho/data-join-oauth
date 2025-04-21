use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use secrecy::ExposeSecret;
use std::collections::HashMap;

use crate::config::{config_get, tnc_authorized_endpoint};
use crate::errors::AuthError;
use crate::models::oauth_provider::OauthProvider;

///
/// Host the BasicClient and any other information required
/// to complete both phases of the OAuth2 strategy.
///
#[derive(Debug, Clone)]
pub struct OauthClient {
    pub client: BasicClient,
    pub scope: String,
    pub identity_server: String,
}
impl OauthClient {
    fn new(client: BasicClient, scope: String, identity_server: String) -> Self {
        OauthClient {
            client,
            scope,
            identity_server,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OauthClients(pub HashMap<OauthProvider, OauthClient>);

impl OauthClients {
    pub fn get(&self, key: &OauthProvider) -> Option<&OauthClient> {
        self.0.get(key)
    }
}

///
/// Initialize the OauthClients by reading in the CONFIG
///
pub fn init() -> Result<OauthClients, AuthError> {
    // generic for all clients
    let endpoint = tnc_authorized_endpoint()?;

    // clients
    let oauth_servers = &config_get()?.oauth_servers;
    let mut oauth_clients = HashMap::new();

    for (auth_service, cfg) in oauth_servers.iter() {
        let redirect_uri = format!("{}/{}", endpoint, &auth_service.to_path());

        oauth_clients.insert(
            auth_service.clone(),
            OauthClient::new(
                BasicClient::new(
                    ClientId::new(cfg.client_id.expose_secret().clone()),
                    Some(ClientSecret::new(cfg.client_secret.expose_secret().clone())),
                    AuthUrl::new(cfg.auth_url.clone()).unwrap(),
                    Some(TokenUrl::new(cfg.token_url.clone()).unwrap()),
                )
                .set_redirect_uri(RedirectUrl::new(redirect_uri.clone()).unwrap()),
                // .set_revocation_uri(
                //     RevocationUrl::new(cfg.revocation_url.clone().unwrap())
                //         .expect("Invalid revocation endpoint URL"),
                // ),
                cfg.scope.clone(),
                cfg.identity_server.clone(),
            ),
        );
    }
    tracing::info!("😈->🙂 oauth clients: {:?}", &oauth_clients.keys());

    Ok(OauthClients(oauth_clients))
}
