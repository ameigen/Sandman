<div align="center">

# Sandman
A simple Rust backed AWS S3 Backup Client
</div>

<div align="left">

---
## Features

* Features 1
* Features 2
* Features 3
* Features 4
* Features 5
---

## Overview

* Overview 1
* Overview 2
* Overview 3
* Overview 4
* Overview 5
----

## Configuration Files

----
Configuration of Sandman can be performed manually with a `sandman_config.toml` file defined as follows.

```toml
title = "Sandman Config"

[aws]
aws_access_key_id = "AWS_ACCESS_KEY_ID"
aws_default_region = "DEFAULT_REGION"
aws_secret_access_key = "AWS_SECRET_ACCESS_KEY"

[directories]
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