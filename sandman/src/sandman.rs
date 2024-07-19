use crate::args::{Args, GatherArgs};
use crate::backup::backup;
use crate::sha::{
    generate_shas, get_prior_shas, get_sha_diff, merge_diff_old, write_file_shas, ShaFile,
};
use clap::Parser;
use env_logger;
use ignore::gitignore::Gitignore;
use ignore::Error;
use log::{error, info};
use sandman_share::config::{AwsConfig, Config};
use sandman_share::consts::{SANDMAN_CONFIG, SANDMAN_HISTORY, SANDMAN_IGNORE};
use sandman_share::paths::{config_dir, verify_config_existence};
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

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
fn get_config(path: String) -> Config {
    let path: PathBuf = PathBuf::from(path);
    let config_string: String = match path.is_file() {
        true => fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Error while processing .sandman_config.toml {}", e)),
        false => {
            let default_path: OsString = config_dir();
            let default_file: &PathBuf = &PathBuf::from(default_path).join(SANDMAN_CONFIG);
            info!(
                "Unable to find `{}` checking system default `{:?}",
                SANDMAN_CONFIG, default_file
            );
            fs::read_to_string(default_file)
                .unwrap_or_else(|e| panic!("Error while processing {:?}: {}", default_file, e))
        }
    };
    toml::from_str(&config_string)
        .unwrap_or_else(|e| panic!("Error while processing {}: {}", SANDMAN_CONFIG, e))
}

/// Use the `verbosity` parameter of the `args` struct to determine the level of our logger.
fn set_loggers(verbosity: bool) {
    if verbosity {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }
}

/// Runs Sandman with an external `.sandman_config.toml` file passed with `Args`. If the path does
/// not exist or was left blank it will check in the default system location. In the case of the
/// directory or file not existing it will be created and the application will exit.
///
/// # Arguments
///
/// * `args` - `Args` used to get the configuration path
async fn with_external_config(args: Args) {
    verify_config_existence();
    let config: Config = get_config(args.config_path);
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
}

/// Runs Sandman with provided CLI arguments in the form of an `Args` struct
///
/// # Arguments
///
/// * `args` - `Args` built from the CLI parameters
async fn with_cli_args(args: Args) {
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

/// Main entry point for running the Sandman application.
pub(crate) async fn run_sandman() {
    let args = Args::parse();
    set_loggers(args.verbosity);
    if args.with_config {
        with_external_config(args).await;
    } else {
        with_cli_args(args).await;
    }
}
