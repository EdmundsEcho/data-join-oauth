use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use uuid::Uuid;

use crate::errors::AuthError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectId(Uuid);

impl fmt::Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        let ProjectId(uuid) = self;
        write!(f, "{}", uuid.to_string())
    }
}
impl TryFrom<&[u8]> for ProjectId {
    type Error = AuthError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        let project_id = ProjectId(
            Uuid::from_slice(slice)
                .map_err(|err| AuthError::ProjectIdError(err.to_string().into()))?,
        );
        Ok(project_id)
    }
}
impl TryFrom<&str> for ProjectId {
    type Error = AuthError;

    fn try_from(slice: &str) -> Result<Self, Self::Error> {
        let project_id = ProjectId(
            Uuid::parse_str(slice)
                .map_err(|err| AuthError::ProjectIdError(err.to_string().into()))?,
        );
        Ok(project_id)
    }
}
impl TryFrom<String> for ProjectId {
    type Error = AuthError;

    fn try_from(input: String) -> Result<Self, Self::Error> {
        ProjectId::try_from(input.as_str())
    }
}
