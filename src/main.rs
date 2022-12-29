use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    env,
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    time,
};

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

// fn hash_files(all_files: Vec<PathBuf>) -> Result<HashMap<PathBuf, Vec<u8>>, Box<dyn Error>> {
//     let mut file_hashes = HashMap::new();

//     for filepath in all_files {
//         let mut hasher = Sha256::new();
//         // let filepath = path.unwrap().path();
//         // println!("Name: {}", filepath.display());
//         let mut file = fs::File::open(filepath.as_path())?;
//         let _bytes_written = io::copy(&mut file, &mut hasher)?;
//         let hash_bytes = hasher.finalize().as_slice().to_vec();
//         file_hashes.insert(filepath, hash_bytes);
//     }

//     Ok(file_hashes)
// }

fn get_file_hash(filepath: PathBuf) -> (PathBuf, Vec<u8>) {
    let mut hasher = Sha256::new();
    let mut file = fs::File::open(filepath.as_path()).unwrap();
    let _bytes_written = io::copy(&mut file, &mut hasher).unwrap();
    let hash_bytes = hasher.finalize().as_slice().to_vec();
    (filepath, hash_bytes)
}

fn hash_files_func(all_files: Vec<PathBuf>) -> HashMap<Vec<u8>, Vec<PathBuf>> {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(256)
        .build()
        .unwrap();

    let hash_groups = pool.install(|| {
        let file_hashes = all_files
            .into_par_iter()
            .map(get_file_hash)
            .collect::<HashMap<PathBuf, Vec<u8>>>();

        let mut hash_groups = HashMap::<Vec<u8>, Vec<PathBuf>>::new();
        for (val, key) in file_hashes {
            hash_groups.entry(key).or_insert(Vec::new()).push(val);
        }
        hash_groups
    });

    hash_groups
}

fn main() -> Result<(), Box<dyn Error>> {
    let now = time::Instant::now();

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage:");
        println!("dupe-be-gone: <directory>");
        return Ok(());
    }

    let target_dir = Path::new(args[1].as_str());

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
    let file_hashes = hash_files_func(all_files);

    for (hash_key, files) in file_hashes {
        println!("{:?}: {:?}", hash_key, files);
    }

    let elapsed_time = now.elapsed();
    println!("Running main() took {} seconds.", elapsed_time.as_secs());

    Ok(())
}

// TODO
// - Traverse recursively [x]
// - Compare hashes[ ]
