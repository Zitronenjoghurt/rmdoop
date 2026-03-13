use clap::Parser;

mod cli;
#[cfg(test)]
mod tests;

fn main() {
    let cli = cli::Cli::parse();

    println!("{:?}", cli.sources);
    println!("{:?}", cli.targets);
}
