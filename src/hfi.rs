// SPDX-License-Identifier: MIT
// Copyright (C) 2023 Akira Moroo

use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
};

use bitfield_struct::bitfield;

use crate::{
    cpuid::{self, Cpuid},
    msr::{self, Msr},
};

#[allow(dead_code)]
#[derive(Debug)]
struct HfiInfo {
    cpu: usize,
    addr: usize,
    size: usize,
    index: usize,
}

impl HfiInfo {
    const PAGE_SIZE: usize = 4096;
    const PAGE_SHIFT: usize = Self::PAGE_SIZE.trailing_zeros() as usize;

    fn new(cpu: usize) -> io::Result<Self> {
        let cpuid = cpuid::ThermalCpuid::read(cpu)?;
        if !cpuid.has_hfi() {
            return Err(io::Error::new(io::ErrorKind::Other, "HFI is not supported"));
        }
        let ptr = msr::HwFeedbackPtr::read(cpu)?;
        let config = msr::HwFeedbackConfig::read(cpu)?;
        if !ptr.valid() || !config.enable() {
            return Err(io::Error::new(io::ErrorKind::Other, "HFI is not enabled"));
        }
        if !cpuid.has_perf_cap() || !cpuid.has_energy_efficiency_cap() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "HFI capability is not supported",
            ));
        }
        Ok(Self {
            cpu,
            addr: (ptr.addr() as usize) << Self::PAGE_SHIFT,
            size: Self::PAGE_SIZE * cpuid.hfi_size(),
            index: cpuid.hfi_row_index(),
        })
    }
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct HfiTable<const NUM_CPUS: usize> {
    global_header: HfiGlobalHeader,
    entries: [HfiEntry; NUM_CPUS],
}

impl<const NUM_CPUS: usize> HfiTable<NUM_CPUS> {
    const NUM_CPUS: usize = NUM_CPUS;

    pub fn new() -> Self {
        Self {
            global_header: HfiGlobalHeader::default(),
            entries: [HfiEntry::default(); NUM_CPUS],
        }
    }

    pub fn read(&mut self) -> io::Result<()> {
        let info = HfiInfo::new(0)?;
        self.global_header.read(&info)?;
        for cpu in 0..Self::NUM_CPUS {
            let info = HfiInfo::new(cpu)?;
            self.entries[cpu].read(&info)?;
        }
        Ok(())
    }
}

#[bitfield(u8)]
#[derive(Default)]
struct PerfCapFlags {
    changed: bool,
    request_idle: bool,
    #[bits(6)]
    _reserved: u8,
}

#[bitfield(u8)]
#[derive(Default)]
struct EnergyEfficiencyCapChanged {
    changed: bool,
    request_idle: bool,
    #[bits(6)]
    _reserved: u8,
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C, packed)]
struct HfiGlobalHeader {
    timestamp: u64,
    perf_cap_flags: PerfCapFlags,
    energy_efficiency_cap_changed: EnergyEfficiencyCapChanged,
    _reserved: [u8; 6],
}

impl HfiGlobalHeader {
    fn read(&mut self, info: &HfiInfo) -> io::Result<()> {
        let mut buf = [0u8; std::mem::size_of::<Self>()];
        let mut fd = File::open("/dev/mem")?;
        fd.seek(SeekFrom::Start(info.addr as u64))?;
        fd.read_exact(&mut buf)?;
        let header = unsafe { std::mem::transmute::<_, Self>(buf) };
        *self = header;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C, packed)]
struct HfiEntry {
    perf_cap: u8,
    energy_efficiency_cap: u8,
    _reserved: [u8; 6],
}

impl HfiEntry {
    fn read(&mut self, info: &HfiInfo) -> io::Result<()> {
        let mut buf = [0u8; std::mem::size_of::<Self>()];
        let mut fd = File::open("/dev/mem")?;
        fd.seek(SeekFrom::Start(
            info.addr as u64
                + std::mem::size_of::<HfiGlobalHeader>() as u64
                + std::mem::size_of::<Self>() as u64 * info.cpu as u64,
        ))?;
        fd.read_exact(&mut buf)?;
        let entry = unsafe { std::mem::transmute::<_, Self>(buf) };
        *self = entry;
        Ok(())
    }
}
