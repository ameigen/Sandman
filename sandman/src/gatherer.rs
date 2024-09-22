use crate::args::GatherArgs;
use crate::backup::backup;
use crate::sha::{
    generate_shas, get_prior_shas, get_sha_diff, merge_diff_old, write_file_shas, ShaFile,
};
use ignore::gitignore::Gitignore;
use log::{error, info};
use sandman_share::config::{AwsConfig, SandmanUploadedFile};
use sandman_share::consts::SANDMAN_HISTORY;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::task::JoinHandle;

pub(crate) struct Gatherer {
    args: GatherArgs,
    aws: Option<AwsConfig>,
    pub(crate) handle: Option<JoinHandle<()>>,
}

impl Gatherer {
    pub(crate) fn new(args: GatherArgs, aws: Option<AwsConfig>) -> Self {
        Gatherer {
            args,
            aws,
            handle: None,
        }
    }

    pub(crate) fn gather(&mut self, oneshot: bool, exit_flag: Option<Arc<Mutex<bool>>>) {
        let handle = tokio::task::spawn(start_gathering(
            self.args.clone(),
            self.aws.clone(),
            oneshot,
            exit_flag,
        ));
        self.handle = Option::from(handle)
    }
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
    if oneshot {
        return gather(&gather_args, &aws_config).await;
    }
    if exit_flag.is_some() {
        info!(
            "[Gatherer - {}] Starting to watch for backup with {} - every {}s",
            gather_args.name, gather_args.local_directory, gather_args.interval
        );
        let exit = exit_flag.unwrap();
        'gathering: loop {
            if *exit.lock().unwrap() {
                break 'gathering;
            };
            gather(&gather_args, &aws_config).await;
            async_std::task::sleep(Duration::from_secs(1)).await;
        }
    }
}


/// Asynchronously checks if the current time has reached or surpassed the configured start time for the gatherer.
/// If the current time is earlier than the start time, the function calculates the wait time and sleeps for that
/// duration before proceeding.
///
/// # Arguments
///
/// * `gather_args` - A reference to the `GatherArgs` struct that contains the gatherer's configuration,
/// including the start time in seconds since the UNIX epoch and the gatherer's name.
///
/// # Panics
///
/// This function will panic if the system time is earlier than the UNIX epoch, which should not normally occur.
async fn check_start_time(gather_args: &GatherArgs) {
    let current_time: Duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!");
    if current_time < Duration::from_secs(gather_args.start_time) {
        let wait_time: Duration = Duration::from_secs(gather_args.start_time) - current_time;
        info!(
            "[Gatherer - {}] Has not reached it's initial start time sleeping for {:?} seconds...",
            gather_args.name, wait_time
        );
        async_std::task::sleep(wait_time).await;
    }
}

/// Asynchronously waits until the configured backup interval has passed since the last backup time.
/// If the interval has not yet passed, the function calculates the remaining wait time and sleeps for that
/// duration before proceeding.
///
/// # Arguments
///
/// * `gather_args` - A reference to the `GatherArgs` struct that contains the gatherer's configuration,
/// including the interval in seconds and the gatherer's name.
/// * `last_time` - A `Duration` representing the last time a backup was made.
///
/// # Panics
///
/// This function will panic if the system time is earlier than the UNIX epoch, which should not normally occur.
async fn await_backup(gather_args: &GatherArgs, last_time: Duration) {
    let current_time: Duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!");
    let time_since: Duration = current_time - last_time;
    let interval_duration: Duration = Duration::from_secs(gather_args.interval);

    if time_since < Duration::from_secs(gather_args.interval) {
        let wait_time: Duration = interval_duration - time_since;
        info!(
            "[Gatherer - {}] Not ready for backup sleeping for {:?} seconds...",
            gather_args.name, wait_time
        );
        async_std::task::sleep(wait_time).await;
    }
}

/// Deletes uploaded files, only called if the files belong to a directory that has been flagged
/// for deletion inside the configuration toml
///
/// # Arguments
///
/// * `uploaded_files` - Array/Vector of `SandmanUploadedFile`s which contains both the local and
/// remote paths
async fn cleanup_deletable(uploaded_files: &[SandmanUploadedFile]) {
    for file in uploaded_files.iter() {
        match tokio::fs::remove_file(&file.path).await {
            Ok(_) => {
                info!("Removed local file: {}", &file.path)
            }
            Err(e) => {
                error!("Unable to remove file: {} {}", &file.path, e)
            }
        }
    }
}

/// Processes the previously generated sha file and calculates the delta to see if enough time has
/// elapsed to perform a new check of the designated directory. If that time hasn't been met, the
/// task will sleep.
///
/// # Arguments
///
/// * `gather_args` - `GatherArgs` detailing the directory to be backed up, interval to
/// check at, whether it should be cleaned, the target bucket, and the prefix for naming.
/// * `aws_config` - `Option<AwsConfig` optional arguments for the target Aws interface, if not
/// provided it will be assumed this is set in `.sandman_config.toml`
async fn gather(gather_args: &GatherArgs, aws_config: &Option<AwsConfig>) {
    let directory: &PathBuf = &PathBuf::from(OsStr::new(&gather_args.local_directory));
    let sha_location: &PathBuf = &directory.join(SANDMAN_HISTORY);
    let old_file_shas: ShaFile = get_prior_shas(sha_location);
    let last_time: Duration = Duration::from_secs(old_file_shas.timestamp);
    let ignore: Gitignore = crate::sandman::get_ignore(&gather_args.local_directory);
    let mut current_file_shas: ShaFile = ShaFile::new();

    check_start_time(gather_args).await;
    await_backup(gather_args, last_time).await;

    info!(
        "[Gatherer - {} ] Ready for backup...of {}",
        gather_args.name, gather_args.local_directory
    );

    generate_shas(
        gather_args.local_directory.clone(),
        &mut current_file_shas,
        &ignore,
    );

    let sha_diff: ShaFile = get_sha_diff(&old_file_shas, current_file_shas);
    let merged_shas: ShaFile = merge_diff_old(old_file_shas, &sha_diff);
    write_file_shas(&merged_shas, sha_location);

    let uploaded_files: Vec<SandmanUploadedFile> =
        backup(sha_diff, gather_args, aws_config).await.unwrap();

    if gather_args.cleanable {
        cleanup_deletable(&uploaded_files).await;
    }
}
