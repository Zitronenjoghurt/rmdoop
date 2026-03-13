use crate::grouping::{DuplicateGroup, group_identical};
use clap::{Parser, ValueHint};
use colored::Colorize;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rmdoop", version)]
pub struct Cli {
    /// Target paths to scan and potentially delete files from.
    /// Can be either files or directories (which will be scanned recursively).
    #[arg(value_hint = ValueHint::AnyPath)]
    pub targets: Vec<PathBuf>,
    /// Source paths to scan, which will be compared to the target paths and never deleted.
    /// Can be either files or directories (which will be scanned recursively).
    #[arg(short, long, value_hint = ValueHint::AnyPath)]
    pub sources: Vec<PathBuf>,
    /// If this is true, the program will print more information about what it's doing.
    #[arg(short, long)]
    pub verbose: bool,
    /// If this is true, the program will print absolutely nothing (besides the final result and errors).
    #[arg(short, long)]
    pub quiet: bool,
    /// If this is true, the program will not prompt for confirmation before deleting files.
    /// It will only delete duplicate files where there is at least one source file.
    #[arg(short, long)]
    pub autonomous: bool,
    /// If this is true, the program will list all files that would have been deleted.
    #[arg(short, long)]
    pub list: bool,
    /// If this is true and a group of duplicate files has no source file, the first file from the group will be promoted to a source file.
    #[arg(short, long)]
    pub promote: bool,
}

impl Cli {
    pub fn execute(&self) -> anyhow::Result<u64> {
        let groups = group_identical(&self.sources, &self.targets, self.grouping_config())?;

        if !self.quiet && !self.list {
            println!(
                "Found {} groups of identical files for deduplication.",
                groups.len().to_string().green().bold()
            );
        }

        if self.list {
            self.print_list(groups);
            return Ok(0);
        }

        if groups.is_empty() {
            return Ok(0);
        }

        let (files, bytes) = if self.autonomous {
            self.autonomous_deletion(groups)?
        } else {
            self.prompted_deletion(groups)?
        };

        println!(
            "Deleted {} files, clearing up {}.",
            files.to_string().green().bold(),
            format_byte_size(bytes).green().bold()
        );

        Ok(files)
    }

    fn print_list(&self, groups: Vec<DuplicateGroup>) {
        if groups.is_empty() {
            println!("There are no duplicate files to delete.");
            return;
        }

        let mut delete_count = 0;
        println!("{}", "Files that would be deleted:".bold());
        for group in groups {
            if group.originals.is_empty() {
                continue;
            }
            for duplicate in &group.duplicates {
                println!("  - {}", duplicate.display().to_string().italic());
            }
            delete_count += group.duplicates.len();
        }

        if delete_count == 0 {
            println!(
                "There are duplications, but no files would be deleted. Check that you provide source paths or use --promote (-p) to automatically promote a random duplicate of each group of identical files to a source file."
            );
        }
    }

    fn autonomous_deletion(&self, groups: Vec<DuplicateGroup>) -> anyhow::Result<(u64, u64)> {
        let group_count = groups.len();
        let mut files = 0;
        let mut total_bytes = 0;
        let mut skipped_groups = 0;
        let mut skipped_files = 0;

        for group in groups {
            if group.originals.is_empty() {
                skipped_groups += 1;
                skipped_files += group.duplicates.len();
                if self.verbose && !self.quiet {
                    println!(
                        "Skipping group of {} files because it has no source file:",
                        group.duplicates.len().to_string().yellow().bold()
                    );

                    for duplicate in &group.duplicates {
                        println!("  - {}", duplicate.display().to_string().italic());
                    }
                }
                continue;
            }

            for duplicate in &group.duplicates {
                let bytes = std::fs::metadata(duplicate)?.len();
                std::fs::remove_file(duplicate)?;
                files += 1;
                total_bytes += bytes;
                if self.verbose && !self.quiet {
                    println!(
                        "Deleted {} ({}).",
                        duplicate.display().to_string().italic(),
                        format_byte_size(bytes).green().bold()
                    );
                }
            }
        }

        if !self.quiet && group_count > 1 {
            println!(
                "Skipped {} out of {} groups ({} files) because they have no source file.",
                skipped_groups.to_string().yellow().bold(),
                group_count.to_string().yellow().bold(),
                skipped_files.to_string().yellow().bold()
            );
        }

        Ok((files, total_bytes))
    }

    fn prompted_deletion(&self, _groups: Vec<DuplicateGroup>) -> anyhow::Result<(u64, u64)> {
        Ok((0, 0))
    }

    fn grouping_config(&self) -> crate::grouping::GroupingConfig {
        crate::grouping::GroupingConfig {
            promote_random_duplicate: self.promote,
            verbose: self.verbose,
            quiet: self.quiet,
        }
    }
}

const BYTE_UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
fn format_byte_size(size: u64) -> String {
    let mut size = size as f64;
    for unit in BYTE_UNITS.iter() {
        if size < 1000.0 {
            return format!("{size:.2} {unit}");
        }
        size /= 1000.0;
    }
    format!("{} {}", size.round(), BYTE_UNITS.last().unwrap())
}
