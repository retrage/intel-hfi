// SPDX-License-Identifier: 0BSD
// Copyright (C) 2023 Akira Moroo

//! Intel Thread Director (ITD) Class 1 example
//! Class 1: Integer or floating-point vectorized code, excluding Intel Deep Learning Boost (Intel DL Boost) code.
//!
//! Intel 64 and IA-32 Architectures Optimization Reference Manual
//! Example 2-2. Class 1 Pseudo-code Snippet

use std::arch::asm;

fn main() {
    loop {
        unsafe {
            let x1 = 0;
            let x2 = 0;
            let x3 = 0;
            let x4 = 0;
            let x5 = 0;
            let x6 = 0;
            let x7 = 0;
            let x8 = 0;
            let x9 = 0;
            let x10 = 0;
            asm!(
                "vfmaddsub132ps {x0:y}, {x1:y}, {x2:y}",
                "vfmaddsub213ps {x0:y}, {x1:y}, {x3:y}",
                "vfmaddsub231ps {x0:y}, {x1:y}, {x4:y}",
                "vfmaddsub132ps {x0:y}, {x1:y}, {x5:y}",
                "vfmaddsub213ps {x0:y}, {x1:y}, {x6:y}",
                "vfmaddsub231ps {x0:y}, {x1:y}, {x7:y}",
                "vfmaddsub132ps {x0:y}, {x1:y}, {x8:y}",
                "vfmaddsub213ps {x0:y}, {x1:y}, {x9:y}",
                "vfmaddsub231ps {x0:y}, {x1:y}, {x10:y}",
                    x0 = out(ymm_reg) _,
                    x1 = in(ymm_reg) x1,
                    x2 = in(ymm_reg) x2,
                    x3 = in(ymm_reg) x3,
                    x4 = in(ymm_reg) x4,
                    x5 = in(ymm_reg) x5,
                    x6 = in(ymm_reg) x6,
                    x7 = in(ymm_reg) x7,
                    x8 = in(ymm_reg) x8,
                    x9 = in(ymm_reg) x9,
                    x10 = in(ymm_reg) x10,
            );
        }
    }
}
