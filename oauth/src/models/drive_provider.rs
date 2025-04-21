use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum_macros::AsRefStr;
use strum_macros::Display;

use crate::errors::AuthError;

///
/// Supported sources of data files
/// Align this with the default.toml configuration
///
#[derive(Clone, Debug, Display, AsRefStr, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(try_from = "String")]
pub enum DriveProvider {
    #[serde(rename(serialize = "google"))]
    Google,
    #[serde(rename(serialize = "msgraph"))]
    MSGraph,
    #[serde(rename(serialize = "dropbox"))]
    DropBox,
    #[serde(rename(serialize = "user"))]
    User, //< User's local drive
    #[serde(rename(serialize = "luci"))]
    Luci, //< Hosted by Luci (provided by user)
    #[serde(rename(serialize = "empty"))]
    Empty,
}
impl Default for DriveProvider {
    fn default() -> DriveProvider {
        DriveProvider::Empty
    }
}
///
/// Align this with the default.toml configuration
///
impl FromStr for DriveProvider {
    type Err = AuthError;
    fn from_str(input: &str) -> Result<DriveProvider, Self::Err> {
        match input {
            "google" => Ok(DriveProvider::Google),
            "msgraph" => Ok(DriveProvider::MSGraph),
            "dropbox" => Ok(DriveProvider::DropBox),
            "user" => Ok(DriveProvider::User),
            "luci" => Ok(DriveProvider::Luci),
            "empty" => Ok(DriveProvider::Empty),
            v => Err(AuthError::UnsupportedProvider(v.into())),
        }
    }
}
impl From<String> for DriveProvider {
    fn from(input: String) -> DriveProvider {
        let input = input.as_str();
        match input {
            "google" => DriveProvider::Google,
            "msgraph" => DriveProvider::MSGraph,
            "dropbox" => DriveProvider::DropBox,
            "user" => DriveProvider::User,
            "luci" => DriveProvider::Luci,
            _ => DriveProvider::Empty,
        }
    }
}
impl From<&str> for DriveProvider {
    fn from(input: &str) -> DriveProvider {
        match input {
            "google" => DriveProvider::Google,
            "msgraph" => DriveProvider::MSGraph,
            "dropbox" => DriveProvider::DropBox,
            "user" => DriveProvider::User,
            "luci" => DriveProvider::Luci,
            _ => DriveProvider::Empty,
        }
    }
}
///
/// Utilized by serde for default values of the Raw, staging versions
/// specified for each provider.
///
impl DriveProvider {
    pub fn to_path(&self) -> &str {
        match self {
            DriveProvider::Google => "google",
            DriveProvider::MSGraph => "msgraph",
            DriveProvider::DropBox => "dropbox",
            _ => "empty",
        }
    }
    pub fn google() -> Self {
        DriveProvider::Google
    }
    pub fn msgraph() -> Self {
        DriveProvider::MSGraph
    }
    pub fn dropbox() -> Self {
        DriveProvider::DropBox
    }
    pub fn user() -> Self {
        DriveProvider::User
    }
    pub fn luci() -> Self {
        DriveProvider::Luci
    }
}
