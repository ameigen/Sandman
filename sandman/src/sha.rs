use log::error;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::string::String;
use std::time::{SystemTime, UNIX_EPOCH};

/// Struct representing SHA file information with a map of file paths to SHA values and a timestamp.
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub(crate) struct ShaFile {
    pub(crate) files: HashMap<String, String>,
    pub(crate) timestamp: u128,
}

impl ShaFile {
    /// Creates a new `ShaFile` instance with the current timestamp.
    pub(crate) fn new() -> Self {
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

/// Generates SHA-256 hashes for files in the given directory, ignoring files specified in the ignore list.
///
/// # Arguments
///
/// * `directory` - The directory to scan for files.
/// * `sha_file` - A mutable reference to a `ShaFile` to store the hashes.
/// * `ignore_list` - A list of file patterns to ignore.
pub(crate) fn generate_shas(
    directory: String,
    sha_file: &mut ShaFile,
    ignore_file: &gitignore::File,
) {
    let paths = match fs::read_dir(directory) {
        Ok(paths) => paths,
        Err(e) => {
            error!("Unable to open directory: {}", e);
            return;
        }
    };

    for entry in paths {
        let path = match entry {
            Ok(entry) => entry.path(),
            Err(e) => {
                error!("Error reading directory entry: {}", e);
                continue;
            }
        };

        let path_str = match path.to_str() {
            Some(p) => p.to_string(),
            None => {
                error!("Invalid path: {:?}", path);
                continue;
            }
        };

        match ignore_file.is_excluded(&*path) {
            Ok(is_excluded) => {
                if is_excluded {
                    continue;
                };
            }
            Err(e) => {
                error!("Error checking path against ignore file {}", e);
            }
        }

        if path.is_dir() {
            generate_shas(path_str, sha_file, ignore_file);
        } else {
            let bytes = match fs::read(&path) {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!("Error while opening file: {:?} {}", path, e);
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

/// Retrieves the prior SHA file information from the given location.
///
/// # Arguments
///
/// * `sha_location` - The file path to the SHA file.
///
/// # Returns
///
/// A `ShaFile` instance with the previously stored SHA information.
pub(crate) fn get_prior_shas(sha_location: &String) -> ShaFile {
    match fs::read_to_string(sha_location) {
        Ok(json) => serde_json::from_str(&json).expect("Unable to deserialize ShaFile"),
        Err(e) => {
            error!("Unable to find ShaFile: {}. Defaulting to empty.", e);
            ShaFile::new()
        }
    }
}

/// Computes the difference in SHA values between old and new SHA files.
///
/// # Arguments
///
/// * `old` - The old `ShaFile`.
/// * `new` - The new `ShaFile`.
///
/// # Returns
///
/// A `ShaFile` containing the differences in SHA values.
pub(crate) fn get_sha_diff(old: &ShaFile, new: ShaFile) -> ShaFile {
    let mut diff = ShaFile::new();

    for (k, v) in &new.files {
        if old.files.get(k) != Some(v) {
            diff.files.insert(k.clone(), v.clone());
        }
    }

    diff
}

/// Merges the differences from the new SHA file into the old SHA file.
///
/// # Arguments
///
/// * `old` - The old `ShaFile`.
/// * `new` - The new `ShaFile` containing the differences.
///
/// # Returns
///
/// A `ShaFile` with the merged SHA values.
pub(crate) fn merge_diff_old(mut old: ShaFile, new: &ShaFile) -> ShaFile {
    for (k, v) in &new.files {
        old.files.insert(k.clone(), v.clone());
    }
    old
}

/// Writes the SHA file information to the specified output path.
///
/// # Arguments
///
/// * `shas` - The `ShaFile` containing SHA information.
/// * `output_path` - The file path to write the SHA file to.
pub(crate) fn write_file_shas(shas: &ShaFile, output_path: &String) {
    let shas_json = serde_json::to_string_pretty(shas).expect("Failed to serialize ShaFile");
    fs::write(output_path, shas_json).expect("Failed to write ShaFile");
}