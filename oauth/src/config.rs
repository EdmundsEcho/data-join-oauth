use anyhow::Result;
use arc_swap::{ArcSwap, Guard};
use axum::http::uri::Uri;
use clap::Parser;
use config::{Config, ConfigError, Environment, File};
use once_cell::sync::OnceCell;
use secrecy::Secret;
use serde::Deserialize;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::{env, fmt};

use crate::errors::AuthError;
use crate::models::drive_provider::DriveProvider;
use crate::models::oauth_provider::OauthProvider;
use crate::models::project_id::ProjectId;
//
//
// Models:
//
// 👉 Settings
//      👉 Options
//      👉 OauthServers
//
// let oauth_servers = &config_get()?.oauth_servers;
// let options = &config_get()?.options;
//
const CONFIG_FILE_PATH: &str = "config/default.toml";
const CONFIG_FILE_ROOT: &str = "config";

//------------------------------------------------------------------------------
///
/// Settings - Hosts the app's configuration specified using the *external*
/// toml files. These configuration models are subsequently read-in by
/// DriveClients. The DriveClients hosts a final, separate config type.
///
/// 🔖 The RUST_ENV is used to determine which configuration is loaded.
///    In the event the RUST_ENV is not set: Development.
///    (This means that RUST_ENV cannot be set by this configuration)
///
/// toml   -- Config package ->
/// config -- DriveClients builder -> DriveClients
///
/// .toml files must follow this structure (or vice-versa)
///
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub options: Options,
    pub oauth_servers: OauthServers,
    pub drive_servers: DriveServers,
}
// create from file
impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mode = env::var("RUST_ENV").unwrap_or_else(|_| "Development".into());

        tracing::info!("\n\n🦀 mode: {}", &mode);

        let settings = Config::builder()
            .add_source(File::with_name(CONFIG_FILE_PATH))
            // use env Development, Production or Testing to set override
            .add_source(
                File::with_name(&format!("{}/{}.toml", CONFIG_FILE_ROOT, mode.to_lowercase()))
                    .required(true),
            )
            .add_source(Environment::default().separator("_"))
            .build()?;

        settings.try_deserialize()
    }
}

//------------------------------------------------------------------------------
///
/// Server options
/// Options that drive where and how to operate in the local tnc network
/// ⬜ harden by requiring Uri instead of String
///
#[derive(Parser, Debug, Deserialize, Clone)]
#[clap(
    name = "oauth service",
    about = "Authenticates and authorizes users and access to user resources"
)]
pub struct Options {
    #[clap(
        short = 'l',
        long = "log",
        default_value = "oauth=trace,tower_http=trace"
    )]
    pub rust_log: String,

    #[clap(short = 'a', long = "addr", default_value = "locahost")]
    pub host: IpAddr,

    #[clap(short = 'p', long = "port", default_value = "3000")]
    pub port: u16,
    //
    pub root_dir: String,
    //
    pub redis_db: Secret<String>,
    pub redis_pool_size: u16,

    pub tnc_authorized_endpoint: String,
    pub tnc_authorized_drive_endpoint: String,
    pub tnc_register_endpoint: String,
    pub tnc_app_endpoint: String,
    pub tnc_drive_token_endpoint: String,
    pub tnc_filesystem_endpoint: String,
}

//------------------------------------------------------------------------------
///
/// 🙂 OauthServer specifications
///
#[derive(Debug, Deserialize, Clone)]
pub struct OauthServer {
    pub auth_url: String,
    pub token_url: String,
    pub client_id: Secret<String>,
    pub client_secret: Secret<String>,
    pub identity_server: String,
    #[serde(default)]
    pub revocation_url: Option<String>,
    pub scope: String,
}
pub type OauthServers = HashMap<OauthProvider, OauthServer>;

//------------------------------------------------------------------------------
///
/// 🗄️  AutDriveServer specifications
///
/// Hosts the configuration information for each of the supported drive providers.
///
/// ⬜ May create specialized, drive-specific "secrets" configuration
/// ⬜ Perhaps be more specific with the types (ie., not just String)
///
#[derive(Debug, Deserialize, Clone)]
pub struct DriveServer {
    pub auth_uri: String,
    pub token_uri: String,
    pub client_id: Secret<String>,
    pub client_secret: Secret<String>,
    pub project_id: Option<String>,
    pub scopes: Vec<String>,
    pub(crate) files_request: FilesRequest,
}
// private
#[derive(Debug, Deserialize, Clone)]
pub(crate) struct FilesRequest {
    pub method: Option<String>,
    pub drive_server: String,
    pub endpoint: String,
    pub query_ls: String,
    pub query_read: Option<String>,
    pub json_body_ls: Option<String>,
}
pub type DriveServers = HashMap<DriveProvider, DriveServer>;

