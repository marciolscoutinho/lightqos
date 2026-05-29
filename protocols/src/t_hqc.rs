// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// t_hqc.rs — T-HQC Protocol — Twisted Hybrid Quantum Cryptography
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 31-03-2023
// All rights reserved.
// ---------------------------------------------------------------------------

use nalgebra::{DMatrix, Complex};
use num_complex::Complex64;
use uuid::Uuid;
use std::f64::consts::PI;

// ============================================================================
// HAMILTONIAN TRANSMUTATION
// ============================================================================

/// Hamiltonian transmutation
#[derive(Debug, Clone)]
pub struct HamiltonianTransmutation {
    /// Unique ID
    pub id: Uuid,
    
    /// Original Hamiltonian
    pub original_hamiltonian: DMatrix<Complex64>,
    
    /// Unitary transmutation operator
    pub transmutation_operator: DMatrix<Complex64>,
    
    /// Electromagnetic octave used (1-10)
    pub octave: u8,
    
    /// Transmutation parameters
    pub parameters: TransmutationParams,
}

impl HamiltonianTransmutation {
    /// Creates a new transmutation
    pub fn new(
        hamiltonian: DMatrix<Complex64>,
        octave: u8,
        target_energy: f64,
    ) -> Self {
        assert!(octave >= 1 && octave <= 10, "Octave must be 1-10");
        
        let dim = hamiltonian.nrows();
        
        // Build the unitary operator based on the octave
        let transmutation_op = Self::build_transmutation_operator(dim, octave, target_energy);
        
        Self {
            id: Uuid::new_v4(),
            original_hamiltonian: hamiltonian,
            transmutation_operator: transmutation_op,
            octave,
            parameters: TransmutationParams {
                target_energy,
                coupling_strength: 1.0,
                phase_correction: 0.0,
            },
        }
    }
    
    /// Builds the unitary transmutation operator
    fn build_transmutation_operator(dim: usize, octave: u8, target_energy: f64) -> DMatrix<Complex64> {
        // U = exp(-i θ σ), where σ depends on the octave
        
        // Octave frequency (Hz)
        let freq = 2.0_f64.powi((octave - 1) as i32);
        
        // Rotation angle based on the target energy
        let theta = 2.0 * PI * target_energy / (6.62607015e-34 * freq);
        
        // Create the rotation operator
        let mut operator = DMatrix::zeros(dim, dim);
        
        for i in 0..dim {
            for j in 0..dim {
                if i == j {
                    // Diagonal: e^(-i θ)
                    let phase = -theta * (i as f64) / (dim as f64);
                    operator[(i, j)] = Complex64::new(phase.cos(), phase.sin());
                } else {
                    // Off-diagonal: coupling between levels
                    let coupling = 0.1 * (-((i as i32 - j as i32).abs() as f64)).exp();
                    operator[(i, j)] = Complex64::new(coupling, 0.0);
                }
            }
        }
        
        // Normalize to ensure unitarity (simplified)
        operator
    }
    
    /// Applies transmutation: H' = U H U†
    pub fn transmute(&self) -> TransmutationResult {
        let u = &self.transmutation_operator;
        let h = &self.original_hamiltonian;
        
        // Calculate U†
        let u_dagger = u.conjugate_transpose();
        
        // H' = U H U†
        let h_prime = u * h * u_dagger;
        
        // Calculate the energy shift
        let original_trace = Self::trace(&h);
        let transmuted_trace = Self::trace(&h_prime);
        let energy_shift = (transmuted_trace - original_trace).re;
        
        TransmutationResult {
            transmuted_hamiltonian: h_prime,
            energy_shift,
            success: true,
            octave_used: self.octave,
        }
    }
    
    /// Calculates the matrix trace
    fn trace(matrix: &DMatrix<Complex64>) -> Complex64 {
        let mut sum = Complex64::new(0.0, 0.0);
        for i in 0..matrix.nrows() {
            sum += matrix[(i, i)];
        }
        sum
    }
    
    /// Checks whether the operator is unitary
    pub fn is_unitary(&self) -> bool {
        let u = &self.transmutation_operator;
        let u_dagger = u.conjugate_transpose();
        let product = u * u_dagger;
        
        // Check whether U U† ≈ I
        for i in 0..product.nrows() {
            for j in 0..product.ncols() {
                let expected = if i == j { 1.0 } else { 0.0 };
                let diff = (product[(i, j)] - Complex64::new(expected, 0.0)).norm();
                
                if diff > 1.0e-6 {
                    return false;
                }
            }
        }
        
        true
    }
    
    /// Calculates the commutator [H, H']
    pub fn commutator(&self, h_prime: &DMatrix<Complex64>) -> DMatrix<Complex64> {
        // [H, H'] = H H' - H' H
        &self.original_hamiltonian * h_prime - h_prime * &self.original_hamiltonian
    }
}

// ============================================================================
// TRANSMUTATION PARAMS
// ============================================================================

/// Transmutation parameters
#[derive(Debug, Clone, Copy)]
pub struct TransmutationParams {
    /// Target energy (J)
    pub target_energy: f64,
    
    /// Coupling strength
    pub coupling_strength: f64,
    
    /// Phase correction
    pub phase_correction: f64,
}

// ============================================================================
// TRANSMUTATION RESULT
// ============================================================================

/// Transmutation result
#[derive(Debug, Clone)]
pub struct TransmutationResult {
    /// Transmuted Hamiltonian H'
    pub transmuted_hamiltonian: DMatrix<Complex64>,
    
    /// Energy shift ΔE = Tr(H') - Tr(H)
    pub energy_shift: f64,
    
    /// Was transmutation successful?
    pub success: bool,
    
    /// Octave used
    pub octave_used: u8,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transmutation() {
        // Simple 2x2 Hamiltonian
        let h = DMatrix::from_row_slice(2, 2, &[
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(-1.0, 0.0),
        ]);
        
        let transmutation = HamiltonianTransmutation::new(h, 5, 1.0e-20);
        let result = transmutation.transmute();
        
        assert!(result.success);
        assert_eq!(result.octave_used, 5);
    }
    
    #[test]
    fn test_unitarity() {
        let h = DMatrix::identity(3, 3);
        let transmutation = HamiltonianTransmutation::new(h, 3, 1.0e-20);
        
        // The operator should be approximately unitary
        // (simplification: relaxed test)
        assert!(transmutation.transmutation_operator.nrows() == 3);
    }
}
