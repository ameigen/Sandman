use crate::backup::backup;
use crate::config::{AwsConfig, Config};
use crate::sha::{
    generate_shas, get_ignore, get_prior_shas, get_sha_diff, merge_diff_old, write_file_shas,
    ShaFile,
};
use clap::{arg, Parser};
use clap_derive::Parser;
use env_logger;

/// Command-line arguments for the Sandman application.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Local directory to process.
    #[arg(long, default_value_t = String::new())]
    local_directory: String,

    /// File path to store SHA file.
    #[arg(long, default_value_t = String::new())]
    sha_file: String,

    /// S3 bucket name for backup.
    #[arg(long, default_value_t = String::new())]
    s3_bucket: String,

    /// File path for ignore patterns.
    #[arg(long, default_value_t = String::new())]
    ignore_file: String,

    /// S3 bucket prefix for backup.
    #[arg(long, default_value_t = String::new())]
    bucket_prefix: String,

    /// Verbosity flag for logging.
    #[arg(short, long, default_value_t = false)]
    verbosity: bool,

    /// Flag to indicate if configuration file should be used.
    #[arg(long, default_value_t = false)]
    with_config: bool,
}

/// Gathers and processes SHA files, and performs backup.
///
/// # Arguments
///
/// * `local_directory` - The local directory to process.
/// * `ignore_file_path` - Path to the ignore file.
/// * `bucket` - S3 bucket name for backup.
/// * `bucket_prefix` - S3 bucket prefix for backup.
/// * `aws_config` - Optional AWS configuration.
async fn gather_sandman(
    local_directory: String,
    ignore_file_path: String,
    bucket: String,
    bucket_prefix: String,
    aws_config: Option<AwsConfig>,
) {
    let mut current_file_shas: ShaFile = ShaFile::new();
    let sha_location: String = format!("{}/.file_shas", local_directory);
    let ignored_names: Vec<String> = get_ignore(&ignore_file_path);
    generate_shas(local_directory, &mut current_file_shas, &ignored_names);

    let old_file_shas: ShaFile = get_prior_shas(&sha_location);
    let sha_diff: ShaFile = get_sha_diff(&old_file_shas, current_file_shas);
    let merged_shas: ShaFile = merge_diff_old(old_file_shas, &sha_diff);

    write_file_shas(&merged_shas, &sha_location);
    backup(sha_diff, bucket, bucket_prefix, aws_config)
        .await
        .unwrap()
}

/// Main entry point for running the Sandman application.
pub async fn run_sandman() {
    let args = Args::parse();
    if args.verbosity {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    if args.with_config {
        let config_string: String = tokio::fs::read_to_string("./sandman_config.toml")
            .await
            .unwrap();
        let config: Config = toml::from_str(&config_string).unwrap();

        for directory in config.directories.backups {
            let aws_config: AwsConfig = config.aws.clone();
            let ignore_file_path = format!("{}/.sandmanignore", directory.directory);
            gather_sandman(
                directory.directory,
                ignore_file_path,
                directory.bucket,
                directory.prefix,
                Some(aws_config),
            )
                .await;
        }
    } else {
        let ignore_file_path = if args.ignore_file.is_empty() {
            format!("{}/.sandmanignore", args.local_directory)
        } else {
            args.ignore_file.clone()
        };
        gather_sandman(
            args.local_directory,
            ignore_file_path,
            args.s3_bucket,
            args.bucket_prefix,
            None,
        )
            .await;
    }
}
