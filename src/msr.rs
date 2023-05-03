// SPDX-License-Identifier: MIT
// Copyright (C) 2023 Akira Moroo

use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
};

use bitfield_struct::bitfield;

pub trait Msr<const ADDR: u32> {
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

const IA32_HW_FEEDBACK_PTR: u32 = 0x17D0;
const IA32_HW_FEEDBACK_CONFIG: u32 = 0x17D1;
const IA32_THREAD_FEEDBACK_CHAR: u32 = 0x17D2;
const IA32_HW_FEEDBACK_THREAD_CONFIG: u32 = 0x17D4;
const IA32_HRESET_ENABLE: u32 = 0x17DA;

#[bitfield(u64)]
pub struct HwFeedbackPtr {
    pub valid: bool,
    #[bits(11)]
    _reserved: u64,
    #[bits(52)]
    pub addr: u64,
}
impl Msr<IA32_HW_FEEDBACK_PTR> for HwFeedbackPtr {}

#[bitfield(u64)]
pub struct HwFeedbackConfig {
    pub enable: bool,
    #[bits(63)]
    _reserved: u64,
}
impl Msr<IA32_HW_FEEDBACK_CONFIG> for HwFeedbackConfig {}

#[bitfield(u64)]
pub struct ThreadFeedbackChar {
    #[bits(8)]
    pub class_id: u64,
    #[bits(55)]
    _reserved: u64,
    pub valid: bool,
}
impl Msr<IA32_THREAD_FEEDBACK_CHAR> for ThreadFeedbackChar {}

#[bitfield(u64)]
pub struct HwFeedbackThreadConfig {
    pub enable: bool,
    #[bits(63)]
    _reserved: u64,
}
impl Msr<IA32_HW_FEEDBACK_THREAD_CONFIG> for HwFeedbackThreadConfig {}

#[bitfield(u64)]
pub struct HresetEnable {
    pub enable: bool,
    #[bits(63)]
    _reserved: u64,
}
impl Msr<IA32_HRESET_ENABLE> for HresetEnable {}
