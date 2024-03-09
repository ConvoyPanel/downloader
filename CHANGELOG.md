# Changelog

This file is a running track of new features and fixes to each version of the panel released starting with `v0.4.0`.

This project follows [Semantic Versioning](http://semver.org) guidelines.

## v2.0.0

### Changes

- Refactored everything
- Added support for custom images metadata JSON. You have to specify the URL as an argument when running the
  binary. [Example](https://images.cdn.convoypanel.com/images.json)
- Improved clean up logic. Downloaded templates will be removed if the process is interrupted (with crtl+c) or
  completed.
- Changed where the templates are downloaded. They will now be downloaded in your system's temporary directory.

## v1.0.1

### Fixed

- Out of memory issue when downloading the templates (thanks for the tip adly)

## v1.0.0

### Changed

- From Go to Rust
- Improved stability

## v0.4.0

### Changed

- Removed AlmaLinux 8 because of mounting issues
- Removed AlmaLinux 9 because of kernel panic issues
- Removed RockyLinux 9 because failed to boot everytime
