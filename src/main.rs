//! `aws-rotate-iam-keys`.
//!
//! This application will allow you to rotate your AWS IAM access keys automatically.

#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::restriction)]

mod app;
mod aws_config;
mod rotate_error;

use crate::aws_config::{
    get_config_path, parse_config_files, write_credentials, AWSConfig, ConfigType,
};
use crate::rotate_error::RotateError;
use log::{debug, error, info};
use rusoto_core::{HttpClient, Region};
use rusoto_credential::StaticProvider;
use rusoto_iam::{
    CreateAccessKeyRequest, DeleteAccessKeyRequest, Iam, IamClient, ListAccessKeysRequest,
};
use std::path::PathBuf;
use tokio::task;
use tokio::time::{sleep, Duration};

async fn check_key(ak: String, sk: String) {
    let client = IamClient::new_with(
        HttpClient::new().unwrap(),
        StaticProvider::new_minimal(ak, sk),
        Region::UsEast1,
    );
    info!("Checking if new access key is active");
    // The key is not active immediately, so we wait for it to be activated.
    let mut active_key = false;
    while !active_key {
        if client
            .list_access_keys(ListAccessKeysRequest::default())
            .await
            .is_ok()
        {
            info!("Key is active!");
            active_key = true
        } else {
            sleep(Duration::from_secs(3)).await;
            info!("Retrying...");
            continue;
        }
    }
}

async fn rotate(
    profile: String,
    old_profile: AWSConfig,
) -> Result<(String, String, String), RotateError> {
    let mut client = IamClient::new_with(
        HttpClient::new().unwrap(),
        StaticProvider::new_minimal(
            String::from(&old_profile.access_key_id),
            String::from(&old_profile.secret_access_key),
        ),
        Region::UsEast1,
    );
    info!("Creating new access key for profile: {}", profile);
    let (new_access_key, new_secret_key) = match client
        .create_access_key(CreateAccessKeyRequest::default())
        .await
    {
        Ok(r) => (r.access_key.access_key_id, r.access_key.secret_access_key),
        Err(e) => {
            let mut err = RotateError::from(e);
            err.profile = Some(profile);
            return Err(err);
        }
    };
    check_key(new_access_key.clone(), new_secret_key.clone()).await;
    client = IamClient::new_with(
        HttpClient::new().unwrap(),
        StaticProvider::new_minimal(new_access_key.clone(), new_secret_key.clone()),
        Region::UsEast1,
    );
    info!("Deleting old access key for profile: {}", profile);
    match client
        .delete_access_key(DeleteAccessKeyRequest {
            access_key_id: String::from(&old_profile.access_key_id),
            ..DeleteAccessKeyRequest::default()
        })
        .await
    {
        Ok(_) => {}
        Err(e) => {
            let mut err = RotateError::from(e);
            err.profile = Some(profile);
            return Err(err);
        }
    }

    Ok((profile, new_access_key, new_secret_key))
}

#[tokio::main]
async fn main() -> Result<(), RotateError> {
    env_logger::init();
    let matches = app::app().get_matches();

    let dry_run: bool = matches.is_present("dry-run");
    let cred_location = match matches.value_of("credfile") {
        Some(p) => PathBuf::from(p),
        None => get_config_path(&ConfigType::Credentials)?,
    };

    let conf_location = match matches.value_of("configfile") {
        Some(p) => PathBuf::from(p),
        None => get_config_path(&ConfigType::Config)?,
    };

    let arg_profiles: Vec<String> = matches
        .values_of("profile")
        .expect("No profiles specified")
        .map(ToString::to_string)
        .collect();

    let mut conf_profiles = parse_config_files(&conf_location, &cred_location)?;
    debug!("{:#?}", conf_profiles);
    let mut tasks = Vec::with_capacity(arg_profiles.len());
    if dry_run {
        println!("Would have rotated {:?}", arg_profiles);
        Ok(())
    } else {
        for profile in arg_profiles {
            let old_profile = match conf_profiles.get(profile.as_str()) {
                Some(p) => p,
                None => {
                    return Err(RotateError::new(
                        &format!(
                            "Profile: {} does not exist in credentials file",
                            profile.as_str()
                        ),
                        Some(profile),
                    ));
                }
            };
            tasks.push(task::spawn(rotate(profile, old_profile.clone())));
        }
        for task in tasks {
            let (profile, ak, sk) = match task.await.expect("Failed to await task") {
                Ok(res) => res,
                Err(e) => {
                    error!("Failed to rotate due to: {}", e.message);
                    let err_profile = e.profile.expect("No profile attached to `RotateError`");
                    // remove failed profile to prevent attempts to rewrite
                    conf_profiles.remove(err_profile.as_str());
                    continue;
                }
            };
            let conf = conf_profiles.get_mut(&profile).unwrap();
            conf.access_key_id = ak;
            conf.secret_access_key = sk;
        }
        write_credentials(&conf_profiles, &cred_location)
    }
}
