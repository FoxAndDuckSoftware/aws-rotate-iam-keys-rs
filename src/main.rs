//! `aws-rotate-iam-keys`.
//!
//! This application will allow you to rotate your AWS IAM access keys automatically.

#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::restriction)]

mod app;
mod aws_config;
mod rotate_error;

use crate::aws_config::{
    get_config_location, parse_config_files, write_credentials, AWSConfig, ConfigType,
};
use crate::rotate_error::RotateError;
use futures::future;
use log::{debug, info};
use rusoto_core::{HttpClient, Region};
use rusoto_credential::StaticProvider;
use rusoto_iam::{CreateAccessKeyRequest, DeleteAccessKeyRequest, Iam, IamClient};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::time::Duration;

async fn rotate(
    profile: String,
    profiles: HashMap<String, AWSConfig>,
) -> Result<(String, String, String), RotateError> {
    let old_profile = if let Some(p) = profiles.get(profile.as_str()) {
        p
    } else {
        return Err(RotateError::new(
            &format!("Profile: {} does not exist in credentials file", profile).as_str(),
        ));
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
    let new_resp = match client
        .create_access_key(CreateAccessKeyRequest::default())
        .await
    {
        Ok(r) => r,
        Err(e) => return Err(RotateError::new(&e)),
    };
    // The key is not active immediately so we wait for 10 seconds
    tokio::time::delay_for(Duration::new(10, 0)).await;
    client = IamClient::new_with(
        HttpClient::new().unwrap(),
        StaticProvider::new_minimal(
            String::from(&new_resp.access_key.access_key_id),
            String::from(&new_resp.access_key.secret_access_key),
        ),
        Region::UsEast1,
    );
    info!("Deleting old access key for profile: {}", profile);
    client
        .delete_access_key(DeleteAccessKeyRequest {
            access_key_id: String::from(&old_profile.access_key_id),
            ..DeleteAccessKeyRequest::default()
        })
        .await
        .unwrap();

    Ok((
        profile,
        new_resp.access_key.access_key_id,
        new_resp.access_key.secret_access_key,
    ))
}

#[tokio::main]
async fn main() -> Result<(), RotateError> {
    env_logger::init();
    let matches = app::app().get_matches();

    let dry_run: bool = matches.is_present("dry-run");
    let cred_location = PathBuf::from(
        matches
            .value_of("credfile")
            .unwrap_or(get_config_location(&ConfigType::Credentials)?.as_str()),
    );

    let conf_location = PathBuf::from(
        matches
            .value_of("credfile")
            .unwrap_or(get_config_location(&ConfigType::Config)?.as_str()),
    );

    let arg_profiles: Vec<String> = matches
        .values_of("profile")
        .unwrap()
        .map(ToString::to_string)
        .collect();

    let mut profiles = parse_config_files(&conf_location, &cred_location)?;
    debug!("{:#?}", profiles);
    let mut tasks = Vec::with_capacity(arg_profiles.len());
    if dry_run {
        println!("Would have rotated {:?}", arg_profiles);
        Ok(())
    } else {
        for profile in arg_profiles {
            tasks.push(tokio::spawn(rotate(profile, profiles.clone())))
        }
        for result in future::join_all(tasks).await {
            let (profile, ak, sk) = result.unwrap().unwrap();
            let conf = profiles.get_mut(&profile).unwrap();
            conf.access_key_id = ak;
            conf.secret_access_key = sk;
        }
        return match write_credentials(&profiles, &cred_location) {
            Ok(_) => Ok(()),
            Err(e) => Err(RotateError::new(
                &format!("Failed to write credentials: {}", e.message).as_str(),
            )),
        };
    }
}