//------------------------------------------------------------------------------
// RUST_ENV
//
// ⬜ #[serde(rename_all = "lowercase")]
#[derive(Clone, Debug, Deserialize)]
#[allow(non_camel_case_types)]
pub enum RUST_ENV {
    Development,
    Testing,
    Production,
}
impl fmt::Display for RUST_ENV {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RUST_ENV::Development => write!(f, "Development"),
            RUST_ENV::Testing => write!(f, "Testing"),
            RUST_ENV::Production => write!(f, "Production"),
        }
    }
}
impl From<&str> for RUST_ENV {
    fn from(env: &str) -> Self {
        match env {
            "Testing" => RUST_ENV::Testing,
            "Production" => RUST_ENV::Production,
            _ => RUST_ENV::Development,
        }
    }
}
//------------------------------------------------------------------------------
// Utility functions to echo endpoints
//
pub fn tnc_authorized_endpoint() -> Result<Uri, AuthError> {
    let endpoint: &String = &config_get()?.options.tnc_authorized_endpoint;

    let endpoint = Uri::try_from(endpoint).map_err(|err| {
        let message = format!("Authorized endpoint uri failed\n{:?}", &err);
        AuthError::InvalidUrl(message.into()).trace()
    })?;

    Ok(endpoint)
}
pub fn tnc_authorized_drive_endpoint() -> Result<Uri, AuthError> {
    let endpoint: &String = &config_get()?.options.tnc_authorized_drive_endpoint;

    let endpoint = Uri::try_from(endpoint).map_err(|err| {
        let message = format!("Tnc: Authorized drive endpoint uri failed\n{:?}", &err);
        AuthError::InvalidUrl(message.into()).trace()
    })?;

    Ok(endpoint)
}
pub fn tnc_register_endpoint() -> Result<Uri, AuthError> {
    let endpoint: &String = &config_get()?.options.tnc_register_endpoint;

    let endpoint = Uri::try_from(endpoint).map_err(|err| {
        let message = format!("Tnc: Register endpoint uri failed\n{:?}", &err);
        AuthError::InvalidUrl(message.into()).trace()
    })?;

    Ok(endpoint)
}
pub fn tnc_app_endpoint() -> Result<Uri, AuthError> {
    let endpoint: &String = &config_get()?.options.tnc_app_endpoint;

    let endpoint = Uri::try_from(endpoint).map_err(|err| {
        let message = format!("Tnc: Auth session uri failed\n{:?}", &err);
        AuthError::InvalidUrl(message.into()).trace()
    })?;

    Ok(endpoint)
}
pub fn tnc_drive_token_endpoint() -> Result<Uri, AuthError> {
    let endpoint: &String = &config_get()?.options.tnc_drive_token_endpoint;

    let endpoint = Uri::try_from(endpoint).map_err(|err| {
        let message = format!("Tnc: Drive token uri failed\n{:?}", &err);
        AuthError::InvalidUrl(message.into()).trace()
    })?;

    Ok(endpoint)
}
///
/// Takes ownership of project_id to promote limited reuse
///
pub fn set_tnc_filesystem_endpoint(project_id: Option<ProjectId>) -> Result<Uri, AuthError> {
    let prefix: &String = &config_get()?.options.tnc_filesystem_endpoint;
    let endpoint = match project_id {
        None => format!("{}/project_id/files", prefix),
        Some(pid) => format!("{}/{}/files", prefix, pid),
    };

    let endpoint = Uri::try_from(endpoint).map_err(|err| {
        let message = format!("Tnc: Filesystem endpoint\n{:?}", &err);
        AuthError::InvalidUrl(message.into()).trace()
    })?;

    Ok(endpoint)
}

//------------------------------------------------------------------------------
// config getter and setter
//
// use once_cell::sync::OnceCell;
// ⬜ See how to integrate this one_cell streamlined approach
// let cell = OnceCell::new();
// let value = cell.get_or_init(|| 92);
// assert_eq!(value, &92);
//
pub fn config_get() -> Result<Guard<Arc<Settings>>, AuthError> {
    let cfg = CFG
        .get()
        .ok_or_else(|| AuthError::ConfigError("Config was not initialized".into()))?
        .load();
    Ok(cfg)
}
pub fn config_init() -> Result<(), AuthError> {
    // instantiate the configuration
    let new_cfg = Settings::new().map_err(|e| AuthError::ConfigError(e.to_string().into()))?;
    // set or get the handle to arc
    if CFG.get().is_none() {
        CFG.set(ArcSwap::from_pointee(new_cfg)).unwrap();
    } else {
        CFG.get().unwrap().store(Arc::new(new_cfg));
    }
    Ok(())
}

pub static CFG: OnceCell<ArcSwap<Settings>> = OnceCell::new();
