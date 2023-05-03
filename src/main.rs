// SPDX-License-Identifier: MIT
// Copyright (C) 2023 Akira Moroo
// Intel Thread Director (ITD) utility

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

trait Cpuid<const EAX: u32, const ECX: u32> {
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
struct ThermalCpuid {
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
    fn has_hw_feedback(&self) -> bool {
        self.eax.hw_feedback()
    }
    fn has_itd(&self) -> bool {
        self.eax.itd()
    }
    fn num_itd_class(&self) -> u8 {
        self.ecx.num_itd_class() as u8
    }
    fn has_perf_cap(&self) -> bool {
        self.edx.perf_cap()
    }
    fn has_energy_efficiency_cap(&self) -> bool {
        self.edx.energy_efficiency_cap()
    }
    fn hw_feedback_size(&self) -> usize {
        self.edx.hw_feedback_size() as usize + 1
    }
    fn hw_feedback_row_index(&self) -> usize {
        self.edx.hw_feedback_row_index() as usize
    }
}

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
        let cpuid = ThermalCpuid::read(cpu)?;
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

fn main() {
    let n_cpus = 1;

    for cpu in 0..n_cpus {
        let thermal_cpuid = ThermalCpuid::read(cpu).unwrap();
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
}
