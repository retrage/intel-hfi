// SPDX-License-Identifier: MIT
// Copyright (C) 2023 Akira Moroo

//! Intel Hardware Feedback Interface (HFI) utility

mod cpuid;
mod hfi;
mod msr;

use crate::{hfi::HfiTable, msr::Msr};
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
    #[arg(short, long)]
    enable: bool,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let info = hfi::HfiInfo::new(cli.cpu)?;
    println!("HFI Table:");
    println!("{}", info);

    match &cli.command {
        Commands::Hfi(args) => {
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
        Commands::Itd(args) => {
            if !info.has_itd() {
                println!("ITD capability is not supported");
                return Ok(());
            }

            println!("ITD Table:");
            let itd_info = hfi::ItdInfo::new(&info);
            if let Some(num_itd_classes) = itd_info.num_itd_classes() {
                println!("  Number of ITD classes: {}", num_itd_classes);
            } else {
                println!("Failed to read number of ITD classes");
                return Ok(());
            }
            println!("  ITD enabled: {}", itd_info.itd_enabled());
            println!("  HRESET enabled: {}", itd_info.hreset_enabled());
            if !itd_info.itd_enabled() && args.enable {
                println!("Enabling ITD...");
                let mut config = msr::HwFeedbackThreadConfig::read(cli.cpu)?;
                config.set_enable(true);
                config.write(cli.cpu, u64::from(config))?;
            }
            if itd_info.has_valid_class_id() {
                if let Some(class_id) = itd_info.class_id() {
                    println!("  ITD class ID: {}", class_id);
                } else {
                    println!("Failed to read ITD class ID");
                    return Ok(());
                }
            } else {
                println!("ITD class ID is not valid");
                return Ok(());
            }
        }
    }

    Ok(())
}
