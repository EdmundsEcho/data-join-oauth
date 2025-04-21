use serde::{Deserialize, Serialize};
use std::fmt;

use crate::models::oauth_provider::OauthProvider;
use crate::models::user::RawUser;

/// Interface for registering a new user
#[derive(Debug, Serialize, Deserialize)]
pub struct UserRegistration {
    pub auth_agent: OauthProvider,
    pub auth_id: String,
    pub email: Option<String>,
}

impl fmt::Display for UserRegistration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UserRegistration")
            .field("auth_agent", &self.auth_agent)
            .field("auth_id", &self.auth_id)
            .field("email", &self.email)
            .finish()
    }
}

impl std::convert::From<RawUser> for UserRegistration {
    fn from(user: RawUser) -> Self {
        UserRegistration {
            auth_agent: user.provider_id.provider,
            auth_id: user.provider_id.id,
            email: user.email,
        }
    }
}
