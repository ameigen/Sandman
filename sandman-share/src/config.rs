use async_trait::async_trait;
use rusoto_credential::{AwsCredentials, CredentialsError, ProvideAwsCredentials};
use serde::Deserialize;

/// Main configuration struct for the application.
#[derive(Deserialize)]
pub struct Config {
    /// AWS configuration details.
    pub aws: AwsConfig,

    /// Directories configuration details.
    pub directories: DirectoriesConfig,
}

/// AWS configuration details.
#[derive(Deserialize, Clone)]
pub struct AwsConfig {
    /// AWS access key ID.
    pub aws_access_key_id: String,

    /// AWS default region.
    pub aws_default_region: String,

    /// AWS secret access key.
    pub aws_secret_access_key: String,
}

/// Implementation of the `ProvideAwsCredentials` trait for `AwsConfig`.
#[async_trait]
impl ProvideAwsCredentials for AwsConfig {
    async fn credentials(&self) -> Result<AwsCredentials, CredentialsError> {
        Ok(AwsCredentials::new(
            self.aws_access_key_id.clone(),
            self.aws_secret_access_key.clone(),
            None,
            None,
        ))
    }
}

/// Configuration for directories to be backed up.
#[derive(Deserialize, Debug)]
pub struct DirectoriesConfig {
    /// List of directories to be backed up.
    pub backups: Vec<SandmanDirectory>,
}

/// Details of a directory to be backed up.
#[derive(Deserialize, Debug)]
pub struct SandmanDirectory {
    /// Designator for this backup
    pub name: String,

    /// Path to the directory.
    pub directory: String,

    /// Interval for backups
    pub interval: u64,

    /// Start time (unix timestamp)
    pub start_time: u64,

    /// S3 bucket prefix for the backup.
    pub prefix: String,

    /// S3 bucket name for the backup.
    pub bucket: String,

    /// Whether files in this directory should be deleted on a successful upload
    pub cleanable: bool,
}

pub struct SandmanUploadedFile {
    /// Local file path
    pub path: String,

    /// Remote name
    pub remote_name: String,
}

impl SandmanUploadedFile {
    pub fn new(path: String, remote_name: String) -> Self {
        SandmanUploadedFile { path, remote_name }
    }
}
