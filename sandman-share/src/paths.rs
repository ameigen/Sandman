use crate::consts::{BASE_CONFIG, BASE_GLOBAL_IGNORE, SANDMAN_CONFIG, SANDMAN_IGNORE};
use directories::ProjectDirs;
use log::info;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

pub fn config_dir() -> OsString {
    if let Some(project_directories) = ProjectDirs::from("", "", "Sandman") {
        return OsString::from(project_directories.config_dir());
    }
    OsString::from("./")
}

pub fn file_in_config(file_name: &str) -> PathBuf {
    let config_path: OsString = config_dir();
    return Path::new(&config_path).join(file_name);
}

pub fn verify_config_existence() {
    let path: &PathBuf = &file_in_config("");
    let config_file: &PathBuf = &file_in_config(SANDMAN_CONFIG);
    let ignore_file: &PathBuf = &file_in_config(SANDMAN_IGNORE);

    if !path.exists() {
        fs::create_dir_all(path)
            .unwrap_or_else(|e| panic!("Error creating default configuration path: {}", e));
    }

    if !ignore_file.exists() {
        fs::write(ignore_file, BASE_GLOBAL_IGNORE).expect("Unable to write default .sandmanignore");
        info!("Creating global .sandmanignore file at {:?}", ignore_file);
    }

    if !config_file.exists() {
        fs::write(config_file, BASE_CONFIG).expect("Unable to write default config.");
        info!("No configuration file was found-- please modify the default found at {:?} and restart Sandman", config_file);
        std::process::exit(-1);
    }
}
