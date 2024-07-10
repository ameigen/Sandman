use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::string::String;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct ShaFile {
    pub files: HashMap<String, String>,
    pub timestamp: u128,
}

impl ShaFile {
    pub fn new() -> Self {
        let start = SystemTime::now();
        let timestamp = start
            .duration_since(UNIX_EPOCH)
            .expect("System time is before Unix epoch")
            .as_millis();

        ShaFile {
            files: HashMap::new(),
            timestamp,
        }
    }
}

pub fn generate_shas(directory: String, sha_file: &mut ShaFile, ignore_list: &[String]) {
    let paths = match fs::read_dir(directory) {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("Unable to open directory: {}", e);
            return;
        }
    };

    for entry in paths {
        let path = match entry {
            Ok(entry) => entry.path(),
            Err(e) => {
                eprintln!("Error reading directory entry: {}", e);
                continue;
            }
        };

        let path_str = match path.to_str() {
            Some(p) => p.to_string(),
            None => {
                eprintln!("Invalid path: {:?}", path);
                continue;
            }
        };

        if ignore_list.iter().any(|ignore| path_str.contains(ignore)) {
            continue;
        }

        if path.is_dir() {
            generate_shas(path_str, sha_file, ignore_list);
        } else {
            let bytes = match fs::read(&path) {
                Ok(bytes) => bytes,
                Err(e) => {
                    eprintln!("Error while opening file: {:?} {}", path, e);
                    continue;
                }
            };

            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let hash = format!("{:x}", hasher.finalize());
            sha_file.files.insert(path_str, hash);
        }
    }
}

pub fn get_prior_shas(sha_location: String) -> ShaFile {
    match fs::read_to_string(sha_location) {
        Ok(json) => serde_json::from_str(&json).expect("Unable to deserialize ShaFile"),
        Err(e) => {
            println!("Unable to find ShaFile: {}. Defaulting to empty.", e);
            ShaFile::new()
        }
    }
}

pub fn get_sha_diff(old: &ShaFile, new: ShaFile) -> ShaFile {
    let mut diff = ShaFile::new();

    for (k, v) in &new.files {
        if old.files.get(k) != Some(v) {
            diff.files.insert(k.clone(), v.clone());
        }
    }

    diff
}

pub fn merge_diff_old(mut old: ShaFile, new: &ShaFile) -> ShaFile {
    for (k, v) in &new.files {
        old.files.insert(k.clone(), v.clone());
    }
    old
}

pub fn write_file_shas(shas: &ShaFile, output_path: String) {
    let shas_json = serde_json::to_string_pretty(shas).expect("Failed to serialize ShaFile");
    fs::write(output_path, shas_json).expect("Failed to write ShaFile");
}

pub fn get_ignore(ignore_path: &str) -> Vec<String> {
    if PathBuf::from(ignore_path).is_file() {
        let content = fs::read_to_string(ignore_path).expect("Failed to read ignore file");
        content
            .lines()
            .map(|line| line.trim_end_matches('\r').to_string())
            .collect()
    } else {
        Vec::new()
    }
}
