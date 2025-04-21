use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AuthReturnValues {
    pub code: String,
    pub state: String,
}
