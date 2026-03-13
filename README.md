[![CI](https://github.com/Zitronenjoghurt/rmdoop/actions/workflows/check.yml/badge.svg)](https://github.com/Zitronenjoghurt/rmdoop/actions/workflows/check.yml)
[![Release](https://github.com/Zitronenjoghurt/rmdoop/actions/workflows/release.yml/badge.svg)](https://github.com/Zitronenjoghurt/rmdoop/actions/workflows/release.yml)

# rmdoop

A CLI tool for recursively removing duplicate files (by content) from different directories.

## Work in progress

This project is still in development. While a minimum viable product is available, only the autonomous mode is
implemented so far. I plan to add a prompted, user-friendly mode for more curated workflows.

## Installation

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Zitronenjoghurt/rmdoop/releases/download/v0.1.0/rmdoop-v0.1.0-installer.sh | sh
```

## Usage

### Basic concept

rmdoop distinguishes between **source** and **target** files:

- **Source files** are never deleted, but are used for comparison.
- **Target files** are candidates for deletion.

A group of identical files needs at least one source file for deduplication to happen.

```sh
# All files in the current directory become targets.
# Without any source files, nothing will be deleted.
rmdoop ./
```

Provide one or more source directories using `--sources` (or `-s`):

```sh
rmdoop ./ -s ~/home/original ~/home/other-originals
```

### Promotion

If you just want to deduplicate files in a directory and don't care about which files are the actual source files, you
can use `--promote` (or `-p`):

```sh
# Promotes the first file of each identical group to a source file,
# deleting the rest.
rmdoop ./ -p
```

### Safe mode

Use `--list` (or `-l`) to preview which files would be removed without actually deleting anything:

```sh
rmdoop ./ -l
```

Combine it with `--promote` to preview deduplication of a single directory:

```sh
rmdoop ./ -lp
```

### Autonomous mode

Use `--autonomous` (or `-a`) to skip confirmation prompts. rmdoop will still only delete duplicates that have at least
one source file.

```sh
# Automatically deduplicate all files in the current directory.
rmdoop ./ -ap
```