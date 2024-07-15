<div align="center">

# Sandman
A simple Rust backed AWS S3 Backup Client

[![Rust Build](https://github.com/ameigen/Sandman/actions/workflows/rust.yml/badge.svg)](https://github.com/ameigen/Sandman/actions/workflows/rust.yml)
</div>

<div align="left">


## Overview

* Overview 1
* Overview 2
* Overview 3
* Overview 4
* Overview 5

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
---
</div>
