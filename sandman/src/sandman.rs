use crate::args::{Args, GatherArgs};
use crate::backup::backup;
use crate::sha::{
    generate_shas, get_prior_shas, get_sha_diff, merge_diff_old, write_file_shas, ShaFile,
};
use clap::Parser;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use log::{error, info};
use sandman_share::config::{AwsConfig, Config};
use sandman_share::consts::{SANDMAN_CONFIG, SANDMAN_HISTORY, SANDMAN_IGNORE};
use sandman_share::paths::{file_in_config, verify_config_existence};
use std::ffi::OsStr;
use std::fs;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::task::JoinHandle;
use uuid::Uuid;

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
fn get_ignore(dir: &String) -> Gitignore {
    let path: &Path = Path::new(OsStr::new(&dir));
    let file: PathBuf = path.join(SANDMAN_IGNORE);
    let global_file: PathBuf = file_in_config(SANDMAN_IGNORE);

    let mut builder: GitignoreBuilder = GitignoreBuilder::new(path);
    builder.add(file);
    builder.add(global_file);
    builder.build().unwrap_or_else(|e| {
        error!("Error processing ignore file: {}", e);
        Gitignore::empty()
    })
}

/// Gathers and processes SHA files, and performs backup.
///
/// # Arguments
///
/// * `gather_args` - Arguments for gathering and backing up.
/// * `aws_config` - Optional AWS configuration.
/// * `oneshot` - bool flag indicating whether the function should return after one run
/// * `exit_flag` - Optional bool used as a signal for when the function should exit
async fn start_gathering(
    gather_args: GatherArgs,
    aws_config: Option<AwsConfig>,
    oneshot: bool,
    exit_flag: Option<Arc<Mutex<bool>>>,
) {
    let uuid: String = Uuid::new_v4().to_string();
    if oneshot {
        gather(&uuid, &gather_args, &aws_config).await;
    } else if exit_flag.is_some() {
        info!(
            "[Gatherer - {uuid}] Starting to watch for backup with {} - every {}s",
            gather_args.local_directory, gather_args.interval
        );
        let exit = exit_flag.unwrap();
        loop {
            let can_exit: bool = *exit.lock().unwrap();
            if can_exit {
                return;
            };
            gather(&uuid, &gather_args, &aws_config).await;
            async_std::task::sleep(Duration::from_secs(1)).await;
        }
    }
}

async fn gather(uuid: &String, gather_args: &GatherArgs, aws_config: &Option<AwsConfig>) {
    let directory: &PathBuf = &PathBuf::from(OsStr::new(&gather_args.local_directory));
    let sha_location: &PathBuf = &directory.join(SANDMAN_HISTORY);
    let old_file_shas: ShaFile = get_prior_shas(sha_location);
    let now: Duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!");
    let interval_duration: Duration = Duration::from_secs(gather_args.interval);
    let last_time: Duration = Duration::from_secs(old_file_shas.timestamp);
    let time_since: Duration = now - last_time;

    info!(
        "{:?} {:?} {:?} {}",
        now, last_time, time_since, gather_args.interval
    );
    if time_since >= Duration::from_secs(gather_args.interval) {
        info!(
            "[Gatherer - {uuid} ] Ready for backup...of {}",
            gather_args.local_directory
        );
        let ignore: Gitignore = get_ignore(&gather_args.local_directory);
        let mut current_file_shas: ShaFile = ShaFile::new();
        generate_shas(
            gather_args.local_directory.clone(),
            &mut current_file_shas,
            &ignore,
        );

        let sha_diff: ShaFile = get_sha_diff(&old_file_shas, current_file_shas);
        let merged_shas: ShaFile = merge_diff_old(old_file_shas, &sha_diff);
        write_file_shas(&merged_shas, sha_location);
        backup(sha_diff, gather_args, aws_config, uuid)
            .await
            .unwrap()
    } else {
        let wait_time: Duration = interval_duration - time_since;
        info!(
            "[Gatherer - {uuid}] Not ready for backup sleeping for {:?} seconds...",
            wait_time
        );
        async_std::task::sleep(wait_time).await;
    }
}

/// Opens and processes the `sandman_config.toml` file into a `Config` struct
fn get_config(path: String) -> Config {
    let path: PathBuf = PathBuf::from(path);
    let config_string: String = match path.is_file() {
        true => fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Error while processing .sandman_config.toml {}", e)),
        false => {
            let default_file: &PathBuf = &file_in_config(SANDMAN_CONFIG);
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
async fn with_external_config(args: &Args) {
    verify_config_existence();
    let exit: Option<Arc<Mutex<bool>>> = Some(Arc::new(Mutex::new(false)));
    let config: Config = get_config(args.config_path.clone());
    let mut gatherers: Vec<JoinHandle<()>> = vec![];

    for directory in config.directories.backups {
        let aws_config: AwsConfig = config.aws.clone();
        let gather_args: GatherArgs = GatherArgs::new(
            directory.directory,
            directory.bucket,
            directory.prefix,
            directory.interval,
            directory.start_time,
        );

        gatherers.push(tokio::task::spawn(start_gathering(
            gather_args,
            Some(aws_config),
            false,
            exit.clone(),
        )));
    }
    for gatherer in gatherers {
        gatherer.await.expect("TODO: panic message");
    }
}

/// Runs Sandman with provided CLI arguments in the form of an `Args` struct
///
/// # Arguments
///
/// * `args` - `Args` built from the CLI parameters
async fn with_cli_args(args: &Args) {
    let gather_args: GatherArgs = GatherArgs::new(
        args.local_directory.clone(),
        args.s3_bucket.clone(),
        args.bucket_prefix.clone(),
        0,
        0,
    );
    start_gathering(gather_args, None, true, None).await;
}

/// Main entry point for running the Sandman application.
pub(crate) async fn run_sandman() {
    let args = Args::parse();
    set_loggers(args.verbosity);
    if args.with_config {
        with_external_config(&args).await;
    } else {
        with_cli_args(&args).await;
    }
}
