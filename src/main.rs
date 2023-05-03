// SPDX-License-Identifier: MIT
// Copyright (C) 2023 Akira Moroo
// Intel Thread Director (ITD) utility

mod cpuid;

use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
};

use crate::cpuid::Cpuid;
use bitfield_struct::bitfield;

const IA32_HW_FEEDBACK_PTR: u32 = 0x17D0;
const IA32_HW_FEEDBACK_CONFIG: u32 = 0x17D1;
const IA32_THREAD_FEEDBACK_CHAR: u32 = 0x17D2;
const IA32_HW_FEEDBACK_THREAD_CONFIG: u32 = 0x17D4;
const IA32_HRESET_ENABLE: u32 = 0x17DA;

trait Msr<const ADDR: u32> {
    const ADDR: u32 = ADDR;

    fn read(cpu: usize) -> io::Result<Self>
    where
        Self: Sized + From<u64>,
    {
        let mut fd = File::open(format!("/dev/cpu/{}/msr", cpu))?;
        let mut buf = [0u8; 8];
        fd.seek(SeekFrom::Start(Self::ADDR as u64))?;
        fd.read_exact(&mut buf)?;
        Ok(Self::from(u64::from_le_bytes(buf)))
    }

    fn write(self, cpu: usize, value: u64) -> io::Result<()>
    where
        Self: Into<u64>,
    {
        let mut fd = File::open(format!("/dev/cpu/{}/msr", cpu))?;
        let buf = value.to_le_bytes();
        fd.seek(SeekFrom::Start(Self::ADDR as u64))?;
        fd.write_all(&buf)?;
        Ok(())
    }
}

#[bitfield(u64)]
struct HwFeedbackPtr {
    valid: bool,
    #[bits(11)]
    _reserved: u64,
    #[bits(52)]
    addr: u64,
}
impl Msr<IA32_HW_FEEDBACK_PTR> for HwFeedbackPtr {}

#[bitfield(u64)]
struct HwFeedbackConfig {
    enable: bool,
    #[bits(63)]
    _reserved: u64,
}
impl Msr<IA32_HW_FEEDBACK_CONFIG> for HwFeedbackConfig {}

#[bitfield(u64)]
struct ThreadFeedbackChar {
    #[bits(8)]
    class_id: u64,
    #[bits(55)]
    _reserved: u64,
    valid: bool,
}
impl Msr<IA32_THREAD_FEEDBACK_CHAR> for ThreadFeedbackChar {}

#[bitfield(u64)]
struct HwFeedbackThreadConfig {
    enable: bool,
    #[bits(63)]
    _reserved: u64,
}
impl Msr<IA32_HW_FEEDBACK_THREAD_CONFIG> for HwFeedbackThreadConfig {}

#[bitfield(u64)]
struct HresetEnable {
    enable: bool,
    #[bits(63)]
    _reserved: u64,
}
impl Msr<IA32_HRESET_ENABLE> for HresetEnable {}

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
        let ptr = HwFeedbackPtr::read(cpu)?;
        let config = HwFeedbackConfig::read(cpu)?;
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
    let ptr = HwFeedbackPtr::read(cpu).unwrap();
    println!("HwFeedbackPtr: {:#x?}", ptr);
    let config = HwFeedbackConfig::read(cpu).unwrap();
    println!("HwFeedbackConfig: {:#x?}", config);
    let thread_config = HwFeedbackThreadConfig::read(cpu).unwrap();
    println!("HwFeedbackThreadConfig: {:#x?}", thread_config);
    let hreset_enable = HresetEnable::read(cpu).unwrap();
    println!("HresetEnable: {:#x?}", hreset_enable);
    let thread_char = ThreadFeedbackChar::read(cpu).unwrap();
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
