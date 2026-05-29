// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// emf_simulator.rs — EMF Simulator — Entangled Memory Fabric simulation
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 17-08-2023
// All rights reserved.
// ---------------------------------------------------------------------------

use crate::quantum_state::QuantumState;
use nalgebra::DVector;
use num_complex::Complex64;
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// EMF SIMULATOR
// ============================================================================

/// Entanglement simulator
#[derive(Debug, Clone)]
pub struct EMFSimulator {
    /// Simulated entangled pairs
    pub pairs: HashMap<Uuid, EntanglementSimulation>,
    
    /// Maximum capacity
    pub max_pairs: usize,
    
    /// Simulation time
    pub time: f64,
}

impl EMFSimulator {
    /// Creates a new simulator
    pub fn new(max_pairs: usize) -> Self {
        Self {
            pairs: HashMap::new(),
            max_pairs,
            time: 0.0,
        }
    }
    
    /// Generates an entangled pair
    pub fn generate_pair(
        &mut self,
        qubit_a: usize,
        qubit_b: usize,
        initial_fidelity: f64,
    ) -> Result<Uuid, String> {
        if self.pairs.len() >= self.max_pairs {
            return Err("Capacity exceeded".to_string());
        }
        
        let pair = EntanglementSimulation::new(qubit_a, qubit_b, initial_fidelity);
        let id = pair.id;
        self.pairs.insert(id, pair);
        
        Ok(id)
    }
    
    /// Gets a pair
    pub fn get_pair(&self, id: &Uuid) -> Option<&EntanglementSimulation> {
        self.pairs.get(id)
    }
    
    /// Removes a pair (consumption)
    pub fn consume_pair(&mut self, id: &Uuid) -> Option<EntanglementSimulation> {
        self.pairs.remove(id)
    }
    
    /// Evolves all pairs
    pub fn evolve(&mut self, dt: f64) {
        for pair in self.pairs.values_mut() {
            pair.evolve(dt);
        }
        self.time += dt;
    }
    
    /// Average fidelity
    pub fn avg_fidelity(&self) -> f64 {
        if self.pairs.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.pairs.values().map(|p| p.current_fidelity).sum();
        sum / self.pairs.len() as f64
    }
    
    /// Statistics
    pub fn statistics(&self) -> EMFStatistics {
        EMFStatistics {
            total_pairs: self.pairs.len(),
            max_capacity: self.max_pairs,
            avg_fidelity: self.avg_fidelity(),
            time: self.time,
        }
    }
}

// ============================================================================
// ENTANGLEMENT SIMULATION
// ============================================================================

/// Entangled pair simulation
#[derive(Debug, Clone)]
pub struct EntanglementSimulation {
    /// Unique ID
    pub id: Uuid,
    
    /// Qubit A
    pub qubit_a: usize,
    
    /// Qubit B
    pub qubit_b: usize,
    
    /// Quantum state of the pair
    pub state: QuantumState,
    
    /// Initial fidelity
    pub initial_fidelity: f64,
    
    /// Current fidelity
    pub current_fidelity: f64,
    
    /// Age (timesteps)
    pub age: u64,
}

impl EntanglementSimulation {
    /// Creates a new pair
    pub fn new(qubit_a: usize, qubit_b: usize, initial_fidelity: f64) -> Self {
        // Ideal Bell pair
        let ideal_bell = QuantumState::bell_state();
        
        // Apply noise to obtain the desired fidelity
        let state = Self::apply_noise(ideal_bell, 1.0 - initial_fidelity);
        
        Self {
            id: Uuid::new_v4(),
            qubit_a,
            qubit_b,
            state,
            initial_fidelity,
            current_fidelity: initial_fidelity,
            age: 0,
        }
    }
    
    /// Applies noise to the state
    fn apply_noise(mut state: QuantumState, noise_level: f64) -> QuantumState {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        for amplitude in state.amplitudes.iter_mut() {
            let noise_re = rng.gen::<f64>() * noise_level - noise_level / 2.0;
            let noise_im = rng.gen::<f64>() * noise_level - noise_level / 2.0;
            
            *amplitude += Complex64::new(noise_re, noise_im);
        }
        
        state.normalize();
        state
    }
    
