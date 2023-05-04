// SPDX-License-Identifier: MIT
// Copyright (C) 2023 Akira Moroo
// Intel Hardware Feedback Interface (HFI) utility

mod cpuid;
mod hfi;
mod msr;

use std::io;

use crate::hfi::HfiTable;

fn main() -> io::Result<()> {
    let mut table = HfiTable::<32>::new();
    table.read()?;

    println!("{:#x?}", table);

    Ok(())
}
