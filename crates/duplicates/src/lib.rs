//! Three-stage exact duplicate detector: byte size, bounded sample hash, then SHA-256.
use diskscope_common::FileEntry;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

#[derive(Debug, Clone, Serialize)]
pub struct DuplicateGroup {
    pub bytes_per_file: u64,
    pub hash: String,
    pub files: Vec<PathBuf>,
}
pub fn find_exact(entries: impl IntoIterator<Item = FileEntry>) -> io::Result<Vec<DuplicateGroup>> {
    let mut sizes: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for item in entries {
        if !item.is_dir && item.size > 0 {
            sizes.entry(item.size).or_default().push(item.path);
        }
    }
    let mut sampled: HashMap<(u64, String), Vec<PathBuf>> = HashMap::new();
    for (size, files) in sizes.into_iter().filter(|(_, values)| values.len() > 1) {
        for file in files {
            sampled
                .entry((size, sample_hash(&file)?))
                .or_default()
                .push(file);
        }
    }
    let mut full: HashMap<(u64, String), Vec<PathBuf>> = HashMap::new();
    for ((size, _), files) in sampled.into_iter().filter(|(_, values)| values.len() > 1) {
        for file in files {
            full.entry((size, full_hash(&file)?))
                .or_default()
                .push(file);
        }
    }
    Ok(full
        .into_iter()
        .filter_map(|((bytes_per_file, hash), files)| {
            (files.len() > 1).then_some(DuplicateGroup {
                bytes_per_file,
                hash,
                files,
            })
        })
        .collect())
}
fn sample_hash(path: &PathBuf) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut data = vec![0; 65_536];
    let read = file.read(&mut data)?;
    data.truncate(read);
    Ok(blake3::hash(&data).to_hex().to_string())
}
fn full_hash(path: &PathBuf) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut data = [0; 131_072];
    loop {
        let read = file.read(&mut data)?;
        if read == 0 {
            break;
        }
        hash.update(&data[..read]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
