use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum_macros::AsRefStr;
use strum_macros::Display;

use crate::errors::AuthError;

///
/// Supported user authentication services
///
#[derive(Clone, Hash, Eq, PartialEq, Debug, Display, AsRefStr, Deserialize, Serialize)]
#[serde(try_from = "String")]
pub enum OauthProvider {
    #[serde(rename(serialize = "google"))]
    Google,
    #[serde(rename(serialize = "azure"))]
    Azure,
    #[serde(rename(serialize = "facebook"))]
    Facebook,
    #[serde(rename(serialize = "twitter"))]
    Twitter,
    #[serde(rename(serialize = "linkedIn"))]
    LinkedIn,
    #[serde(rename(serialize = "github"))]
    Github,
    #[serde(rename(serialize = "discord"))]
    Discord,
    #[serde(rename(serialize = "luci"))]
    Luci, //< If, when Luci hosts user/password
    #[serde(rename(serialize = "empty"))]
    Empty,
}
impl Default for OauthProvider {
    fn default() -> OauthProvider {
        OauthProvider::Empty
    }
}
///
/// Supported sources of data files
/// Align this with the default.toml configuration
///
impl From<String> for OauthProvider {
    fn from(input: String) -> OauthProvider {
        let input = input.as_str();
        match input {
            "google" => OauthProvider::Google,
            "azure" => OauthProvider::Azure,
            "facebook" => OauthProvider::Facebook,
            "twitter" => OauthProvider::Twitter,
            "linkedIn" => OauthProvider::LinkedIn,
            "github" => OauthProvider::Github,
            "discord" => OauthProvider::Discord,
            "luci" => OauthProvider::Luci,
            _ => OauthProvider::Empty,
        }
    }
}
///
/// Utilized by serde for default values of the Raw, staging versions
/// specified for each provider.
///
/// Align this with the default.toml configuration
///
impl FromStr for OauthProvider {
    type Err = AuthError;
    fn from_str(input: &str) -> Result<OauthProvider, Self::Err> {
        match input {
            "google" => Ok(OauthProvider::Google),
            "azure" => Ok(OauthProvider::Azure),
            "facebook" => Ok(OauthProvider::Facebook),
            "twitter" => Ok(OauthProvider::Twitter),
            "linkedIn" => Ok(OauthProvider::LinkedIn),
            "github" => Ok(OauthProvider::Github),
            "discord" => Ok(OauthProvider::Discord),
            "luci" => Ok(OauthProvider::Luci),
            v => Err(AuthError::UnsupportedProvider(v.into())),
        }
    }
}
///
/// Utilized by serde for default values of the Raw, staging versions
/// specified for each provider.
///
impl OauthProvider {
    pub fn to_path(&self) -> &str {
        match self {
            OauthProvider::Google => "google",
            OauthProvider::Azure => "azure",
            OauthProvider::Facebook => "facebook",
            OauthProvider::Twitter => "twitter",
            OauthProvider::LinkedIn => "linkedIn",
            OauthProvider::Github => "github",
            OauthProvider::Discord => "discord",
            OauthProvider::Luci => "luci",
            _ => "empty",
        }
    }
    pub fn google() -> Self {
        OauthProvider::Google
    }
    pub fn azure() -> Self {
        OauthProvider::Azure
    }
    pub fn facebook() -> Self {
        OauthProvider::Facebook
    }
    pub fn twitter() -> Self {
        OauthProvider::Twitter
    }
    pub fn linked_in() -> Self {
        OauthProvider::LinkedIn
    }
    pub fn github() -> Self {
        OauthProvider::Github
    }
    pub fn discord() -> Self {
        OauthProvider::Discord
    }
    pub fn luci() -> Self {
        OauthProvider::Luci
    }
}
