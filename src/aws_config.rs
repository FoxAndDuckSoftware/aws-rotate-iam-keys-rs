use crate::RotateError;
use dirs::home_dir;
use ini::Ini;
use std::collections::HashMap;
use std::env::var;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub struct AWSConfig {
    pub(crate) region: String,
    pub(crate) access_key_id: String,
    pub(crate) secret_access_key: String,
}

impl AWSConfig {
    fn new(region: String, access_key_id: String, secret_access_key: String) -> AWSConfig {
        AWSConfig {
            region,
            access_key_id,
            secret_access_key,
        }
    }
}

pub enum ConfigType {
    Credentials,
    Config,
}

pub fn parse_config_files(
    conf_path: PathBuf,
    cred_path: PathBuf,
) -> Result<HashMap<String, AWSConfig>, RotateError> {
    let conf = Ini::load_from_file(conf_path).unwrap();
    let cred = Ini::load_from_file(cred_path).unwrap();
    let mut res: HashMap<String, AWSConfig> = HashMap::new();
    for (sec, prop) in conf.iter() {
        let profile_name: String = match sec {
            // profile is usually "profile <profile_name>", so split and take the 2nd item.
            Some(s) => s.split_whitespace().collect()[1].to_string(),
            None => continue,
        };
        let reg: String = match prop.get("region") {
            Some(r) => r.to_string(),
            None => Err(RotateError::new(format!(
                "No region for profile: {p}",
                p = profile_name
            ))),
        };
        let ak: String = match cred
            .section(Some(&profile_name))
            .unwrap()
            .get("aws_access_key_id")
        {
            Some(a) => a.to_string(),
            None => Err(RotateError::new(format!(
                "No access key for profile: {p}",
                p = profile_name
            ))),
        };
        let sk: String = match cred
            .section(Some(&profile_name))
            .unwrap()
            .get("aws_secret_access_key")
        {
            Some(s) => s.to_string(),
            None => Err(RotateError::new(format!(
                "No secret key for profile: {p}",
                p = profile_name
            ))),
        };
        res.insert(profile_name, AWSConfig::new(reg, ak, sk))
    }
    return Ok(res);
}

pub fn get_config_location(config_type: ConfigType) -> PathBuf {
    let env_var;
    let mut default_path = match home_dir() {
        Some(mut home) => {
            home.push(".aws");
            home
        }
        None => Err(RotateError::new("Cannot determine home directory")),
    };
    match config_type {
        ConfigType::Credentials => {
            env_var = "AWS_SHARED_CREDENTIALS_FILE";
            default_path.push("credentials")
        }
        ConfigType::Config => {
            env_var = "AWS_CONFIG_FILE";
            default_path.push("config")
        }
    }
    match var(env_var) {
        Ok(value) => {
            if value.is_empty() {
                default_path
            } else {
                PathBuf::from(value.as_str());
            }
        }
        Err(_) => default_path,
    }
}
