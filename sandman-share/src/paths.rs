use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use directories::ProjectDirs;
use log::info;
use crate::consts::{BASE_CONFIG, SANDMAN_CONFIG};

pub fn config_dir<'a>() -> OsString {
    if let Some(project_directories) = ProjectDirs::from("", "", "Sandman") {
        return OsString::from(project_directories.config_dir())
    }
    OsString::from("./")
}

pub fn verify_config_existence() {
    let config_str: OsString = config_dir();
    let path: &Path = Path::new(&config_str);
    let file: PathBuf = path.join(SANDMAN_CONFIG);
    if !path.exists() {
            fs::create_dir_all(path).unwrap_or_else(|e| panic!("Error creating default configuration path: {}", e));
        }
    if !file.exists() {
        fs::write(path.join(SANDMAN_CONFIG), BASE_CONFIG).expect("Unable to write default config.");
        info!("No configuration file was found-- please modify the default found at {:?} and restart Sandman", path);
        std::process::exit(-1);
    }
}