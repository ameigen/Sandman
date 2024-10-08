use crate::args::GatherArgs;
use crate::sha::ShaFile;
use chrono::prelude::*;
use log::debug;
use log::error;
use rusoto_core::{HttpClient, Region, RusotoError};
use rusoto_s3::PutObjectError;
use rusoto_s3::{PutObjectOutput, PutObjectRequest, S3Client, StreamingBody, S3};
use sandman_share::config::{AwsConfig, SandmanUploadedFile};
use std::error::Error;
use std::str::FromStr;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

/// Performs a backup of the files in the given SHA file difference to the specified S3 bucket.
///
/// # Arguments
///
/// * `diff` - A `ShaFile` representing the differences in files.
/// * `args` - GatherArgs carrying the target bucket location for backup.
/// * `credentials` - Optional AWS credentials configuration.
///
/// # Returns
///
/// A `Result` which is `Ok` if the backup was successful, or an error if it failed.
pub(crate) async fn backup(
    diff: ShaFile,
    args: &GatherArgs,
    credentials: &Option<AwsConfig>,
) -> Result<Vec<SandmanUploadedFile>, Box<dyn Error>> {
    let now: DateTime<Utc> = Utc::now();
    let formatted_time = now.format("%Y-%m-%d--%H-%M-%S").to_string();

    // Create the S3 client using provided credentials or default region
    let client: S3Client = match credentials {
        None => S3Client::new(Region::UsEast1),
        Some(credentials) => {
            let region: Region =
                Region::from_str(&credentials.aws_default_region).expect("Invalid region string");
            S3Client::new_with(HttpClient::new().unwrap(), credentials.clone(), region)
        }
    };

    // Iterate over the files in the SHA file difference and upload them to S3
    // If a file is successfully uploaded store it's remote name and local file path
    let mut uploaded_files: Vec<SandmanUploadedFile> = vec![];

    for (file_path, _) in diff.files {
        let bucket_location: String =
            format!("{}/{}/{}", args.bucket_prefix, formatted_time, file_path);

        let mut file: File = File::open(&file_path).await?;
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer).await?;

        let upload_result: Result<PutObjectOutput, RusotoError<PutObjectError>> = client
            .put_object(PutObjectRequest {
                bucket: args.bucket.clone(),
                key: bucket_location.clone(),
                body: Some(StreamingBody::from(buffer)),
                ..Default::default()
            })
            .await;

        match upload_result {
            Ok(_) => {
                debug!(
                    "[Gatherer - {}] Successfully uploaded: {}",
                    args.name, bucket_location
                );
                uploaded_files.push(SandmanUploadedFile::new(file_path, bucket_location))
            }
            Err(e) => error!(
                "[Gatherer - {}] Error uploading {}: {}",
                args.name, bucket_location, e
            ),
        }
    }

    Ok(uploaded_files)
}
