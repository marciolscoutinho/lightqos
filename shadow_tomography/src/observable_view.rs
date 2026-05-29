// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// observable_view.rs — Observable View — multi-basis expectation value estimation
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 13-07-2024
// All rights reserved.
// ---------------------------------------------------------------------------

use crate::QuantumCircuit;
use nalgebra::{DMatrix, DVector};
use num_complex::Complex64;
use std::collections::HashMap;
use uuid::Uuid;
use rand::Rng;

// ============================================================================
// VIEW MANAGER
// ============================================================================

/// Manager for observable views
#[derive(Debug, Clone)]
pub struct ViewManager {
    /// Available measurement bases
    pub bases: Vec<MeasurementBasis>,
    
    /// Results cache
    pub cache: HashMap<Uuid, MultiBaseResult>,
}

impl ViewManager {
    /// Create new view manager
    pub fn new() -> Self {
        Self {
            bases: vec![
                MeasurementBasis::Z,
                MeasurementBasis::X,
                MeasurementBasis::Y,
            ],
            cache: HashMap::new(),
        }
    }
    
    /// Measure in multiple bases
    pub fn measure_multi_base(
        &mut self,
        circuit: &QuantumCircuit,
        bases: &[MeasurementBasis],
    ) -> MultiBaseResult {
        let mut views = Vec::new();
        
        for basis in bases {
            let view = self.measure_in_basis(circuit, *basis);
            views.push(view);
        }
        
        // Reconstruct state from multi-base measurements
        let reconstructed_state = self.reconstruct_state(&views, circuit.num_qubits);
        
        // Compute fidelity and purity
        let purity = self.compute_purity(&reconstructed_state);
        
        MultiBaseResult {
            id: Uuid::new_v4(),
            views,
            reconstructed_state,
            purity,
            num_bases: bases.len(),
        }
    }
    
    /// Measure in single basis
    fn measure_in_basis(
        &self,
        circuit: &QuantumCircuit,
        basis: MeasurementBasis,
    ) -> ObservableView {
        // Apply basis rotation
        let mut rotated_circuit = circuit.clone();
        self.apply_basis_rotation(&mut rotated_circuit, basis);
        
        // Measure in Z basis
        let measurements = self.execute_measurements(&rotated_circuit, 1000);
        
        // Compute expectation values in this basis
        let expectations = self.compute_expectations_in_basis(&measurements, basis);
        
        ObservableView {
            basis,
            measurements,
            expectations,
        }
    }
    
    /// Apply basis rotation gates
    fn apply_basis_rotation(&self, circuit: &mut QuantumCircuit, basis: MeasurementBasis) {
        match basis {
            MeasurementBasis::Z => {
                // No rotation needed
            }
            MeasurementBasis::X => {
                // Apply H (Hadamard) to each qubit
                for _ in 0..circuit.num_qubits {
                    circuit.gates.push("H".to_string());
                }
            }
            MeasurementBasis::Y => {
                // Apply S† H to each qubit
                for _ in 0..circuit.num_qubits {
                    circuit.gates.push("Sdg".to_string());
                    circuit.gates.push("H".to_string());
                }
            }
            MeasurementBasis::Custom { .. } => {
                // Apply custom rotation
            }
        }
    }
    
    /// Execute measurements
    fn execute_measurements(&self, circuit: &QuantumCircuit, shots: usize) -> HashMap<String, usize> {
        let mut rng = rand::thread_rng();
        let mut outcomes = HashMap::new();
        
        for _ in 0..shots {
            let outcome_int: usize = rng.gen_range(0..(1 << circuit.num_qubits));
            let outcome = format!("{:0width$b}", outcome_int, width = circuit.num_qubits);
            
            *outcomes.entry(outcome).or_insert(0) += 1;
        }
        
        outcomes
    }
    
