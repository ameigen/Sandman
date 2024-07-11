use clap_derive::Parser;

/// Command-line arguments for the Sandman application.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    /// Local directory to process.
    #[arg(long, default_value_t = String::new())]
    pub(crate) local_directory: String,

    /// File path to store SHA file.
    #[arg(long, default_value_t = String::new())]
    sha_file: String,

    /// S3 bucket name for backup.
    #[arg(long, default_value_t = String::new())]
    pub(crate) s3_bucket: String,

    /// File path for ignore patterns.
    #[arg(long, default_value_t = String::new())]
    pub(crate) ignore_file: String,

    /// S3 bucket prefix for backup.
    #[arg(long, default_value_t = String::new())]
    pub(crate) bucket_prefix: String,

    /// Verbosity flag for logging.
    #[arg(short, long, default_value_t = false)]
    pub(crate) verbosity: bool,

    /// Flag to indicate if configuration file should be used.
    #[arg(long, default_value_t = false)]
    pub(crate) with_config: bool,
}

pub(crate) struct GatherArgs {
    pub(crate) local_directory: String,
    pub(crate) ignore_file: String,
    pub(crate) bucket: String,
    pub(crate) bucket_prefix: String,
}

impl GatherArgs {
    pub(crate) fn new(local_directory: String, ignore_file: String, bucket: String, bucket_prefix: String) -> GatherArgs {
        GatherArgs{
            local_directory,
            ignore_file,
            bucket,
            bucket_prefix
        }
    }
}
