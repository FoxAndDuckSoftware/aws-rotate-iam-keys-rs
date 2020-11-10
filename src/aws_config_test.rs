use super::{write_credentials, AWSConfig};
use crate::aws_config::{get_config_path, ConfigType};
use ini::Ini;
use std::collections::HashMap;
use std::env;
use std::env::var;
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::PathBuf;

const TEST_PROFILE: &str = "test";
const TEST_AK: &str = "ThisIsAnAccessKey";
const TEST_SK: &str = "ThisIsASecretKey";
const TEST_PROFILE2: &str = "test2";
const TEST_AK2: &str = "ThisIsAnAccessKey2";
const TEST_SK2: &str = "ThisIsASecretKey2";

#[test]
fn aws_config() {
    let a = AWSConfig::new(&TEST_AK, &TEST_SK);
    assert_eq!(TEST_AK, a.access_key_id);
    assert_eq!(TEST_SK, a.secret_access_key);
}

#[test]
fn write_test_credentials() {
    let mut config = HashMap::<String, AWSConfig>::new();
    config.insert(TEST_PROFILE.to_string(), AWSConfig::new(&TEST_AK, &TEST_SK));

    let mut temp_path = env::current_dir().unwrap();
    temp_path.push("write_test_creds");

    File::create(&temp_path).unwrap();
    write_credentials(&config, &temp_path).unwrap();

    let ini_file = Ini::load_from_file(&temp_path).unwrap();
    assert!(!ini_file.is_empty());
    assert!(ini_file.section(Some(TEST_PROFILE)).is_some());

    let test_section = ini_file.section(Some(TEST_PROFILE)).unwrap();
    assert_eq!(test_section.get("aws_access_key_id").unwrap(), TEST_AK);
    assert_eq!(test_section.get("aws_secret_access_key").unwrap(), TEST_SK);

    //cleanup
    remove_file(&temp_path.as_path()).unwrap()
}

#[test]
fn test_nondestructive_write() {
    let mut config = HashMap::<String, AWSConfig>::new();
    config.insert(TEST_PROFILE.to_string(), AWSConfig::new(&TEST_AK, &TEST_SK));

    let mut temp_path = env::current_dir().unwrap();
    temp_path.push("nondestructive-test");

    File::create(&temp_path).unwrap();
    write_credentials(&config, &temp_path).unwrap();
    config.clear();
    config.insert(
        TEST_PROFILE2.to_string(),
        AWSConfig::new(&TEST_AK2, &TEST_SK2),
    );
    write_credentials(&config, &temp_path).unwrap();

    let ini_file = Ini::load_from_file(&temp_path).unwrap();
    assert!(!ini_file.is_empty());
    assert!(ini_file.section(Some(TEST_PROFILE)).is_some());
    assert!(ini_file.section(Some(TEST_PROFILE2)).is_some());

    let mut test_section = ini_file.section(Some(TEST_PROFILE)).unwrap();
    assert_eq!(test_section.get("aws_access_key_id").unwrap(), TEST_AK);
    assert_eq!(test_section.get("aws_secret_access_key").unwrap(), TEST_SK);

    test_section = ini_file.section(Some(TEST_PROFILE2)).unwrap();
    assert_eq!(test_section.get("aws_access_key_id").unwrap(), TEST_AK2);
    assert_eq!(test_section.get("aws_secret_access_key").unwrap(), TEST_SK2);

    //cleanup
    remove_file(&temp_path.as_path()).unwrap()
}

#[test]
fn test_failed_write() {
    let mut config = HashMap::<String, AWSConfig>::new();
    config.insert(TEST_PROFILE.to_string(), AWSConfig::new(&TEST_AK, &TEST_SK));

    let mut temp_path = env::current_dir().unwrap();
    temp_path.push("failed-write");

    let mut file = File::create(&temp_path).unwrap();
    file.write_all("asdfda;lkjhgs;oijfdjlkjkldsajlkd".as_ref())
        .unwrap();
    assert!(
        write_credentials(&config, &temp_path).is_err(),
        "cred file should be invalid"
    );
    //cleanup
    remove_file(&temp_path.as_path()).unwrap()
}

#[test]
fn test_get_config_path() {
    let mut target = match env::consts::OS {
        "windows" => PathBuf::from(format!(
            r#"C:\Users\{}\.aws"#,
            var("USERNAME").expect("USERNAME env var not defined")
        )),
        "linux" => PathBuf::from(format!(
            "/home/{}/.aws",
            var("USER").expect("USER env var not defined")
        )),
        "macos" => PathBuf::from(format!(
            "/Users/{}/.aws",
            var("USER").expect("USER env var not defined")
        )),
        _ => panic!("Unsupported operating system"),
    };
    let config_path = get_config_path(&ConfigType::Config).unwrap();
    target.push("config");
    assert_eq!(config_path, target);
    target.pop();
    let credential_path = get_config_path(&ConfigType::Credentials).unwrap();
    target.push("credentials");
    assert_eq!(credential_path, target);
}
