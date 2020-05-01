use crate::RotateError;
use dirs::home_dir;
use ini::Ini;
use log::{debug, info};
use std::collections::HashMap;
use std::env::var;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AWSConfig {
    pub(crate) access_key_id: String,
    pub(crate) secret_access_key: String,
}

impl AWSConfig {
    fn new<S>(access_key: S, secret_key: S) -> AWSConfig
    where
        S: ToString,
    {
        AWSConfig {
            access_key_id: access_key.to_string(),
            secret_access_key: secret_key.to_string(),
        }
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
        let section = match cred.section(Some(&profile_name)) {
            Some(s) => s,
            None => {
                info!("Profile: {} has no credentials", &profile_name);
                continue;
            }
        };
        let ak: &str = match section.get("aws_access_key_id") {
            Some(a) => a,
            None => {
                return Err(RotateError::new(format!(
                    "No access key for profile: {}",
                    profile_name
                )));
            }
        };
        let sk: &str = match section.get("aws_secret_access_key") {
            Some(s) => s,
            None => {
                return Err(RotateError::new(format!(
                    "No secret key for profile: {}",
                    profile_name
                )));
            }
        };
        res.insert(profile_name, AWSConfig::new(ak, sk));
    }
    return Ok(res);
}

pub fn get_config_location(config_type: ConfigType) -> Result<String, RotateError> {
    let env_var;
    let mut default_path = match home_dir() {
        Some(mut home) => {
            home.push(".aws");
            home
        }
        None => return Err(RotateError::new("Cannot determine home directory")),
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
    let path = match var(env_var) {
        Ok(value) => {
            if value.is_empty() {
                default_path
            } else {
                PathBuf::from(value.as_str())
            }
        }
        Err(_) => default_path,
    };
    Ok(path.to_str().unwrap().to_string())
}

pub fn write_credentials(
    configs: HashMap<String, AWSConfig>,
    cred_path: &PathBuf,
) -> Result<(), RotateError> {
    let mut cred = Ini::load_from_file(cred_path).unwrap();

    for (name, conf) in configs {
        cred.with_section(Some(name))
            .set("aws_access_key_id", conf.access_key_id)
            .set("aws_secret_access_key", conf.secret_access_key);
    }
    Ok(cred.write_to_file(cred_path).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{remove_file, File};

    const TEST_PROFILE: &str = "test";
    const TEST_AK: &str = "ThisIsAnAccessKey";
    const TEST_SK: &str = "ThisIsASecretKey";
    const TEST_CRED_PATH: &str = "/tmp/test-credentials";

    #[test]
    fn aws_config() {
        let a = AWSConfig::new(TEST_AK, TEST_SK);
        assert_eq!(TEST_AK, a.access_key_id);
        assert_eq!(TEST_SK, a.secret_access_key);
    }

    #[test]
    fn write_test_credentials() {
        let mut config = HashMap::<String, AWSConfig>::new();
        config.insert(TEST_PROFILE.to_string(), AWSConfig::new(TEST_AK, TEST_SK));
        let temp_path = PathBuf::from(TEST_CRED_PATH);
        File::create(TEST_CRED_PATH).unwrap();
        write_credentials(config, &temp_path).unwrap();
        let ini_file = Ini::load_from_file(TEST_CRED_PATH).unwrap();
        assert!(!ini_file.is_empty());
        assert!(ini_file.section(Some(TEST_PROFILE)).is_some());
        let test_section = ini_file.section(Some(TEST_PROFILE)).unwrap();
        assert_eq!(test_section.get("aws_access_key_id").unwrap(), TEST_AK);
        assert_eq!(test_section.get("aws_secret_access_key").unwrap(), TEST_SK);
        //cleanup
        remove_file(TEST_CRED_PATH).unwrap()
    }
}
