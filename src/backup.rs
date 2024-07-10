use crate::sha::ShaFile;
use chrono::prelude::*;
use rusoto_core::Region;
use rusoto_s3::{PutObjectRequest, S3Client, StreamingBody, S3};
use tokio::io::AsyncReadExt;
use tokio::fs::File;
use std::error::Error;

pub async fn backup(diff: ShaFile, bucket: String, bucket_directory: String) -> Result<(), Box<dyn Error>> {
    let now: DateTime<Utc> = Utc::now();
    let formatted_time = now.format("--%Y-%m-%d--%H-%M-%S").to_string();
    let client = S3Client::new(Region::UsEast1);

    for (k, _v) in diff.files {
        let bucket_location = format!("{}{}/{}", bucket_directory, formatted_time, k);

        let mut file = File::open(&k).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        let upload_result = client.put_object(PutObjectRequest {
            bucket: bucket.clone(),
            key: bucket_location.clone(),
            body: Some(StreamingBody::from(buffer)),
            ..Default::default()
        }).await;

        match upload_result {
            Ok(_) => println!("Successfully uploaded: {}", bucket_location),
            Err(e) => println!("Error uploading {}: {}", bucket_location, e),
        }
    }

    Ok(())
}
