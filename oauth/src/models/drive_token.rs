///
/// Hosts the data and transformation flow for the DriveToken
/// Drive token provider -> Instance hosted in a Tnc store.
///
use oauth2::basic::{BasicTokenResponse, BasicTokenType};
use oauth2::{helpers, AccessToken, RefreshToken, Scope, TokenResponse, TokenUrl};
use serde::{Deserialize, Serialize, Serializer};

use std::fmt::Debug;
use std::time::Duration;

use crate::models::drive_provider::DriveProvider;
use crate::models::project_id::ProjectId;
///
/// Hosts the return value from the providers' token endpoint.
/// Unlike the RawUser model the return values from each provider
/// does not require a separate model.
///
/// The Builder is deserialized (instantiated from json).
///
/// The builder pattern: Use the builder to design a configuration.
/// Once completed, run .build() to produce the DriveToken.
///
/// 🔖 Avoid DriveToken::new();  logic is hosted in builder.
///
#[derive(Deserialize)]
pub struct Builder {
    access_token: AccessToken,
    #[serde(deserialize_with = "helpers::deserialize_untagged_enum_case_insensitive")]
    token_type: BasicTokenType,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_in: Option<Duration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<RefreshToken>,
    // #[serde(deserialize_with = "helpers::deserialize_space_delimited_vec")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    scopes: Option<Vec<Scope>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_uri: Option<TokenUrl>,
}

impl Builder {
    pub fn new(resp: &BasicTokenResponse, token_uri: Option<&TokenUrl>) -> Builder {
        Builder {
            access_token: resp.access_token().to_owned(),
            token_type: resp.token_type().to_owned(),
            expires_in: resp.expires_in().to_owned(),
            refresh_token: resp.refresh_token().cloned(),
            scopes: resp.scopes().cloned(),
            token_uri: token_uri.cloned(),
        }
    }
    pub fn build(self, project_id: &ProjectId, provider: &DriveProvider) -> DriveToken {
        DriveToken {
            project_id: project_id.clone(),
            drive_provider: provider.clone(),
            token_uri: self.token_uri,
            access_token: self.access_token,
            token_type: self.token_type,
            expires_in: self.expires_in,
            refresh_token: self.refresh_token,
            scopes: self.scopes,
        }
    }
}
///
/// Step 2. Host the augmented version.
/// The token instance has all the information required to
/// deliver a drive token on-demand.
///
/// The DriveToken is serialized; sent to the postgres db
/// and instantiated in Rust.
///
#[derive(Clone, Debug, Serialize)]
pub struct DriveToken {
    project_id: ProjectId,
    drive_provider: DriveProvider,
    access_token: AccessToken,
    // #[serde(deserialize_with = "helpers::deserialize_untagged_enum_case_insensitive")]
    token_type: BasicTokenType,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "duration_to_secs")]
    expires_in: Option<Duration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<RefreshToken>,
    // #[serde(deserialize_with = "helpers::deserialize_space_delimited_vec")]
    // #[serde(serialize_with = "helpers::serialize_space_delimited_vec")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // #[serde(default)]
    scopes: Option<Vec<Scope>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_uri: Option<TokenUrl>,
}
///
/// Serialize Duration -> u64 seconds
///
fn duration_to_secs<S>(maybe_duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let duration = maybe_duration.unwrap();
    serializer.serialize_u64(duration.as_secs())
}
// type Seconds = u64;
