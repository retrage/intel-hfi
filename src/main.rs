// SPDX-License-Identifier: MIT
// Copyright (C) 2023 Akira Moroo

//! Intel Hardware Feedback Interface (HFI) utility

mod cpuid;
mod hfi;
mod msr;

use crate::hfi::HfiTable;
use clap::{Args, Parser, Subcommand};
use std::io;

const NUM_CPUS: usize = 32;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Dumps HFI table
    Hfi(HfiArgs),
}

#[derive(Args)]
struct HfiArgs {
    /// CPU number
    #[arg(short, long, default_value = "0")]
    cpu: usize,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Hfi(args) => {
            let info = hfi::HfiInfo::new(args.cpu)?;
            println!("HFI Table:");
            println!("{}", info);

            let mut table = HfiTable::<NUM_CPUS>::new();
            table.read(&info)?;

            println!("{}", table.header);
            println!("  CPU {}:", args.cpu);
            println!("{}", table.entries[args.cpu]);
        }
    }

    Ok(())
}
