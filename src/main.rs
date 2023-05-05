// SPDX-License-Identifier: MIT
// Copyright (C) 2023 Akira Moroo

//! Intel Hardware Feedback Interface (HFI) utility

mod cpuid;
mod hfi;
mod msr;

use std::io;

use crate::hfi::HfiTable;

fn main() -> io::Result<()> {
    let cpu = 1;
    let info = hfi::HfiInfo::new(cpu)?;
    println!("{}", info);

    let mut table = HfiTable::<32>::new();
    table.read(&info)?;

    print!("{}", table);

    Ok(())
}
