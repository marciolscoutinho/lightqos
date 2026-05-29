// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// qcr.rs — QCR Protocol — Quantum Channel Routing
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 19-05-2023
// All rights reserved.
// ---------------------------------------------------------------------------

use nalgebra::DVector;
use num_complex::Complex64;
use uuid::Uuid;
use rand::Rng;

// ============================================================================
// QUANTUM CYCLE REGENERATIVE
// ============================================================================

/// QCR protocol
#[derive(Debug, Clone)]
pub struct QuantumCycleRegenerative {
    /// Unique ID
    pub id: Uuid,
    
    /// Regeneration cycles
    pub cycles: Vec<RegenerationCycle>,
    
    /// Maximum number of cycles
    pub max_cycles: usize,
    
    /// Target fidelity threshold
    pub target_fidelity: f64,
    
    /// Statistics
    pub stats: RegenerationStats,
}

impl QuantumCycleRegenerative {
    /// Creates a new QCR protocol
    pub fn new(max_cycles: usize, target_fidelity: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            cycles: Vec::new(),
            max_cycles,
            target_fidelity,
            stats: RegenerationStats::default(),
        }
    }
    
    /// Regenerates a quantum state
    pub fn regenerate(
        &mut self,
        degraded_state: DVector<Complex64>,
        ideal_state: DVector<Complex64>,
    ) -> RegenerationResult {
        assert_eq!(degraded_state.len(), ideal_state.len(), "States must have same size");
        
        let mut current_state = degraded_state.clone();
        let mut fidelity = Self::calculate_fidelity(&current_state, &ideal_state);
        
        let mut cycle_count = 0;
        
        // Regeneration cycles
        while fidelity < self.target_fidelity && cycle_count < self.max_cycles {
            // Create a cycle
            let cycle = RegenerationCycle::new(cycle_count, fidelity);
            
            // Apply purification
            current_state = self.apply_purification(&current_state, &ideal_state);
            
            // Recalculate fidelity
            fidelity = Self::calculate_fidelity(&current_state, &ideal_state);
            
            self.cycles.push(cycle);
            cycle_count += 1;
        }
        
        // Update statistics
        self.stats.total_regenerations += 1;
        if fidelity >= self.target_fidelity {
            self.stats.successful_regenerations += 1;
        }
        self.stats.avg_cycles = (self.stats.avg_cycles * (self.stats.total_regenerations - 1) as f64 
                                 + cycle_count as f64) / self.stats.total_regenerations as f64;
        
        RegenerationResult {
            regenerated_state: current_state,
            initial_fidelity: Self::calculate_fidelity(&degraded_state, &ideal_state),
            final_fidelity: fidelity,
            cycles_used: cycle_count,
            success: fidelity >= self.target_fidelity,
        }
    }
    
    /// Applies harmonic purification
    fn apply_purification(
        &self,
        state: &DVector<Complex64>,
        ideal: &DVector<Complex64>,
    ) -> DVector<Complex64> {
        let mut purified = state.clone();
        
        // Purification: move closer to the ideal state
        let learning_rate = 0.1; // 10% por ciclo
        
        for i in 0..purified.len() {
            let error = ideal[i] - purified[i];
            purified[i] += error * learning_rate;
        }
        
        // Renormalize
        Self::normalize(&mut purified);
        
        purified
    }
    
    /// Calculates fidelity F = |⟨ψ|φ⟩|²
    fn calculate_fidelity(state1: &DVector<Complex64>, state2: &DVector<Complex64>) -> f64 {
        let overlap: Complex64 = state1.iter()
            .zip(state2.iter())
            .map(|(a, b)| a.conj() * b)
            .sum();
        
        overlap.norm_sqr()
    }
    
    /// Normalizes the state
    fn normalize(state: &mut DVector<Complex64>) {
        let norm = state.iter().map(|z| z.norm_sqr()).sum::<f64>().sqrt();
        if norm > 1.0e-10 {
            *state /= norm;
        }
    }
    
    /// Applies purification with a specific octave
    pub fn purify_with_octave(
        &mut self,
        state: &DVector<Complex64>,
        ideal: &DVector<Complex64>,
        octave: u8,
    ) -> DVector<Complex64> {
        assert!(octave >= 1 && octave <= 10, "Octave must be 1-10");
        
        // Harmonic resonance based on the octave
        let resonance_freq = 2.0_f64.powi((octave - 1) as i32);
        let phase_factor = Complex64::new(
            0.0,
            std::f64::consts::PI * resonance_freq / 1000.0
        ).exp();
        
        let mut purified = state.clone();
        
        // Apply resonance
        for amplitude in purified.iter_mut() {
            *amplitude *= phase_factor;
        }
        
        // Move closer to the ideal state
        for i in 0..purified.len() {
            let error = ideal[i] - purified[i];
            purified[i] += error * 0.1;
        }
        
        Self::normalize(&mut purified);
        purified
    }
    
    /// Clears the cycle history
    pub fn clear_cycles(&mut self) {
        self.cycles.clear();
    }
}

