// SPDX-License-Identifier: MIT
// Copyright (C) 2023 Akira Moroo

use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
};

use bitfield_struct::bitfield;

pub trait Cpuid<const EAX: u32, const ECX: u32> {
    const EAX: u32 = EAX;
    const ECX: u32 = ECX;

    fn read(cpu: usize) -> io::Result<Self>
    where
        Self: Sized + From<[u32; 4]>,
    {
        let mut fd = File::open(format!("/dev/cpu/{}/cpuid", cpu))?;
        let mut buf = [0u8; 16];
        let pos = ((Self::ECX as u64) << 32) | (Self::EAX as u64);
        fd.seek(SeekFrom::Start(pos))?;
        fd.read_exact(buf.as_mut())?;
        let b1 = [buf[0], buf[1], buf[2], buf[3]];
        let b2 = [buf[4], buf[5], buf[6], buf[7]];
        let b3 = [buf[8], buf[9], buf[10], buf[11]];
        let b4 = [buf[12], buf[13], buf[14], buf[15]];
        Ok(Self::from([
            u32::from_le_bytes(b1),
            u32::from_le_bytes(b2),
            u32::from_le_bytes(b3),
            u32::from_le_bytes(b4),
        ]))
    }
}

#[bitfield(u32)]
struct ThermalCpuidEax {
    #[bits(19)]
    _reserved: u32,
    hw_feedback: bool,
    #[bits(3)]
    _reserved: u32,
    itd: bool,
    #[bits(8)]
    _reserved: u32,
}

#[bitfield(u32)]
struct ThermalCpuidEbx {
    #[bits(32)]
    _reserved: u32,
}

#[bitfield(u32)]
struct ThermalCpuidEcx {
    #[bits(8)]
    _reserved: u32,
    #[bits(8)]
    num_itd_class: u32,
    #[bits(16)]
    _reserved: u32,
}

#[bitfield(u32)]
struct ThermalCpuidEdx {
    perf_cap: bool,
    energy_efficiency_cap: bool,
    #[bits(6)]
    _reserved: u32,
    #[bits(4)]
    hw_feedback_size: u32,
    #[bits(4)]
    _reserved: u32,
    #[bits(16)]
    hw_feedback_row_index: u32,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ThermalCpuid {
    eax: ThermalCpuidEax,
    ebx: ThermalCpuidEbx,
    ecx: ThermalCpuidEcx,
    edx: ThermalCpuidEdx,
}

impl From<[u32; 4]> for ThermalCpuid {
    fn from(value: [u32; 4]) -> Self {
        let eax = ThermalCpuidEax::from(value[0]);
        let ebx = ThermalCpuidEbx::from(value[1]);
        let ecx = ThermalCpuidEcx::from(value[2]);
        let edx = ThermalCpuidEdx::from(value[3]);
        Self { eax, ebx, ecx, edx }
    }
}

impl Cpuid<0x06, 0x0> for ThermalCpuid {}

impl ThermalCpuid {
    pub fn has_hw_feedback(&self) -> bool {
        self.eax.hw_feedback()
    }
    #[allow(dead_code)]
    pub fn has_itd(&self) -> bool {
        self.eax.itd()
    }
    #[allow(dead_code)]
    pub fn num_itd_class(&self) -> u8 {
        self.ecx.num_itd_class() as u8
    }
    pub fn has_perf_cap(&self) -> bool {
        self.edx.perf_cap()
    }
    pub fn has_energy_efficiency_cap(&self) -> bool {
        self.edx.energy_efficiency_cap()
    }
    pub fn hw_feedback_size(&self) -> usize {
        self.edx.hw_feedback_size() as usize + 1
    }
    pub fn hw_feedback_row_index(&self) -> usize {
        self.edx.hw_feedback_row_index() as usize
    }
}
