// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// lib.rs — Kernel entry point — PyO3 module registration and public API
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 21-05-2023
// All rights reserved.
// ---------------------------------------------------------------------------

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::wrap_pyfunction;

// Internal modules
pub mod efal;
pub mod emf;
pub mod tlm;
pub mod hio;
pub mod drivers;
pub mod math;

// PyO3 modules (bindings)
mod py_emf;
mod py_tlm;
mod py_hio;

// ============================================================================
// MAIN PYTHON MODULE
// ============================================================================

/// LightQOS Rust Kernel
///
/// PyO3 bindings for Python access to the high-performance kernel.
#[pymodule]
fn lightqos(_py: Python, m: &PyModule) -> PyResult<()> {
    // Module information
    m.add("__version__", "0.1.0")?;
    m.add("__author__", "LightQOS Team")?;
    
    // Submodules
    m.add_class::<py_emf::PyEMFManager>()?;
    m.add_class::<py_emf::PyEntangledPair>()?;
    
    m.add_class::<py_tlm::PyContractManager>()?;
    m.add_class::<py_tlm::PyTemporalContract>()?;
    m.add_class::<py_tlm::PyHarmonicScheduler>()?;
    
    m.add_class::<py_hio::PyShadowCollector>()?;
    m.add_class::<py_hio::PyQuantumShadow>()?;
    
    // Utility functions
    m.add_function(wrap_pyfunction!(get_kernel_info, m)?)?;
    m.add_function(wrap_pyfunction!(benchmark_emf, m)?)?;
    
    Ok(())
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Gets information about the Rust kernel
#[pyfunction]
fn get_kernel_info(py: Python) -> PyResult<PyObject> {
    let dict = PyDict::new(py);
    
    dict.set_item("version", "0.1.0")?;
    dict.set_item("rust_version", "1.75+")?;
    dict.set_item("pyo3_version", "0.20")?;
    dict.set_item("build_type", if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    })?;
    
    dict.set_item("modules", vec![
        "emf", "tlm", "hio", "drivers", "math"
    ])?;
    
    Ok(dict.into())
}

/// EMF benchmark (for Python vs Rust comparison)
#[pyfunction]
fn benchmark_emf(num_iterations: usize) -> PyResult<f64> {
    use std::time::Instant;
    
    let start = Instant::now();
    
    // Simulate EMF operations
    for _ in 0..num_iterations {
        let _ = std::hint::black_box(calculate_fidelity(0.95, 0.02));
    }
    
    let elapsed = start.elapsed();
    Ok(elapsed.as_secs_f64())
}

fn calculate_fidelity(initial: f64, decay: f64) -> f64 {
    initial * (1.0 - decay)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculate_fidelity() {
        let result = calculate_fidelity(0.95, 0.02);
        assert!((result - 0.931).abs() < 0.001);
    }
}
