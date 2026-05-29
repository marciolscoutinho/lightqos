// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// quantum_state.rs — Quantum State — state-vector and density matrix representation
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 06-05-2026
// All rights reserved.
// ---------------------------------------------------------------------------

use nalgebra::{DVector, DMatrix, Complex};
use num_complex::Complex64;
use rand::Rng;
use std::collections::HashMap;

const SQRT_2: f64 = std::f64::consts::SQRT_2;

// ============================================================================
// QUANTUM STATE
// ============================================================================

/// Pure quantum state
#[derive(Debug, Clone)]
pub struct QuantumState {
    /// State vector |ψ⟩
    pub amplitudes: DVector<Complex64>,
    
    /// Number of qubits
    pub num_qubits: usize,
}

impl QuantumState {
    /// Creates state |0⟩^⊗n
    pub fn zero_state(num_qubits: usize) -> Self {
        let dim = 2_usize.pow(num_qubits as u32);
        let mut amplitudes = DVector::zeros(dim);
        amplitudes[0] = Complex64::new(1.0, 0.0);
        
        Self {
            amplitudes,
            num_qubits,
        }
    }
    
    /// Creates a state from amplitudes
    pub fn from_amplitudes(amplitudes: Vec<Complex64>, num_qubits: usize) -> Self {
        let dim = 2_usize.pow(num_qubits as u32);
        assert_eq!(amplitudes.len(), dim, "Invalid number of amplitudes");
        
        let mut state = Self {
            amplitudes: DVector::from_vec(amplitudes),
            num_qubits,
        };
        state.normalize();
        state
    }
    
    /// Creates a computational basis state
    pub fn basis_state(index: usize, num_qubits: usize) -> Self {
        let dim = 2_usize.pow(num_qubits as u32);
        assert!(index < dim, "Index out of bounds");
        
        let mut amplitudes = DVector::zeros(dim);
        amplitudes[index] = Complex64::new(1.0, 0.0);
        
        Self {
            amplitudes,
            num_qubits,
        }
    }
    
    /// State |+⟩ = (|0⟩ + |1⟩)/√2
    pub fn plus_state() -> Self {
        Self::from_amplitudes(
            vec![
                Complex64::new(1.0 / SQRT_2, 0.0),
                Complex64::new(1.0 / SQRT_2, 0.0),
            ],
            1,
        )
    }
    
    /// State |-⟩ = (|0⟩ - |1⟩)/√2
    pub fn minus_state() -> Self {
        Self::from_amplitudes(
            vec![
                Complex64::new(1.0 / SQRT_2, 0.0),
                Complex64::new(-1.0 / SQRT_2, 0.0),
            ],
            1,
        )
    }
    
    /// Bell pair |Φ⁺⟩ = (|00⟩ + |11⟩)/√2
    pub fn bell_state() -> Self {
        Self::from_amplitudes(
            vec![
                Complex64::new(1.0 / SQRT_2, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0 / SQRT_2, 0.0),
            ],
            2,
        )
    }
    
    /// GHZ state: (|000⟩ + |111⟩)/√2
    pub fn ghz_state(num_qubits: usize) -> Self {
        let dim = 2_usize.pow(num_qubits as u32);
        let mut amplitudes = DVector::zeros(dim);
        
        amplitudes[0] = Complex64::new(1.0 / SQRT_2, 0.0);
        amplitudes[dim - 1] = Complex64::new(1.0 / SQRT_2, 0.0);
        
        Self {
            amplitudes,
            num_qubits,
        }
    }
    
    /// Normalizes the state
    pub fn normalize(&mut self) {
        let norm: f64 = self.amplitudes.iter().map(|z| z.norm_sqr()).sum::<f64>().sqrt();
        if norm > 1.0e-10 {
            self.amplitudes /= norm;
        }
    }
    
    /// Checks whether the state is normalized
    pub fn is_normalized(&self) -> bool {
        let norm_sq: f64 = self.amplitudes.iter().map(|z| z.norm_sqr()).sum();
        (norm_sq - 1.0).abs() < 1.0e-10
    }
    
    /// Applies a unitary gate
    pub fn apply_gate(&mut self, gate: &DMatrix<Complex64>) {
        assert_eq!(gate.nrows(), gate.ncols(), "Gate must be square");
        assert_eq!(gate.nrows(), self.amplitudes.len(), "Gate dimension mismatch");
        
        self.amplitudes = gate * &self.amplitudes;
    }
    
    /// Applies a 1-qubit gate to a specific qubit
    pub fn apply_single_qubit_gate(&mut self, gate: &DMatrix<Complex64>, target: usize) {
        assert!(target < self.num_qubits, "Target qubit out of bounds");
        assert_eq!(gate.nrows(), 2, "Single qubit gate must be 2x2");
        
        let full_gate = self.expand_gate(gate, target);
        self.apply_gate(&full_gate);
    }
    
    /// Expands a 1-qubit gate to the whole system
    fn expand_gate(&self, gate: &DMatrix<Complex64>, target: usize) -> DMatrix<Complex64> {
        let dim = 2_usize.pow(self.num_qubits as u32);
        let mut full_gate = DMatrix::zeros(dim, dim);
        
        for i in 0..dim {
            for j in 0..dim {
                // Checks whether non-target qubits are equal
                let mut match_others = true;
                for q in 0..self.num_qubits {
                    if q != target {
                        let bit_i = (i >> q) & 1;
                        let bit_j = (j >> q) & 1;
                        if bit_i != bit_j {
                            match_others = false;
                            break;
                        }
                    }
                }
                
                if match_others {
                    let bit_i = (i >> target) & 1;
                    let bit_j = (j >> target) & 1;
                    full_gate[(i, j)] = gate[(bit_i, bit_j)];
                }
            }
        }
        
        full_gate
    }
    
