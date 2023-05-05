// SPDX-License-Identifier: 0BSD
// Copyright (C) 2023 Akira Moroo

//! Intel Thread Director (ITD) Class 0 example
//! Class 0: Non-vectorizable integer or floating-point code.
//!
//! Intel 64 and IA-32 Architectures Optimization Reference Manual
//! Example 2-1. Class 0 Pseudo-code Snippet

use std::arch::asm;

fn main() {
    loop {
        unsafe {
            asm!("xor {x}, {x}",
                    "add {x}, 5",
                    "inc {x}",
                    x = out(reg) _,
            );
        }
    }
}
