// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// metrics.rs — EMF Metrics — fidelity, entropy and concurrence calculations
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 13-10-2021
// All rights reserved.
// ---------------------------------------------------------------------------

use std::f64::consts::PI;
use nalgebra::{Complex, DMatrix, DVector};
use serde::{Serialize, Deserialize};

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// State of an entangled pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntangledPairState {
    /// 4x4 density matrix (2-qubit system)
    pub density_matrix: Vec<Complex<f64>>,
    
    /// Fidelity with the ideal Bell state
    pub fidelity: f64,
    
    /// Concurrence (entanglement measure)
    pub concurrence: f64,
    
    /// Entanglement entropy
    pub entanglement_entropy: f64,
    
    /// Ergotropy (extractable work)
    pub ergotropy: f64,
    
    /// Negativity
    pub negativity: f64,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Number of times reused
    pub reuse_count: usize,
}

/// Aggregated metrics of the EMF pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EMFPoolMetrics {
    /// Total number of pairs in the pool
    pub total_pairs: usize,
    
    /// Available pairs (high fidelity)
    pub available_pairs: usize,
    
    /// Degraded pairs (low fidelity)
    pub degraded_pairs: usize,
    
    /// Average fidelity
    pub avg_fidelity: f64,
    
    /// Average concurrence
    pub avg_concurrence: f64,
    
    /// Total ergotropy
    pub total_ergotropy: f64,
    
    /// Total entropy
    pub total_entropy: f64,
    
    /// Recycling rate (pairs/second)
    pub recycling_rate: f64,
    
    /// Thermodynamic efficiency (useful work / total energy)
    pub thermodynamic_efficiency: f64,
}

/// Thermodynamic cycle of a pair
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThermodynamicPhase {
    /// Generation (4+ → 1+): high-ergotropy creation
    Generation,
    
    /// Usage (1+): pair in active use
    Active,
    
    /// Degradation (1+ → 0=): loss of coherence
    Degradation,
    
    /// Radiation (1- → 4-): final dissipation
    Radiation,
    
    /// Inertia (0=): resting/recycling state
    Inertia,
}

// ============================================================================
// METRICS CALCULATOR
// ============================================================================

pub struct EntanglementMetricsCalculator;

impl EntanglementMetricsCalculator {
    /// Calculates all metrics for an entangled pair
    pub fn calculate_all_metrics(
        density_matrix: &DMatrix<Complex<f64>>,
    ) -> Result<EntangledPairState, String> {
        // Validate dimension (must be 4x4 for 2 qubits)
        if density_matrix.nrows() != 4 || density_matrix.ncols() != 4 {
            return Err(format!(
                "Density matrix must be 4x4, received {}x{}",
                density_matrix.nrows(),
                density_matrix.ncols()
            ));
        }
        
        // Calculate metrics individuais
        let fidelity = Self::calculate_fidelity(density_matrix)?;
        let concurrence = Self::calculate_concurrence(density_matrix)?;
        let entanglement_entropy = Self::calculate_entanglement_entropy(density_matrix)?;
        let ergotropy = Self::calculate_ergotropy(density_matrix)?;
        let negativity = Self::calculate_negativity(density_matrix)?;
        
        Ok(EntangledPairState {
            density_matrix: density_matrix.iter().cloned().collect(),
            fidelity,
            concurrence,
            entanglement_entropy,
            ergotropy,
            negativity,
            created_at: Self::current_timestamp(),
            reuse_count: 0,
        })
    }
    
    // ========================================================================
    // FIDELITY
    // ========================================================================
    
    /// Calculates fidelity with the ideal Bell state |Φ+⟩ = (|00⟩ + |11⟩)/√2
    pub fn calculate_fidelity(
        rho: &DMatrix<Complex<f64>>,
    ) -> Result<f64, String> {
        // Ideal Bell state: |Φ+⟩⟨Φ+|
        let phi_plus = Self::bell_state_phi_plus();
        
        // Fidelity: F = Tr(ρ |Φ+⟩⟨Φ+|)
        let product = rho * &phi_plus;
        let trace = product.trace();
        
        Ok(trace.re.max(0.0).min(1.0))
    }
    