    /// Measures the state in the computational basis
    pub fn measure(&self) -> (usize, f64) {
        let mut rng = rand::thread_rng();
        let random: f64 = rng.gen();
        
        let mut cumulative = 0.0;
        for (i, amplitude) in self.amplitudes.iter().enumerate() {
            let prob = amplitude.norm_sqr();
            cumulative += prob;
            
            if random <= cumulative {
                return (i, prob);
            }
        }
        
        // Fallback (should not happen)
        let last_idx = self.amplitudes.len() - 1;
        (last_idx, self.amplitudes[last_idx].norm_sqr())
    }
    
    /// Measurement probabilities
    pub fn probabilities(&self) -> HashMap<String, f64> {
        let mut probs = HashMap::new();
        
        for (i, amplitude) in self.amplitudes.iter().enumerate() {
            let prob = amplitude.norm_sqr();
            if prob > 1.0e-10 {
                let bitstring = format!("{:0width$b}", i, width = self.num_qubits);
                probs.insert(bitstring, prob);
            }
        }
        
        probs
    }
    
    /// Fidelity with another state
    pub fn fidelity(&self, other: &QuantumState) -> f64 {
        assert_eq!(self.num_qubits, other.num_qubits, "States must have same size");
        
        let overlap: Complex64 = self.amplitudes.iter()
            .zip(other.amplitudes.iter())
            .map(|(a, b)| a.conj() * b)
            .sum();
        
        overlap.norm_sqr()
    }
    
    /// Tensor product with another state
    pub fn tensor_product(&self, other: &QuantumState) -> QuantumState {
        let new_num_qubits = self.num_qubits + other.num_qubits;
        let new_dim = 2_usize.pow(new_num_qubits as u32);
        let mut new_amplitudes = DVector::zeros(new_dim);
        
        for i in 0..self.amplitudes.len() {
            for j in 0..other.amplitudes.len() {
                let idx = i * other.amplitudes.len() + j;
                new_amplitudes[idx] = self.amplitudes[i] * other.amplitudes[j];
            }
        }
        
        QuantumState {
            amplitudes: new_amplitudes,
            num_qubits: new_num_qubits,
        }
    }
}

// ============================================================================
// QUANTUM GATES
// ============================================================================

pub struct Pauli;

impl Pauli {
    /// I gate (identity)
    pub fn i() -> DMatrix<Complex64> {
        DMatrix::from_row_slice(2, 2, &[
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
        ])
    }
    
    /// X gate (NOT)
    pub fn x() -> DMatrix<Complex64> {
        DMatrix::from_row_slice(2, 2, &[
            Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
        ])
    }
    
    /// Y gate
    pub fn y() -> DMatrix<Complex64> {
        DMatrix::from_row_slice(2, 2, &[
            Complex64::new(0.0, 0.0), Complex64::new(0.0, -1.0),
            Complex64::new(0.0, 1.0), Complex64::new(0.0, 0.0),
        ])
    }
    
    /// Z gate
    pub fn z() -> DMatrix<Complex64> {
        DMatrix::from_row_slice(2, 2, &[
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(-1.0, 0.0),
        ])
    }
    
    /// H gate (Hadamard)
    pub fn h() -> DMatrix<Complex64> {
        let val = 1.0 / SQRT_2;
        DMatrix::from_row_slice(2, 2, &[
            Complex64::new(val, 0.0), Complex64::new(val, 0.0),
            Complex64::new(val, 0.0), Complex64::new(-val, 0.0),
        ])
    }
    
    /// S gate (phase)
    pub fn s() -> DMatrix<Complex64> {
        DMatrix::from_row_slice(2, 2, &[
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(0.0, 1.0),
        ])
    }
    
    /// T gate
    pub fn t() -> DMatrix<Complex64> {
        let phase = std::f64::consts::PI / 4.0;
        DMatrix::from_row_slice(2, 2, &[
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(phase.cos(), phase.sin()),
        ])
    }
    
    /// CNOT gate (2 qubits)
    pub fn cnot() -> DMatrix<Complex64> {
        DMatrix::from_row_slice(4, 4, &[
            Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
        ])
    }
}

// ============================================================================
// ALIASES
// ============================================================================

pub type Qubit = QuantumState;
pub type StateBasis = QuantumState;

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_zero_state() {
        let state = QuantumState::zero_state(2);
        assert_eq!(state.num_qubits, 2);
        assert!(state.is_normalized());
    }
    
    #[test]
    fn test_bell_state() {
        let bell = QuantumState::bell_state();
        assert!(bell.is_normalized());
        
        let probs = bell.probabilities();
        assert!((probs["00"] - 0.5).abs() < 1.0e-10);
        assert!((probs["11"] - 0.5).abs() < 1.0e-10);
    }
    
    #[test]
    fn test_pauli_x() {
        let mut state = QuantumState::zero_state(1);
        state.apply_gate(&Pauli::x());
        
        // Should become |1⟩
        assert!((state.amplitudes[1].norm() - 1.0).abs() < 1.0e-10);
    }
    
    #[test]
    fn test_hadamard() {
        let mut state = QuantumState::zero_state(1);
        state.apply_gate(&Pauli::h());
        
        // Should become |+⟩
        let probs = state.probabilities();
        assert!((probs["0"] - 0.5).abs() < 1.0e-10);
        assert!((probs["1"] - 0.5).abs() < 1.0e-10);
    }
    
    #[test]
    fn test_fidelity() {
        let state1 = QuantumState::zero_state(1);
        let state2 = QuantumState::zero_state(1);
        
        assert!((state1.fidelity(&state2) - 1.0).abs() < 1.0e-10);
    }
}
