// SPDX-License-Identifier: MIT
// Copyright (C) 2023 Akira Moroo

//! Intel Hardware Feedback Interface (HFI) utility

mod cpuid;
mod hfi;
mod itd;
mod msr;

use crate::{hfi::HfiTable, itd::ItdInfo, cpuid::Cpuid};
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
    /// Dumps ITD table
    Itd(ItdArgs),
}

#[derive(Args)]
struct HfiArgs {
    #[arg(short, long)]
    all: bool,
}

#[derive(Args)]
struct ItdArgs {
    #[arg(short, long)]
    all: bool,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let cpuid = cpuid::NativeModelIdCpuid::read(cli.cpu)?;
    println!("CPU: {}", cli.cpu);
    println!("  CoreType: {:?}", cpuid.core_type());

    let hfi_info = hfi::HfiInfo::new(cli.cpu)?;
    println!("HFI Table:");
    println!("{}", hfi_info);

    match &cli.command {
        Commands::Hfi(args) => {
            let mut table = HfiTable::<NUM_CPUS>::new();
            table.read(&hfi_info)?;

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
        Commands::Itd(args) => {
            if !hfi_info.has_itd() {
                println!("ITD capability is not supported");
                return Ok(());
            }

            if args.all {
                for cpu in 0..NUM_CPUS {
                    let hfi_info = hfi::HfiInfo::new(cpu)?;
                    let itd_info = ItdInfo::new(&hfi_info);
                    println!("ITD Table (CPU {}):", cpu);
                    println!("{}", itd_info);
                }
            } else {
                let itd_info = ItdInfo::new(&hfi_info);
                println!("ITD Table (CPU {}):", cli.cpu);
                println!("{}", itd_info);
            }
        }
    }

    Ok(())
}