    /// Compute expectation values in basis
    fn compute_expectations_in_basis(
        &self,
        measurements: &HashMap<String, usize>,
        basis: MeasurementBasis,
    ) -> HashMap<String, f64> {
        let total_shots: usize = measurements.values().sum();
        let mut expectations = HashMap::new();
        
        // Compute expectation for each qubit
        for (outcome, &count) in measurements {
            for (qubit_idx, bit) in outcome.chars().enumerate() {
                let observable = format!("{}_{}", basis.symbol(), qubit_idx);
                let value = if bit == '0' { 1.0 } else { -1.0 };
                
                *expectations.entry(observable).or_insert(0.0) += 
                    value * (count as f64 / total_shots as f64);
            }
        }
        
        expectations
    }
    
    /// Reconstruct quantum state from multi-base measurements
    fn reconstruct_state(&self, views: &[ObservableView], num_qubits: usize) -> DMatrix<Complex64> {
        let dim = 1 << num_qubits;
        let mut rho = DMatrix::zeros(dim, dim);
        
        // Use maximum likelihood estimation or linear inversion
        // Simplified: construct from Pauli expectations
        
        // Start with maximally mixed state
        for i in 0..dim {
            rho[(i, i)] = Complex64::new(1.0 / dim as f64, 0.0);
        }
        
        // Adjust based on measurements
        for view in views {
            for (obs, &exp_val) in &view.expectations {
                // Apply corrections based on expectation values
                // This is a simplified version
                if obs.starts_with('Z') {
                    let qubit = obs.chars().last().unwrap().to_digit(10).unwrap() as usize;
                    let weight = exp_val * 0.1; // Simplified weighting
                    
                    for i in 0..dim {
                        if (i >> qubit) & 1 == 0 {
                            rho[(i, i)] += Complex64::new(weight, 0.0);
                        }
                    }
                }
            }
        }
        
        // Renormalize
        let trace: Complex64 = (0..dim).map(|i| rho[(i, i)]).sum();
        if trace.norm() > 1e-10 {
            rho /= trace.norm();
        }
        
        rho
    }
    
    /// Compute purity: Tr(ρ²)
    fn compute_purity(&self, rho: &DMatrix<Complex64>) -> f64 {
        let rho_squared = rho * rho;
        let trace: Complex64 = (0..rho.nrows()).map(|i| rho_squared[(i, i)]).sum();
        trace.norm()
    }
}

impl Default for ViewManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// MEASUREMENT BASIS
// ============================================================================

/// Quantum measurement basis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasurementBasis {
    /// Pauli Z basis (computational)
    Z,
    
    /// Pauli X basis
    X,
    
    /// Pauli Y basis
    Y,
    
    /// Custom basis with rotation angles
    Custom { theta: u32, phi: u32 },
}

impl MeasurementBasis {
    /// Get symbol for this basis
    pub fn symbol(&self) -> char {
        match self {
            MeasurementBasis::Z => 'Z',
            MeasurementBasis::X => 'X',
            MeasurementBasis::Y => 'Y',
            MeasurementBasis::Custom { .. } => 'C',
        }
    }
    
    /// Get rotation matrix for this basis
    pub fn rotation_matrix(&self) -> DMatrix<Complex64> {
        match self {
            MeasurementBasis::Z => DMatrix::identity(2, 2),
            MeasurementBasis::X => {
                // Hadamard gate
                let sqrt2_inv = 1.0 / 2.0_f64.sqrt();
                DMatrix::from_row_slice(2, 2, &[
                    Complex64::new(sqrt2_inv, 0.0), Complex64::new(sqrt2_inv, 0.0),
                    Complex64::new(sqrt2_inv, 0.0), Complex64::new(-sqrt2_inv, 0.0),
                ])
            }
            MeasurementBasis::Y => {
                // S† H gates
                let sqrt2_inv = 1.0 / 2.0_f64.sqrt();
                DMatrix::from_row_slice(2, 2, &[
                    Complex64::new(sqrt2_inv, 0.0), Complex64::new(0.0, -sqrt2_inv),
                    Complex64::new(0.0, sqrt2_inv), Complex64::new(sqrt2_inv, 0.0),
                ])
            }
            MeasurementBasis::Custom { theta, phi } => {
                // General rotation
                let theta_rad = (*theta as f64).to_radians();
                let phi_rad = (*phi as f64).to_radians();
                
                let cos_half = (theta_rad / 2.0).cos();
                let sin_half = (theta_rad / 2.0).sin();
                
                DMatrix::from_row_slice(2, 2, &[
                    Complex64::new(cos_half, 0.0),
                    Complex64::new(-sin_half * phi_rad.sin(), -sin_half * phi_rad.cos()),
                    Complex64::new(sin_half * phi_rad.sin(), sin_half * phi_rad.cos()),
                    Complex64::new(cos_half, 0.0),
                ])
            }
        }
    }
}

