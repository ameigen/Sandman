use crate::args::{Args, GatherArgs};
use crate::backup::backup;
use crate::sha::{
    generate_shas, get_prior_shas, get_sha_diff, merge_diff_old, write_file_shas, ShaFile,
};
use clap::Parser;
use env_logger;
use ignore::gitignore::Gitignore;
use ignore::Error;
use log::error;
use sandman_share::config::{AwsConfig, Config};
use std::fs;

pub const SANDMAN_HISTORY: &str = ".sandman_history";
pub const SANDMAN_CONFIG: &str = ".sandman_config.toml";
pub const SANDMAN_IGNORE: &str = ".sandmanignore";

/// Creates a `Gitignore` file matcher from the provided path. If an error occurs it will
/// return a default match accepting any and all files
///
/// # Arguments
///
/// * `path` - String of the path to the `.sandmanignore` file
///
/// # Returns
///
/// `Gitignore` - glob matcher
fn get_ignore(path: String) -> Gitignore {
    let ignore_result: (Gitignore, Option<Error>) = Gitignore::new(path);
    let ignore_error: Option<Error> = ignore_result.1;
    match ignore_error {
        None => ignore_result.0,
        Some(e) => {
            error!("Error processing ignore file: {}", e);
            Gitignore::empty()
        }
    }
}

/// Gathers and processes SHA files, and performs backup.
///
/// # Arguments
///
/// * `gather_args` - Arguments for gathering and backing up.
/// * `aws_config` - Optional AWS configuration.
async fn gather(gather_args: GatherArgs, aws_config: Option<AwsConfig>) {
    let mut current_file_shas: ShaFile = ShaFile::new();
    let sha_location: String = format!("{}/{}", gather_args.local_directory, SANDMAN_HISTORY);
    let ignore: Gitignore = get_ignore(gather_args.ignore_file);
    generate_shas(
        gather_args.local_directory.clone(),
        &mut current_file_shas,
        &ignore,
    );

    let old_file_shas: ShaFile = get_prior_shas(&sha_location);
    let sha_diff: ShaFile = get_sha_diff(&old_file_shas, current_file_shas);
    let merged_shas: ShaFile = merge_diff_old(old_file_shas, &sha_diff);

    write_file_shas(&merged_shas, &sha_location);
    backup(
        sha_diff,
        gather_args.bucket,
        gather_args.bucket_prefix,
        aws_config,
    )
    .await
    .unwrap()
}

/// Opens and processes the `sandman_config.toml` file into a `Config` struct
fn get_config() -> Config {
    let config_string: String = fs::read_to_string(format!("./{}", SANDMAN_CONFIG))
        .unwrap_or_else(|_e| panic!("Error while processing .sandman_config.toml"));
    toml::from_str(&config_string).unwrap()
}

/// Use the `verbosity` parameter of the `args` struct to determine the level of our logger.
fn set_loggers(verbosity: bool) {
    if verbosity {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }
}

/// Main entry point for running the Sandman application.
pub(crate) async fn run_sandman() {
    let args = Args::parse();
    set_loggers(args.verbosity);
    if args.with_config {
        let config: Config = get_config();
        for directory in config.directories.backups {
            let aws_config: AwsConfig = config.aws.clone();
            let ignore_file_path = format!("{}/{}", directory.directory, SANDMAN_IGNORE);
            let gather_args: GatherArgs = GatherArgs::new(
                directory.directory,
                ignore_file_path,
                directory.bucket,
                directory.prefix,
            );
            gather(gather_args, Some(aws_config)).await;
        }
    } else {
        let ignore_file_path = if args.ignore_file.is_empty() {
            format!("{}/{}", args.local_directory, SANDMAN_IGNORE)
        } else {
            args.ignore_file.clone()
        };
        let gather_args: GatherArgs = GatherArgs::new(
            args.local_directory,
            ignore_file_path,
            args.s3_bucket,
            args.bucket_prefix,
        );
        gather(gather_args, None).await;
    }
}
