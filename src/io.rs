use jwalk::WalkDir;
use rapidhash::fast::RapidHasher;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hasher;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct FileRef {
    pub path: PathBuf,
    pub deletable: bool,
}

pub fn walk_path_group_by_size(path: &Path, map: &mut HashMap<u64, Vec<FileRef>>, deletable: bool) {
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file()
            && let Ok(meta) = entry.metadata()
        {
            map.entry(meta.len()).or_default().push({
                FileRef {
                    path: entry.path().to_owned(),
                    deletable,
                }
            });
        }
    }
}

pub fn hash_file(path: &Path) -> anyhow::Result<u64> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = RapidHasher::default();
    let mut buf = [0u8; 8192];

    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => hasher.write(&buf[..n]),
            Err(e) => return Err(anyhow::anyhow!("Failed to read file: {}", e)),
        }
    }

    Ok(hasher.finish())
}

pub fn files_identical(path1: &Path, path2: &Path) -> anyhow::Result<bool> {
    let mut file1 = File::open(path1)?;
    let mut file2 = File::open(path2)?;

    let mut buf1 = [0u8; 8192];
    let mut buf2 = [0u8; 8192];

    loop {
        let n1 = file1.read(&mut buf1)?;
        let n2 = file2.read(&mut buf2)?;

        if n1 != n2 || buf1[..n1] != buf2[..n2] {
            return Ok(false);
        }

        if n1 == 0 {
            return Ok(true);
        }
    }
}
