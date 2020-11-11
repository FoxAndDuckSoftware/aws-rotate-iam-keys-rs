//! Represents an AWS IAM Key, Access Key ID and Secret Access Key

use crate::RotateError;
use ini::Ini;
use log::{debug, info};
use std::collections::HashMap;
use std::env::var;
use std::fmt;
use std::path::PathBuf;

#[cfg(test)]
#[path = "./aws_config_test.rs"]
mod aws_config_test;

/// A simple object containing the Access Key ID and Secret Key
pub struct AWSConfig {
    pub(crate) access_key_id: String,
    pub(crate) secret_access_key: String,
}

impl AWSConfig {
    /// Create a new `AWSConfig`
    ///
    /// * `access_key` — The Access Key ID for this `AWSConfig`.
    /// * `secret_key` — The Secret Key for this `AWSConfig`.
    fn new<S>(access_key: &S, secret_key: &S) -> Self
    where
        S: ToString,
    {
        Self {
            access_key_id: access_key.to_string(),
            secret_access_key: secret_key.to_string(),
        }
    }
}

impl Clone for AWSConfig {
    fn clone(&self) -> Self {
        Self {
            access_key_id: self.access_key_id.clone(),
            secret_access_key: self.secret_access_key.clone(),
        }
    }
}

impl fmt::Display for AWSConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AK: {}, SK: {}",
            &self.access_key_id, &self.secret_access_key
        )
    }
}

impl fmt::Debug for AWSConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AWSConfig")
            .field("access_key_id", &self.access_key_id)
            .field("secret_access_key", &self.secret_access_key)
            .finish()
    }
}

pub enum ConfigType {
    Credentials,
    Config,
}

pub fn parse_config_files(
    conf_path: &PathBuf,
    cred_path: &PathBuf,
) -> Result<HashMap<String, AWSConfig>, RotateError> {
    let conf = Ini::load_from_file(conf_path).unwrap();
    let cred = Ini::load_from_file(cred_path).unwrap();
    let mut res: HashMap<String, AWSConfig> = HashMap::new();
    for (sec, prop) in conf.iter() {
        if prop.contains_key("source_profile") {
            // profiles that have a source_profile are not useful.
            continue;
        }
        let profile_name: String = match sec {
            // profile in .aws/config is usually [profile <profile_name>], so split and take the 2nd item.
            Some(s) => {
                if !(s.starts_with("profile")) {
                    continue;
                }
                s.split_whitespace().collect::<Vec<&str>>()[1].to_string()
            }
            None => continue,
        };
        debug!("{}: {:#?}", profile_name, prop);
        let section = if let Some(s) = cred.section(Some(&profile_name)) {
            s
        } else {
            info!("Profile: {} has no credentials", &profile_name);
            continue;
        };
        let ak: &str = if let Some(a) = section.get("aws_access_key_id") {
            a
        } else {
            return Err(RotateError::new(&format!(
                "No access key for profile: {}",
                profile_name
            )));
        };
        let sk: &str = if let Some(s) = section.get("aws_secret_access_key") {
            s
        } else {
            return Err(RotateError::new(&format!(
                "No secret key for profile: {}",
                profile_name
            )));
        };
        res.insert(profile_name, AWSConfig::new(&ak, &sk));
    }
    Ok(res)
}

pub fn get_config_path(config_type: &ConfigType) -> Result<PathBuf, RotateError> {
    let mut home = match dirs_next::home_dir() {
        Some(mut p) => {
            p.push(".aws");
            p
        }
        None => return Err(RotateError::new(&"Failed to find home directory")),
    };
    let env_var = match config_type {
        ConfigType::Credentials => {
            home.push("credentials");
            "AWS_SHARED_CREDENTIALS_FILE"
        }
        ConfigType::Config => {
            home.push("config");
            "AWS_CONFIG_FILE"
        }
    };
    let final_path = match var(env_var) {
        Ok(value) => {
            if value.is_empty() {
                home
            } else {
                PathBuf::from(value.as_str())
            }
        }
        Err(_) => home,
    };
    Ok(final_path)
}

pub fn write_credentials(
    configs: &HashMap<String, AWSConfig>,
    cred_path: &PathBuf,
) -> Result<(), RotateError> {
    let mut cred = match Ini::load_from_file(cred_path) {
        Ok(i) => i,
        Err(e) => {
            return Err(RotateError::new(&format!(
                "Failed to load the credential file at: {}, reason {}",
                cred_path.to_str().unwrap(),
                e
            )));
        }
    };

    for (name, conf) in configs {
        cred.with_section(Some(name))
            .set("aws_access_key_id", &conf.access_key_id)
            .set("aws_secret_access_key", &conf.secret_access_key);
    }
    match cred.write_to_file(cred_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(RotateError::new(&format!(
            "Failed to write credentials: {}",
            e
        ))),
    }
}
