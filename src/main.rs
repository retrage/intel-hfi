// SPDX-License-Identifier: MIT
// Copyright (C) 2023 Akira Moroo

use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
};

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

fn main() {
    let n_cpus = 1;

    for cpu in 0..n_cpus {
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
}
