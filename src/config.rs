use async_trait::async_trait;
use rusoto_credential::{AwsCredentials, CredentialsError, ProvideAwsCredentials};
use serde::Deserialize;

/// Main configuration struct for the application.
#[derive(Deserialize, Debug)]
pub (crate) struct Config {
    /// AWS configuration details.
    pub (crate) aws: AwsConfig,

    /// Directories configuration details.
    pub (crate) directories: DirectoriesConfig,
}

/// AWS configuration details.
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct AwsConfig {
    /// AWS access key ID.
    pub (crate) aws_access_key_id: String,

    /// AWS default region.
    pub (crate) aws_default_region: String,

    /// AWS secret access key.
    pub (crate) aws_secret_access_key: String,
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
pub (crate) struct DirectoriesConfig {
    /// List of directories to be backed up.
    pub (crate) backups: Vec<SandmanDirectory>,
}

/// Details of a directory to be backed up.
#[derive(Deserialize, Debug)]
pub (crate) struct SandmanDirectory {
    /// Path to the directory.
    pub (crate) directory: String,

    /// S3 bucket prefix for the backup.
    pub (crate) prefix: String,

    /// S3 bucket name for the backup.
    pub (crate) bucket: String,
}
