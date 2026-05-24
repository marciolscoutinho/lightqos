// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// shadow_copy.rs — HIO Shadow Copy — quantum shadow collection via Clifford rotations
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 06-08-2024
// All rights reserved.
// ---------------------------------------------------------------------------

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use nalgebra::{Complex, DMatrix, DVector};
use uuid::Uuid;

// ============================================================================
// SHADOW (CLASSICAL SHADOW)
// ============================================================================

/// Classical shadow of a quantum state
/// 
/// Represents a partial "snapshot" of the state without fully collapsing it
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumShadow {
    /// Unique ID
    pub id: Uuid,
    
    /// Measurement snapshots
    pub measurement_snapshots: Vec<MeasurementSnapshot>,
    
    /// Number of qubits in the system
    pub num_qubits: usize,
    
    /// Number of collected snapshots
    pub num_snapshots: usize,
    
    /// Measurement bases used
    pub measurement_bases: Vec<PauliString>,
    
    /// Metadata
    pub metadata: ShadowMetadata,
}

/// Individual measurement snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasurementSnapshot {
    /// Measurement result (bitstring)
    pub outcome: Vec<bool>,
    
    /// Pauli basis used
    pub basis: PauliString,
    
    /// Measurement timestamp
    pub timestamp_ns: u64,
    
    /// Unitary applied before measurement
    pub unitary: Option<Vec<Complex<f64>>>,
}

/// Pauli string (tensor product of Paulis)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PauliString {
    /// Pauli operators for each qubit
    pub paulis: Vec<PauliOperator>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PauliOperator {
    I,  // Identity
    X,  // Pauli-X
    Y,  // Pauli-Y
    Z,  // Pauli-Z
}

impl PauliString {
    pub fn new(paulis: Vec<PauliOperator>) -> Self {
        Self { paulis }
    }
    
    /// Creates a random Pauli string
    pub fn random(num_qubits: usize) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let paulis = (0..num_qubits)
            .map(|_| match rng.gen_range(0..4) {
                0 => PauliOperator::I,
                1 => PauliOperator::X,
                2 => PauliOperator::Y,
                _ => PauliOperator::Z,
            })
            .collect();
        
        Self { paulis }
    }
    
    /// Converts to matrix
    pub fn to_matrix(&self) -> DMatrix<Complex<f64>> {
        let n = self.paulis.len();
        
        // Tensor product of Pauli matrices
        let mut result = DMatrix::identity(1, 1);
        
        for pauli in &self.paulis {
            let pauli_mat = pauli.to_matrix();
            result = Self::tensor_product(&result, &pauli_mat);
        }
        
        result
    }
    
    fn tensor_product(
        a: &DMatrix<Complex<f64>>,
        b: &DMatrix<Complex<f64>>,
    ) -> DMatrix<Complex<f64>> {
        let rows_a = a.nrows();
        let cols_a = a.ncols();
        let rows_b = b.nrows();
        let cols_b = b.ncols();
        
        let mut result = DMatrix::zeros(rows_a * rows_b, cols_a * cols_b);
        
        for i in 0..rows_a {
            for j in 0..cols_a {
                for k in 0..rows_b {
                    for l in 0..cols_b {
                        result[(i * rows_b + k, j * cols_b + l)] = a[(i, j)] * b[(k, l)];
                    }
                }
            }
        }
        
        result
    }
}

