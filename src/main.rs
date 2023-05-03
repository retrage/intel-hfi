// SPDX-License-Identifier: MIT
// Copyright (C) 2023 Akira Moroo
// Intel Thread Director (ITD) utility

mod cpuid;
mod msr;

use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
};

use crate::cpuid::Cpuid;
use crate::msr::Msr;

#[allow(dead_code)]
#[derive(Debug)]
struct HwFeedbackInfo {
    cpu: usize,
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
        if !cpuid.has_perf_cap() || !cpuid.has_energy_efficiency_cap() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "HwFeedback capability is not supported",
            ));
        }
        Ok(Self {
            cpu,
            addr: (ptr.addr() as usize) << Self::PAGE_SHIFT,
            size: Self::PAGE_SIZE * cpuid.hw_feedback_size(),
            index: cpuid.hw_feedback_row_index(),
        })
    }
}

#[repr(C, packed)]
struct HwFeedbackInterfaceTable<const NUM_CPUS: usize> {
    global_header: HwFeedbackGlobalHeader,
    entries: [HwFeedbackInterfaceEntry; NUM_CPUS],
}

impl<const NUM_CPUS: usize> HwFeedbackInterfaceTable<NUM_CPUS> {
    const NUM_CPUS: usize = NUM_CPUS;

    fn new() -> Self {
        Self {
            global_header: HwFeedbackGlobalHeader::default(),
            entries: [HwFeedbackInterfaceEntry::default(); NUM_CPUS],
        }
    }

    fn read(&mut self) -> io::Result<()> {
        let info = HwFeedbackInfo::new(0)?;
        self.global_header.read(&info)?;
        for cpu in 0..Self::NUM_CPUS {
            let info = HwFeedbackInfo::new(cpu)?;
            self.entries[cpu].read(&info)?;
        }
        Ok(())
    }
}

#[derive(Default)]
#[repr(C, packed)]
struct HwFeedbackGlobalHeader {
    timestamp: u64,
    perf_cap_flags: u8,
    energy_efficiency_cap_changed: u8,
    _reserved: [u8; 6],
}

impl HwFeedbackGlobalHeader {
    fn read(&mut self, info: &HwFeedbackInfo) -> io::Result<()> {
        let mut buf = [0u8; std::mem::size_of::<Self>()];
        let mut fd = File::open("/dev/mem")?;
        fd.seek(SeekFrom::Start(info.addr as u64))?;
        fd.read_exact(&mut buf)?;
        let header = unsafe { std::mem::transmute::<_, Self>(buf) };
        *self = header;
        Ok(())
    }
}

#[derive(Copy, Clone, Default)]
#[repr(C, packed)]
struct HwFeedbackInterfaceEntry {
    perf_cap: u8,
    energy_efficiency_cap: u8,
    _reserved: [u8; 6],
}

impl HwFeedbackInterfaceEntry {
    fn read(&mut self, info: &HwFeedbackInfo) -> io::Result<()> {
        let mut buf = [0u8; std::mem::size_of::<Self>()];
        let mut fd = File::open("/dev/mem")?;
        fd.seek(SeekFrom::Start(
            info.addr as u64
                + std::mem::size_of::<HwFeedbackGlobalHeader>() as u64
                + std::mem::size_of::<Self>() as u64 * info.cpu as u64,
        ))?;
        fd.read_exact(&mut buf)?;
        let entry = unsafe { std::mem::transmute::<_, Self>(buf) };
        *self = entry;
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let mut table = HwFeedbackInterfaceTable::<1>::new();
    table.read()?;

    Ok(())
}
