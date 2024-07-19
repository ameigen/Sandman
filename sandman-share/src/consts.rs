
pub const SANDMAN_HISTORY: &str = ".sandman_history";
pub const SANDMAN_CONFIG: &str = ".sandman_config.toml";
pub const SANDMAN_IGNORE: &str = ".sandmanignore";
pub(crate) static BASE_CONFIG: &str = r#"
title = "Sandman Config"

[aws]
aws_access_key_id = "AKIAU2KKXXLEYTEWFUVI"
aws_default_region = "us-east-1"
aws_secret_access_key = "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"

[directories]
backups = [
    {directory="./sandman", prefix="NightlyBackup", bucket="rust-bucket-test-test-test"},
]"#;