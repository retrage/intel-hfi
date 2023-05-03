// SPDX-License-Identifier: MIT
// Copyright (C) 2023 Akira Moroo
// Intel Thread Director (ITD) utility

mod cpuid;
mod msr;

use std::io;

use crate::cpuid::Cpuid;
use crate::msr::Msr;

#[allow(dead_code)]
#[derive(Debug)]
struct HwFeedbackInfo {
    addr: usize,
    size: usize,
    index: usize,
}

impl HwFeedbackInfo {
    const PAGE_SIZE: usize = 4096;
    const PAGE_SHIFT: usize = Self::PAGE_SIZE.trailing_zeros() as usize;

    fn new(cpu: usize) -> io::Result<Self> {
        let cpuid = cpuid::ThermalCpuid::read(cpu)?;
        if !cpuid.has_hw_feedback() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "HwFeedback is not supported",
            ));
        }
        let ptr = msr::HwFeedbackPtr::read(cpu)?;
        let config = msr::HwFeedbackConfig::read(cpu)?;
        if !ptr.valid() || !config.enable() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "HwFeedback is not enabled",
            ));
        }
        Ok(Self {
            addr: (ptr.addr() as usize) << Self::PAGE_SHIFT,
            size: Self::PAGE_SIZE * cpuid.hw_feedback_size(),
            index: cpuid.hw_feedback_row_index(),
        })
    }
}

#[allow(dead_code)]
fn dump_cpu_thermal_info(cpu: usize) {
    let thermal_cpuid = cpuid::ThermalCpuid::read(cpu).unwrap();
    println!("ThermalCpuid: {:#x?}", thermal_cpuid);
    let ptr = msr::HwFeedbackPtr::read(cpu).unwrap();
    println!("HwFeedbackPtr: {:#x?}", ptr);
    let config = msr::HwFeedbackConfig::read(cpu).unwrap();
    println!("HwFeedbackConfig: {:#x?}", config);
    let thread_config = msr::HwFeedbackThreadConfig::read(cpu).unwrap();
    println!("HwFeedbackThreadConfig: {:#x?}", thread_config);
    let hreset_enable = msr::HresetEnable::read(cpu).unwrap();
    println!("HresetEnable: {:#x?}", hreset_enable);
    let thread_char = msr::ThreadFeedbackChar::read(cpu).unwrap();
    println!("ThreadFeedbackChar: {:#x?}", thread_char);
}

#[allow(dead_code)]
fn print_cpu_hw_feedback_info(cpu: usize) {
    let thermal_cpuid = cpuid::ThermalCpuid::read(cpu).unwrap();
    print!("cpu #{}: ", cpu);
    if thermal_cpuid.has_hw_feedback() {
        print!(
            "hw feedback supported: perf cap={}, energy efficiency cap={}, size={}, row index={} ",
            thermal_cpuid.has_perf_cap(),
            thermal_cpuid.has_energy_efficiency_cap(),
            thermal_cpuid.hw_feedback_size(),
            thermal_cpuid.hw_feedback_row_index()
        );
        if thermal_cpuid.has_itd() {
            print!("itd: num_itd_class={}", thermal_cpuid.num_itd_class());
        }
    }
    println!();
}

fn main() -> io::Result<()> {
    let n_cpus = 32;

    for cpu in 0..n_cpus {
        let hw_feedback_info = HwFeedbackInfo::new(cpu)?;
        println!("cpu #{}: {:#x?}", cpu, hw_feedback_info);
    }
    Ok(())
}
