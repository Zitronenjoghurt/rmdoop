use clap::{Parser, ValueHint};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rmdoop", version)]
pub struct Cli {
    #[arg(short, long, required = true, num_args = 1.., value_hint = ValueHint::DirPath)]
    pub sources: Vec<PathBuf>,
    #[arg(short, long, required = true, num_args = 1.., value_hint = ValueHint::DirPath)]
    pub targets: Vec<PathBuf>,
}
