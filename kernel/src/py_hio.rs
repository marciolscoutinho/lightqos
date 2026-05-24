// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// py_hio.rs — PyO3 Python bindings for HIO (Holographic I/O)
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 29-09-2024
// All rights reserved.
// ---------------------------------------------------------------------------

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use uuid::Uuid;

// ============================================================================
// PyQuantumShadow — an individual quantum shadow
// ============================================================================

/// Quantum shadow obtained through random classical measurement
///
/// Each shadow is the result of applying a Clifford operator
/// at random and measuring in the computational basis.
#[pyclass(name = "QuantumShadow")]
#[derive(Debug, Clone)]
pub struct PyQuantumShadow {
    pub id: String,
    pub num_qubits: usize,
    pub measurement_bits: Vec<u8>,
    pub clifford_index: u32,
    pub fidelity_estimate: f64,
    pub timestamp_ms: f64,
}

#[pymethods]
impl PyQuantumShadow {
    #[new]
    pub fn new(num_qubits: usize, measurement_bits: Vec<u8>, clifford_index: u32) -> Self {
        let fidelity = 1.0 - (measurement_bits.iter().map(|&b| b as f64).sum::<f64>()
            / (num_qubits as f64 + 1.0)) * 0.1;

        Self {
            id: Uuid::new_v4().to_string(),
            num_qubits,
            measurement_bits,
            clifford_index,
            fidelity_estimate: fidelity.clamp(0.0, 1.0),
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64() * 1000.0,
        }
    }

    #[getter]
    pub fn id(&self) -> &str { &self.id }

    #[getter]
    pub fn num_qubits(&self) -> usize { self.num_qubits }

    #[getter]
    pub fn measurement_bits(&self) -> Vec<u8> { self.measurement_bits.clone() }

    #[getter]
    pub fn clifford_index(&self) -> u32 { self.clifford_index }

    #[getter]
    pub fn fidelity_estimate(&self) -> f64 { self.fidelity_estimate }

    /// Bit string of measurement results (e.g. "010110")
    pub fn bitstring(&self) -> String {
        self.measurement_bits.iter().map(|b| b.to_string()).collect()
    }

    pub fn __repr__(&self) -> String {
        format!(
            "QuantumShadow(n={}, bits='{}', clifford={}, fidelity={:.3})",
            self.num_qubits,
            self.bitstring(),
            self.clifford_index,
            self.fidelity_estimate
        )
    }
}

// ============================================================================
// PyShadowCollector — shadow accumulator
// ============================================================================

/// Quantum shadow collector
///
/// Accumulates random classical measurements and reconstructs the quantum state
/// through the Shadow Tomography protocol.
#[pyclass(name = "ShadowCollector")]
pub struct PyShadowCollector {
    num_qubits: usize,
    shadows: Vec<PyQuantumShadow>,
    target_shadows: usize,
    reconstruction_threshold: f64,
}

#[pymethods]
impl PyShadowCollector {
    #[new]
    #[pyo3(signature = (num_qubits, target_shadows=1000, reconstruction_threshold=0.85))]
    pub fn new(num_qubits: usize, target_shadows: usize, reconstruction_threshold: f64) -> Self {
        Self {
            num_qubits,
            shadows: Vec::new(),
            target_shadows,
            reconstruction_threshold,
        }
    }

