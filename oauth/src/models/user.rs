use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::string::ToString;
use uuid::Uuid;

use crate::models::oauth_provider::OauthProvider;

///
///{
///    "standard": {
///         "provider": "String google | twitter",
///         "id": "String",
///         "emails": [
///            {
///                "value": "email",
///                "type": "String work | home",
///                "validated": true
///            }
///         ],
///         "displayName": "String",
///         "name": {
///            "givenName": "String",
///            "familyName": "String",
///         },
///         "pictures": ["String"],
///         "locale": "en"
///    },
///}
///

///
/// Staging struct for User
///
#[derive(Debug, Deserialize, Serialize)]
pub struct RawUser {
    pub id: Uuid,
    pub username: Option<String>,
    pub email: Option<String>,
    pub provider_id: ProviderId,
}
impl Default for RawUser {
    fn default() -> RawUser {
        RawUser {
            id: Uuid::new_v4(),
            username: None,
            email: None,
            provider_id: ProviderId::default(),
        }
    }
}
impl RawUser {}

#[derive(Debug, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub provider_id: ProviderId,
    pub email: String,
    pub username: String,
    pub created_on: DateTime<Utc>,
}
impl From<RawUser> for User {
    // username is not optional
    fn from(input: RawUser) -> User {
        User {
            id: input.id,
            provider_id: input.provider_id,
            email: input.email.unwrap(),
            username: input.username.unwrap(),
            created_on: Utc::now(),
        }
    }
}

/* --------------------------------------------------------------------------------------------- */
/* RawFromGoogle serde */
/* --------------------------------------------------------------------------------------------- */
// Google -> RawUser
/* --------------------------------------------------------------------------------------------- */
#[derive(Debug, Deserialize)]
pub struct RawFromGoogle {
    pub id: String,
    #[serde(default = "OauthProvider::google")]
    pub provider: OauthProvider,
    pub email: String,
    pub verified_email: bool,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub locale: Option<String>,
}
impl From<RawFromGoogle> for RawUser {
    fn from(input: RawFromGoogle) -> Self {
        RawUser {
            id: Uuid::new_v4(),
            provider_id: ProviderId {
                id: input.id,
                provider: input.provider,
            },
            email: Some(input.email),
            username: None,
        }
    }
}
/* --------------------------------------------------------------------------------------------- */
// Azure
/* --------------------------------------------------------------------------------------------- */
#[derive(Debug, Deserialize)]
pub struct RawFromAzure {
    id: String,
    #[serde(default = "OauthProvider::azure")]
    provider: OauthProvider,
    mail: Option<String>,
    #[allow(dead_code)]
    user_principal_name: Option<String>,
    display_name: Option<String>,
    #[allow(dead_code)]
    given_name: Option<String>,
    #[allow(dead_code)]
    family_name: Option<String>,
}
impl From<RawFromAzure> for RawUser {
    fn from(input: RawFromAzure) -> Self {
        RawUser {
            id: Uuid::new_v4(),
            provider_id: ProviderId {
                id: input.id,
                provider: input.provider,
            },
            email: input.mail,
            username: input.display_name,
        }
    }
}
/* --------------------------------------------------------------------------------------------- */
// Twitter
/* --------------------------------------------------------------------------------------------- */
#[derive(Debug, Deserialize)]
pub struct RawFromTwitter {
    pub data: FromTwitter,
}
#[derive(Debug, Deserialize)]
pub struct FromTwitter {
    id: String,
    #[serde(default = "OauthProvider::twitter")]
    provider: OauthProvider,
    username: String,
    #[allow(dead_code)]
    name: Option<String>,
}
impl From<RawFromTwitter> for RawUser {
    fn from(input: RawFromTwitter) -> Self {
        RawUser {
            id: Uuid::new_v4(),
            provider_id: ProviderId {
                id: input.data.id,
                provider: input.data.provider,
            },
            email: None,
            username: Some(input.data.username),
        }
    }
}
/* --------------------------------------------------------------------------------------------- */
// LinkedIn
/* --------------------------------------------------------------------------------------------- */
#[derive(Debug, Deserialize)]
pub struct RawFromLinkedIn {
    id: String,
    #[serde(default = "OauthProvider::linked_in")]
    provider: OauthProvider,
    username: String,
    #[allow(dead_code)]
    name: Option<String>,
}
impl From<RawFromLinkedIn> for RawUser {
    fn from(input: RawFromLinkedIn) -> Self {
        RawUser {
            id: Uuid::new_v4(),
            provider_id: ProviderId {
                id: input.id,
                provider: input.provider,
            },
            email: None,
            username: Some(input.username),
        }
    }
}
/* --------------------------------------------------------------------------------------------- */
// Github
/* --------------------------------------------------------------------------------------------- */
#[derive(Debug, Deserialize)]
pub struct RawFromGithub {
    id: u32,
    #[serde(default = "OauthProvider::github")]
    provider: OauthProvider,
    email: String,
    login: String,
    #[allow(dead_code)]
    name: Option<String>,
    #[allow(dead_code)]
    location: Option<String>,
}
impl From<RawFromGithub> for RawUser {
    fn from(input: RawFromGithub) -> Self {
        RawUser {
            id: Uuid::new_v4(),
            provider_id: ProviderId {
                id: input.id.to_string(),
                provider: input.provider,
            },
            email: set_maybe_email(&input.email),
            username: Some(input.login),
        }
    }
}
/* --------------------------------------------------------------------------------------------- */
// Discord
/* --------------------------------------------------------------------------------------------- */
#[derive(Debug, Deserialize)]
pub struct RawFromDiscord {
    id: String,
    #[serde(default = "OauthProvider::discord")]
    provider: OauthProvider,
    email: String,
    username: String,
    #[allow(dead_code)]
    verified: bool,
    #[allow(dead_code)]
    locale: String,
    #[allow(dead_code)]
    name: Option<String>,
}
impl From<RawFromDiscord> for RawUser {
    fn from(input: RawFromDiscord) -> Self {
        RawUser {
            id: Uuid::new_v4(),
            provider_id: ProviderId {
                id: input.id,
                provider: input.provider,
            },
            email: set_maybe_email(&input.email),
            username: Some(input.username),
        }
    }
}
/* --------------------------------------------------------------------------------------------- */
// Luci
/* --------------------------------------------------------------------------------------------- */
#[derive(Debug, Deserialize)]
pub struct RawFromLuci {
    id: u32,
    #[serde(default = "OauthProvider::luci")]
    provider: OauthProvider,
    email: String,
    username: String,
    #[allow(dead_code)]
    name: Option<String>,
}
impl From<RawFromLuci> for RawUser {
    fn from(input: RawFromLuci) -> Self {
        RawUser {
            id: Uuid::new_v4(),
            provider_id: ProviderId {
                id: input.id.to_string(),
                provider: input.provider,
            },
            email: Some(input.email),
            username: Some(input.username),
        }
    }
}
/* --------------------------------------------------------------------------------------------- */
#[derive(Debug, Deserialize, Serialize)]
pub struct ProviderId {
    pub id: String,
    pub provider: OauthProvider,
}
impl Default for ProviderId {
    fn default() -> Self {
        ProviderId {
            id: Uuid::new_v4().to_string(),
            provider: OauthProvider::Luci,
        }
    }
}
/* --------------------------------------------------------------------------------------------- */
// Likely something in std::option
//
fn set_maybe_email(input: &String) -> Option<String> {
    let mut email = Some(input.to_owned());
    if input.is_empty() {
        email.take();
    }
    email
}
