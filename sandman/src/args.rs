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

    /// S3 bucket prefix for backup.
    #[arg(long, default_value_t = String::new())]
    pub(crate) bucket_prefix: String,

    /// Verbosity flag for logging.
    #[arg(short, long, default_value_t = false)]
    pub(crate) verbosity: bool,

    /// Flag to indicate if configuration file should be used.
    #[arg(long, default_value_t = false)]
    pub(crate) with_config: bool,

    #[arg(long, default_value_t = String::new())]
    pub(crate) config_path: String,
}

#[derive(Clone)]
pub(crate) struct GatherArgs {
    pub(crate) name: String,
    pub(crate) local_directory: String,
    pub(crate) bucket: String,
    pub(crate) bucket_prefix: String,
    pub(crate) interval: u64,
    pub(crate) start_time: u64,
    pub(crate) cleanable: bool,
}

impl GatherArgs {
    pub(crate) fn new(
        name: String,
        local_directory: String,
        bucket: String,
        bucket_prefix: String,
        interval: u64,
        start_time: u64,
        cleanable: bool,
    ) -> Self {
        GatherArgs {
            name,
            local_directory,
            bucket,
            bucket_prefix,
            interval,
            start_time,
            cleanable,
        }
    }
}
