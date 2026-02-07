use clap::Parser;
use std::error::Error;

mod cli;
mod common;
mod paired;
mod window;

use crate::{cli::Commands, paired::pair_pileups, window::window_pileup};

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Cli::parse();

    match args.command {
        Commands::Window(window_args) => window_pileup(window_args)?,
        Commands::Paired(pair_args) => pair_pileups(pair_args)?,
    };
    Ok(())
}
