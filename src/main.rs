use clap::Parser;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    error::Error,
    fs, io,
    os::unix::prelude::OsStrExt,
    path::{Path, PathBuf},
    time,
};

/// Simple program find and remove duplicate files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the directory to start dupelicate search
    #[arg(value_name = "FILE_DIR")]
    directory: Option<PathBuf>,

    /// Whether to consider compare files from different directories
    #[arg(short, long, action)]
    single_dir: bool,

    /// Whether to print outputs of details.
    #[arg(short, long, action)]
    verbose: bool,

    /// Whether to prompt the user when removing duplicates
    #[arg(short, long, action)]
    interactive: bool,
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
    let mut parent = filepath
        .parent()
        .unwrap_or(Path::new("/"))
        .as_os_str()
        .as_bytes();

    let _bytes_written = io::copy(&mut file, &mut hasher).unwrap();
    let _bytes_written = io::copy(&mut parent, &mut hasher).unwrap();
    let hash_bytes = hasher.finalize().as_slice().to_vec();
    (filepath, hash_bytes)
}

fn get_file_hash_single_dir(filepath: PathBuf) -> (PathBuf, Vec<u8>) {
    let mut hasher = Sha256::new();
    let mut file = fs::File::open(filepath.as_path()).unwrap();

    let _bytes_written = io::copy(&mut file, &mut hasher).unwrap();
    let hash_bytes = hasher.finalize().as_slice().to_vec();
    (filepath, hash_bytes)
}

fn hash_files_func(all_files: Vec<PathBuf>, single_dir: bool) -> HashMap<Vec<u8>, Vec<PathBuf>> {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(256)
        .build()
        .unwrap();

    let h_func = if single_dir {
        get_file_hash_single_dir
    } else {
        get_file_hash
    };

    let hash_groups = pool.install(|| {
        let file_hashes = all_files
            .into_par_iter()
            .map(h_func)
            .collect::<HashMap<PathBuf, Vec<u8>>>();

        let mut hash_groups = HashMap::new();

        for (val, key) in file_hashes {
            hash_groups.entry(key).or_insert(Vec::new()).push(val);
        }
        hash_groups
            .into_par_iter()
            .filter(|(_key, val)| val.len() > 1)
            .collect()
    });

    hash_groups
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let now = time::Instant::now();

    // let args: Vec<String> = env::args().collect();

    // if args.len() != 2 {
    //     println!("Usage:");
    //     println!("dupe-be-gone: <directory>");
    //     return Ok(());
    // }

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

    // let target_dir = fs::read_dir(target_dir)?;
    // let target_dir = target_dir.filter(|x| !x.as_ref().unwrap().path().is_dir());

    let mut all_files = Vec::new();
    visit_dirs(target_dir, &mut all_files)?;
    let file_hashes = hash_files_func(all_files, args.single_dir);

    for files in file_hashes.values() {
        for file in files {
            println!("{:?}", file);
        }
        println!();
    }

    let elapsed_time = now.elapsed();
    println!("Running main() took {} seconds.", elapsed_time.as_secs());

    Ok(())
}

// TODO
// - Traverse recursively [x]
// - Compare hashes[x]
// - Compare parent[x]
// - Add getops/argparse [ ]
