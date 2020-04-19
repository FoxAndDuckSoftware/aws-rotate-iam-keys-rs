extern crate clap;

mod aws_config;

use crate::aws_config::{get_config_location, parse_config_files, ConfigType};
use clap::{App, Arg};
use dirs::home_dir;
use rusoto_core::{HttpClient, Region};
use rusoto_credential::{CredentialsError, EnvironmentProvider};
use rusoto_iam::{CreateAccessKeyRequest, Iam, IamClient};
use std::env;
use std::path::{Path, PathBuf};

/// Represents an Error that has occurred with rotate-iam-keys.
///
/// This is an error message from the application, not underlying libraries.
#[derive(Clone, Debug, PartialEq)]
pub struct RotateError {
    /// The underlying error message for rotate error.
    pub message: String,
}

impl RotateError {
    /// Create a new Rotate Error.
    ///
    /// * `message` — The Error message for this RotateError.
    pub fn new<S>(message: S) -> RotateError
    where
        S: ToString,
    {
        RotateError {
            message: message.to_string(),
        }
    }
}

fn main() {
    let matches = App::new("aws-rotate-iam-keys")
        .version("1.0.0")
        .author("Martin Kemp <me@martinke.mp>")
        .about("Rotates your IAM Access Keys")
        .arg(
            Arg::with_name("profile")
                .short("p")
                .long("profile")
                .takes_value(true)
                .help("profile to rotate")
                .long_help("profile to rotate, you can specify multiple profiles for example, --profile dev --profile prod to rotate all of those specified")
                .number_of_values(1)
                .multiple(true)
        )
        .arg(
            Arg::with_name("credfile")
                .short("c")
                .long("credfile")
                .takes_value(true)
                .help("location of your aws credential file")
                .number_of_values(1)
                .multiple(false)
        )
        .arg(
            Arg::with_name("configfile")
                .short("f")
                .long("configfile")
                .takes_value(true)
                .help("location of your aws config file")
                .number_of_values(1)
                .multiple(false)
        )
        .arg(
            Arg::with_name("disable")
                .short("d")
                .long("disable")
                .takes_value(false)
                .help("disable the access key instead of deleting it")
                .multiple(false)
        )
        .get_matches();

    let cred_location: &Path = match matches.is_present("credfile") {
        true => Path::new(matches.value_of("credfile").unwrap()),
        false => get_config_location(ConfigType::Credentials),
    };

    let conf_location: &Path = match matches.is_present("configfile") {
        true => Path::new(matches.value_of("configfile").unwrap()),
        false => get_config_location(ConfigType::Config),
    };

    let profiles = parse_config_files(conf_location, cred_location).unwrap();
    for profile in matches.value_of("profile").unwrap().collect() {
        if !profiles.contains_key(profile) {
            Err("Profile does not exist in credentials file")
        }
        env::set_var(
            "AWS_ACCESS_KEY_ID",
            profiles.get(&profile).aws_access_key_id(),
        );
        env::set_var(
            "AWS_SECRET_ACCESS_KEY",
            profiles.get(&profile).aws_secret_access_key(),
        );
        let client = IamClient::new_with(
            HttpClient::new().unwrap(),
            EnvironmentProvider::new(),
            Region::EuWest1,
        );
        let resp = client.create_access_key().await.unwrap();
    }
}
