use crate::args::{Args, GatherArgs};
use crate::gatherer::Gatherer;
use clap::Parser;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use log::{error, info};
use sandman_share::config::{AwsConfig, Config};
use sandman_share::consts::{SANDMAN_CONFIG, SANDMAN_IGNORE};
use sandman_share::paths::{file_in_config, verify_config_existence};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

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
pub(crate) fn get_ignore(dir: &String) -> Gitignore {
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

/// Use the `verbosity` parameter of the `args` struct to determine the level of our logger.
fn set_loggers(verbosity: bool) {
    if verbosity {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
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
    let mut gatherers: Vec<Gatherer> = vec![];

    for directory in config.directories.backups {
        let aws_config: AwsConfig = config.aws.clone();
        let gather_args: GatherArgs = GatherArgs::new(
            directory.name,
            directory.directory,
            directory.bucket,
            directory.prefix,
            directory.interval,
            directory.start_time,
            directory.cleanable
        );

        gatherers.push(Gatherer::new(gather_args, Some(aws_config)));
        let len: usize = gatherers.len() - 1;
        let gatherer: &mut Gatherer = &mut gatherers[len];
        let _ = &gatherer.gather(false, exit.clone());
    }
    for gatherer in gatherers {
        gatherer.handle.unwrap().await.expect("TODO: panic message");
    }
}

/// Runs Sandman with provided CLI arguments in the form of an `Args` struct
///
/// # Arguments
///
/// * `args` - `Args` built from the CLI parameters
async fn with_cli_args(args: &Args) {
    let gather_args: GatherArgs = GatherArgs::new(
        "OneShotter".to_string(),
        args.local_directory.clone(),
        args.s3_bucket.clone(),
        args.bucket_prefix.clone(),
        0,
        0,
        false
    );
    let mut gatherer: Gatherer = Gatherer::new(gather_args, None);
    gatherer.gather(true, None);
    let _ = &gatherer.handle.unwrap().await;
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