// ============================================================================
// REGENERATION CYCLE
// ============================================================================

/// Regeneration cycle
#[derive(Debug, Clone)]
pub struct RegenerationCycle {
    /// Cycle number
    pub cycle_number: usize,
    
    /// Fidelity at the beginning of the cycle
    pub initial_fidelity: f64,
    
    /// Octave used (1-10)
    pub octave: u8,
    
    /// Applied energy (J)
    pub energy_applied: f64,
}

impl RegenerationCycle {
    pub fn new(cycle_number: usize, initial_fidelity: f64) -> Self {
        let mut rng = rand::thread_rng();
        
        Self {
            cycle_number,
            initial_fidelity,
            octave: rng.gen_range(1..=10), // Random octave
            energy_applied: rng.gen::<f64>() * 1.0e-20,
        }
    }
}

// ============================================================================
// REGENERATION RESULT
// ============================================================================

/// Regeneration result
#[derive(Debug, Clone)]
pub struct RegenerationResult {
    /// Regenerated state
    pub regenerated_state: DVector<Complex64>,
    
    /// Initial fidelity
    pub initial_fidelity: f64,
    
    /// Final fidelity
    pub final_fidelity: f64,
    
    /// Number of cycles used
    pub cycles_used: usize,
    
    /// Was regeneration successful?
    pub success: bool,
}

// ============================================================================
// STATISTICS
// ============================================================================

/// Regeneration statistics
#[derive(Debug, Clone, Default)]
pub struct RegenerationStats {
    /// Total regenerations
    pub total_regenerations: usize,
    
    /// Successful regenerations
    pub successful_regenerations: usize,
    
    /// Average number of cycles
    pub avg_cycles: f64,
}

impl RegenerationStats {
    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_regenerations == 0 {
            return 0.0;
        }
        self.successful_regenerations as f64 / self.total_regenerations as f64
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_qcr_creation() {
        let qcr = QuantumCycleRegenerative::new(10, 0.95);
        assert_eq!(qcr.max_cycles, 10);
        assert_eq!(qcr.target_fidelity, 0.95);
    }
    
    #[test]
    fn test_regeneration() {
        let mut qcr = QuantumCycleRegenerative::new(20, 0.95);
        
        // Ideal state
        let ideal = DVector::from_vec(vec![
            Complex64::new(1.0 / 2.0_f64.sqrt(), 0.0),
            Complex64::new(1.0 / 2.0_f64.sqrt(), 0.0),
        ]);
        
        // Degraded state
        let degraded = DVector::from_vec(vec![
            Complex64::new(0.8, 0.0),
            Complex64::new(0.6, 0.0),
        ]);
        
        let result = qcr.regenerate(degraded, ideal);
        
        assert!(result.final_fidelity > result.initial_fidelity);
    }
    
    #[test]
    fn test_fidelity() {
        let state1 = DVector::from_vec(vec![
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
        ]);
        
        let state2 = DVector::from_vec(vec![
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
        ]);
        
        let fidelity = QuantumCycleRegenerative::calculate_fidelity(&state1, &state2);
        assert!((fidelity - 1.0).abs() < 1.0e-10);
    }
}
