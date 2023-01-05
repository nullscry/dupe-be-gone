# dupe-be-gone

A simple CLI to recursively find and remove duplicate files

This is a project that I like to work on to improve my Rust skills.

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
