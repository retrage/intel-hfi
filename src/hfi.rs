// SPDX-License-Identifier: MIT
// Copyright (C) 2023 Akira Moroo

use std::{
    fmt,
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
pub struct HfiInfo {
    cpu: usize,
    pub addr: usize,
    pub size: usize,
    index: usize,
}

impl HfiInfo {
    const PAGE_SIZE: usize = 4096;
    const PAGE_SHIFT: usize = Self::PAGE_SIZE.trailing_zeros() as usize;

    pub fn new(cpu: usize) -> io::Result<Self> {
        let cpuid = cpuid::ThermalCpuid::read(cpu)?;
        if !cpuid.has_hfi() {
            return Err(io::Error::new(io::ErrorKind::Other, "HFI is not supported"));
        }
        let ptr = msr::HwFeedbackPtr::read(cpu)?;
        let config = msr::HwFeedbackConfig::read(cpu)?;
        if !ptr.valid() || !config.enable() {
            return Err(io::Error::new(io::ErrorKind::Other, "HFI is not enabled"));
        }
        if !cpuid.has_perf_cap() || !cpuid.has_ee_cap() {
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

impl fmt::Display for HfiInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HFI Table: [{:#x}-{:#x}]",
            self.addr,
            self.addr + self.size - 1
        )
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

    pub fn read(&mut self, info: &HfiInfo) -> io::Result<()> {
        self.global_header.read(info)?;
        for cpu in 0..Self::NUM_CPUS {
            let info = HfiInfo::new(cpu)?;
            self.entries[cpu].read(&info)?;
        }
        Ok(())
    }
}

impl<const NUM_CPUS: usize> fmt::Display for HfiTable<NUM_CPUS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.global_header)?;
        for cpu in 0..Self::NUM_CPUS {
            writeln!(f, "CPU #{}: {}", cpu, self.entries[cpu])?;
        }
        Ok(())
    }
}

#[bitfield(u8)]
#[derive(Default)]
struct PerfCap {
    changed: bool,
    request_idle: bool,
    #[bits(6)]
    _reserved: u8,
}

impl fmt::Display for PerfCap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Changed: {}, RequestIdle: {}",
            self.changed(),
            self.request_idle()
        )
    }
}

#[bitfield(u8)]
#[derive(Default)]
struct EECap {
    changed: bool,
    request_idle: bool,
    #[bits(6)]
    _reserved: u8,
}

impl fmt::Display for EECap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Changed: {}, RequestIdle: {}",
            self.changed(),
            self.request_idle()
        )
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C, packed)]
struct HfiGlobalHeader {
    timestamp: u64,
    perf_cap: PerfCap,
    ee_cap: EECap,
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

impl fmt::Display for HfiGlobalHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let timestamp = self.timestamp;
        writeln!(f, "Timestamp: {}", timestamp)?;
        writeln!(f, "PerfCap: {}", self.perf_cap)?;
        writeln!(f, "EECap: {}", self.ee_cap)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C, packed)]
struct HfiEntry {
    perf_cap: u8,
    ee_cap: u8,
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

impl fmt::Display for HfiEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PerfCap: {}, EECap: {}", self.perf_cap, self.ee_cap)
    }
}
