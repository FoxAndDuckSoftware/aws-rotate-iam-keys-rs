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
    region: String,
    access_key_id: String,
    secret_access_key: String,
}

pub enum ConfigType {
    Credentials,
    Config,
}

pub fn parse_config_files(
    conf_path: &Path,
    cred_path: &Path,
) -> Result<HashMap<String, AWSConfig>, RotateError> {
    let conf = Ini::load_from_file(conf_path).unwrap();
    let cred = Ini::load_from_file(cred_path).unwrap();
}

pub fn get_config_location(config_type: ConfigType) -> &Path {
    let env_var;
    let mut default_path = match home_dir() {
        Some(mut home) => {
            home.push(".aws");
            home
        }
        None => Err(RotateError("Cannot determine home directory")),
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
                default_path.as_path()
            } else {
                Some(Path::new(value.as_str()))
            }
        }
        Err(_) => default_path.as_path(),
    }
}
