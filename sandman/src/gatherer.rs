use crate::args::GatherArgs;
use crate::backup::backup;
use crate::sha::{
    generate_shas, get_prior_shas, get_sha_diff, merge_diff_old, write_file_shas, ShaFile,
};
use ignore::gitignore::Gitignore;
use log::info;
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

async fn cleanup_deletables(gather_args: &GatherArgs, uploaded_files: &[SandmanUploadedFile]) {
}

async fn gather(gather_args: &GatherArgs, aws_config: &Option<AwsConfig>) {
    let directory: &PathBuf = &PathBuf::from(OsStr::new(&gather_args.local_directory));
    let sha_location: &PathBuf = &directory.join(SANDMAN_HISTORY);
    let old_file_shas: ShaFile = get_prior_shas(sha_location);
    let now: Duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!");
    let interval_duration: Duration = Duration::from_secs(gather_args.interval);
    let last_time: Duration = Duration::from_secs(old_file_shas.timestamp);
    let time_since: Duration = now - last_time;
    let ignore: Gitignore = crate::sandman::get_ignore(&gather_args.local_directory);
    let mut current_file_shas: ShaFile = ShaFile::new();

    if time_since < Duration::from_secs(gather_args.interval) {
        let wait_time: Duration = interval_duration - time_since;
        info!(
            "[Gatherer - {}] Not ready for backup sleeping for {:?} seconds...",
            gather_args.name, wait_time
        );
        async_std::task::sleep(wait_time).await;
    }

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

    let uploaded_files: Vec<SandmanUploadedFile> = backup(sha_diff, gather_args, aws_config).await.unwrap();
    cleanup_deletables(gather_args, &uploaded_files).await;
}
