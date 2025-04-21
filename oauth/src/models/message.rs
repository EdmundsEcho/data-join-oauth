use serde::Serialize;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Default, Error, Serialize)]
pub struct Message(pub Option<String>);

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<&str> for Message {
    fn from(msg: &str) -> Message {
        if msg.is_empty() {
            Message(None)
        } else {
            Message(Some(msg.to_string()))
        }
    }
}
impl From<String> for Message {
    fn from(msg: String) -> Message {
        if msg.is_empty() {
            Message(None)
        } else {
            Message(Some(msg))
        }
    }
}
/// Convert frequently encountered errors to a message within AuthError
impl From<serde_json::Error> for Message {
    fn from(err: serde_json::Error) -> Message {
        Message(Some(err.to_string()))
    }
}
/// Convert frequently encountered errors to a message within AuthError
impl From<reqwest::Error> for Message {
    fn from(err: reqwest::Error) -> Message {
        Message(Some(err.to_string()))
    }
}

impl<T> From<&T> for Message
where
    T: std::fmt::Display,
{
    fn from(msg: &T) -> Message {
        Message(Some(msg.to_string()))
    }
}
