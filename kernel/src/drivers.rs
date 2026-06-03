// -----------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// drivers.rs — Kernel driver module bridge
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// All rights reserved.
// -----------------------------------------------------------------------------

//! Kernel-side driver bridge for LightQOS.
//!
//! This module is intentionally minimal for now. The full hardware driver
//! implementations live in the workspace `drivers` crate. This kernel module
//! provides a stable internal namespace so the kernel crate can compile while
//! driver integration evolves.

/// Generic quantum driver status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverStatus {
    Uninitialized,
    Ready,
    Busy,
    Error,
}

/// Minimal driver descriptor used by the kernel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverDescriptor {
    pub name: String,
    pub backend: String,
    pub status: DriverStatus,
}

impl DriverDescriptor {
    /// Creates a new driver descriptor.
    pub fn new(name: impl Into<String>, backend: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            backend: backend.into(),
            status: DriverStatus::Uninitialized,
        }
    }

    /// Marks the driver as ready.
    pub fn mark_ready(&mut self) {
        self.status = DriverStatus::Ready;
    }

    /// Marks the driver as busy.
    pub fn mark_busy(&mut self) {
        self.status = DriverStatus::Busy;
    }

    /// Marks the driver as failed.
    pub fn mark_error(&mut self) {
        self.status = DriverStatus::Error;
    }
}
