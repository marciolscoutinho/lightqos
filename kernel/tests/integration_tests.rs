// -----------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// integration_tests.rs — Kernel integration smoke tests
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// All rights reserved.
// -----------------------------------------------------------------------------

#![allow(warnings)]

#[test]
fn kernel_version_is_exported() {
    assert!(!lightqos_kernel::VERSION.is_empty());
}

#[test]
fn kernel_runtime_initializes() {
    let runtime = lightqos_kernel::init();

    assert_eq!(runtime.version, lightqos_kernel::VERSION);
}