    /// Evolves the pair (degradation)
    pub fn evolve(&mut self, dt: f64) {
        self.age += 1;
        
        // Exponential degradation
        let decay_rate = 0.01; // 1% por timestep
        self.current_fidelity *= f64::exp(-decay_rate * dt);
        
        // Apply noise to the state
        let noise = 0.001 * dt;
        self.state = Self::apply_noise(self.state.clone(), noise);
    }
    
    /// Calculates concurrence (entanglement measure)
    pub fn concurrence(&self) -> f64 {
        // Simplification: concurrence for the Bell state
        // C = |⟨ψ|Φ⁺⟩|²
        let ideal_bell = QuantumState::bell_state();
        self.state.fidelity(&ideal_bell).sqrt()
    }
    
    /// Measures the pair (Bell measurement)
    pub fn measure_bell_basis(&self) -> BellMeasurement {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let probs = self.state.probabilities();
        let random: f64 = rng.gen();
        
        let mut cumulative = 0.0;
        
        // |Φ⁺⟩ = |00⟩ + |11⟩
        if let Some(&p) = probs.get("00") {
            cumulative += p;
            if random < cumulative {
                return BellMeasurement::PhiPlus;
            }
        }
        
        if let Some(&p) = probs.get("11") {
            cumulative += p;
            if random < cumulative {
                return BellMeasurement::PhiPlus;
            }
        }
        
        // |Φ⁻⟩ = |00⟩ - |11⟩
        if let Some(&p) = probs.get("01") {
            cumulative += p;
            if random < cumulative {
                return BellMeasurement::PhiMinus;
            }
        }
        
        // |Ψ⁺⟩ = |01⟩ + |10⟩
        if let Some(&p) = probs.get("10") {
            cumulative += p;
            if random < cumulative {
                return BellMeasurement::PsiPlus;
            }
        }
        
        // Fallback
        BellMeasurement::PsiMinus
    }
    
    /// Checks whether the pair is usable
    pub fn is_usable(&self, threshold: f64) -> bool {
        self.current_fidelity >= threshold
    }
}

// ============================================================================
// BELL MEASUREMENT
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BellMeasurement {
    /// |Φ⁺⟩ = (|00⟩ + |11⟩)/√2
    PhiPlus,
    
    /// |Φ⁻⟩ = (|00⟩ - |11⟩)/√2
    PhiMinus,
    
    /// |Ψ⁺⟩ = (|01⟩ + |10⟩)/√2
    PsiPlus,
    
    /// |Ψ⁻⟩ = (|01⟩ - |10⟩)/√2
    PsiMinus,
}

// ============================================================================
// STATISTICS
// ============================================================================

#[derive(Debug, Clone)]
pub struct EMFStatistics {
    pub total_pairs: usize,
    pub max_capacity: usize,
    pub avg_fidelity: f64,
    pub time: f64,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_emf_simulator() {
        let sim = EMFSimulator::new(100);
        assert_eq!(sim.max_pairs, 100);
        assert_eq!(sim.pairs.len(), 0);
    }
    
    #[test]
    fn test_generate_pair() {
        let mut sim = EMFSimulator::new(100);
        let result = sim.generate_pair(0, 1, 0.95);
        
        assert!(result.is_ok());
        assert_eq!(sim.pairs.len(), 1);
    }
    
    #[test]
    fn test_capacity_limit() {
        let mut sim = EMFSimulator::new(2);
        
        sim.generate_pair(0, 1, 0.95).unwrap();
        sim.generate_pair(2, 3, 0.95).unwrap();
        
        let result = sim.generate_pair(4, 5, 0.95);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_evolution() {
        let mut sim = EMFSimulator::new(100);
        sim.generate_pair(0, 1, 0.95).unwrap();
        
        let initial_fidelity = sim.avg_fidelity();
        
        sim.evolve(1.0);
        
        let final_fidelity = sim.avg_fidelity();
        assert!(final_fidelity < initial_fidelity);
    }
    
    #[test]
    fn test_concurrence() {
        let pair = EntanglementSimulation::new(0, 1, 0.99);
        let concurrence = pair.concurrence();
        
        assert!(concurrence > 0.9);
        assert!(concurrence <= 1.0);
    }
    
    #[test]
    fn test_consume_pair() {
        let mut sim = EMFSimulator::new(100);
        let id = sim.generate_pair(0, 1, 0.95).unwrap();
        
        let pair = sim.consume_pair(&id);
        assert!(pair.is_some());
        assert_eq!(sim.pairs.len(), 0);
    }
}
