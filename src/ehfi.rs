// SPDX-License-Identifier: MIT
// Copyright (C) 2023 Akira Moroo

use std::{
    fmt,
    fs::File,
    io::{self, Read, Seek, SeekFrom},
};

use bitfield_struct::bitfield;

use crate::hfi::HfiInfo;

const NUM_CAPS: usize = 2;
const NUM_CLASSES: usize = 4;

#[derive(Debug)]
#[repr(C)]
pub struct EhfiTable<const NUM_CPUS: usize> {
    pub header: EhfiHeader,
    pub entries: [EhfiEntry; NUM_CPUS],
}

impl<const NUM_CPUS: usize> EhfiTable<NUM_CPUS> {
    const NUM_CPUS: usize = NUM_CPUS;

    pub fn new() -> Self {
        Self {
            header: EhfiHeader::default(),
            entries: [EhfiEntry::default(); NUM_CPUS],
        }
    }

    pub fn read(&mut self, info: &HfiInfo) -> io::Result<()> {
        self.header.read(info)?;
        for cpu in 0..Self::NUM_CPUS {
            let info = HfiInfo::new(cpu)?;
            self.entries[cpu].read(&info)?;
        }
        Ok(())
    }
}

impl<const NUM_CPUS: usize> fmt::Display for EhfiTable<NUM_CPUS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.header)?;
        for cpu in 0..Self::NUM_CPUS {
            writeln!(f, "CPU #{}: {}", cpu, self.entries[cpu])?;
        }
        Ok(())
    }
}

#[bitfield(u8)]
struct CapFlags {
    changed: bool,
    request_idle: bool,
    #[bits(6)]
    _reserved: u8,
}

impl fmt::Display for CapFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "    Updated: {}", self.changed())?;
        write!(f, "    Idle Requested: {}", self.request_idle())
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct EhfiHeader {
    timestamp: u64,
    caps: [CapFlags; NUM_CAPS],
}

impl EhfiHeader {
    const SIZE: usize = std::mem::size_of::<Self>();

    fn read(&mut self, info: &HfiInfo) -> io::Result<()> {
        let mut buf = [0u8; Self::SIZE];
        let mut fd = File::open("/dev/mem")?;
        fd.seek(SeekFrom::Start(info.addr as u64))?;
        fd.read_exact(&mut buf)?;
        let header = unsafe { std::mem::transmute::<[u8; Self::SIZE], Self>(buf) };
        *self = header;
        Ok(())
    }
}

impl fmt::Display for EhfiHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let timestamp = self.timestamp;
        writeln!(f, "  Timestamp: {timestamp}")?;
        for cap in 0..NUM_CAPS {
            writeln!(f, "  Capability #{cap}:")?;
            write!(f, "{}", self.caps[cap])?;
            if cap < NUM_CAPS - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C, packed)]
pub struct EhfiCapEntry {
    cap: [u8; NUM_CAPS],
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C, packed)]
pub struct EhfiEntry {
    caps: [EhfiCapEntry; NUM_CLASSES],
}

impl EhfiEntry {
    const SIZE: usize = std::mem::size_of::<Self>();

    fn read(&mut self, info: &HfiInfo) -> io::Result<()> {
        let mut buf = [0u8; Self::SIZE];
        let mut fd = File::open("/dev/mem")?;
        fd.seek(SeekFrom::Start(
            info.addr as u64
                + std::mem::size_of::<EhfiHeader>() as u64
                + std::mem::size_of::<Self>() as u64 * info.cpu as u64,
        ))?;
        fd.read_exact(&mut buf)?;
        let entry = unsafe { std::mem::transmute::<[u8; Self::SIZE], Self>(buf) };
        *self = entry;
        Ok(())
    }
}

impl fmt::Display for EhfiEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, class) in self.caps.iter().enumerate() {
            writeln!(f, "  Class #{index}:")?;
            for (index, cap) in class.cap.iter().enumerate() {
                write!(f, "    Capability #{index}: {cap}")?;
                if index < NUM_CLASSES - 1 {
                    writeln!(f)?;
                }
            }
        }
        Ok(())
    }
}
