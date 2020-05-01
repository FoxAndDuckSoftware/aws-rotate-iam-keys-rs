mod aws_config;
mod rotate_error;

use crate::aws_config::{
    get_config_location, parse_config_files, write_credentials, AWSConfig, ConfigType,
};
use crate::rotate_error::RotateError;
use clap::{App, AppSettings, Arg};
use futures::future;
use log::info;
use rusoto_core::{HttpClient, Region};
use rusoto_credential::StaticProvider;
use rusoto_iam::{CreateAccessKeyResponse, DeleteAccessKeyRequest, Iam, IamClient};
use std::collections::HashMap;
use std::path::PathBuf;

async fn rotate(
    profile: String,
    profiles: HashMap<String, AWSConfig>,
    dry_run: bool,
) -> Result<(String, String, String), RotateError> {
    let old_profile = match profiles.get(profile.as_str()) {
        Some(p) => p,
        None => {
            return Err(RotateError::new(format!(
                "Profile: {} does not exist in credentials file",
                profile
            )));
        }
    };
    let mut client = IamClient::new_with(
        HttpClient::new().unwrap(),
        StaticProvider::new_minimal(
            String::from(&old_profile.access_key_id),
            String::from(&old_profile.secret_access_key),
        ),
        Region::UsEast1,
    );
    info!("Creating new access key for profile: {}", profile);
    let mut new_resp = CreateAccessKeyResponse::default();
    if !dry_run {
        new_resp = match client.create_access_key(Default::default()).await {
            Ok(r) => r,
            Err(e) => return Err(RotateError::new(e)),
        };
        client = IamClient::new_with(
            HttpClient::new().unwrap(),
            StaticProvider::new_minimal(
                String::from(&new_resp.access_key.access_key_id),
                String::from(&new_resp.access_key.secret_access_key),
            ),
            Region::UsEast1,
        );
    }
    info!("Deleting old access key for profile: {}", profile);
    if !dry_run {
        client
            .delete_access_key(DeleteAccessKeyRequest {
                access_key_id: String::from(&old_profile.access_key_id),
                ..Default::default()
            })
            .await
            .unwrap();
    }

    return Ok((
        profile,
        new_resp.access_key.access_key_id,
        new_resp.access_key.secret_access_key,
    ));
}

#[tokio::main]
async fn main() -> Result<(), RotateError> {
    env_logger::init();
    let matches = App::new("aws-rotate-iam-keys")
        .setting(AppSettings::ArgRequiredElseHelp)
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
                .required(true)
        )
        .arg(
            Arg::with_name("credfile")
                .long("credfile")
                .takes_value(true)
                .help("location of your aws credential file")
                .number_of_values(1)
                .multiple(false)
        )
        .arg(
            Arg::with_name("configfile")
                .long("configfile")
                .takes_value(true)
                .help("location of your aws config file")
                .number_of_values(1)
                .multiple(false)
        )
        .arg(
            Arg::with_name("disable")
                .short("D")
                .long("disable")
                .takes_value(false)
                .help("disable the access key instead of deleting it")
                .multiple(false)
        )
        .arg(
            Arg::with_name("dry_run")
                .short("d")
                .long("dry_run")
                .help("runs without affecting anything, useful to check before commiting")
                .multiple(false)
        )
        .get_matches();

    let dry_run: bool = *&matches.is_present("dry_run");
    let cred_location = PathBuf::from(
        matches
            .value_of("credfile")
            .unwrap_or(get_config_location(ConfigType::Credentials)?.as_str()),
    );

    let conf_location = PathBuf::from(
        matches
            .value_of("credfile")
            .unwrap_or(get_config_location(ConfigType::Config)?.as_str()),
    );

    let arg_profiles: Vec<String> = matches
        .values_of("profile")
        .unwrap()
        .map(|s| s.to_string())
        .collect();

    let mut profiles = parse_config_files(&conf_location, &cred_location)?;

    let mut tasks = Vec::with_capacity(arg_profiles.len());
    for profile in arg_profiles {
        tasks.push(tokio::spawn(rotate(profile, profiles.clone(), dry_run)))
    }
    for result in future::join_all(tasks).await {
        let (profile, ak, sk) = result.unwrap().unwrap();
        let conf = profiles.get_mut(&profile).unwrap();
        conf.access_key_id = ak;
        conf.secret_access_key = sk;
    }

    if !dry_run {
        return match write_credentials(profiles, &cred_location) {
            Ok(_) => Ok(()),
            Err(e) => Err(RotateError::new(format!(
                "Failed to write credentials: {}",
                e.message
            ))),
        };
    } else {
        Ok(())
    }
}
