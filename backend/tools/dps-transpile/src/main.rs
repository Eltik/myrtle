use std::process::ExitCode;

use clap::Parser;
use myrtle_dps_transpile::Cli;

fn main() -> ExitCode {
    match myrtle_dps_transpile::run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}
