use std::path::Path;
use std::str::FromStr;

use serde::{de, Deserialize, Deserializer};

use revault_tx::{
    bitcoin::util::bip32::ExtendedPrivKey,
    scripts::{CpfpDescriptor, DepositDescriptor, EmergencyAddress, UnvaultDescriptor},
};

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_vecfromstr")]
    pub keys: Vec<ExtendedPrivKey>,
    pub descriptors: Option<Descriptors>,
    pub emergency_address: Option<EmergencyAddress>,
}

#[derive(Debug, Deserialize)]
pub struct Descriptors {
    #[serde(deserialize_with = "deserialize_fromstr")]
    pub deposit_descriptor: DepositDescriptor,
    #[serde(deserialize_with = "deserialize_fromstr")]
    pub unvault_descriptor: UnvaultDescriptor,
    #[serde(deserialize_with = "deserialize_fromstr")]
    pub cpfp_descriptor: CpfpDescriptor,
}

impl Config {
    pub fn new(keys: Vec<ExtendedPrivKey>) -> Self {
        Self {
            keys,
            descriptors: None,
            emergency_address: None,
        }
    }
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        std::fs::read(path)
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => ConfigError::NotFound,
                _ => ConfigError::ReadingFile(format!("Reading configuration file: {}", e)),
            })
            .and_then(|file_content| {
                toml::from_slice::<Config>(&file_content).map_err(|e| {
                    ConfigError::ReadingFile(format!("Parsing configuration file: {}", e))
                })
            })
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ConfigError {
    NotFound,
    ReadingFile(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Config file not found"),
            Self::ReadingFile(e) => write!(f, "Error while reading file: {}", e),
        }
    }
}

fn deserialize_fromstr<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    let string = String::deserialize(deserializer)?;
    T::from_str(&string)
        .map_err(|e| de::Error::custom(format!("Error parsing descriptor '{}': '{}'", string, e)))
}

fn deserialize_vecfromstr<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    let array: Vec<String> = Vec::deserialize(deserializer)?;
    let mut result: Vec<T> = Vec::new();
    for item in array {
        let res = T::from_str(&item)
            .map_err(|e| de::Error::custom(format!("Error parsing '{}': '{}'", item, e)))?;
        result.push(res);
    }
    Ok(result)
}
