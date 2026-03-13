use crate::grouping::group_identical;
use clap::Parser;

mod cli;
mod grouping;
mod io;
#[cfg(test)]
mod tests;

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();

    let groups = group_identical(&cli.sources, &cli.targets)?;
    println!("{groups:#?}");

    Ok(())
}
