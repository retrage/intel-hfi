// SPDX-License-Identifier: MIT
// Copyright (C) 2023 Akira Moroo

//! Intel Thread Director (ITD)

use std::fmt;

use crate::{
    cpuid::{self, Cpuid},
    hfi::HfiInfo,
    msr::{self, Msr},
};

#[allow(dead_code)]
pub struct ItdInfo {
    cpu: usize,
    addr: usize,
    size: usize,
}

impl ItdInfo {
    pub fn new(hfi_info: &HfiInfo) -> Self {
        assert!(hfi_info.has_itd());
        Self {
            cpu: hfi_info.cpu,
            addr: hfi_info.addr,
            size: hfi_info.size,
        }
    }

    #[allow(dead_code)]
    pub fn num_itd_classes(&self) -> Option<usize> {
        match cpuid::ThermalCpuid::read(self.cpu) {
            Ok(cpuid) => Some(cpuid.num_itd_classes() as usize),
            Err(_) => None,
        }
    }

    pub fn itd_enabled(&self) -> bool {
        match msr::HwFeedbackThreadConfig::read(self.cpu) {
            Ok(config) => config.enable(),
            Err(_) => false,
        }
    }

    pub fn hreset_enabled(&self) -> bool {
        match msr::HresetEnable::read(self.cpu) {
            Ok(config) => config.enable(),
            Err(_) => false,
        }
    }

    #[allow(dead_code)]
    pub fn has_valid_class_id(&self) -> bool {
        match msr::ThreadFeedbackChar::read(self.cpu) {
            Ok(char) => char.valid(),
            Err(_) => false,
        }
    }

    pub fn class_id(&self) -> Option<usize> {
        match msr::ThreadFeedbackChar::read(self.cpu) {
            Ok(char) => match char.valid() {
                true => Some(char.class_id() as usize),
                false => None,
            },
            Err(_) => None,
        }
    }
}

impl fmt::Display for ItdInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  ITD enabled: {}", self.itd_enabled())?;
        writeln!(f, "  HRESET enabled: {}", self.hreset_enabled())?;
        write!(f, "  ITD class ID: ")?;
        match self.class_id() {
            Some(class_id) => write!(f, "{class_id}"),
            None => write!(f, "None"),
        }
    }
}
