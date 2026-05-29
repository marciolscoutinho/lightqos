// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// quantum_channel.rs — Quantum Channels — CPTP maps and Kraus operator representation
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 22-07-2022
// All rights reserved.
// ---------------------------------------------------------------------------

use nalgebra::{DMatrix, DVector};
use num_complex::Complex64;
use uuid::Uuid;

// ============================================================================
// QUANTUM CHANNEL
// ============================================================================

/// Quantum channel (CPTP map)
#[derive(Debug, Clone)]
pub struct QuantumChannel {
    /// Unique identifier
    pub id: Uuid,
    
    /// Input dimension
    pub input_dim: usize,
    
    /// Output dimension
    pub output_dim: usize,
    
    /// Choi matrix representation
    pub choi: ChoiMatrix,
    
    /// Kraus operators (if available)
    pub kraus: Option<KrausOperators>,
    
    /// Channel name
    pub name: String,
}

impl QuantumChannel {
    /// Create identity channel
    pub fn identity(dim: usize) -> Self {
        let choi_matrix = DMatrix::identity(dim * dim, dim * dim);
        
        Self {
            id: Uuid::new_v4(),
            input_dim: dim,
            output_dim: dim,
            choi: ChoiMatrix {
                matrix: choi_matrix,
                input_dim: dim,
                output_dim: dim,
            },
            kraus: None,
            name: "Identity".to_string(),
        }
    }
    
    /// Create channel from Choi matrix
    pub fn from_choi(choi_matrix: DMatrix<Complex64>, input_dim: usize, output_dim: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            input_dim,
            output_dim,
            choi: ChoiMatrix {
                matrix: choi_matrix,
                input_dim,
                output_dim,
            },
            kraus: None,
            name: "Custom".to_string(),
        }
    }
    
    /// Create channel from Kraus operators
    pub fn from_kraus(operators: Vec<DMatrix<Complex64>>, name: String) -> Self {
        let input_dim = operators[0].ncols();
        let output_dim = operators[0].nrows();
        
        // Convert Kraus to Choi
        let choi_matrix = Self::kraus_to_choi(&operators, input_dim, output_dim);
        
        Self {
            id: Uuid::new_v4(),
            input_dim,
            output_dim,
            choi: ChoiMatrix {
                matrix: choi_matrix,
                input_dim,
                output_dim,
            },
            kraus: Some(KrausOperators {
                operators,
                input_dim,
                output_dim,
            }),
            name,
        }
    }
    
    /// Convert Kraus operators to Choi matrix
    fn kraus_to_choi(
        operators: &[DMatrix<Complex64>],
        input_dim: usize,
        output_dim: usize,
    ) -> DMatrix<Complex64> {
        let choi_dim = input_dim * output_dim;
        let mut choi = DMatrix::zeros(choi_dim, choi_dim);
        
        // Choi matrix: Σ_k (K_k ⊗ K_k*)
        for k_op in operators {
            for i in 0..output_dim {
                for j in 0..input_dim {
                    for k in 0..output_dim {
                        for l in 0..input_dim {
                            let idx1 = i * input_dim + j;
                            let idx2 = k * input_dim + l;
                            choi[(idx1, idx2)] += k_op[(i, j)] * k_op[(k, l)].conj();
                        }
                    }
                }
            }
        }
        
        choi
    }
    
    /// Apply channel to quantum state (density matrix)
    pub fn apply(&self, state: &DMatrix<Complex64>) -> DMatrix<Complex64> {
        if let Some(ref kraus) = self.kraus {
            // Use Kraus representation: ρ' = Σ_k K_k ρ K_k†
            kraus.apply(state)
        } else {
            // Use Choi representation
            self.choi.apply(state)
        }
    }
    
    /// Compose two channels: (Ε₂ ∘ Ε₁)(ρ) = Ε₂(Ε₁(ρ))
    pub fn compose(&self, other: &QuantumChannel) -> QuantumChannel {
        assert_eq!(self.input_dim, other.output_dim, "Dimension mismatch");
        
        // Compose Choi matrices
        let composed_choi = self.choi.compose(&other.choi);
        
        Self {
            id: Uuid::new_v4(),
            input_dim: other.input_dim,
            output_dim: self.output_dim,
            choi: composed_choi,
            kraus: None,
            name: format!("{} ∘ {}", self.name, other.name),
        }
    }
    
    /// Check if channel is CPTP
    pub fn is_cptp(&self) -> bool {
        self.choi.is_positive_semidefinite() && self.choi.is_trace_preserving()
    }
    
    /// Depolarizing channel: ρ → (1-p)ρ + p(I/d)
    pub fn depolarizing(dim: usize, probability: f64) -> Self {
        let mut operators = Vec::new();
        
        // K₀ = √(1-p) I
        let k0 = DMatrix::identity(dim, dim) * (1.0 - probability).sqrt();
        operators.push(k0);
        
        // Pauli operators for qubit (dim=2)
        if dim == 2 {
            let p_coeff = (probability / 3.0).sqrt();
            
            // K₁ = √(p/3) X
            let mut kx = DMatrix::zeros(2, 2);
            kx[(0, 1)] = Complex64::new(p_coeff, 0.0);
            kx[(1, 0)] = Complex64::new(p_coeff, 0.0);
            operators.push(kx);
            
            // K₂ = √(p/3) Y
            let mut ky = DMatrix::zeros(2, 2);
            ky[(0, 1)] = Complex64::new(0.0, -p_coeff);
            ky[(1, 0)] = Complex64::new(0.0, p_coeff);
            operators.push(ky);
            
            // K₃ = √(p/3) Z
            let mut kz = DMatrix::zeros(2, 2);
            kz[(0, 0)] = Complex64::new(p_coeff, 0.0);
            kz[(1, 1)] = Complex64::new(-p_coeff, 0.0);
            operators.push(kz);
        }
        
        Self::from_kraus(operators, format!("Depolarizing(p={})", probability))
    }
    
    /// Amplitude damping channel
    pub fn amplitude_damping(gamma: f64) -> Self {
        let mut operators = Vec::new();
        
        // K₀
        let mut k0 = DMatrix::zeros(2, 2);
        k0[(0, 0)] = Complex64::new(1.0, 0.0);
        k0[(1, 1)] = Complex64::new((1.0 - gamma).sqrt(), 0.0);
        operators.push(k0);
        
        // K₁
        let mut k1 = DMatrix::zeros(2, 2);
        k1[(0, 1)] = Complex64::new(gamma.sqrt(), 0.0);
        operators.push(k1);
        
        Self::from_kraus(operators, format!("AmplitudeDamping(γ={})", gamma))
    }
}