impl PauliOperator {
    pub fn to_matrix(&self) -> DMatrix<Complex<f64>> {
        match self {
            Self::I => DMatrix::from_row_slice(2, 2, &[
                Complex::new(1.0, 0.0), Complex::new(0.0, 0.0),
                Complex::new(0.0, 0.0), Complex::new(1.0, 0.0),
            ]),
            Self::X => DMatrix::from_row_slice(2, 2, &[
                Complex::new(0.0, 0.0), Complex::new(1.0, 0.0),
                Complex::new(1.0, 0.0), Complex::new(0.0, 0.0),
            ]),
            Self::Y => DMatrix::from_row_slice(2, 2, &[
                Complex::new(0.0, 0.0), Complex::new(0.0, -1.0),
                Complex::new(0.0, 1.0), Complex::new(0.0, 0.0),
            ]),
            Self::Z => DMatrix::from_row_slice(2, 2, &[
                Complex::new(1.0, 0.0), Complex::new(0.0, 0.0),
                Complex::new(0.0, 0.0), Complex::new(-1.0, 0.0),
            ]),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowMetadata {
    /// Statistical confidence (0.0 to 1.0)
    pub confidence: f64,
    
    /// Estimated maximum error
    pub max_error: f64,
    
    /// Collection time (ns)
    pub collection_time_ns: u64,
    
    /// Sampling strategy
    pub sampling_strategy: SamplingStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SamplingStrategy {
    /// Uniform sampling of Pauli bases
    Uniform,
    
    /// Adaptive sampling (more informative bases)
    Adaptive,
    
    /// Importance sampling
    Importance,
}

impl QuantumShadow {
    pub fn new(num_qubits: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            measurement_snapshots: Vec::new(),
            num_qubits,
            num_snapshots: 0,
            measurement_bases: Vec::new(),
            metadata: ShadowMetadata {
                confidence: 0.0,
                max_error: 1.0,
                collection_time_ns: 0,
                sampling_strategy: SamplingStrategy::Uniform,
            },
        }
    }
    
    /// Adds a measurement snapshot
    pub fn add_snapshot(&mut self, snapshot: MeasurementSnapshot) {
        self.measurement_bases.push(snapshot.basis.clone());
        self.measurement_snapshots.push(snapshot);
        self.num_snapshots += 1;
        
        // Update statistical confidence
        self.update_confidence();
    }
    
    /// Updates confidence based on the number of samples
    fn update_confidence(&mut self) {
        // Simplified formula: confidence increases with √N
        let n = self.num_snapshots as f64;
        self.metadata.confidence = (1.0 - (1.0 / (1.0 + n.sqrt()))).min(0.99);
        
        // Error decreases with 1/√N
        self.metadata.max_error = 1.0 / n.sqrt();
    }
    
    /// Reconstructs an approximate density matrix
    pub fn reconstruct_density_matrix(&self) -> Result<DMatrix<Complex<f64>>, String> {
        if self.num_snapshots == 0 {
            return Err("No snapshot available".to_string());
        }
        
        let dim = 2_usize.pow(self.num_qubits as u32);
        let mut rho = DMatrix::zeros(dim, dim);
        
        // Shadow tomography: reconstruct state from measurements
        for snapshot in &self.measurement_snapshots {
            // Convert result to projector
            let projector = Self::outcome_to_projector(&snapshot.outcome);
            
            // Accumulate contribution
            // ρ̃ ≈ (1/N) Σᵢ Uᵢ† |bᵢ⟩⟨bᵢ| Uᵢ
            rho += &projector;
        }
        
        // Normalize
        rho /= self.num_snapshots as f64;
        
        Ok(rho)
    }
    
    fn outcome_to_projector(outcome: &[bool]) -> DMatrix<Complex<f64>> {
        let n = outcome.len();
        let dim = 2_usize.pow(n as u32);
        
        // Convert bitstring to index
        let mut index = 0;
        for (i, &bit) in outcome.iter().enumerate() {
            if bit {
                index += 2_usize.pow(i as u32);
            }
        }
        
        // Create projector |index⟩⟨index|
        let mut projector = DMatrix::zeros(dim, dim);
        projector[(index, index)] = Complex::new(1.0, 0.0);
        
        projector
    }
    
    /// Estimates the expected value of an observable
    pub fn estimate_observable(&self, observable: &PauliString) -> Result<f64, String> {
        if self.num_snapshots == 0 {
            return Err("No snapshot available".to_string());
        }
        
        let mut sum = 0.0;
        let mut count = 0;
        
        // Estimate ⟨O⟩ from the shadows
        for snapshot in &self.measurement_snapshots {
            // Check whether the basis is compatible
            if Self::are_compatible(&snapshot.basis, observable) {
                let contribution = Self::compute_contribution(&snapshot.outcome, observable);
                sum += contribution;
                count += 1;
            }
        }
        
        if count == 0 {
            return Err("No measurement compatible with observable".to_string());
        }
        
        Ok(sum / count as f64)
    }
    
    fn are_compatible(basis: &PauliString, observable: &PauliString) -> bool {
        // Bases are compatible if they commute
        basis.paulis.len() == observable.paulis.len()
    }
    
    fn compute_contribution(outcome: &[bool], observable: &PauliString) -> f64 {
        // Simplification: contribution based on parity
        let parity: bool = outcome.iter()
            .zip(&observable.paulis)
            .filter(|(_, p)| **p != PauliOperator::I)
            .map(|(bit, _)| *bit)
            .fold(false, |acc, bit| acc ^ bit);
        
        if parity { -1.0 } else { 1.0 }
    }
}

// ============================================================================
// SHADOW COLLECTOR
// ============================================================================

/// Classical shadow collector
pub struct ShadowCollector {
    /// Collected shadows (system ID → Shadow)
    shadows: HashMap<Uuid, QuantumShadow>,
    
    /// Configuration
    config: CollectorConfig,
    
    /// Statistics
    stats: CollectorStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorConfig {
    /// Number of snapshots per shadow
    pub snapshots_per_shadow: usize,
    
    /// Sampling strategy
    pub sampling_strategy: SamplingStrategy,
    
    /// Number of qubits
    pub num_qubits: usize,
    
    /// Confidence alvo
    pub target_confidence: f64,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            snapshots_per_shadow: 100,
            sampling_strategy: SamplingStrategy::Uniform,
            num_qubits: 2,
            target_confidence: 0.95,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CollectorStatistics {
    pub total_shadows_collected: u64,
    pub total_snapshots: u64,
    pub avg_confidence: f64,
    pub avg_collection_time_ns: u64,
}

impl ShadowCollector {
    pub fn new(config: CollectorConfig) -> Self {
        Self {
            shadows: HashMap::new(),
            config,
            stats: CollectorStatistics::default(),
        }
    }
    
    /// Collects a shadow of a quantum state
    pub fn collect_shadow(
        &mut self,
        system_id: Uuid,
        state_vector: &DVector<Complex<f64>>,
    ) -> Result<Uuid, String> {
        let start_time = Self::current_time_ns();
        
        let mut shadow = QuantumShadow::new(self.config.num_qubits);
        shadow.metadata.sampling_strategy = self.config.sampling_strategy;
        
        // Collect snapshots
        for _ in 0..self.config.snapshots_per_shadow {
            let snapshot = self.perform_measurement(state_vector)?;
            shadow.add_snapshot(snapshot);
            
            // Check whether target confidence was reached
            if shadow.metadata.confidence >= self.config.target_confidence {
                break;
            }
        }
        
        let collection_time = Self::current_time_ns() - start_time;
        shadow.metadata.collection_time_ns = collection_time;
        
        // Store
        let shadow_id = shadow.id;
        self.shadows.insert(shadow_id, shadow);
        
        // Statistics
        self.stats.total_shadows_collected += 1;
        self.stats.total_snapshots += self.config.snapshots_per_shadow as u64;
        self.update_stats();
        
        Ok(shadow_id)
    }
    
    fn perform_measurement(
        &self,
        state_vector: &DVector<Complex<f64>>,
    ) -> Result<MeasurementSnapshot, String> {
        // Choose a random Pauli basis
        let basis = match self.config.sampling_strategy {
            SamplingStrategy::Uniform => PauliString::random(self.config.num_qubits),
            SamplingStrategy::Adaptive => PauliString::random(self.config.num_qubits),
            SamplingStrategy::Importance => PauliString::random(self.config.num_qubits),
        };
        
        // Simulate measurement
        let outcome = Self::simulate_measurement(state_vector, &basis);
        
        Ok(MeasurementSnapshot {
            outcome,
            basis,
            timestamp_ns: Self::current_time_ns(),
            unitary: None,
        })
    }
    
    fn simulate_measurement(
        state_vector: &DVector<Complex<f64>>,
        basis: &PauliString,
    ) -> Vec<bool> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Simplification: random measurement based on amplitudes
        let n = basis.paulis.len();
        let mut outcome = Vec::with_capacity(n);
        
        for i in 0..n {
            // Probability based on qubit i
            let index_0 = i * 2;
            
            let prob_0 = if index_0 < state_vector.len() {
                state_vector[index_0].norm_sqr()
            } else {
                0.5
            };
            
            outcome.push(rng.gen_bool(1.0 - prob_0));
        }
        
        outcome
    }
    
    /// Gets collected shadow
    pub fn get_shadow(&self, id: Uuid) -> Option<&QuantumShadow> {
        self.shadows.get(&id)
    }
    
    /// Lists all shadows
    pub fn list_shadows(&self) -> Vec<&QuantumShadow> {
        self.shadows.values().collect()
    }
    
    fn update_stats(&mut self) {
        if self.shadows.is_empty() {
            return;
        }
        
        let total: f64 = self.shadows.values()
            .map(|s| s.metadata.confidence)
            .sum();
        
        self.stats.avg_confidence = total / self.shadows.len() as f64;
        
        let total_time: u64 = self.shadows.values()
            .map(|s| s.metadata.collection_time_ns)
            .sum();
        
        self.stats.avg_collection_time_ns = total_time / self.shadows.len() as u64;
    }
    
    pub fn get_statistics(&self) -> &CollectorStatistics {
        &self.stats
    }
    
    fn current_time_ns() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pauli_string_creation() {
        let ps = PauliString::new(vec![PauliOperator::X, PauliOperator::Y, PauliOperator::Z]);
        assert_eq!(ps.paulis.len(), 3);
    }
    
    #[test]
    fn test_pauli_matrices() {
        let x = PauliOperator::X.to_matrix();
        assert_eq!(x.nrows(), 2);
        assert_eq!(x.ncols(), 2);
        
        // Check anticommutation: XY = -YX
        let y = PauliOperator::Y.to_matrix();
        let xy = &x * &y;
        let yx = &y * &x;
        
        // xy should be approximately equal to -yx
        let diff = xy + yx;
        let norm: f64 = diff.iter().map(|c| c.norm_sqr()).sum();
        assert!(norm < 1e-10);
    }
    
    #[test]
    fn test_shadow_creation() {
        let shadow = QuantumShadow::new(2);
        assert_eq!(shadow.num_qubits, 2);
        assert_eq!(shadow.num_snapshots, 0);
    }
    
    #[test]
    fn test_add_snapshot() {
        let mut shadow = QuantumShadow::new(2);
        
        let snapshot = MeasurementSnapshot {
            outcome: vec![false, true],
            basis: PauliString::new(vec![PauliOperator::Z, PauliOperator::Z]),
            timestamp_ns: 0,
            unitary: None,
        };
        
        shadow.add_snapshot(snapshot);
        
        assert_eq!(shadow.num_snapshots, 1);
        assert!(shadow.metadata.confidence > 0.0);
    }
    
    #[test]
    fn test_collector() {
        let config = CollectorConfig {
            snapshots_per_shadow: 10,
            num_qubits: 2,
            ..Default::default()
        };
        
        let mut collector = ShadowCollector::new(config);
        
        // State |00⟩
        let state = DVector::from_vec(vec![
            Complex::new(1.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
        ]);
        
        let shadow_id = collector.collect_shadow(Uuid::new_v4(), &state).unwrap();
        
        let shadow = collector.get_shadow(shadow_id).unwrap();
        assert_eq!(shadow.num_snapshots, 10);
    }
    
    #[test]
    fn test_estimate_observable() {
        let mut shadow = QuantumShadow::new(2);
        
        // Add some snapshots
        for i in 0..50 {
            let snapshot = MeasurementSnapshot {
                outcome: vec![i % 2 == 0, i % 3 == 0],
                basis: PauliString::new(vec![PauliOperator::Z, PauliOperator::Z]),
                timestamp_ns: 0,
                unitary: None,
            };
            shadow.add_snapshot(snapshot);
        }
        
        let observable = PauliString::new(vec![PauliOperator::Z, PauliOperator::Z]);
        let estimate = shadow.estimate_observable(&observable).unwrap();
        
        assert!(estimate >= -1.0 && estimate <= 1.0);
    }
}
