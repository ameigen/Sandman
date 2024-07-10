use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Config {
    title: String,
    aws: AwsConfig,
    directories: DirectoriesConfig
}

#[derive(Deserialize, Debug)]
struct AwsConfig {
    aws_access_key_id: String,
    aws_default_region: String,
    aws_secret_access_key: String
}

#[derive(Deserialize, Debug)]
struct DirectoriesConfig {
    backups: Vec<SandmanDirectory>
}

#[derive(Deserialize, Debug)]
struct SandmanDirectory {
    directory: String,
    prefix: String,
    bucket: String,
}
