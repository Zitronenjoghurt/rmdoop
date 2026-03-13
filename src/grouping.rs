use crate::io::{FileRef, files_identical, hash_file, walk_path_group_by_size};
use colored::Colorize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug)]
pub struct DuplicateGroup {
    pub originals: Vec<PathBuf>,
    pub duplicates: Vec<PathBuf>,
}

impl DuplicateGroup {
    pub fn from_refs(refs: Vec<FileRef>) -> DuplicateGroup {
        let (originals, duplicates) = refs.into_iter().fold(
            (Vec::new(), Vec::new()),
            |(mut origs, mut dups), file_ref| {
                if file_ref.deletable {
                    dups.push(file_ref.path);
                } else {
                    origs.push(file_ref.path);
                }
                (origs, dups)
            },
        );
        DuplicateGroup {
            originals,
            duplicates,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct GroupingConfig {
    pub promote_random_duplicate: bool,
    pub quiet: bool,
    pub verbose: bool,
}

impl GroupingConfig {
    pub fn verbose(&self) -> bool {
        self.verbose && !self.quiet
    }
}

pub fn group_identical(
    sources: &[PathBuf],
    targets: &[PathBuf],
    config: GroupingConfig,
) -> anyhow::Result<Vec<DuplicateGroup>> {
    let mut refs_by_size: HashMap<u64, Vec<FileRef>> = HashMap::new();

    for source in sources {
        walk_path_group_by_size(source, &mut refs_by_size, false);
    }

    for target in targets {
        walk_path_group_by_size(target, &mut refs_by_size, true);
    }

    if !config.quiet {
        let source_count = refs_by_size
            .values()
            .map(|refs| refs.iter().filter(|ref_| !ref_.deletable).count())
            .sum::<usize>();
        let target_count = refs_by_size
            .values()
            .map(|refs| refs.iter().filter(|ref_| ref_.deletable).count())
            .sum::<usize>();
        let total_count = source_count + target_count;
        println!(
            "Found {} files to check for duplicates: {} source, {} target.",
            total_count.to_string().green().bold(),
            source_count.to_string().green().bold(),
            target_count.to_string().yellow().bold()
        );
    }

    let hashed_groups = group_duplicates_by_hash(refs_by_size)?;
    let raw_groups = hashed_groups
        .into_values()
        .try_fold(Vec::new(), |mut acc, refs| {
            let mut verified = verify_duplicates_byte_by_byte(refs)?;
            acc.append(&mut verified);
            Ok::<Vec<Vec<FileRef>>, anyhow::Error>(acc)
        })?;

    let mut groups: Vec<DuplicateGroup> = raw_groups
        .into_iter()
        .map(DuplicateGroup::from_refs)
        .collect();

    if config.promote_random_duplicate {
        if !config.quiet {
            let promotable_group_count = groups
                .iter()
                .filter(|group| group.originals.is_empty())
                .count();
            println!(
                "{} target files will be promoted to source files.",
                promotable_group_count.to_string().green().bold()
            );
        }

        groups
            .iter_mut()
            .filter(|group| group.originals.is_empty())
            .for_each(|group| {
                let promoted = group.duplicates.remove(0);
                if config.verbose() {
                    println!(
                        "Promoted {} to source file.",
                        promoted.to_string_lossy().italic()
                    );
                }
                group.originals.push(promoted);
            });
    }

    Ok(groups)
}

fn group_duplicates_by_hash(
    map: HashMap<u64, Vec<FileRef>>,
) -> anyhow::Result<HashMap<u64, Vec<FileRef>>> {
    let to_hash: Vec<FileRef> = map
        .into_values()
        .filter(|refs| refs.len() > 1)
        .flatten()
        .collect();

    let hashed: Result<Vec<(u64, FileRef)>, _> = to_hash
        .into_iter()
        .map(|file_ref| {
            let hash = hash_file(&file_ref.path)?;
            Ok::<(u64, FileRef), anyhow::Error>((hash, file_ref))
        })
        .collect();

    let mut by_hash = hashed?.into_iter().fold(
        HashMap::new(),
        |mut acc: HashMap<u64, Vec<FileRef>>, (hash, file_ref)| {
            acc.entry(hash).or_default().push(file_ref);
            acc
        },
    );

    by_hash.retain(|_, refs| refs.len() > 1);
    Ok(by_hash)
}

fn verify_duplicates_byte_by_byte(
    mut candidates: Vec<FileRef>,
) -> anyhow::Result<Vec<Vec<FileRef>>> {
    let mut verified_groups: Vec<Vec<FileRef>> = Vec::new();

    while let Some(reference) = candidates.pop() {
        let mut current_group = vec![reference];
        let mut remaining_candidates = Vec::new();

        for candidate in candidates {
            if files_identical(&current_group[0].path, &candidate.path)? {
                current_group.push(candidate);
            } else {
                remaining_candidates.push(candidate);
            }
        }

        if current_group.len() > 1 {
            verified_groups.push(current_group);
        }

        candidates = remaining_candidates;
    }

    Ok(verified_groups)
}
