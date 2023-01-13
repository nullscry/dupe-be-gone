# dupe-be-gone

[![Build Status](https://github.com/nullscry/dupe-be-gone/actions/workflows/ci.yml/badge.svg)](https://github.com/nullscry/dupe-be-gone/actions/workflows/ci.yml)
[![Release Status](https://github.com/nullscry/dupe-be-gone/actions/workflows/release.yml/badge.svg)](https://github.com/nullscry/dupe-be-gone/releases)
[![Crate Status](https://img.shields.io/crates/v/dupe-be-gone.svg)](https://crates.io/crates/dupe-be-gone)
[![Docs Status](https://docs.rs/dupe-be-gone/badge.svg)](https://docs.rs/crate/dupe-be-gone/)

A Multi-threaded duplicate file cleaner usuable as a CLI application.

## Target Plaforms

- aarch64-linux
- x86_64-linux
- x86_64-macos
- x86_64-windows

## Usage

```sh
dupe-be-gone --help
A simple CLI to recursively find and remove duplicate files

Usage: dupe-be-gone [OPTIONS] [FILE_DIR]

Arguments:
  [FILE_DIR]  Name of the directory to start recursive dupelicate search

Options:
  -c, --combined           Whether to consider comparing files from different directories
  -s, --silent             Whether to print outputs of details
  -t, --threads <THREADS>  Number of threads to use. Higher values will speed up the process. But higher values might also hog resources [default: 128]
  -h, --help               Print help information
  -V, --version            Print version information
```
