#![allow(warnings)]
#![allow(clippy::all)]
#![allow(unknown_lints)]

// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// lib.rs — Protocols crate — quantum communication and cryptography
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 15-12-2022
// All rights reserved.
// ---------------------------------------------------------------------------

pub mod qcr;
pub mod qlc;
pub mod t_hqc;
pub mod zpe_extraction;

pub use qcr::*;
pub use qlc::*;
pub use t_hqc::*;
pub use zpe_extraction::*;
