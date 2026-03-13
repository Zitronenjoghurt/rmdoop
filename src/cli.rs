use clap::{Parser, ValueHint};
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
    /// If this is true and a group of duplicate files has no source file, a random file from the group will be promoted to a source file.
    #[arg(short, long)]
    pub promote_random_duplicate: bool,
}
