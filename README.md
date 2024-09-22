<div align="center">

# Sandman
A simple Rust backed AWS S3 Backup Client

[![Rust Build](https://github.com/ameigen/Sandman/actions/workflows/rust.yml/badge.svg)](https://github.com/ameigen/Sandman/actions/workflows/rust.yml)
</div>

<div align="left">

### Command Line Arguments

----
```markdown
- **Local Directory to Process**
    - **Flag:** `--local-directory`
    - **Description:** Specifies the local directory to process.
    - **Default Value:** An empty string.

- **File Path to Store SHA File**
    - **Flag:** `--sha-file`
    - **Description:** Specifies the file path where the SHA file will be stored.
    - **Default Value:** An empty string.

- **S3 Bucket Name for Backup**
    - **Flag:** `--s3-bucket`
    - **Description:** Specifies the S3 bucket name for backup.
    - **Default Value:** An empty string.

- **S3 Bucket Prefix for Backup**
    - **Flag:** `--bucket-prefix`
    - **Description:** Specifies the S3 bucket prefix for backup.
    - **Default Value:** An empty string.

- **Verbosity Flag for Logging**
    - **Flags:** `-v`, `--verbosity`
    - **Description:** Increases the verbosity of logging.
    - **Default Value:** `false` (verbosity off).

- **Flag to Indicate if Configuration File Should be Used**
    - **Flag:** `--with-config`
    - **Description:** Indicates whether a configuration file should be used.
    - **Default Value:** `false` (configuration file not used).

- **Configuration File Path**
    - **Flag:** `--config-path`
    - **Description:** Specifies the path to the configuration file.
    - **Default Value:** An empty string.
```
---
## Building

```shell
git clone https://github.com/ameigen/Sandman.git

# Build only the Sandman executable
cd ./sandman/sandman
cargo build

# Build only the Sandman configuration tool
cd ./sandman/sandman-config
cargo build
```


---
## Running

```shell
# Run the Sandman application
cd ./sandman/sandman

#Run with .sandman_config.toml in the same directory
cargo run -- --with-config

#Run with CLI arguments
cargo run -- --local-directory "backup_path" --sha-file "location of diff" --bucket-prefix "prefix to prepend to s3 upload"

# Run the Sandman Configuration Tool
cd ./sandman/sandman-config
cargo run
```
---
## Features

---

### Per Directory Ignore File

Supports the use of `.gitignore` style ignore files in the form of a `.sandmanignore` file. This file should be placed
in the root of any directory that is intended to be backed up. The struct of this file should match the exact spec of
the standard `.gitignore` file

```markdown
/target
/.git
/.idea
*.txt
```

### Sandman Config

Sandman makes use of a `.sandman_config.toml` file to designate the directories to be backed up as well as any AWS
credentials required to perform the uploads.

Configuration of Sandman can be performed manually with a `sandman_config.toml` file defined as follows.


```toml
title = "Sandman Config"

[aws]
aws_access_key_id = "AWS_ACCESS_KEY_ID"
aws_default_region = "DEFAULT_REGION"
aws_secret_access_key = "AWS_SECRET_ACCESS_KEY"

[directories]
# While relative directories !WILL! work you should prefer the use of absolute directories
backups = [
    {directory="DIRECTORY_HERE", prefix="PREFIX_NAME", bucket="BUCKET_NAME"},
]

```

You can also make use of the `sandman-config` utility to modify it via a GUI
```shell
sandman-config
```
If no `--config-path` parameter is passed Sandman will check in an OS dependent default location which follows
```
Windows - C:\\Users\\%USERNAME%\\AppData\\Roaming\\Sandman\\config
Unix - $HOME/$USER/.config/Sandman/config
```

On first runtime (with the `--with-config` flag set) and if no configuration file has been provided the application will create the default directory and
exit, prompting the modify the `sandman_config.toml` as needed.

---
</div>

### Example

#### Project Hierarchy
```markdown
backup_2/
├─ .sandmanignore
├─ foo.txt
backup_1/
├─ .sandmanignore
├─ backup_1_directory/
│  ├─ foo.txt
│  ├─ bar.txt
```

#### Sandman Configuration
```toml
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
start_time = 0
cleanable = true
```