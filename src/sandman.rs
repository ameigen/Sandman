use std::fmt::format;
use crate::backup::backup;
use crate::sha::{
    generate_shas, get_ignore, get_prior_shas, get_sha_diff, merge_diff_old, write_file_shas,
    ShaFile,
};
use crate::config::Config;
use clap::Parser;
use clap_derive::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    local_directory: String,

    #[arg(long)]
    sha_file: String,

    #[arg(long)]
    s3_bucket: String,

    #[arg(long, default_value_t = String::new())]
    ignore_file: String,

    #[arg(long)]
    bucket_prefix: String,

    #[arg(short, long, default_value_t = false)]
    verbosity: bool,

    #[arg(long, default_value_t = false)]
    with_config: bool,
}

// Break the upload functionality into it's own function

pub async fn run_sandman() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.with_config{
        let config_string: String = tokio::fs::read_to_string("./sandman_config.toml").await.unwrap();
        let config: Config = toml::from_str(&config_string).unwrap();

        Ok(for directory in config.directories.backups {
            let mut current_file_shas: ShaFile = ShaFile::new();
            let ignore_file_path = format!("{}/{}", directory.directory, "./sandmanignore");
            let ignore = get_ignore(&ignore_file_path);
            generate_shas(directory.directory, &mut current_file_shas, &ignore);

        })
    }
    else {
        let mut current_file_shas = ShaFile::new();
        let ignore_file_path = if args.ignore_file.is_empty() {
            format!("{}/{}", args.local_directory, ".sandmanignore")
        } else {
            args.ignore_file.clone()
        };

        let ignore = get_ignore(&ignore_file_path);
        generate_shas(args.local_directory, &mut current_file_shas, &ignore);

        let old_file_shas = get_prior_shas(args.sha_file.clone());
        let diff = get_sha_diff(&old_file_shas, current_file_shas);
        let merged = merge_diff_old(old_file_shas, &diff);

        write_file_shas(&merged, args.sha_file);
        backup(diff, args.s3_bucket, args.bucket_prefix).await?;

        Ok(())
    }
}