    /// Creates the density matrix of the state |Φ+⟩
    fn bell_state_phi_plus() -> DMatrix<Complex<f64>> {
        let mut phi = DMatrix::zeros(4, 4);
        
        // |Φ+⟩ = (|00⟩ + |11⟩)/√2
        // In the computational basis: (1,0,0,1)/√2
        let sqrt2_inv = 1.0 / 2.0_f64.sqrt();
        
        // |Φ+⟩⟨Φ+| = 1/2 * [[1,0,0,1], [0,0,0,0], [0,0,0,0], [1,0,0,1]]
        phi[(0, 0)] = Complex::new(0.5, 0.0);
        phi[(0, 3)] = Complex::new(0.5, 0.0);
        phi[(3, 0)] = Complex::new(0.5, 0.0);
        phi[(3, 3)] = Complex::new(0.5, 0.0);
        
        phi
    }
    
    // ========================================================================
    // CONCURRENCE
    // ========================================================================
    
    /// Calculates concurrence (entanglement measure for 2 qubits)
    /// C = max(0, λ1 - λ2 - λ3 - λ4)
    /// where λi are eigenvalues de R = sqrt(sqrt(ρ) ρ_tilde sqrt(ρ))
    pub fn calculate_concurrence(
        rho: &DMatrix<Complex<f64>>,
    ) -> Result<f64, String> {
        // Calculate ρ_tilde = (σy ⊗ σy) ρ* (σy ⊗ σy)
        let sigma_y_tensor = Self::sigma_y_tensor();
        let rho_conj = rho.map(|c| c.conj());
        let rho_tilde = &sigma_y_tensor * &rho_conj * &sigma_y_tensor;
        
        // Calculate R = sqrt(rho) * rho_tilde * sqrt(rho)
        let sqrt_rho = Self::matrix_sqrt(rho)?;
        let r = &sqrt_rho * &rho_tilde * &sqrt_rho;
        
        // Get eigenvalues of R
        let eigenvalues = Self::get_eigenvalues(&r)?;
        
        // Sort in descending order
        let mut lambdas: Vec<f64> = eigenvalues.iter().map(|c| c.norm()).collect();
        lambdas.sort_by(|a, b| b.partial_cmp(a).unwrap());
        
        // C = max(0, λ1 - λ2 - λ3 - λ4)
        let concurrence = (lambdas[0] - lambdas[1] - lambdas[2] - lambdas[3]).max(0.0);
        
        Ok(concurrence.min(1.0))
    }
    
    /// Returns tensor product σy ⊗ σy
    fn sigma_y_tensor() -> DMatrix<Complex<f64>> {
        let mut result = DMatrix::zeros(4, 4);
        let i = Complex::new(0.0, 1.0);
        
        // σy = [[0, -i], [i, 0]]
        // σy ⊗ σy for base |00⟩, |01⟩, |10⟩, |11⟩
        result[(0, 3)] = Complex::new(-1.0, 0.0);
        result[(1, 2)] = Complex::new(1.0, 0.0);
        result[(2, 1)] = Complex::new(1.0, 0.0);
        result[(3, 0)] = Complex::new(-1.0, 0.0);
        
        result
    }
    
    // ========================================================================
    // ENTANGLEMENT ENTROPY
    // ========================================================================
    
    /// Calculates entanglement entropy S = -Tr(ρA log ρA)
    /// where ρA is the reduced density matrix of the first qubit
    pub fn calculate_entanglement_entropy(
        rho: &DMatrix<Complex<f64>>,
    ) -> Result<f64, String> {
        // Partial trace over the second qubit
        let rho_a = Self::partial_trace_second_qubit(rho);
        
        // Get eigenvalues
        let eigenvalues = Self::get_eigenvalues(&rho_a)?;
        
        // S = -Σ λi log2(λi)
        let mut entropy = 0.0;
        for lambda in eigenvalues {
            let p = lambda.re;
            if p > 1e-10 {
                entropy -= p * p.log2();
            }
        }
        
        Ok(entropy.max(0.0))
    }
    
