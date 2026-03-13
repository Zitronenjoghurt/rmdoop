use clap::Parser;

mod cli;
mod grouping;
mod io;
#[cfg(test)]
mod tests;

fn main() -> anyhow::Result<()> {
    let _ = cli::Cli::parse().execute()?;
    Ok(())
}