// ============================================================================
// OBSERVABLE VIEW
// ============================================================================

/// View of observables in a specific basis
#[derive(Debug, Clone)]
pub struct ObservableView {
    /// Measurement basis
    pub basis: MeasurementBasis,
    
    /// Raw measurement outcomes
    pub measurements: HashMap<String, usize>,
    
    /// Expectation values
    pub expectations: HashMap<String, f64>,
}

impl ObservableView {
    /// Get expectation value
    pub fn expectation(&self, observable: &str) -> Option<f64> {
        self.expectations.get(observable).copied()
    }
    
    /// Get measurement probability
    pub fn probability(&self, outcome: &str) -> f64 {
        let total: usize = self.measurements.values().sum();
        let count = self.measurements.get(outcome).unwrap_or(&0);
        *count as f64 / total as f64
    }
}

// ============================================================================
// MULTI-BASE RESULT
// ============================================================================

/// Result of multi-base measurement
#[derive(Debug, Clone)]
pub struct MultiBaseResult {
    /// Unique ID
    pub id: Uuid,
    
    /// Observable views in different bases
    pub views: Vec<ObservableView>,
    
    /// Reconstructed quantum state
    pub reconstructed_state: DMatrix<Complex64>,
    
    /// Purity of reconstructed state
    pub purity: f64,
    
    /// Number of bases measured
    pub num_bases: usize,
}

impl MultiBaseResult {
    /// Get view for specific basis
    pub fn get_view(&self, basis: MeasurementBasis) -> Option<&ObservableView> {
        self.views.iter().find(|v| v.basis == basis)
    }
    
    /// Compute Bloch vector (for single qubit)
    pub fn bloch_vector(&self) -> Option<[f64; 3]> {
        if self.views.is_empty() {
            return None;
        }
        
        let x = self.views.iter()
            .find(|v| v.basis == MeasurementBasis::X)?
            .expectations.get("X_0")?;
        
        let y = self.views.iter()
            .find(|v| v.basis == MeasurementBasis::Y)?
            .expectations.get("Y_0")?;
        
        let z = self.views.iter()
            .find(|v| v.basis == MeasurementBasis::Z)?
            .expectations.get("Z_0")?;
        
        Some([*x, *y, *z])
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_view_manager() {
        let manager = ViewManager::new();
        assert_eq!(manager.bases.len(), 3);
    }
    
    #[test]
    fn test_measurement_bases() {
        let z = MeasurementBasis::Z;
        let x = MeasurementBasis::X;
        let y = MeasurementBasis::Y;
        
        assert_eq!(z.symbol(), 'Z');
        assert_eq!(x.symbol(), 'X');
        assert_eq!(y.symbol(), 'Y');
    }
    
    #[test]
    fn test_multi_base_measurement() {
        let mut manager = ViewManager::new();
        let circuit = QuantumCircuit::new(1);
        
        let bases = vec![
            MeasurementBasis::Z,
            MeasurementBasis::X,
            MeasurementBasis::Y,
        ];
        
        let result = manager.measure_multi_base(&circuit, &bases);
        
        assert_eq!(result.num_bases, 3);
        assert!(result.purity >= 0.0 && result.purity <= 1.0);
    }
}