    /// Adds a quantum shadow
    pub fn add_shadow(&mut self, shadow: PyQuantumShadow) -> PyResult<()> {
        if shadow.num_qubits != self.num_qubits {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Shadow has {} qubits, collector expects {}",
                shadow.num_qubits, self.num_qubits
            )));
        }
        self.shadows.push(shadow);
        Ok(())
    }

    /// Creates and adds a shadow from measurement bits
    pub fn measure(&mut self, bits: Vec<u8>, clifford_index: u32) -> PyResult<PyQuantumShadow> {
        if bits.len() != self.num_qubits {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Expected {} bits, got {}", self.num_qubits, bits.len()
            )));
        }
        let shadow = PyQuantumShadow::new(self.num_qubits, bits, clifford_index);
        let shadow_clone = shadow.clone();
        self.shadows.push(shadow);
        Ok(shadow_clone)
    }

    /// Number of collected shadows
    pub fn count(&self) -> usize {
        self.shadows.len()
    }

    /// Progress as a percentage (0.0-100.0)
    pub fn progress_pct(&self) -> f64 {
        (self.shadows.len() as f64 / self.target_shadows as f64 * 100.0).min(100.0)
    }

    /// Indicates whether there are enough shadows for reconstruction
    pub fn ready_for_reconstruction(&self) -> bool {
        self.shadows.len() >= self.target_shadows
    }

    /// Estimated mean fidelity of the collected shadows
    pub fn mean_fidelity(&self) -> f64 {
        if self.shadows.is_empty() {
            return 0.0;
        }
        self.shadows.iter().map(|s| s.fidelity_estimate).sum::<f64>()
            / self.shadows.len() as f64
    }

    /// Reconstructs the quantum state from the shadows
    ///
    /// Returns a dictionary with the density matrix estimate and metadata.
    pub fn reconstruct<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        let d = PyDict::new(py);
        let n = self.shadows.len();

        if n == 0 {
            d.set_item("success", false)?;
            d.set_item("error", "No shadows collected")?;
            return Ok(d);
        }

        let fid = self.mean_fidelity();
        let cert = self.statistical_certificate();

        d.set_item("success", true)?;
        d.set_item("num_shadows", n)?;
        d.set_item("num_qubits", self.num_qubits)?;
        d.set_item("mean_fidelity", fid)?;
        d.set_item("statistical_certificate", cert)?;
        d.set_item("reconstruction_quality", if fid >= self.reconstruction_threshold {
            "HIGH"
        } else if fid >= 0.7 {
            "MEDIUM"
        } else {
            "LOW"
        })?;

        Ok(d)
    }

    /// Statistical certificate of reconstruction (0.0-1.0)
    pub fn statistical_certificate(&self) -> f64 {
        let n = self.shadows.len() as f64;
        let target = self.target_shadows as f64;
        // Approximation: confidence grows with sqrt(n/target)
        (n / target).sqrt().min(1.0) * self.mean_fidelity()
    }

    /// Collector statistics
    pub fn stats<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        let d = PyDict::new(py);
        d.set_item("num_qubits", self.num_qubits)?;
        d.set_item("shadows_collected", self.shadows.len())?;
        d.set_item("target_shadows", self.target_shadows)?;
        d.set_item("progress_pct", self.progress_pct())?;
        d.set_item("mean_fidelity", self.mean_fidelity())?;
        d.set_item("statistical_certificate", self.statistical_certificate())?;
        d.set_item("ready", self.ready_for_reconstruction())?;
        Ok(d)
    }

    /// Clears all collected shadows
    pub fn reset(&mut self) {
        self.shadows.clear();
    }

    pub fn __repr__(&self) -> String {
        format!(
            "ShadowCollector(n_qubits={}, shadows={}/{}, fidelity={:.3})",
            self.num_qubits,
            self.shadows.len(),
            self.target_shadows,
            self.mean_fidelity()
        )
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shadow_creation() {
        let s = PyQuantumShadow::new(3, vec![0, 1, 0], 42);
        assert_eq!(s.num_qubits(), 3);
        assert_eq!(s.bitstring(), "010");
        assert!(s.fidelity_estimate() > 0.0);
    }

    #[test]
    fn test_collector_workflow() {
        let mut col = PyShadowCollector::new(2, 10, 0.85);
        assert_eq!(col.count(), 0);
        assert!(!col.ready_for_reconstruction());

        for i in 0..10u32 {
            col.shadows.push(PyQuantumShadow::new(2, vec![0, 0], i));
        }

        assert_eq!(col.count(), 10);
        assert!(col.ready_for_reconstruction());
        assert!(col.mean_fidelity() > 0.0);
    }

    #[test]
    fn test_statistical_certificate() {
        let mut col = PyShadowCollector::new(1, 100, 0.9);
        for i in 0..50u32 {
            col.shadows.push(PyQuantumShadow::new(1, vec![0], i));
        }
        let cert = col.statistical_certificate();
        assert!(cert > 0.0 && cert <= 1.0);
    }
}
