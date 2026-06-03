// -----------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// tlm_tests.rs — TLM smoke tests
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// All rights reserved.
// -----------------------------------------------------------------------------

#![allow(warnings)]

#[test]
fn tlm_module_is_available() {
    let _version = lightqos_kernel::VERSION;
    assert!(!_version.is_empty());
}
