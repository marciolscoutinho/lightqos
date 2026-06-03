// -----------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// hio_tests.rs — HIO smoke tests
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// All rights reserved.
// -----------------------------------------------------------------------------

#![allow(warnings)]

#[test]
fn hio_module_is_available() {
    let _version = lightqos_kernel::VERSION;
    assert!(!_version.is_empty());
}
