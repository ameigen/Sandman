use log::debug;
use log::error;
use crate::config::AwsConfig;
use crate::sha::ShaFile;
use chrono::prelude::*;
use rusoto_core::{HttpClient, Region};
use rusoto_s3::{PutObjectRequest, S3Client, StreamingBody, S3};
use std::error::Error;
use std::str::FromStr;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

/// Performs a backup of the files in the given SHA file difference to the specified S3 bucket.
///
/// # Arguments
///
/// * `diff` - A `ShaFile` representing the differences in files.
/// * `bucket` - The name of the S3 bucket to upload to.
/// * `bucket_directory` - The directory within the S3 bucket to upload to.
/// * `credentials` - Optional AWS credentials configuration.
///
/// # Returns
///
/// A `Result` which is `Ok` if the backup was successful, or an error if it failed.
pub(crate) async fn backup(
    diff: ShaFile,
    bucket: String,
    bucket_directory: String,
    credentials: Option<AwsConfig>,
) -> Result<(), Box<dyn Error>> {
    let now: DateTime<Utc> = Utc::now();
    let formatted_time = now.format("--%Y-%m-%d--%H-%M-%S").to_string();

    // Create the S3 client using provided credentials or default region
    let client = match credentials {
        None => S3Client::new(Region::UsEast1),
        Some(credentials) => {
            let region: Region = Region::from_str(&credentials.aws_default_region)
                .expect("Invalid region string");
            S3Client::new_with(HttpClient::new().unwrap(), credentials, region)
        }
    };

    // Iterate over the files in the SHA file difference and upload them to S3
    for (file_path, _) in diff.files {
        let bucket_location = format!("{}{}/{}", bucket_directory, formatted_time, file_path);

        let mut file = File::open(&file_path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        let upload_result = client
            .put_object(PutObjectRequest {
                bucket: bucket.clone(),
                key: bucket_location.clone(),
                body: Some(StreamingBody::from(buffer)),
                ..Default::default()
            })
            .await;

        match upload_result {
            Ok(_) => debug!("Successfully uploaded: {}", bucket_location),
            Err(e) => error!("Error uploading {}: {}", bucket_location, e),
        }
    }

    Ok(())
}