    /// Partial trace over the second qubit (2-qubit system)
    fn partial_trace_second_qubit(
        rho: &DMatrix<Complex<f64>>,
    ) -> DMatrix<Complex<f64>> {
        let mut rho_a = DMatrix::zeros(2, 2);
        
        // Base: |00⟩, |01⟩, |10⟩, |11⟩
        // Trace over qubit 2: Tr_2(ρ) = Σ_j ⟨j|_2 ρ |j⟩_2
        
        // ⟨0|_2 ρ |0⟩_2
        rho_a[(0, 0)] = rho[(0, 0)] + rho[(1, 1)];
        rho_a[(0, 1)] = rho[(0, 2)] + rho[(1, 3)];
        rho_a[(1, 0)] = rho[(2, 0)] + rho[(3, 1)];
        rho_a[(1, 1)] = rho[(2, 2)] + rho[(3, 3)];
        
        rho_a
    }
    
    // ========================================================================
    // ERGOTROPY
    // ========================================================================
    
    /// Calculates ergotropy (maximum extractable work)
    /// W = Tr(ρH) - Tr(ρ_passive H)
    /// where H is the Hamiltonian and ρ_passive is the passive state
    pub fn calculate_ergotropy(
        rho: &DMatrix<Complex<f64>>,
    ) -> Result<f64, String> {
        // Hamiltoniano simples: H = diag(3, 1, 1, 0) (níveis de energia)
        let hamiltonian = DMatrix::from_diagonal(&DVector::from_vec(vec![
            Complex::new(3.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(0.0, 0.0),
        ]));
        
        // Current energy
        let energy = (rho * &hamiltonian).trace().re;
        
        // Passive state (eigenvalues sorted in descending order)
        let eigenvalues = Self::get_eigenvalues(rho)?;
        let mut sorted_eigenvalues: Vec<f64> = eigenvalues.iter()
            .map(|c| c.re)
            .collect();
        sorted_eigenvalues.sort_by(|a, b| b.partial_cmp(a).unwrap());
        
        // Passive-state energy
        let passive_energy: f64 = sorted_eigenvalues.iter()
            .enumerate()
            .map(|(i, &lambda)| lambda * (3 - i) as f64)
            .sum();
        
        let ergotropy = (energy - passive_energy).max(0.0);
        
        Ok(ergotropy)
    }
    
    // ========================================================================
    // NEGATIVITY
    // ========================================================================
    
    /// Calculates negativity (separability measure)
    /// N = (||ρ^TA|| - 1) / 2
    /// where ρ^TA is the partial transpose
    pub fn calculate_negativity(
        rho: &DMatrix<Complex<f64>>,
    ) -> Result<f64, String> {
        // Partial transpose over the first qubit
        let rho_ta = Self::partial_transpose_first_qubit(rho);
        
        // Trace norm: ||A|| = Tr(sqrt(A† A))
        let rho_ta_dagger = rho_ta.adjoint();
        let product = &rho_ta_dagger * &rho_ta;
        let sqrt_product = Self::matrix_sqrt(&product)?;
        let trace_norm = sqrt_product.trace().re;
        
        let negativity = (trace_norm - 1.0) / 2.0;
        
        Ok(negativity.max(0.0))
    }
    
    /// Partial transpose over the first qubit
    fn partial_transpose_first_qubit(
        rho: &DMatrix<Complex<f64>>,
    ) -> DMatrix<Complex<f64>> {
        let mut rho_ta = rho.clone();
        
        // Transpose 2x2 blocks
        // Base: |00⟩, |01⟩, |10⟩, |11⟩
        // Transpose over qubit 1: |i1,i2⟩ → |i2,i1⟩
        
        // Swap (01) ↔ (10)
        for i in 0..4 {
            let temp = rho_ta[(1, i)];
            rho_ta[(1, i)] = rho_ta[(2, i)];
            rho_ta[(2, i)] = temp;
        }
        
        for i in 0..4 {
            let temp = rho_ta[(i, 1)];
            rho_ta[(i, 1)] = rho_ta[(i, 2)];
            rho_ta[(i, 2)] = temp;
        }
        
        rho_ta
    }
    
    // ========================================================================
    // UTILITIES
    // ========================================================================
    
    /// Square root of a positive Hermitian matrix
    fn matrix_sqrt(
        mat: &DMatrix<Complex<f64>>,
    ) -> Result<DMatrix<Complex<f64>>, String> {
        // Simplification: use spectral decomposition
        let eigenvalues = Self::get_eigenvalues(mat)?;
        
        // sqrt(M) = Σ sqrt(λi) |ei⟩⟨ei|
        // Here we use a diagonal approximation
        let mut result = mat.clone();
        for i in 0..mat.nrows() {
            for j in 0..mat.ncols() {
                if i == j {
                    result[(i, j)] = Complex::new(eigenvalues[i].re.sqrt(), 0.0);
                } else {
                    result[(i, j)] = Complex::new(0.0, 0.0);
                }
            }
        }
        
        Ok(result)
    }
    
    /// Gets eigenvalues (simplified)
    fn get_eigenvalues(
        mat: &DMatrix<Complex<f64>>,
    ) -> Result<Vec<Complex<f64>>, String> {
        // Simplification: use the power method or an external library
        // Here we return an approximation
        let mut eigenvalues = Vec::new();
        for i in 0..mat.nrows() {
            eigenvalues.push(mat[(i, i)]);
        }
        Ok(eigenvalues)
    }
    
    fn current_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

// ============================================================================
// THERMODYNAMIC PHASE CLASSIFIER
// ============================================================================

pub struct ThermodynamicPhaseClassifier;

impl ThermodynamicPhaseClassifier {
    /// Classifies the thermodynamic phase based on metrics
    pub fn classify_phase(state: &EntangledPairState) -> ThermodynamicPhase {
        // Criteria based on TUCU
        
        // High ergotropy + high fidelity → Generation
        if state.ergotropy > 0.8 && state.fidelity > 0.95 {
            return ThermodynamicPhase::Generation;
        }
        
        // Good fidelity + active use → Active
        if state.fidelity > 0.85 && state.reuse_count > 0 {
            return ThermodynamicPhase::Active;
        }
        
        // Degraded fidelity → Degradation
        if state.fidelity < 0.85 && state.fidelity > 0.5 {
            return ThermodynamicPhase::Degradation;
        }
        
        // Low fidelity → Radiation
        if state.fidelity < 0.5 {
            return ThermodynamicPhase::Radiation;
        }
        
        // Default → Inertia
        ThermodynamicPhase::Inertia
    }
    
    /// Decides whether a pair should be recycled
    pub fn should_recycle(state: &EntangledPairState) -> bool {
        let phase = Self::classify_phase(state);
        
        matches!(
            phase,
            ThermodynamicPhase::Radiation | ThermodynamicPhase::Degradation
        )
    }
}

// ============================================================================
// POOL METRICS AGGREGATOR
// ============================================================================

pub struct EMFPoolMetricsAggregator;

impl EMFPoolMetricsAggregator {
    /// Calculates aggregated pool metrics
    pub fn aggregate_metrics(
        pairs: &[EntangledPairState],
    ) -> EMFPoolMetrics {
        let total_pairs = pairs.len();
        
        if total_pairs == 0 {
            return EMFPoolMetrics {
                total_pairs: 0,
                available_pairs: 0,
                degraded_pairs: 0,
                avg_fidelity: 0.0,
                avg_concurrence: 0.0,
                total_ergotropy: 0.0,
                total_entropy: 0.0,
                recycling_rate: 0.0,
                thermodynamic_efficiency: 0.0,
            };
        }
        
        let mut available_pairs = 0;
        let mut degraded_pairs = 0;
        let mut sum_fidelity = 0.0;
        let mut sum_concurrence = 0.0;
        let mut total_ergotropy = 0.0;
        let mut total_entropy = 0.0;
        
        for pair in pairs {
            if pair.fidelity >= 0.85 {
                available_pairs += 1;
            } else {
                degraded_pairs += 1;
            }
            
            sum_fidelity += pair.fidelity;
            sum_concurrence += pair.concurrence;
            total_ergotropy += pair.ergotropy;
            total_entropy += pair.entanglement_entropy;
        }
        
        let avg_fidelity = sum_fidelity / total_pairs as f64;
        let avg_concurrence = sum_concurrence / total_pairs as f64;
        
        // Eficiência termodinâmica: trabalho útil / energia total
        let max_possible_ergotropy = total_pairs as f64 * 3.0;  // 3.0 é energia máxima
        let thermodynamic_efficiency = if max_possible_ergotropy > 0.0 {
            total_ergotropy / max_possible_ergotropy
        } else {
            0.0
        };
        
        // Recycling rate (simplified)
        let recycling_rate = degraded_pairs as f64 / total_pairs as f64;
        
        EMFPoolMetrics {
            total_pairs,
            available_pairs,
            degraded_pairs,
            avg_fidelity,
            avg_concurrence,
            total_ergotropy,
            total_entropy,
            recycling_rate,
            thermodynamic_efficiency,
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bell_state_fidelity() {
        // Estado de Bell perfeito |Φ+⟩
        let phi_plus = EntanglementMetricsCalculator::bell_state_phi_plus();
        
        let fidelity = EntanglementMetricsCalculator::calculate_fidelity(&phi_plus)
            .unwrap();
        
        assert!((fidelity - 1.0).abs() < 1e-6, "Bell-state fidelity must be 1.0");
    }
    
    #[test]
    fn test_separable_state_concurrence() {
        // Separable state |00⟩⟨00| must have concurrence 0
        let mut rho = DMatrix::zeros(4, 4);
        rho[(0, 0)] = Complex::new(1.0, 0.0);
        
        let concurrence = EntanglementMetricsCalculator::calculate_concurrence(&rho)
            .unwrap();
        
        assert!(concurrence < 0.1, "Separable state must have concurrence ~0");
    }
    
    #[test]
    fn test_phase_classification() {
        let state = EntangledPairState {
            density_matrix: vec![],
            fidelity: 0.97,
            concurrence: 0.9,
            entanglement_entropy: 1.0,
            ergotropy: 0.85,
            negativity: 0.5,
            created_at: 0,
            reuse_count: 0,
        };
        
        let phase = ThermodynamicPhaseClassifier::classify_phase(&state);
        assert_eq!(phase, ThermodynamicPhase::Generation);
    }
    
    #[test]
    fn test_metrics_aggregation() {
        let pairs = vec![
            EntangledPairState {
                density_matrix: vec![],
                fidelity: 0.95,
                concurrence: 0.8,
                entanglement_entropy: 0.9,
                ergotropy: 0.7,
                negativity: 0.4,
                created_at: 0,
                reuse_count: 0,
            },
            EntangledPairState {
                density_matrix: vec![],
                fidelity: 0.60,
                concurrence: 0.3,
                entanglement_entropy: 0.5,
                ergotropy: 0.2,
                negativity: 0.1,
                created_at: 0,
                reuse_count: 3,
            },
        ];
        
        let metrics = EMFPoolMetricsAggregator::aggregate_metrics(&pairs);
        
        assert_eq!(metrics.total_pairs, 2);
        assert_eq!(metrics.available_pairs, 1);
        assert_eq!(metrics.degraded_pairs, 1);
        assert!((metrics.avg_fidelity - 0.775).abs() < 0.01);
    }
}
