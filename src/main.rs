use clap::Parser;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    error::Error,
    fs,
    hash::Hasher,
    io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

mod hasher;
use hasher::FxHasher;

use nohash_hasher::BuildNoHashHasher;

/// A simple CLI to recursively find and remove duplicate files.
#[derive(Parser, Debug)]
#[command(name = "dupe-be-gone", author = "nullscry", version = "0.1", about, long_about = None)]
struct Args {
    /// Name of the directory to start recursive dupelicate search.
    #[arg(value_name = "FILE_DIR")]
    directory: Option<PathBuf>,

    /// Whether to consider comparing files from different directories.
    #[arg(short, long, action)]
    combined: bool,

    /// Whether to print outputs of details.
    #[arg(short, long, action)]
    silent: bool,

    /// Number of threads to use. Higher values will speed up the process. But higher values might also hog resources.
    #[arg(short, long, default_value_t = 128)]
    threads: usize,
}

// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &Path, all_files: &mut Vec<PathBuf>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, all_files)?;
            } else {
                all_files.push(path);
            }
        }
    }
    Ok(())
}

fn get_file_hash(filepath: PathBuf) -> (PathBuf, Vec<u8>) {
    let mut hasher = Sha256::new();
    let mut file = fs::File::open(filepath.as_path()).unwrap();
    let parent = filepath
        .parent()
        .unwrap_or_else(|| Path::new("/"))
        .display()
        .to_string();
    let mut parent = parent.as_bytes();

    let _bytes_written = io::copy(&mut file, &mut hasher).unwrap();
    let _bytes_written = io::copy(&mut parent, &mut hasher).unwrap();
    let hash_bytes = hasher.finalize().as_slice().to_vec();
    (filepath, hash_bytes)
}

fn get_file_hash_combined(filepath: PathBuf) -> (PathBuf, Vec<u8>) {
    let mut hasher = Sha256::new();
    let mut file = fs::File::open(filepath.as_path()).unwrap();

    let _bytes_written = io::copy(&mut file, &mut hasher).unwrap();
    let hash_bytes = hasher.finalize().as_slice().to_vec();
    (filepath, hash_bytes)
}

fn get_file_hash_rustc(filepath: PathBuf) -> (u64, PathBuf) {
    let mut hasher = FxHasher::default();
    let mut file = fs::File::open(filepath.as_path()).unwrap();
    let parent = filepath
        .parent()
        .unwrap_or_else(|| Path::new("/"))
        .display()
        .to_string();
    let mut parent = parent.as_bytes();

    let _bytes_written = io::copy(&mut file, &mut hasher).unwrap();
    let _bytes_written = io::copy(&mut parent, &mut hasher).unwrap();
    let hash_bytes = hasher.finish();
    (hash_bytes, filepath)
}

fn get_file_hash_combined_rustc(filepath: PathBuf) -> (u64, PathBuf) {
    let mut hasher = FxHasher::default();
    let mut file = fs::File::open(filepath.as_path()).unwrap();

    let _bytes_written = io::copy(&mut file, &mut hasher).unwrap();
    let hash_bytes = hasher.finish();
    (hash_bytes, filepath)
}

fn hash_files_func(
    all_files: Vec<PathBuf>,
    combined: bool,
    num_threads: usize,
) -> HashMap<u64, Vec<PathBuf>> {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    let h_func: fn(PathBuf) -> (u64, PathBuf) = if combined {
        get_file_hash_combined_rustc
    } else {
        get_file_hash_rustc
    };

    pool.install(|| {
        let hash_groups: HashMap<u64, Vec<PathBuf>, BuildNoHashHasher<u64>> =
            HashMap::with_hasher(BuildNoHashHasher::default());
        let hash_groups = Arc::new(Mutex::new(Box::new(hash_groups)));

        all_files.into_par_iter().for_each(|file| {
            let (file_hash, file_path) = h_func(file);
            hash_groups
                .lock()
                .unwrap()
                .entry(file_hash)
                .or_insert_with(Vec::new)
                .push(file_path);
        });
        // let file_hashes = all_files
        //     .into_par_iter()
        //     .map(h_func)
        //     .collect::<HashMap<PathBuf, Vec<u8>>>();

        // let mut hash_groups = HashMap::new();

        // for (val, key) in file_hashes {
        //     hash_groups.entry(key).or_insert_with(Vec::new).push(val);
        // }
        let hash_groups = &*hash_groups.lock().unwrap();
        hash_groups
            .to_owned()
            .into_par_iter()
            .filter(|(_key, val)| val.len() > 1)
            .collect()
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let target_dir = args
        .directory
        .expect("Please provide a proper path for directory.");

    let target_dir = &target_dir.as_path();

    if !Path::new(target_dir).exists() {
        println!(
            "Given path to directory {} does not exist.",
            target_dir.display()
        );
        return Ok(());
    }

    let mut all_files = Vec::new();

    if !args.silent {
        println!("Gathering all files...");
    }
    visit_dirs(target_dir, &mut all_files)?;

    if !args.silent {
        println!("Finding duplicates...");
    }
    let file_hashes = hash_files_func(all_files, args.combined, args.threads);

    if file_hashes.is_empty() {
        if !args.silent {
            println!("No duplicates found!");
        }

        return Ok(());
    }

    let mut marked_files = Vec::new();

    let mut user_input = String::new();
    let mut user_choice;
    let stdin = io::stdin();
    if !args.silent {
        println!("Starting duplicate cleaning procedure!");
    }

    for files in file_hashes.values() {
        println!("0: <TO KEEP ALL>");
        for (i, file) in files.iter().enumerate() {
            println!("{}: {:?}", i + 1, file);
        }

        println!("Enter the number of the file you'd like TO KEEP: ");
        loop {
            user_input.clear();
            match stdin.read_line(&mut user_input) {
                Ok(_) => {}
                Err(_) => {
                    eprintln!("Please enter a valid input.");
                    continue;
                }
            }
            match user_input.trim().parse::<usize>() {
                // If `optional` destructures, evaluate the block.
                Ok(n) => {
                    if n > files.len() {
                        println!("Please enter a valid choice.");
                    } else {
                        user_choice = n;
                        break;
                    }
                    // ^ Requires 3 indentations!
                }
                // Quit the loop when the destructure fails:
                Err(e) => {
                    eprintln!("Error encountered: {}", e);
                    eprintln!("Please enter a valid number.");
                } // ^ Why should this be required? There must be a better way!
            }
        }

        if user_choice != 0 {
            for (i, file) in files.iter().enumerate() {
                if i + 1 != user_choice {
                    if !args.silent {
                        println!("Marking file for deletion: {:?}", file);
                    }
                    marked_files.push(file);
                }
            }
        }

        println!();
    }

    for file in marked_files {
        match fs::remove_file(file) {
            Ok(()) => {
                if !args.silent {
                    println!("Deleted file {:?}", file)
                }
            }
            Err(e) => {
                eprintln!(
                    "Encountered error {} when trying to delete file {:?}.",
                    e, file
                )
            }
        }
    }

    if !args.silent {
        println!("Finished cleaning dupes!");
    }

    Ok(())
}
