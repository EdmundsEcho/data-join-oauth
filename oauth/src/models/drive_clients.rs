use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use secrecy::ExposeSecret;
use std::collections::HashMap;

use crate::config::{config_get, tnc_authorized_drive_endpoint};
use crate::errors::AuthError;
use crate::models::drive_provider::DriveProvider;

///
/// Host the BasicClient and any other information required
/// to complete the strategy once authenticated (e.g., scope)
///
#[derive(Debug, Clone)]
pub struct DriveClient {
    pub client: BasicClient,
    pub scopes: Vec<String>,
    pub files_request: FilesRequest,
}
impl DriveClient {
    fn new(client: BasicClient, scopes: Vec<String>, files_request: FilesRequest) -> Self {
        DriveClient {
            client,
            scopes,
            files_request,
        }
    }
}
/// final version (instantiated using model in config)
/// public
#[derive(Debug, Clone)]
pub struct FilesRequest {
    pub method: String,
    pub drive_server: String,
    pub endpoint: String,
    pub query_ls: String,
    pub query_read: Option<String>,
    pub json_body_ls: Option<String>,
}
#[derive(Debug, Clone)]
pub struct DriveClients(pub HashMap<DriveProvider, DriveClient>);

#[allow(dead_code)]
impl DriveClients {
    pub fn get(&self, key: &DriveProvider) -> Option<&DriveClient> {
        self.0.get(key)
    }
}
///
/// Initialize the Clients by reading in the CONFIG (config_get)
///
/// toml   -- Config package ->
/// config -- DriveClients builder -> DriveClients
///
pub fn init() -> Result<DriveClients, AuthError> {
    // generic for all drive clients
    let endpoint = tnc_authorized_drive_endpoint()?;

    // clients
    let drive_servers = &config_get()?.drive_servers;
    let mut clients = HashMap::new();

    for (drive_service, cfg) in drive_servers.iter() {
        let redirect_uri = format!("{}/{}", endpoint, &drive_service.to_path());

        clients.insert(
            drive_service.clone(),
            DriveClient::new(
                BasicClient::new(
                    ClientId::new(cfg.client_id.expose_secret().clone()),
                    Some(ClientSecret::new(cfg.client_secret.expose_secret().clone())),
                    AuthUrl::new(cfg.auth_uri.clone()).unwrap(),
                    Some(TokenUrl::new(cfg.token_uri.clone()).unwrap()),
                )
                .set_redirect_uri(
                    RedirectUrl::new(redirect_uri.clone())
                        .map_err(|err| AuthError::InvalidUrl(err.to_string().into()))?,
                ),
                cfg.scopes.clone(),
                FilesRequest {
                    method: cfg
                        .files_request
                        .method
                        .clone()
                        .unwrap_or_else(|| "post".to_string()),
                    drive_server: cfg.files_request.drive_server.clone(),
                    endpoint: cfg.files_request.endpoint.clone(),
                    query_ls: cfg.files_request.query_ls.clone(),
                    query_read: cfg.files_request.query_read.clone(),
                    json_body_ls: cfg.files_request.json_body_ls.clone(),
                },
            ),
        );
    }
    tracing::info!("🔐->🗄️  drive clients: {:?}", &clients.keys());

    Ok(DriveClients(clients))
}
