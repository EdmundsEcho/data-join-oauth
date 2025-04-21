// Copyright (c) 2021 bc1pgiorgio
// Distributed under the MIT software license

use std::fmt::Display;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serializer};

pub fn deserialize_number_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + serde::Deserialize<'de>,
    <T as FromStr>::Err: Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt<T> {
        String(String),
        Number(T),
    }

    match StringOrInt::<T>::deserialize(deserializer)? {
        StringOrInt::String(s) => s.parse::<T>().map_err(serde::de::Error::custom),
        StringOrInt::Number(i) => Ok(i),
    }
}

// Url to String and back
fn url_to_str<S>(url: &Option<Url>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value = if let Some(url) = url {
        url.as_str()
    } else {
        ""
    };
    serializer.serialize_str(value)
}
fn str_to_url<'de, D>(deserializer: D) -> Result<Option<Url>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Url::from_str(&s).ok())
}

/*
 pub homepage: Option<Url>,
    #[serde(serialize_with = "url_to_str")]
    #[serde(deserialize_with = "str_to_url")]
*/
/* Usage
#[derive(Deserialize, Debug)]
pub struct PoolStats {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub luck_b10: f32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub luck_b50: f32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub luck_b250: f32,
    pub hash_rate_unit: String,
    pub pool_scoring_hash_rate: f64,
    pub pool_active_workers: u32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub round_probability: f32,
    pub round_started: u32,
    pub round_duration: u32,
}
*/
