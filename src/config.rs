use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    title: String,
    pub aws: AwsConfig,
    pub directories: DirectoriesConfig
}

#[derive(Deserialize, Debug)]
struct AwsConfig {
    pub aws_access_key_id: String,
    pub aws_default_region: String,
    pub aws_secret_access_key: String
}

#[derive(Deserialize, Debug)]
struct DirectoriesConfig {
    pub backups: Vec<SandmanDirectory>
}

#[derive(Deserialize, Debug)]
struct SandmanDirectory {
    pub directory: String,
    pub prefix: String,
    pub bucket: String,
}
