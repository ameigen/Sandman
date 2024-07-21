pub const SANDMAN_HISTORY: &str = ".sandman_history";
pub const SANDMAN_CONFIG: &str = ".sandman_config.toml";
pub const SANDMAN_IGNORE: &str = ".sandmanignore";
pub(crate) static BASE_CONFIG: &str = r#"
title = "Example Sandman Config"

[aws]
aws_access_key_id = "AWS_ACCESS_KEY_ID"
aws_default_region = "DEFAULT_REGION"
aws_secret_access_key = "AWS_SECRET_ACCESS_KEY"

# Backup configurations with names directory paths, prefixes, buckets, intervals, and start times.
[[directories.backups]]
name = "Example Backup 1"
directory = "PATH-GOES HERE"
prefix = "ExampleBackup1"
bucket = "our-bucket-name"
interval = 10
start_time = 0

[[directories.backups]]
name = "Example Backup 2"
directory = "PATH-GOES HERE"
prefix = "ExampleBackup2"
bucket = "our-bucket-name"
interval = 180
start_time = 0"#;

pub(crate) static BASE_GLOBAL_IGNORE: &str = r#"
.sandman_history
.sandman_config.toml
.sandmanignore
"#;