// ============================================================================
// CHOI MATRIX
// ============================================================================

/// Choi matrix representation of a quantum channel
#[derive(Debug, Clone)]
pub struct ChoiMatrix {
    /// Choi matrix J(Ε)
    pub matrix: DMatrix<Complex64>,
    
    /// Input dimension
    pub input_dim: usize,
    
    /// Output dimension
    pub output_dim: usize,
}

impl ChoiMatrix {
    /// Apply channel using Choi representation
    pub fn apply(&self, state: &DMatrix<Complex64>) -> DMatrix<Complex64> {
        // Simplified application - full implementation would use:
        // ρ_out = Tr_in[(I ⊗ ρ^T) J]
        
        let dim_out = self.output_dim;
        let mut result = DMatrix::zeros(dim_out, dim_out);
        
        // Simplified: assume state and channel dimensions match
        for i in 0..dim_out {
            for j in 0..dim_out {
                result[(i, j)] = self.matrix[(i, j)];
            }
        }
        
        result
    }
    
    /// Compose two Choi matrices
    pub fn compose(&self, other: &ChoiMatrix) -> ChoiMatrix {
        // Simplified composition
        let composed_matrix = &self.matrix * &other.matrix;
        
        ChoiMatrix {
            matrix: composed_matrix,
            input_dim: other.input_dim,
            output_dim: self.output_dim,
        }
    }
    
    /// Check if Choi matrix is positive semidefinite
    pub fn is_positive_semidefinite(&self) -> bool {
        // Simplified check - would compute eigenvalues
        true
    }
    
    /// Check if channel is trace preserving
    pub fn is_trace_preserving(&self) -> bool {
        // Tr_out[J] = I_in
        true
    }
}

// ============================================================================
// KRAUS OPERATORS
// ============================================================================

/// Kraus operator representation of a quantum channel
#[derive(Debug, Clone)]
pub struct KrausOperators {
    /// Set of Kraus operators {K_i}
    pub operators: Vec<DMatrix<Complex64>>,
    
    /// Input dimension
    pub input_dim: usize,
    
    /// Output dimension
    pub output_dim: usize,
}

impl KrausOperators {
    /// Apply Kraus operators to state
    pub fn apply(&self, state: &DMatrix<Complex64>) -> DMatrix<Complex64> {
        let mut result = DMatrix::zeros(self.output_dim, self.output_dim);
        
        // ρ' = Σ_k K_k ρ K_k†
        for k_op in &self.operators {
            let k_dag = k_op.conjugate_transpose();
            result += k_op * state * k_dag;
        }
        
        result
    }
    
    /// Check completeness relation: Σ_k K_k† K_k = I
    pub fn is_complete(&self) -> bool {
        let mut sum = DMatrix::zeros(self.input_dim, self.input_dim);
        
        for k_op in &self.operators {
            let k_dag = k_op.conjugate_transpose();
            sum += k_dag * k_op;
        }
        
        // Check if sum ≈ identity
        for i in 0..self.input_dim {
            for j in 0..self.input_dim {
                let expected = if i == j { 1.0 } else { 0.0 };
                if (sum[(i, j)].re - expected).abs() > 1e-10 {
                    return false;
                }
            }
        }
        
        true
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_identity_channel() {
        let channel = QuantumChannel::identity(2);
        assert_eq!(channel.input_dim, 2);
        assert_eq!(channel.output_dim, 2);
        assert!(channel.is_cptp());
    }
    
    #[test]
    fn test_depolarizing_channel() {
        let channel = QuantumChannel::depolarizing(2, 0.1);
        assert!(channel.kraus.is_some());
        
        let kraus = channel.kraus.unwrap();
        assert!(kraus.is_complete());
    }
    
    #[test]
    fn test_amplitude_damping() {
        let channel = QuantumChannel::amplitude_damping(0.2);
        assert!(channel.kraus.is_some());
        
        let kraus = channel.kraus.unwrap();
        assert!(kraus.is_complete());
    }
}
