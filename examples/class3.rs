// SPDX-License-Identifier: 0BSD
// Copyright (C) 2023 Akira Moroo

//! Intel Thread Director (ITD) Class 3 example
//! Class 3: Pause (spin-wait) dominated code.
//!
//! Intel 64 and IA-32 Architectures Optimization Reference Manual
//! Example 2-4. Class 3 Pseudo-code Snippet

use std::arch::asm;

fn main() {
    loop {
        unsafe {
            asm!("pause");
        }
    }
}
