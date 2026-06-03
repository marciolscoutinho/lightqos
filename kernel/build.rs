// -----------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// build.rs — PyO3 extension-module linker configuration
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// All rights reserved.
// -----------------------------------------------------------------------------

fn main() {
    pyo3_build_config::add_extension_module_link_args();
}
