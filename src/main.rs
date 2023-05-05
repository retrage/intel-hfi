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
    /// CPU number
    #[arg(short, long, default_value = "0")]
    cpu: usize,
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
    #[arg(short, long)]
    all: bool,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let info = hfi::HfiInfo::new(cli.cpu)?;

    match &cli.command {
        Commands::Hfi(args) => {
            println!("HFI Table:");
            println!("{}", info);

            let mut table = HfiTable::<NUM_CPUS>::new();
            table.read(&info)?;

            println!("{}", table.header);

            if args.all {
                for cpu in 0..NUM_CPUS {
                    println!("  CPU {}:", cpu);
                    println!("{}", table.entries[cpu]);
                }
            } else {
                println!("  CPU {}:", cli.cpu);
                println!("{}", table.entries[cli.cpu]);
            }
        }
    }

    Ok(())
}
